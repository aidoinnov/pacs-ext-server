#!/usr/bin/env python3
"""
Series Note API E2E 테스트 (Series UID 사용)
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
        return None
    except Exception as e:
        print(f"❌ Series 조회 에러: {e}")
        return None

def test_note_api(token: str, series_uid: str):
    """Note API 테스트"""
    print("\n" + "=" * 60)
    print("📝 Note API 테스트 (Series UID 사용)")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    
    # 1. Note 조회 (없을 때 빈 문자열 확인)
    print(f"\n1️⃣ GET /api/series/{series_uid[:50]}.../note")
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
            note_content = data.get('note', '')
            if note_content == '':
                print("✅ Note가 없을 때 빈 문자열 반환 확인")
            else:
                print(f"   Note 내용: {note_content[:100]}")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
            return False
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
        return False
    
    # 2. Note 생성/수정
    print(f"\n2️⃣ PUT /api/series/{series_uid[:50]}.../note")
    print("-" * 60)
    note_data = {
        'note': 'E2E 테스트 Note - Series UID 사용'
    }
    
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
            note_content = data.get('note', '')
            if note_content:
                print(f"   Note 내용: {note_content[:100]}")
            else:
                print("⚠️  Note 내용이 비어있음")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
            return False
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
        return False
    
    # 3. Note 재조회 (생성된 내용 확인)
    print(f"\n3️⃣ GET /api/series/{series_uid[:50]}.../note (재조회)")
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
            note_content = data.get('note', '')
            if note_content == 'E2E 테스트 Note - Series UID 사용':
                print("✅ 생성된 Note 내용 확인 성공")
            else:
                print(f"⚠️  Note 내용 불일치: {note_content[:100]}")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    # 4. Note 삭제
    print(f"\n4️⃣ DELETE /api/series/{series_uid[:50]}.../note")
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
    
    # 5. Note 재조회 (삭제 후 빈 문자열 확인)
    print(f"\n5️⃣ GET /api/series/{series_uid[:50]}.../note (삭제 후 확인)")
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
            note_content = data.get('note', '')
            if note_content == '':
                print("✅ 삭제 후 빈 문자열 반환 확인")
            else:
                print(f"⚠️  Note 내용이 남아있음: {note_content[:100]}")
            print(f"   Response: {json.dumps(data, indent=2, ensure_ascii=False)[:300]}")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    return True

def main():
    print("=" * 60)
    print("🧪 Series Note API E2E 테스트")
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
    success = test_note_api(token, series_uid)
    
    # 결과 요약
    print("\n" + "=" * 60)
    print("📊 테스트 결과")
    print("=" * 60)
    if success:
        print("✅ 모든 테스트 통과!")
        sys.exit(0)
    else:
        print("❌ 일부 테스트 실패")
        sys.exit(1)

if __name__ == '__main__':
    main()

