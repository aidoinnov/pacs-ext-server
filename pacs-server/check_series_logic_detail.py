#!/usr/bin/env python3
"""
Series API 로직 상세 확인
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

def check_series_with_details(token, project_id=2):
    """Series API 상세 확인"""
    print("=" * 60)
    print(f"🔍 Series API 상세 분석 (project_id={project_id})")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    # 여러 페이지 확인
    for page in [1, 2, 3]:
        url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page={page}&page_size=100'
        print(f"\n📡 Page {page}: {url}")
        print("-" * 60)
        
        try:
            resp = requests.get(url, headers=headers, timeout=30)
            print(f"Status Code: {resp.status_code}")
            
            if resp.status_code == 200:
                data = resp.json()
                
                if isinstance(data, list):
                    count = len(data)
                    print(f"   Series 개수: {count}")
                    
                    if count > 0:
                        # 첫 번째와 마지막 Series 정보
                        first = data[0]
                        last = data[-1]
                        print(f"   첫 번째 Series UID: {first.get('0020000E', {}).get('Value', ['N/A'])[0]}")
                        print(f"   마지막 Series UID: {last.get('0020000E', {}).get('Value', ['N/A'])[0]}")
                        
                        # Study UID별 그룹화
                        study_groups = {}
                        for series in data:
                            study_uid_tag = series.get('0020000D', {})
                            study_uid = study_uid_tag.get('Value', [None])[0] if isinstance(study_uid_tag, dict) else None
                            if study_uid:
                                if study_uid not in study_groups:
                                    study_groups[study_uid] = []
                                series_uid_tag = series.get('0020000E', {})
                                series_uid = series_uid_tag.get('Value', [None])[0] if isinstance(series_uid_tag, dict) else None
                                if series_uid:
                                    study_groups[study_uid].append(series_uid)
                        
                        print(f"   Study 개수: {len(study_groups)}")
                        for study_uid, series_list in list(study_groups.items())[:5]:
                            print(f"     Study {study_uid[:50]}...: {len(series_list)}개 Series")
                    else:
                        print("   ⚠️  Series 목록이 비어있습니다")
                        break
                else:
                    print(f"   ⚠️  예상치 못한 응답 형식: {type(data)}")
            else:
                print(f"   ❌ 에러: {resp.status_code}")
                print(f"   {resp.text[:200]}")
                break
        except Exception as e:
            print(f"   ❌ 요청 에러: {e}")
            break

def check_different_project_ids(token):
    """다른 project_id로 확인"""
    print("\n" + "=" * 60)
    print("🔍 다른 project_id로 확인")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    for project_id in [1, 2, 3]:
        url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page=1&page_size=10'
        print(f"\n📡 project_id={project_id}")
        print("-" * 60)
        
        try:
            resp = requests.get(url, headers=headers, timeout=30)
            if resp.status_code == 200:
                data = resp.json()
                if isinstance(data, list):
                    count = len(data)
                    print(f"   Series 개수: {count}")
                elif isinstance(data, dict):
                    series_list = data.get('series', [])
                    count = len(series_list)
                    print(f"   Series 개수: {count}")
            else:
                print(f"   Status: {resp.status_code}")
        except Exception as e:
            print(f"   에러: {e}")

def main():
    print("=" * 60)
    print("🔍 Series API 상세 확인")
    print("=" * 60)
    
    # 1. 토큰 획득
    print("\n1️⃣ 토큰 획득")
    print("-" * 60)
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        return
    print("✅ 토큰 획득 성공")
    
    # 2. project_id=2 상세 확인
    print("\n2️⃣ project_id=2 상세 확인")
    print("-" * 60)
    check_series_with_details(token, project_id=2)
    
    # 3. 다른 project_id 확인
    print("\n3️⃣ 다른 project_id 확인")
    print("-" * 60)
    check_different_project_ids(token)
    
    print("\n" + "=" * 60)
    print("✅ 확인 완료")
    print("=" * 60)
    print("\n💡 서버 로그를 확인하여 다음을 확인하세요:")
    print("   1. 'get_allowed_series_uids' 쿼리 결과")
    print("   2. QIDO 응답 Series 개수")
    print("   3. 필터링 후 Series 개수")
    print("   4. RBAC 평가 결과")

if __name__ == '__main__':
    main()

