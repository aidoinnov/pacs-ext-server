#!/usr/bin/env python3
"""
Series 개수 문제 상세 분석
"""
import requests
import json

BASE_URL = "http://localhost:8080"

def get_token():
    """로그인하여 토큰 획득"""
    try:
        resp = requests.post(f'{BASE_URL}/api/auth/login', json={
            'username': 'iaid-pacs-admin',
            'password': 'Qlalfqjsgh1!'
        }, timeout=10)
        
        if resp.status_code == 200:
            data = resp.json()
            return data.get('token') or data.get('access_token')
    except Exception as e:
        print(f"❌ 로그인 에러: {e}")
    return None

def check_qido_directly(token):
    """QIDO를 직접 호출해서 전체 Series 확인"""
    print("=" * 60)
    print("🔍 QIDO 직접 호출 (필터링 없이)")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    # QIDO 직접 호출 (admin API 사용)
    url = f'{BASE_URL}/api/admin/dicom/series?limit=1000'
    print(f"\n📡 URL: {url}")
    print("-" * 60)
    
    try:
        resp = requests.get(url, headers=headers, timeout=60)
        print(f"Status Code: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                total_series = len(data)
                print(f"✅ QIDO 전체 Series 개수: {total_series}개")
                
                # Series UID 추출
                series_uids = []
                for series in data:
                    series_uid_tag = series.get('0020000E', {})
                    series_uid = series_uid_tag.get('Value', [None])[0] if isinstance(series_uid_tag, dict) else None
                    if series_uid:
                        series_uids.append(series_uid)
                
                print(f"   고유 Series UID: {len(set(series_uids))}개")
                return series_uids
            else:
                print(f"⚠️  예상치 못한 응답 형식: {type(data)}")
        elif resp.status_code == 403:
            print("❌ 403 Forbidden - admin 권한이 없습니다")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    return []

def compare_series_lists(all_series, filtered_series):
    """전체 Series와 필터링된 Series 비교"""
    print("\n" + "=" * 60)
    print("📊 Series 비교 분석")
    print("=" * 60)
    
    all_set = set(all_series)
    filtered_set = set(filtered_series)
    
    print(f"\n전체 Series: {len(all_set)}개")
    print(f"필터링된 Series (project_id=2): {len(filtered_set)}개")
    print(f"차이: {len(all_set) - len(filtered_set)}개")
    
    # 필터링에서 제외된 Series
    excluded = all_set - filtered_set
    if excluded:
        print(f"\n⚠️  필터링에서 제외된 Series ({len(excluded)}개):")
        for i, uid in enumerate(list(excluded)[:10], 1):
            print(f"   {i}. {uid}")
        if len(excluded) > 10:
            print(f"   ... 외 {len(excluded) - 10}개")
    
    # 필터링에 포함된 Series
    included = filtered_set & all_set
    if included:
        print(f"\n✅ 필터링에 포함된 Series ({len(included)}개):")
        for i, uid in enumerate(list(included)[:10], 1):
            print(f"   {i}. {uid}")
        if len(included) > 10:
            print(f"   ... 외 {len(included) - 10}개")

def check_project_data_api(token):
    """프로젝트 데이터 API로 확인"""
    print("\n" + "=" * 60)
    print("🔍 프로젝트 데이터 API 확인")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    # 프로젝트 Study 목록 확인
    url = f'{BASE_URL}/api/project-data/2/studies'
    print(f"\n📡 URL: {url}")
    print("-" * 60)
    
    try:
        resp = requests.get(url, headers=headers, timeout=30)
        print(f"Status Code: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, dict):
                studies = data.get('studies', [])
                print(f"✅ 프로젝트 Study 개수: {len(studies)}개")
                
                # 각 Study의 Series 개수 확인
                total_series = 0
                for study in studies:
                    study_id = study.get('id')
                    study_uid = study.get('study_uid', 'N/A')
                    
                    # Study의 Series 조회
                    series_url = f'{BASE_URL}/api/project-data/2/studies/{study_id}/series'
                    series_resp = requests.get(series_url, headers=headers, timeout=30)
                    if series_resp.status_code == 200:
                        series_data = series_resp.json()
                        if isinstance(series_data, dict):
                            series_list = series_data.get('series', [])
                            count = len(series_list)
                            total_series += count
                            print(f"   Study {study_uid[:50]}...: {count}개 Series")
                
                print(f"\n📊 프로젝트 데이터 API 총 Series: {total_series}개")
                return total_series
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    return 0

def main():
    print("=" * 60)
    print("🔍 Series 개수 문제 상세 분석")
    print("=" * 60)
    
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        return
    
    # 1. QIDO 전체 Series 확인
    all_series = check_qido_directly(token)
    
    # 2. 필터링된 Series 확인
    print("\n" + "=" * 60)
    print("🔍 필터링된 Series 확인 (project_id=2)")
    print("=" * 60)
    
    headers = {'Authorization': f'Bearer {token}'}
    url = f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=1000'
    
    try:
        resp = requests.get(url, headers=headers, timeout=60)
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                filtered_series_list = data
            else:
                filtered_series_list = data.get('series', [])
            
            filtered_series_uids = []
            for series in filtered_series_list:
                series_uid_tag = series.get('0020000E', {})
                series_uid = series_uid_tag.get('Value', [None])[0] if isinstance(series_uid_tag, dict) else None
                if series_uid:
                    filtered_series_uids.append(series_uid)
            
            print(f"✅ 필터링된 Series: {len(filtered_series_uids)}개")
    except Exception as e:
        print(f"❌ 에러: {e}")
        filtered_series_uids = []
    
    # 3. 비교 분석
    if all_series and filtered_series_uids:
        compare_series_lists(all_series, filtered_series_uids)
    
    # 4. 프로젝트 데이터 API 확인
    project_series_count = check_project_data_api(token)
    
    # 5. 요약
    print("\n" + "=" * 60)
    print("📊 분석 결과 요약")
    print("=" * 60)
    print(f"QIDO 전체 Series: {len(set(all_series)) if all_series else 'N/A'}개")
    print(f"필터링된 Series (project_id=2): {len(set(filtered_series_uids)) if filtered_series_uids else 'N/A'}개")
    print(f"프로젝트 데이터 API Series: {project_series_count}개")
    
    if filtered_series_uids and len(set(filtered_series_uids)) == 11:
        print("\n💡 결론:")
        print("   - project_id=2에 실제로 11개 Series만 할당되어 있습니다")
        print("   - 이전에 28개였던 것은 다른 데이터였거나")
        print("   - 데이터가 삭제/이동되었을 가능성이 있습니다")
        print("\n💡 해결 방법:")
        print("   - 모든 데이터를 다시 할당하려면:")
        print("     python3 assign_all_data_from_db.py")

if __name__ == '__main__':
    main()

