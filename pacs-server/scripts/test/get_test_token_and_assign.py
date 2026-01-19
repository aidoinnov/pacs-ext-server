#!/usr/bin/env python3
"""
테스트 토큰 생성 후 프로젝트에 모든 데이터 할당
"""
import requests
import json
import sys
import time
from typing import List, Dict, Optional

BASE_URL = "http://localhost:8080"
PROJECT_ID = 2
USER_ID = 1

def get_login_token():
    """로그인 API를 통해 토큰 획득"""
    url = f"{BASE_URL}/api/auth/login"
    data = {
        "username": "iaid-pacs-admin",
        "password": "Qlalfqjsgh1!"
    }
    
    try:
        print("🔐 로그인 중...")
        response = requests.post(url, json=data, timeout=30)
        
        if response.status_code == 200:
            result = response.json()
            # LoginResponse에는 token 필드가 있음
            token = result.get('token') or result.get('access_token')
            if token:
                print(f"✅ 로그인 성공!")
                print(f"   User ID: {result.get('user_id', 'N/A')}")
                print(f"   Username: {result.get('username', 'N/A')}")
                return token
            else:
                print(f"❌ 토큰을 찾을 수 없습니다")
                print(json.dumps(result, indent=2))
                return None
        else:
            print(f"❌ 로그인 실패: {response.status_code}")
            print(f"Response: {response.text}")
            return None
    except Exception as e:
        print(f"❌ 에러: {e}")
        if hasattr(e, 'response') and e.response:
            print(f"   Response: {e.response.text[:200]}")
        return None

def get_all_studies_from_series(token: str, limit: int = 10000) -> List[Dict]:
    """Series API를 통해 모든 Study 조회 (Study UID 추출)"""
    print(f"📋 모든 Series 조회 중... (limit={limit}, project_id={PROJECT_ID})")
    
    # /api/me/dicom/series 사용 (사용자 관점 API)
    url = f"{BASE_URL}/api/me/dicom/series"
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    params = {
        "project_id": PROJECT_ID,
        "page": 1,
        "page_size": min(limit, 200)  # 최대 200
    }
    
    all_series = []
    page = 1
    
    try:
        while True:
            params["page"] = page
            response = requests.get(url, headers=headers, params=params, timeout=60)
            response.raise_for_status()
            result = response.json()
            
            # 응답 형식 확인
            if isinstance(result, dict) and "data" in result:
                series_list = result["data"]
            elif isinstance(result, list):
                series_list = result
            else:
                print(f"⚠️  예상치 못한 응답 형식: {type(result)}")
                break
            
            if not series_list or len(series_list) == 0:
                break
                
            all_series.extend(series_list)
            print(f"   페이지 {page}: {len(series_list)}개 Series 수집 (총 {len(all_series)}개)")
            
            if len(series_list) < params["page_size"]:
                break
                
            page += 1
            if len(all_series) >= limit:
                break
        
        # Series에서 Study UID 추출 (중복 제거)
        study_uids = set()
        for series in all_series:
            study_uid = extract_study_uid_from_series(series)
            if study_uid:
                study_uids.add(study_uid)
        
        print(f"✅ {len(all_series)}개 Series에서 {len(study_uids)}개 Study 발견")
        
        # Study 목록 생성 (간단한 형식)
        studies = []
        for study_uid in study_uids:
            studies.append({
                "0020000D": {
                    "vr": "UI",
                    "Value": [study_uid]
                }
            })
        
        return studies
    except requests.exceptions.RequestException as e:
        print(f"❌ Series 조회 실패: {e}")
        if hasattr(e, 'response') and e.response:
            print(f"   Response: {e.response.text[:200]}")
        return []

def extract_study_uid_from_series(series_data: Dict) -> Optional[str]:
    """Series 데이터에서 Study UID 추출"""
    # 여러 가능한 형식 확인
    if "0020000D" in series_data:
        tag_data = series_data["0020000D"]
        if isinstance(tag_data, dict) and "Value" in tag_data:
            value = tag_data["Value"]
            if isinstance(value, list) and len(value) > 0:
                return str(value[0])
    # 직접 study_uid 필드 확인
    if "study_uid" in series_data:
        return str(series_data["study_uid"])
    if "studyInstanceUID" in series_data:
        return str(series_data["studyInstanceUID"])
    return None

def get_study_series(token: str, study_uid: str, limit: int = 1000) -> List[Dict]:
    """특정 Study의 모든 Series 조회 (사용자 관점 API 사용)"""
    url = f"{BASE_URL}/api/me/dicom/studies/{study_uid}/series"
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    params = {
        "project_id": PROJECT_ID,
        "page": 1,
        "page_size": min(limit, 200)
    }
    
    all_series = []
    page = 1
    
    try:
        while True:
            params["page"] = page
            response = requests.get(url, headers=headers, params=params, timeout=30)
            response.raise_for_status()
            result = response.json()
            
            if isinstance(result, dict) and "data" in result:
                series_list = result["data"]
            elif isinstance(result, list):
                series_list = result
            else:
                break
            
            if not series_list or len(series_list) == 0:
                break
                
            all_series.extend(series_list)
            
            if len(series_list) < params["page_size"]:
                break
                
            page += 1
            if len(all_series) >= limit:
                break
        
        return all_series
    except requests.exceptions.RequestException as e:
        print(f"  ⚠️  Study {study_uid[:50]}...의 Series 조회 실패: {e}")
        if hasattr(e, 'response') and e.response:
            print(f"     Response: {e.response.text[:100]}")
        return []

def extract_study_uid(study_data: Dict) -> Optional[str]:
    """Study 데이터에서 Study UID 추출 (DICOM 태그 형식)"""
    if "0020000D" in study_data:
        tag_data = study_data["0020000D"]
        if isinstance(tag_data, dict) and "Value" in tag_data:
            value = tag_data["Value"]
            if isinstance(value, list) and len(value) > 0:
                return str(value[0])
    return None

def extract_tag_value(series_data: Dict, tag: str, default=""):
    """DICOM 태그에서 값 추출"""
    if tag in series_data:
        tag_data = series_data[tag]
        if isinstance(tag_data, dict) and "Value" in tag_data:
            value = tag_data["Value"]
            if isinstance(value, list) and len(value) > 0:
                return str(value[0])
    return default

def assign_series(token: str, study_uid: str, series_data: Dict, assigned_series: set, failed_series: list) -> bool:
    """Series를 프로젝트에 할당"""
    series_uid = extract_tag_value(series_data, "0020000E")
    
    if not series_uid:
        return False
        
    if series_uid in assigned_series:
        return True
    
    url = f"{BASE_URL}/api/projects/{PROJECT_ID}/series/assign"
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    
    payload = {
        "study_uid": study_uid,
        "series_uid": series_uid,
        "series_description": extract_tag_value(series_data, "0008103E", ""),
        "modality": extract_tag_value(series_data, "00080060", ""),
        "series_number": None
    }
    
    # Series Number 추출
    series_num_str = extract_tag_value(series_data, "00200011", "")
    if series_num_str:
        try:
            payload["series_number"] = int(series_num_str)
        except (ValueError, TypeError):
            pass
    
    try:
        response = requests.post(url, json=payload, headers=headers, timeout=30)
        if response.status_code in [200, 201]:
            assigned_series.add(series_uid)
            return True
        elif response.status_code == 409:
            # 이미 할당됨
            assigned_series.add(series_uid)
            return True
        else:
            error_msg = response.text[:200] if response.text else "Unknown error"
            failed_series.append({
                "study_uid": study_uid,
                "series_uid": series_uid,
                "error": f"{response.status_code}: {error_msg}"
            })
            return False
    except requests.exceptions.RequestException as e:
        failed_series.append({
            "study_uid": study_uid,
            "series_uid": series_uid,
            "error": str(e)
        })
        return False

def main():
    print("=" * 60)
    print("🚀 프로젝트에 모든 데이터 할당 시작")
    print("=" * 60)
    print(f"프로젝트 ID: {PROJECT_ID}")
    print(f"Base URL: {BASE_URL}")
    print()
    
    # 1. 로그인하여 토큰 획득
    token = get_login_token()
    if not token:
        print("❌ 로그인 실패. 종료합니다.")
        sys.exit(1)
    
    print()
    
    # 2. 모든 Study 조회 (Series API를 통해)
    studies = get_all_studies_from_series(token, limit=10000)
    
    if not studies:
        print("❌ Study가 없습니다. 종료합니다.")
        sys.exit(1)
    
    print()
    
    # 3. 각 Study와 Series 할당
    assigned_series = set()
    failed_series = []
    total_studies = len(studies)
    total_series = 0
    
    for idx, study_data in enumerate(studies, 1):
        study_uid = extract_study_uid(study_data)
        if not study_uid:
            print(f"  [{idx}/{total_studies}] Study UID를 찾을 수 없음, 건너뜀")
            continue
        
        print(f"  [{idx}/{total_studies}] Study: {study_uid[:50]}...")
        
        # Study의 모든 Series 조회
        series_list = get_study_series(token, study_uid, limit=1000)
        total_series += len(series_list)
        
        if not series_list:
            print(f"    ⚠️  Series 없음")
            continue
        
        # 각 Series 할당
        for series_idx, series_data in enumerate(series_list, 1):
            success = assign_series(token, study_uid, series_data, assigned_series, failed_series)
            if series_idx % 10 == 0 or series_idx == len(series_list):
                print(f"    [{series_idx}/{len(series_list)}] Series 할당 중... ({len(assigned_series)}개 성공, {len(failed_series)}개 실패)")
            
            # API 부하 방지를 위한 짧은 대기
            time.sleep(0.05)
        
        print(f"    ✅ {len(series_list)}개 Series 처리 완료")
        print()
    
    # 결과 출력
    print("=" * 60)
    print("📊 할당 결과")
    print("=" * 60)
    print(f"✅ 할당된 Series: {len(assigned_series)}개")
    print(f"❌ 실패한 Series: {len(failed_series)}개")
    print(f"📋 총 처리한 Study: {total_studies}개")
    print(f"📋 총 처리한 Series: {total_series}개")
    
    if failed_series:
        print("\n⚠️  실패한 Series 목록 (최대 10개):")
        for failed in failed_series[:10]:
            study_uid = failed.get('study_uid', 'Unknown')[:50]
            series_uid = failed.get('series_uid', 'Unknown')[:50]
            error = failed.get('error', 'Unknown error')[:100]
            print(f"  - Study: {study_uid}...")
            print(f"    Series: {series_uid}...")
            print(f"    Error: {error}")
    
    print("\n✅ 완료!")

if __name__ == '__main__':
    main()

