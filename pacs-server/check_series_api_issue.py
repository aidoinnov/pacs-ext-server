#!/usr/bin/env python3
"""
Series API 문제 진단 스크립트
"""
import requests
import json
import base64
from datetime import datetime

BASE_URL = "http://localhost:8080"
TOKEN = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJ5X1EwUDd0MDhjcEZZeFNEUFdseGdKcGFUcWtsOFd0eUJYRGRGaVVUQXBJIn0.eyJleHAiOjE3NjcyNDc2ODEsImlhdCI6MTc2NzI0NzM4MSwianRpIjoiZTc3YjU2YzctY2EwZS00ZjljLTlkZjAtYzA2MmQ3NDMyMGEzIiwiaXNzIjoiaHR0cHM6Ly9rZXljbG9hay5wYWNzLmFpLWRvLmNvLmtyL3JlYWxtcy9kY200Y2hlIiwiYXVkIjpbImlhaWQtcGFjcy1jbGllbnQiLCJhY2NvdW50Il0sInN1YiI6ImY0ZTJlMzU1LTIxMDItNGZiNi04YzZmLTg4YzI3NDQzZjVkOCIsInR5cCI6IkJlYXJlciIsImF6cCI6ImlhaWQtcGFjcy1jbGllbnQiLCJzZXNzaW9uX3N0YXRlIjoiNzY2MWFhZDEtNmUzYS00MTVjLTlhY2ItZTQ3YzZiZmM0ZmExIiwiYWNyIjoiMSIsImFsbG93ZWQtb3JpZ2lucyI6WyIqIl0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJvZmZsaW5lX2FjY2VzcyIsImRlZmF1bHQtcm9sZXMtZGNtNGNoZSIsInVtYV9hdXRob3JpemF0aW9uIiwidXNlciJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoiUEFDUy1hdWRpZW5jZS1zZXJ2aWNlIHByb2ZpbGUgZW1haWwiLCJzaWQiOiI3NjYxYWFkMS02ZTNhLTQxNWMtOWFjYi1lNDdjNmJmYzRmYTEiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwicHJlZmVycmVkX3VzZXJuYW1lIjoiaWFpZC1wYWNzLWFkbWluIn0.EtaYxku0tSEspnskmvkJ8O_pj_YtahTSFHSGzqjo1tiXS3W97HhbZw4ME0SbtALvGxBZAipyWNfZvfAlLnUOnmtDvEz0d9KkgCvBt7d0mWMuLRFIP86Pxh3PWdlRMr82h_O9_OBgyDtGxjzSloX1ZE1UcRNhdydnPaFISElsNfMRsCA6EMIk18Xb-ZGhPaSd5klUsRPPEg6Jiry9OEs5NDxvRFiUPi-YN7a_3ReBD6hNGcTI4TV8nZZzT3i3pd-_aksreqfONlw1zERpfACJbTtAt0kRmKV0Y8bp7Xkb_HOfvDVzzLSyKLG8IQRf4XkiYDlrkqT7EstX_N4u9b9Syg"

def decode_jwt(token):
    """JWT 토큰 디코딩"""
    try:
        parts = token.split('.')
        if len(parts) != 3:
            return None
        
        # Payload 디코딩
        payload = parts[1]
        # Base64 padding 추가
        payload += '=' * (4 - len(payload) % 4)
        decoded = base64.urlsafe_b64decode(payload)
        return json.loads(decoded)
    except Exception as e:
        print(f"❌ 토큰 디코딩 에러: {e}")
        return None

def check_token_expiry(token):
    """토큰 만료 시간 확인"""
    payload = decode_jwt(token)
    if not payload:
        return None
    
    exp = payload.get('exp')
    iat = payload.get('iat')
    
    if exp:
        exp_time = datetime.fromtimestamp(exp)
        now = datetime.now()
        is_expired = now > exp_time
        
        print(f"📅 토큰 정보:")
        print(f"   발급 시간 (iat): {datetime.fromtimestamp(iat) if iat else 'N/A'}")
        print(f"   만료 시간 (exp): {exp_time}")
        print(f"   현재 시간: {now}")
        print(f"   상태: {'❌ 만료됨' if is_expired else '✅ 유효함'}")
        
        if is_expired:
            print(f"   만료된 지: {(now - exp_time).total_seconds() / 3600:.2f} 시간")
        else:
            print(f"   남은 시간: {(exp_time - now).total_seconds() / 3600:.2f} 시간")
        
        return not is_expired
    
    return None

def test_api(token):
    """API 테스트"""
    print("\n" + "=" * 60)
    print("🔍 API 테스트")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    url = f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=200'
    print(f"\n📡 요청 URL: {url}")
    print("-" * 60)
    
    try:
        resp = requests.get(url, headers=headers, timeout=30)
        print(f"Status Code: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                count = len(data)
                print(f"✅ 응답 성공: {count}개 Series")
                if count == 0:
                    print("⚠️  Series 목록이 비어있습니다!")
                else:
                    print(f"   첫 번째 Series UID: {data[0].get('0020000E', {}).get('Value', ['N/A'])[0]}")
            elif isinstance(data, dict):
                series_list = data.get('series', [])
                count = len(series_list)
                print(f"✅ 응답 성공: {count}개 Series")
                if count == 0:
                    print("⚠️  Series 목록이 비어있습니다!")
            else:
                print(f"⚠️  예상치 못한 응답 형식: {type(data)}")
                print(f"   {str(data)[:200]}")
        elif resp.status_code == 401:
            print("❌ 인증 실패 (401 Unauthorized)")
            print("   토큰이 만료되었거나 유효하지 않습니다.")
        elif resp.status_code == 403:
            print("❌ 권한 없음 (403 Forbidden)")
            print("   프로젝트에 대한 접근 권한이 없습니다.")
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:500]}")
        
        return resp.status_code == 200
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
        return False

def get_new_token():
    """새 토큰 획득"""
    print("\n" + "=" * 60)
    print("🔑 새 토큰 획득")
    print("=" * 60)
    
    try:
        resp = requests.post(f'{BASE_URL}/api/auth/login', json={
            'username': 'iaid-pacs-admin',
            'password': 'Qlalfqjsgh1!'
        }, timeout=10)
        
        if resp.status_code == 200:
            data = resp.json()
            token = data.get('token') or data.get('access_token')
            if token:
                print("✅ 새 토큰 획득 성공")
                return token
        else:
            print(f"❌ 로그인 실패: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 로그인 에러: {e}")
    
    return None

def main():
    print("=" * 60)
    print("🔍 Series API 문제 진단")
    print("=" * 60)
    
    # 1. 토큰 만료 확인
    print("\n1️⃣ 토큰 만료 확인")
    print("-" * 60)
    is_valid = check_token_expiry(TOKEN)
    
    # 2. API 테스트
    print("\n2️⃣ API 테스트 (기존 토큰)")
    print("-" * 60)
    api_success = test_api(TOKEN)
    
    # 3. 토큰이 만료되었거나 API가 실패한 경우 새 토큰으로 재시도
    if not is_valid or not api_success:
        new_token = get_new_token()
        if new_token:
            print("\n3️⃣ API 테스트 (새 토큰)")
            print("-" * 60)
            test_api(new_token)
    
    # 결과 요약
    print("\n" + "=" * 60)
    print("📊 진단 결과 요약")
    print("=" * 60)
    print(f"토큰 상태: {'✅ 유효' if is_valid else '❌ 만료'}")
    print(f"API 응답: {'✅ 성공' if api_success else '❌ 실패'}")
    
    if not is_valid:
        print("\n💡 해결 방법:")
        print("   1. 새 토큰을 발급받아 사용하세요")
        print("   2. 또는 자동으로 새 토큰을 발급받아 재시도했습니다")
    elif not api_success:
        print("\n💡 가능한 원인:")
        print("   1. project_data에 데이터가 없을 수 있습니다")
        print("   2. 사용자가 프로젝트에 할당되지 않았을 수 있습니다")
        print("   3. Dcm4chee 연결 문제일 수 있습니다")
        print("   4. 서버 로그를 확인하세요")

if __name__ == '__main__':
    main()

