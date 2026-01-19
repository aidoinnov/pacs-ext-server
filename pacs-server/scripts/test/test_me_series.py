#!/usr/bin/env python3
"""
/api/me/dicom/series 엔드포인트 확인
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

def extract_series_uid(series):
    """Series UID 추출"""
    series_uid_tag = series.get('0020000E', {})
    if isinstance(series_uid_tag, dict):
        value = series_uid_tag.get('Value', [])
        if isinstance(value, list) and len(value) > 0:
            return str(value[0])
    return None

def main():
    print("=" * 60)
    print("🔍 /api/me/dicom/series 엔드포인트 확인")
    print("=" * 60)
    
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        return
    
    headers = {'Authorization': f'Bearer {token}'}
    project_id = 2
    
    # 1. 기본 요청 (page, page_size 없이)
    print("\n1️⃣ 기본 요청 (page, page_size 없이)")
    print("-" * 60)
    url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}'
    resp = requests.get(url, headers=headers, timeout=60)
    
    print(f"URL: {url}")
    print(f"Status: {resp.status_code}")
    if resp.status_code == 200:
        data = resp.json()
        if isinstance(data, list):
            series_list = data
        else:
            series_list = data.get('series', [])
        
        print(f"Series 개수: {len(series_list)}")
        
        if len(series_list) > 0:
            print(f"\n처음 5개 Series UID:")
            for i, series in enumerate(series_list[:5], 1):
                uid = extract_series_uid(series)
                print(f"   {i}. {uid}")
        else:
            print("⚠️  Series가 0개입니다!")
    else:
        print(f"❌ 에러: {resp.status_code}")
        print(f"   {resp.text[:200]}")
    
    # 2. page, page_size 포함
    print("\n2️⃣ page, page_size 포함")
    print("-" * 60)
    url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page=1&page_size=100'
    resp = requests.get(url, headers=headers, timeout=60)
    
    print(f"URL: {url}")
    print(f"Status: {resp.status_code}")
    if resp.status_code == 200:
        data = resp.json()
        if isinstance(data, list):
            series_list = data
        else:
            series_list = data.get('series', [])
        
        print(f"Series 개수: {len(series_list)}")
        
        if len(series_list) > 0:
            print(f"\n처음 5개 Series UID:")
            for i, series in enumerate(series_list[:5], 1):
                uid = extract_series_uid(series)
                print(f"   {i}. {uid}")
        else:
            print("⚠️  Series가 0개입니다!")
    else:
        print(f"❌ 에러: {resp.status_code}")
        print(f"   {resp.text[:200]}")
    
    # 3. DB에서 확인한 Series UID와 비교
    print("\n3️⃣ DB에서 확인한 Series UID (참고용)")
    print("-" * 60)
    db_series_uids = [
        "1.2.840.113619.2.311.168624790352053237183428645578553404611",
        "1.2.840.113619.2.495.11554579.117236.29274.1645718974.446",
        "1.3.12.2.1107.5.1.4.73676.30000020120700101330300041752",
        "1.3.12.2.1107.5.1.4.73676.30000020120700101330300041811",
        "1.3.12.2.1107.5.1.4.73676.30000020120700101330300041870"
    ]
    
    for i, uid in enumerate(db_series_uids, 1):
        print(f"   {i}. {uid}")

if __name__ == '__main__':
    main()

