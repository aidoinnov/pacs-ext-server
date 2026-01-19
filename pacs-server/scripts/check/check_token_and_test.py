#!/usr/bin/env python3
"""
토큰 확인 및 새 토큰 발급 후 API 테스트
"""
import requests
import json
import jwt
from datetime import datetime

BASE_URL = "http://localhost:8080"
OLD_TOKEN = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJ5X1EwUDd0MDhjcEZZeFNEUFdseGdKcGFUcWtsOFd0eUJYRGRGaVVUQXBJIn0.eyJleHAiOjE3NjY2NzQ2MDksImlhdCI6MTc2NjY3MjgwOSwianRpIjoiNjJhZDhmZWItNjRkZS00MjUyLThlOWItZGM4Nzg3YTNlNzA1IiwiaXNzIjoiaHR0cHM6Ly9rZXljbG9hay5wYWNzLmFpLWRvLmtyL3JlYWxtcy9kY200Y2hlIiwiYXVkIjpbImlhaWQtcGFjcy1jbGllbnQiLCJhY2NvdW50Il0sInN1YiI6ImY0ZTJlMzU1LTIxMDItNGZiNi04YzZmLTg4YzI3NDQzZjVkOCIsInR5cCI6IkJlYXJlciIsImF6cCI6ImlhaWQtcGFjcy1jbGllbnQiLCJzZXNzaW9uX3N0YXRlIjoiMzhiODZiNTYtYjJmOS00OTNjLWIwY2QtMDljMjAyYzViYTNkIiwiYWNyIjoiMSIsImFsbG93ZWQtb3JpZ2lucyI6WyIqIl0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJvZmZsaW5lX2FjY2VzcyIsImRlZmF1bHQtcm9sZXMtZGNtNGNoZSIsInVtYV9hdXRob3JpemF0aW9uIiwidXNlciJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoiUEFDUy1hdWRpZW5jZS1zZXJ2aWNlIHByb2ZpbGUgZW1haWwiLCJzaWQiOiIzOGI4NmI1Ni1iMmY5LTQ5M2MtYjBjZC0wOWMyMDJjNWJhM2QiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwicHJlZmVycmVkX3VzZXJuYW1lIjoiaWFpZC1wYWNzLWFkbWluIn0.SwUxz95PkFeOaWnYYXnRwiNg2_1cNB7fSwsABpUfo_zYt01jKO4abkVVMaDc9LtIEYeVtX0jlPD2kIErEqNaqHWXCQZvB2dSVUHtzm70J6fPMk1SnQncA6o029ipwx19j6GpLDnGXwnM9kcTYhBUQwyGKdo590O4Rh-ZtDme1_mLb73qlDdEUyiGhMnSFQVQRuv3653VWzq9HDf3EUbuR46TgEHlyXPs6yW6Np0w8g_ZipwQAwmhXv64JSmTnm9kl2QxeWSco4aqhx17GDWlmrJR_b0vjpcS93IsKEYQ_Yt4eXPxIH2arvXVs24rk2d-wbkd4NxnGjTKaEyDXYQJTg"

print("=" * 60)
print("🔍 토큰 확인 및 새 토큰 발급")
print("=" * 60)

# 1. 기존 토큰 확인
print("\n1️⃣ 기존 토큰 확인")
print("-" * 60)
try:
    # 토큰 디코딩 (검증 없이)
    decoded = jwt.decode(OLD_TOKEN, options={"verify_signature": False})
    exp = decoded.get('exp', 0)
    iat = decoded.get('iat', 0)
    exp_time = datetime.fromtimestamp(exp)
    iat_time = datetime.fromtimestamp(iat)
    now = datetime.now()
    
    print(f"발급 시간: {iat_time}")
    print(f"만료 시간: {exp_time}")
    print(f"현재 시간: {now}")
    
    if now > exp_time:
        print("❌ 토큰이 만료되었습니다!")
        print(f"   만료된 지: {now - exp_time}")
    else:
        print(f"✅ 토큰이 아직 유효합니다 (만료까지: {exp_time - now})")
    
    print(f"\n사용자: {decoded.get('preferred_username', 'N/A')}")
    print(f"Subject: {decoded.get('sub', 'N/A')}")
    
except Exception as e:
    print(f"⚠️  토큰 디코딩 실패: {e}")

# 2. 새 토큰 발급
print("\n2️⃣ 새 토큰 발급")
print("-" * 60)
try:
    login_resp = requests.post(f'{BASE_URL}/api/auth/login', json={
        'username': 'iaid-pacs-admin',
        'password': 'Qlalfqjsgh1!'
    }, timeout=10)
    
    if login_resp.status_code == 200:
        data = login_resp.json()
        new_token = data.get('token') or data.get('access_token')
        if new_token:
            print("✅ 새 토큰 발급 성공!")
            print(f"토큰: {new_token[:50]}...")
            
            # 토큰 정보 확인
            try:
                decoded = jwt.decode(new_token, options={"verify_signature": False})
                exp = decoded.get('exp', 0)
                exp_time = datetime.fromtimestamp(exp)
                print(f"만료 시간: {exp_time}")
            except:
                pass
        else:
            print("❌ 토큰을 찾을 수 없습니다")
            print(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        print(f"❌ 로그인 실패: {login_resp.status_code}")
        print(f"응답: {login_resp.text[:200]}")
        new_token = None
        
except Exception as e:
    print(f"❌ 로그인 에러: {e}")
    new_token = None

# 3. 새 토큰으로 API 테스트
if new_token:
    print("\n3️⃣ 새 토큰으로 API 테스트")
    print("-" * 60)
    headers = {
        'Authorization': f'Bearer {new_token}',
        'Accept': 'application/json'
    }
    
    url = f"{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=200"
    
    try:
        response = requests.get(url, headers=headers, timeout=30)
        print(f"Status Code: {response.status_code}")
        
        if response.status_code == 200:
            try:
                data = response.json()
                if isinstance(data, list):
                    print(f"✅ Series 개수: {len(data)}")
                    if len(data) == 0:
                        print("❌ 빈 배열 반환")
                        print("\n문제 원인 확인 필요:")
                        print("1. 서버 로그 확인:")
                        print("   - '🔍 Gateway /series: Found {} allowed series UIDs'")
                        print("   - '🔍 Gateway /series: QIDO returned {} series'")
                        print("   - '🔍 Gateway /series: Filtered {} series'")
                        print("2. DB 직접 확인 (test_get_allowed_series_uids.sql)")
                    else:
                        print(f"\n첫 번째 Series:")
                        print(json.dumps(data[0], indent=2, ensure_ascii=False)[:500])
                elif isinstance(data, dict):
                    series_list = data.get('series', [])
                    print(f"✅ Series 개수: {len(series_list)}")
                    print(f"Total: {data.get('total', 0)}")
                else:
                    print(f"⚠️  예상치 못한 응답 형식: {type(data)}")
            except json.JSONDecodeError:
                print(f"❌ JSON 파싱 실패")
                print(f"응답: {response.text[:500]}")
        elif response.status_code == 401:
            print("❌ 401 Unauthorized - 토큰이 여전히 유효하지 않습니다")
            print(f"응답: {response.text[:200]}")
        else:
            print(f"❌ 에러 응답: {response.status_code}")
            print(f"응답: {response.text[:500]}")
            
    except Exception as e:
        print(f"❌ API 호출 에러: {e}")

print("\n" + "=" * 60)
print("✅ 완료")
print("=" * 60)

if new_token:
    print(f"\n새 토큰 (전체):")
    print(new_token)

