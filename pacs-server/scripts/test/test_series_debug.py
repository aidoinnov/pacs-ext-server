#!/usr/bin/env python3
"""
Series API 디버깅 - QIDO 응답과 필터링 확인
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
    print("🔍 Series API 디버깅")
    print("=" * 60)
    
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        return
    
    headers = {'Authorization': f'Bearer {token}'}
    project_id = 2
    
    # 1. API 응답 확인
    print("\n1️⃣ API 응답 확인")
    print("-" * 60)
    url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page=1&page_size=1000'
    resp = requests.get(url, headers=headers, timeout=60)
    
    print(f"Status: {resp.status_code}")
    if resp.status_code == 200:
        data = resp.json()
        if isinstance(data, list):
            series_list = data
        else:
            series_list = data.get('series', [])
        
        print(f"Series 개수: {len(series_list)}")
        
        if len(series_list) == 0:
            print("\n⚠️  Series가 0개입니다!")
            print("\n💡 가능한 원인:")
            print("   1. QIDO에서 Series를 반환하지 않음")
            print("   2. allowed_series_uids가 비어있음")
            print("   3. QIDO 응답의 Series UID와 DB의 Series UID가 매칭되지 않음")
            print("\n💡 확인 방법:")
            print("   - 서버 로그에서 다음 메시지 확인:")
            print("     'Found X allowed series UIDs for project 2'")
            print("     'QIDO returned X series'")
            print("     'Filtered X series from Y QIDO results'")
        else:
            print(f"\nSeries UID 목록:")
            for i, series in enumerate(series_list[:10], 1):
                uid = extract_series_uid(series)
                print(f"   {i}. {uid}")
    
    # 2. DB에서 직접 확인한 Series UID와 비교
    print("\n2️⃣ DB에서 확인한 Series UID")
    print("-" * 60)
    print("   (이전 테스트에서 확인한 5개 Series UID)")
    db_series_uids = [
        "1.2.840.113619.2.311.168624790352053237183428645578553404611",
        "1.2.840.113619.2.495.11554579.117236.29274.1645718974.446",
        "1.3.12.2.1107.5.1.4.73676.30000020120700101330300041752",
        "1.3.12.2.1107.5.1.4.73676.30000020120700101330300041811",
        "1.3.12.2.1107.5.1.4.73676.30000020120700101330300041870"
    ]
    
    for i, uid in enumerate(db_series_uids, 1):
        print(f"   {i}. {uid}")
    
    print("\n💡 문제 분석:")
    print("   - DB에는 5개 Series가 있음")
    print("   - API는 0개를 반환함")
    print("   - 이는 QIDO에서 해당 Series를 반환하지 않았거나")
    print("   - QIDO 응답의 Series UID와 DB의 Series UID가 매칭되지 않았을 가능성이 높음")
    print("\n💡 해결 방법:")
    print("   1. 서버 로그 확인 (특히 QIDO 응답과 필터링 로그)")
    print("   2. QIDO에서 실제로 해당 Series를 반환하는지 확인")
    print("   3. Series UID 형식이 일치하는지 확인")

if __name__ == '__main__':
    main()

