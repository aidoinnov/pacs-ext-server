#!/usr/bin/env python3
"""
Series UID 기반 API E2E 테스트
"""
import requests
import json
import sys
from typing import Optional

BASE_URL = "http://localhost:8080"

def get_token() -> Optional[str]:
    """로그인하여 토큰 획득"""
    try:
        resp = requests.post(f'{BASE_URL}/api/auth/login', json={
            'username': 'iaid-pacs-admin',
            'password': 'Qlalfqjsgh1!'
        }, timeout=10)
        
        if resp.status_code == 200:
            data = resp.json()
            return data.get('token') or data.get('access_token')
        else:
            print(f"❌ 로그인 실패: {resp.status_code}")
            print(f"   {resp.text[:200]}")
            return None
    except Exception as e:
        print(f"❌ 로그인 에러: {e}")
        return None

def get_series_uid(token: str) -> Optional[str]:
    """Series 목록에서 첫 번째 Series UID 가져오기"""
    try:
        headers = {'Authorization': f'Bearer {token}'}
        resp = requests.get(
            f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=1',
            headers=headers,
            timeout=30
        )
        
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list) and len(data) > 0:
                series_uid = data[0].get('0020000E', {}).get('Value', [None])[0]
                return series_uid
            elif isinstance(data, dict):
                series_list = data.get('series', [])
                if series_list:
                    series_uid = series_list[0].get('0020000E', {}).get('Value', [None])[0]
                    return series_uid
        else:
            print(f"❌ Series 조회 실패: {resp.status_code}")
            print(f"   {resp.text[:200]}")
            return None
    except Exception as e:
        print(f"❌ Series 조회 에러: {e}")
        return None

def test_note_api_with_series_uid(token: str, series_uid: str):
    """Series UID로 Note API 테스트"""
    print("\n" + "=" * 60)
    print("📝 Note API 테스트 (Series UID 사용)")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    
    # 테스트용 Note 데이터
    note_data = {
        'content': 'E2E 테스트 Note - Series UID 사용',
        'tags': ['test', 'e2e']
    }
    
    # 1. Note 생성/수정
    print(f"\n1️⃣ PUT /api/series/{series_uid[:50]}.../note")
    print("-" * 60)
    try:
        resp = requests.put(
            f'{BASE_URL}/api/series/{series_uid}/note',
            headers=headers,
            json=note_data,
            timeout=30
        )
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            print("✅ Note 생성/수정 성공")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        elif resp.status_code == 400:
            error_text = resp.text[:200]
            if "can not parse" in error_text.lower() or "i32" in error_text.lower():
                print(f"❌ 여전히 i32 파싱 에러 발생!")
                print(f"   {error_text}")
                return False
            else:
                print(f"⚠️  Bad Request: {error_text}")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
            return False
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
        return False
    
    # 2. Note 조회
    print(f"\n2️⃣ GET /api/series/{series_uid[:50]}.../note")
    print("-" * 60)
    try:
        resp = requests.get(
            f'{BASE_URL}/api/series/{series_uid}/note',
            headers=headers,
            timeout=30
        )
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            print("✅ Note 조회 성공")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        elif resp.status_code == 404:
            print("⚠️  Note가 없음 (정상일 수 있음)")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    # 3. Note 삭제
    print(f"\n3️⃣ DELETE /api/series/{series_uid[:50]}.../note")
    print("-" * 60)
    try:
        resp = requests.delete(
            f'{BASE_URL}/api/series/{series_uid}/note',
            headers=headers,
            timeout=30
        )
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 200:
            print("✅ Note 삭제 성공")
        elif resp.status_code == 404:
            print("⚠️  Note가 없음 (정상일 수 있음)")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    return True

def test_report_api_with_series_uid(token: str, series_uid: str):
    """Series UID로 Report API 테스트"""
    print("\n" + "=" * 60)
    print("📄 Report API 테스트 (Series UID 사용)")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    
    # 테스트용 Report 데이터
    report_data = {
        'status': 'draft',
        'content': 'E2E 테스트 Report - Series UID 사용',
        'findings': '테스트 findings'
    }
    
    # 1. Report 생성/수정
    print(f"\n1️⃣ PUT /api/series/{series_uid[:50]}.../report")
    print("-" * 60)
    try:
        resp = requests.put(
            f'{BASE_URL}/api/series/{series_uid}/report',
            headers=headers,
            json=report_data,
            timeout=30
        )
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            print("✅ Report 생성/수정 성공")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        elif resp.status_code == 400:
            error_text = resp.text[:200]
            if "can not parse" in error_text.lower() or "i32" in error_text.lower():
                print(f"❌ 여전히 i32 파싱 에러 발생!")
                print(f"   {error_text}")
                return False
            else:
                print(f"⚠️  Bad Request: {error_text}")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
            return False
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
        return False
    
    # 2. Report 조회
    print(f"\n2️⃣ GET /api/series/{series_uid[:50]}.../report")
    print("-" * 60)
    try:
        resp = requests.get(
            f'{BASE_URL}/api/series/{series_uid}/report',
            headers=headers,
            timeout=30
        )
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            print("✅ Report 조회 성공")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        elif resp.status_code == 404:
            print("⚠️  Report가 없음 (정상일 수 있음)")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    return True

def main():
    print("=" * 60)
    print("🧪 Series UID 기반 API E2E 테스트")
    print("=" * 60)
    
    # 1. 로그인
    print("\n1️⃣ 로그인")
    print("-" * 60)
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패. 종료합니다.")
        sys.exit(1)
    print("✅ 토큰 획득 성공")
    
    # 2. Series UID 가져오기
    print("\n2️⃣ Series UID 가져오기")
    print("-" * 60)
    series_uid = get_series_uid(token)
    if not series_uid:
        print("❌ Series UID 획득 실패. 종료합니다.")
        sys.exit(1)
    print(f"✅ Series UID: {series_uid}")
    
    # 3. Note API 테스트
    note_success = test_note_api_with_series_uid(token, series_uid)
    
    # 4. Report API 테스트
    report_success = test_report_api_with_series_uid(token, series_uid)
    
    # 결과 요약
    print("\n" + "=" * 60)
    print("📊 테스트 결과 요약")
    print("=" * 60)
    print(f"Note API: {'✅ 성공' if note_success else '❌ 실패'}")
    print(f"Report API: {'✅ 성공' if report_success else '❌ 실패'}")
    
    if note_success and report_success:
        print("\n✅ 모든 테스트 통과!")
        sys.exit(0)
    else:
        print("\n❌ 일부 테스트 실패")
        sys.exit(1)

if __name__ == '__main__':
    main()

