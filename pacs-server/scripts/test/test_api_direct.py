#!/usr/bin/env python3
"""
제공된 토큰으로 API 직접 테스트
"""
import requests
import json

BASE_URL = "http://localhost:8080"
TOKEN = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJ5X1EwUDd0MDhjcEZZeFNEUFdseGdKcGFUcWtsOFd0eUJYRGRGaVVUQXBJIn0.eyJleHAiOjE3NjY2NzQ2MDksImlhdCI6MTc2NjY3MjgwOSwianRpIjoiNjJhZDhmZWItNjRkZS00MjUyLThlOWItZGM4Nzg3YTNlNzA1IiwiaXNzIjoiaHR0cHM6Ly9rZXljbG9hay5wYWNzLmFpLWRvLmtyL3JlYWxtcy9kY200Y2hlIiwiYXVkIjpbImlhaWQtcGFjcy1jbGllbnQiLCJhY2NvdW50Il0sInN1YiI6ImY0ZTJlMzU1LTIxMDItNGZiNi04YzZmLTg4YzI3NDQzZjVkOCIsInR5cCI6IkJlYXJlciIsImF6cCI6ImlhaWQtcGFjcy1jbGllbnQiLCJzZXNzaW9uX3N0YXRlIjoiMzhiODZiNTYtYjJmOS00OTNjLWIwY2QtMDljMjAyYzViYTNkIiwiYWNyIjoiMSIsImFsbG93ZWQtb3JpZ2lucyI6WyIqIl0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJvZmZsaW5lX2FjY2VzcyIsImRlZmF1bHQtcm9sZXMtZGNtNGNoZSIsInVtYV9hdXRob3JpemF0aW9uIiwidXNlciJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoiUEFDUy1hdWRpZW5jZS1zZXJ2aWNlIHByb2ZpbGUgZW1haWwiLCJzaWQiOiIzOGI4NmI1Ni1iMmY5LTQ5M2MtYjBjZC0wOWMyMDJjNWJhM2QiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwicHJlZmVycmVkX3VzZXJuYW1lIjoiaWFpZC1wYWNzLWFkbWluIn0.SwUxz95PkFeOaWnYYXnRwiNg2_1cNB7fSwsABpUfo_zYt01jKO4abkVVMaDc9LtIEYeVtX0jlPD2kIErEqNaqHWXCQZvB2dSVUHtzm70J6fPMk1SnQncA6o029ipwx19j6GpLDnGXwnM9kcTYhBUQwyGKdo590O4Rh-ZtDme1_mLb73qlDdEUyiGhMnSFQVQRuv3653VWzq9HDf3EUbuR46TgEHlyXPs6yW6Np0w8g_ZipwQAwmhXv64JSmTnm9kl2QxeWSco4aqhx17GDWlmrJR_b0vjpcS93IsKEYQ_Yt4eXPxIH2arvXVs24rk2d-wbkd4NxnGjTKaEyDXYQJTg"

headers = {
    'Authorization': f'Bearer {TOKEN}',
    'Accept': 'application/json'
}

print("=" * 60)
print("🔍 API 직접 테스트")
print("=" * 60)

# 1. API 호출
print("\n1️⃣ /api/me/dicom/series?project_id=2 호출")
print("-" * 60)
url = f"{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=200"

try:
    response = requests.get(url, headers=headers, timeout=30)
    print(f"Status Code: {response.status_code}")
    print(f"Response Headers: {dict(response.headers)}")
    
    if response.status_code == 200:
        try:
            data = response.json()
            print(f"\n응답 타입: {type(data)}")
            
            if isinstance(data, list):
                print(f"✅ Series 개수: {len(data)}")
                if len(data) == 0:
                    print("❌ 빈 배열 반환")
                else:
                    print(f"\n첫 번째 Series:")
                    print(json.dumps(data[0], indent=2, ensure_ascii=False)[:500])
            elif isinstance(data, dict):
                series_list = data.get('series', [])
                total = data.get('total', 0)
                print(f"✅ Series 개수: {len(series_list)}")
                print(f"Total: {total}")
                
                if len(series_list) == 0:
                    print("❌ 빈 배열 반환")
                    print(f"\n전체 응답:")
                    print(json.dumps(data, indent=2, ensure_ascii=False)[:1000])
                else:
                    print(f"\n첫 번째 Series:")
                    print(json.dumps(series_list[0], indent=2, ensure_ascii=False)[:500])
            else:
                print(f"⚠️  예상치 못한 응답 형식: {type(data)}")
                print(f"\n응답 내용:")
                print(json.dumps(data, indent=2, ensure_ascii=False)[:1000])
                
        except json.JSONDecodeError:
            print(f"❌ JSON 파싱 실패")
            print(f"응답 내용: {response.text[:500]}")
    else:
        print(f"❌ 에러 응답: {response.status_code}")
        print(f"응답 내용: {response.text[:500]}")
        
except requests.exceptions.Timeout:
    print("❌ 요청 타임아웃")
except requests.exceptions.ConnectionError:
    print("❌ 연결 실패")
except Exception as e:
    print(f"❌ 에러: {e}")
    import traceback
    traceback.print_exc()

# 2. 문제 진단
print("\n2️⃣ 문제 진단")
print("-" * 60)
print("""
가능한 원인:
1. DB에서 허용된 Series UID가 0개
   → get_allowed_series_uids 쿼리가 빈 결과 반환
   
2. Dcm4chee QIDO가 빈 결과 반환
   → Dcm4chee 연결 실패 또는 실제로 Series가 없음
   
3. 필터링 후 결과가 0개
   → Series UID 형식 불일치 또는 필터링 로직 문제

확인 방법:
1. 서버 로그 확인:
   - '🔍 Gateway /series: Found {} allowed series UIDs'
   - '🔍 Gateway /series: QIDO returned {} series'
   - '🔍 Gateway /series: Filtered {} series'
   
2. DB 직접 확인:
   - test_get_allowed_series_uids.sql 실행
   
3. Dcm4chee 직접 확인:
   - Dcm4chee QIDO 엔드포인트 직접 호출
""")

print("\n" + "=" * 60)
print("✅ 테스트 완료")
print("=" * 60)

