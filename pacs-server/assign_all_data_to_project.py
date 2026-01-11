#!/usr/bin/env python3
"""
프로젝트에 모든 DICOM 데이터 할당 스크립트

사용법:
    python3 assign_all_data_to_project.py --project-id 2 --base-url http://localhost:8080 --token YOUR_TOKEN
"""

import argparse
import requests
import sys
import time
from typing import List, Dict, Optional
from urllib.parse import urljoin

class ProjectDataAssigner:
    def __init__(self, base_url: str, token: str, project_id: int):
        self.base_url = base_url.rstrip('/')
        self.token = token
        self.project_id = project_id
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }
        self.assigned_studies = set()
        self.assigned_series = set()
        self.failed_series = []
        
    def get_all_studies(self, limit: int = 10000) -> List[Dict]:
        """QIDO를 통해 모든 Study 조회 (Admin API 사용)"""
        print(f"📋 모든 Study 조회 중... (limit={limit})")
        
        # Admin API 사용 (project_id 없이 모든 Study 조회)
        url = f"{self.base_url}/api/admin/dicom/studies"
        params = {"limit": str(limit)}
        
        try:
            response = requests.get(url, headers=self.headers, params=params, timeout=60)
            response.raise_for_status()
            studies = response.json()
            
            if isinstance(studies, list):
                print(f"✅ {len(studies)}개 Study 발견")
                return studies
            else:
                print(f"⚠️  예상치 못한 응답 형식: {type(studies)}")
                return []
        except requests.exceptions.RequestException as e:
            print(f"❌ Study 조회 실패: {e}")
            print(f"   URL: {url}")
            print(f"   Response: {e.response.text if hasattr(e, 'response') and e.response else 'N/A'}")
            return []
    
    def get_study_series(self, study_uid: str, limit: int = 1000) -> List[Dict]:
        """특정 Study의 모든 Series 조회 (Admin API 사용)"""
        url = f"{self.base_url}/api/admin/dicom/studies/{study_uid}/series"
        params = {"limit": str(limit)}
        
        try:
            response = requests.get(url, headers=self.headers, params=params, timeout=30)
            response.raise_for_status()
            series_list = response.json()
            
            if isinstance(series_list, list):
                return series_list
            else:
                return []
        except requests.exceptions.RequestException as e:
            print(f"  ⚠️  Study {study_uid}의 Series 조회 실패: {e}")
            return []
    
    def assign_study(self, study_uid: str, study_data: Dict) -> bool:
        """Study를 프로젝트에 할당"""
        if study_uid in self.assigned_studies:
            return True
            
        url = f"{self.base_url}/api/projects/{self.project_id}/studies/assign"
        
        # Study 데이터 추출
        payload = {
            "study_uid": study_uid,
            "study_description": study_data.get("00081030", {}).get("Value", [""])[0] if "00081030" in study_data else "",
            "patient_id": study_data.get("00100020", {}).get("Value", [""])[0] if "00100020" in study_data else "",
            "patient_name": study_data.get("00100010", {}).get("Value", [""])[0] if "00100010" in study_data else "",
            "study_date": study_data.get("00080020", {}).get("Value", [""])[0] if "00080020" in study_data else None
        }
        
        try:
            response = requests.post(url, json=payload, headers=self.headers, timeout=30)
            if response.status_code in [200, 201]:
                self.assigned_studies.add(study_uid)
                return True
            elif response.status_code == 409:
                # 이미 할당됨
                self.assigned_studies.add(study_uid)
                return True
            else:
                print(f"  ⚠️  Study {study_uid} 할당 실패: {response.status_code} - {response.text[:100]}")
                return False
        except requests.exceptions.RequestException as e:
            print(f"  ⚠️  Study {study_uid} 할당 실패: {e}")
            return False
    
    def assign_series(self, study_uid: str, series_data: Dict) -> bool:
        """Series를 프로젝트에 할당"""
        # DICOM 태그 형식에서 Series UID 추출
        series_uid = None
        if "0020000E" in series_data:
            tag_data = series_data["0020000E"]
            if isinstance(tag_data, dict) and "Value" in tag_data:
                value = tag_data["Value"]
                if isinstance(value, list) and len(value) > 0:
                    series_uid = str(value[0])
        
        if not series_uid:
            return False
            
        if series_uid in self.assigned_series:
            return True
        
        url = f"{self.base_url}/api/projects/{self.project_id}/series/assign"
        
        # Series 데이터 추출 (DICOM 태그 형식)
        def extract_tag_value(tag: str, default=""):
            if tag in series_data:
                tag_data = series_data[tag]
                if isinstance(tag_data, dict) and "Value" in tag_data:
                    value = tag_data["Value"]
                    if isinstance(value, list) and len(value) > 0:
                        return str(value[0])
            return default
        
        payload = {
            "study_uid": study_uid,
            "series_uid": series_uid,
            "series_description": extract_tag_value("0008103E", ""),
            "modality": extract_tag_value("00080060", ""),
            "series_number": None
        }
        
        # Series Number 추출 (숫자 변환 시도)
        series_num_str = extract_tag_value("00200011", "")
        if series_num_str:
            try:
                payload["series_number"] = int(series_num_str)
            except (ValueError, TypeError):
                pass
        
        try:
            response = requests.post(url, json=payload, headers=self.headers, timeout=30)
            if response.status_code in [200, 201]:
                self.assigned_series.add(series_uid)
                return True
            elif response.status_code == 409:
                # 이미 할당됨
                self.assigned_series.add(series_uid)
                return True
            else:
                error_msg = response.text[:200] if response.text else "Unknown error"
                self.failed_series.append({
                    "study_uid": study_uid,
                    "series_uid": series_uid,
                    "error": f"{response.status_code}: {error_msg}"
                })
                return False
        except requests.exceptions.RequestException as e:
            self.failed_series.append({
                "study_uid": study_uid,
                "series_uid": series_uid,
                "error": str(e)
            })
            return False
    
    def extract_study_uid(self, study_data: Dict) -> Optional[str]:
        """Study 데이터에서 Study UID 추출 (DICOM 태그 형식)"""
        # DICOM 태그 형식: {"0020000D": {"vr": "UI", "Value": ["1.2.3.4"]}}
        if "0020000D" in study_data:
            tag_data = study_data["0020000D"]
            if isinstance(tag_data, dict) and "Value" in tag_data:
                value = tag_data["Value"]
                if isinstance(value, list) and len(value) > 0:
                    return str(value[0])
        return None
    
    def run(self):
        """모든 데이터 할당 실행"""
        print(f"🚀 프로젝트 {self.project_id}에 모든 데이터 할당 시작")
        print(f"   Base URL: {self.base_url}")
        print("=" * 60)
        
        # 1. 모든 Study 조회
        studies = self.get_all_studies(limit=10000)
        
        if not studies:
            print("❌ Study가 없습니다. 종료합니다.")
            return
        
        total_studies = len(studies)
        total_series = 0
        
        # 2. 각 Study와 Series 할당
        for idx, study_data in enumerate(studies, 1):
            study_uid = self.extract_study_uid(study_data)
            if not study_uid:
                print(f"  [{idx}/{total_studies}] Study UID를 찾을 수 없음, 건너뜀")
                continue
            
            print(f"  [{idx}/{total_studies}] Study: {study_uid[:50]}...")
            
            # Study 할당 (선택사항, Series 할당 시 자동 생성됨)
            # self.assign_study(study_uid, study_data)
            
            # Study의 모든 Series 조회
            series_list = self.get_study_series(study_uid, limit=1000)
            total_series += len(series_list)
            
            if not series_list:
                print(f"    ⚠️  Series 없음")
                continue
            
            # 각 Series 할당
            for series_idx, series_data in enumerate(series_list, 1):
                success = self.assign_series(study_uid, series_data)
                if series_idx % 10 == 0 or series_idx == len(series_list):
                    print(f"    [{series_idx}/{len(series_list)}] Series 할당 중... ({len(self.assigned_series)}개 성공, {len(self.failed_series)}개 실패)")
                
                # API 부하 방지를 위한 짧은 대기
                time.sleep(0.05)
            
            print(f"    ✅ {len(series_list)}개 Series 처리 완료")
        
        # 결과 출력
        print("\n" + "=" * 60)
        print("📊 할당 결과")
        print("=" * 60)
        print(f"✅ 할당된 Study: {len(self.assigned_studies)}개")
        print(f"✅ 할당된 Series: {len(self.assigned_series)}개")
        print(f"❌ 실패한 Series: {len(self.failed_series)}개")
        
        if self.failed_series:
            print("\n⚠️  실패한 Series 목록 (최대 10개):")
            for failed in self.failed_series[:10]:
                study_uid = failed.get('study_uid', 'Unknown')[:50]
                series_uid = failed.get('series_uid', 'Unknown')[:50]
                error = failed.get('error', 'Unknown error')[:100]
                print(f"  - Study: {study_uid}...")
                print(f"    Series: {series_uid}...")
                print(f"    Error: {error}")
        
        print("\n✅ 완료!")

def main():
    parser = argparse.ArgumentParser(description='프로젝트에 모든 DICOM 데이터 할당')
    parser.add_argument('--project-id', type=int, required=True, help='프로젝트 ID')
    parser.add_argument('--base-url', type=str, default='http://localhost:8080', help='API Base URL')
    parser.add_argument('--token', type=str, required=True, help='JWT 토큰')
    
    args = parser.parse_args()
    
    assigner = ProjectDataAssigner(args.base_url, args.token, args.project_id)
    assigner.run()

if __name__ == '__main__':
    main()
