#!/usr/bin/env python3
"""View Selection 빠른 테스트"""

import requests
import json

BASE_URL = 'http://localhost:8080'

print("="*60)
print("🚀 View Selection API 빠른 테스트")
print("="*60)

# 1. Health Check
print("\n1️⃣ Health Check...")
try:
    resp = requests.get(f'{BASE_URL}/health', timeout=5)
    print(f"   Status: {resp.status_code}")
    if resp.status_code == 200:
        print(f"   ✅ 서버 정상: {resp.json()}")
    else:
        print(f"   ❌ 서버 오류")
        exit(1)
except Exception as e:
    print(f"   ❌ 에러: {e}")
    exit(1)

# 2. Test Token 얻기
print("\n2️⃣ Test Token 획득...")
try:
    token_resp = requests.post(
        f'{BASE_URL}/api/auth/test-token',
        json={'user_id': 1, 'keycloak_id': 'test-user-1'},
        headers={'Content-Type': 'application/json'},
        timeout=10
    )
    print(f"   Status: {token_resp.status_code}")
    
    if token_resp.status_code != 200:
        print(f"   Response: {token_resp.text[:200]}")
        print("   ⚠️  Token 획득 실패, 인증 없이 테스트 진행")
        token = None
    else:
        token_data = token_resp.json()
        token = token_data.get('token') or token_data.get('access_token', '')
        if token:
            print(f"   ✅ Token 획득: {token[:50]}...")
        else:
            print(f"   ❌ Token 없음: {token_data}")
            token = None
except Exception as e:
    print(f"   ⚠️  Token API 에러: {e}")
    token = None

# 3. View Selection 생성
print("\n3️⃣ View Selection 생성...")
headers = {'Content-Type': 'application/json'}
if token:
    headers['Authorization'] = f'Bearer {token}'

try:
    create_resp = requests.post(
        f'{BASE_URL}/api/v1/view-selections',
        json={
            'series': [
                {'study_uid': '1.2.840.113619.2.1.1.123', 'series_uid': '1.2.840.113619.2.1.2.124'},
                {'study_uid': '1.2.840.113619.2.1.1.125', 'series_uid': '1.2.840.113619.2.1.2.126'}
            ]
        },
        headers=headers,
        timeout=10
    )
    print(f"   Status: {create_resp.status_code}")
    print(f"   Response: {create_resp.text[:300]}")
    
    if create_resp.status_code == 201:
        result = create_resp.json()
        selection_id = result.get('selection_id')
        if selection_id:
            print(f"   ✅ Selection 생성 성공: {selection_id}")
            
            # 4. Selection 조회
            print(f"\n4️⃣ Selection 조회: {selection_id}")
            get_resp = requests.get(
                f'{BASE_URL}/api/v1/view-selections/{selection_id}',
                headers=headers,
                timeout=10
            )
            print(f"   Status: {get_resp.status_code}")
            print(f"   Response: {get_resp.text[:300]}")
            
            if get_resp.status_code == 200:
                get_result = get_resp.json()
                print(f"   ✅ Selection 조회 성공")
                print(f"   - Selection ID: {get_result.get('selection_id')}")
                print(f"   - Series 수: {len(get_result.get('series', []))}")
                print(f"   - User ID: {get_result.get('user_id')}")
                
                # 5. Selection 삭제
                print(f"\n5️⃣ Selection 삭제: {selection_id}")
                delete_resp = requests.delete(
                    f'{BASE_URL}/api/v1/view-selections/{selection_id}',
                    headers=headers,
                    timeout=10
                )
                print(f"   Status: {delete_resp.status_code}")
                if delete_resp.status_code == 204:
                    print(f"   ✅ Selection 삭제 성공")
                else:
                    print(f"   Response: {delete_resp.text[:200]}")
            else:
                print(f"   ❌ Selection 조회 실패")
        else:
            print(f"   ❌ Selection ID 없음")
    elif create_resp.status_code == 401:
        print(f"   ⚠️  인증 필요 (401)")
    elif create_resp.status_code == 404:
        print(f"   ⚠️  API 엔드포인트 없음 (404) - 서버 재시작 필요")
    else:
        print(f"   ❌ 생성 실패")
        
except Exception as e:
    print(f"   ❌ 에러: {e}")

print("\n" + "="*60)
print("✅ 테스트 완료")
print("="*60)


