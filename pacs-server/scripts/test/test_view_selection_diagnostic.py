#!/usr/bin/env python3
"""
View Selection API 진단 및 테스트 스크립트

이 스크립트는 다음을 확인합니다:
1. 서버 상태
2. Redis 연결 상태
3. API 엔드포인트 등록 여부
4. 실제 API 동작 테스트
"""

import requests
import json
import socket
import sys

BASE_URL = 'http://localhost:8080'

def print_section(title):
    """섹션 제목 출력"""
    print("\n" + "="*60)
    print(f"🔍 {title}")
    print("="*60)

def test_health():
    """서버 Health Check"""
    print_section("1. 서버 Health Check")
    try:
        resp = requests.get(f'{BASE_URL}/health', timeout=5)
        print(f"Status: {resp.status_code}")
        if resp.status_code == 200:
            data = resp.json()
            print(f"✅ 서버 정상: {data}")
            return True
        else:
            print(f"❌ 서버 오류: {resp.text[:200]}")
            return False
    except Exception as e:
        print(f"❌ 연결 실패: {e}")
        return False

def test_redis_connection():
    """Redis 연결 테스트"""
    print_section("2. Redis 연결 테스트")
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(3)
        result = sock.connect_ex(('localhost', 6379))
        
        if result == 0:
            print("✅ Redis 포트 6379 연결 가능")
            try:
                # 간단한 PING 시도
                sock.send(b'*1\r\n$4\r\nPING\r\n')
                response = sock.recv(1024)
                sock.close()
                if b'PONG' in response or b'+PONG' in response:
                    print("✅ Redis PING 성공")
                    return True
                else:
                    print(f"⚠️  Redis 응답 이상: {response}")
                    return False
            except Exception as e:
                print(f"⚠️  Redis 통신 에러: {e}")
                sock.close()
                return False
        else:
            print(f"❌ Redis 포트 6379 연결 실패 (코드: {result})")
            print("   → SSH 터널이 실행 중인지 확인하세요: lsof -i :6379")
            return False
    except Exception as e:
        print(f"❌ Redis 연결 테스트 에러: {e}")
        return False

def test_openapi_endpoints():
    """OpenAPI 문서에서 엔드포인트 확인"""
    print_section("3. OpenAPI 엔드포인트 확인")
    try:
        resp = requests.get(f'{BASE_URL}/api-docs/openapi.json', timeout=5)
        if resp.status_code == 200:
            data = resp.json()
            paths = data.get('paths', {})
            view_paths = [k for k in paths.keys() if 'view' in k.lower()]
            
            if view_paths:
                print(f"✅ View Selection 엔드포인트 {len(view_paths)}개 발견:")
                for path in view_paths:
                    methods = list(paths[path].keys())
                    print(f"   - {path} [{', '.join(methods).upper()}]")
                return True
            else:
                print("❌ View Selection 엔드포인트 없음")
                return False
        else:
            print(f"❌ OpenAPI 문서 조회 실패: {resp.status_code}")
            return False
    except Exception as e:
        print(f"❌ OpenAPI 조회 에러: {e}")
        return False

def test_api_endpoints():
    """실제 API 엔드포인트 테스트"""
    print_section("4. 실제 API 엔드포인트 테스트")
    
    # 인증 없이 테스트 (401이 나와야 정상, 404면 라우트 미등록)
    print("\n4-1. POST /api/v1/view-selections (인증 없이)")
    try:
        resp = requests.post(
            f'{BASE_URL}/api/v1/view-selections',
            json={'series': [{'study_uid': '1.2.3', 'series_uid': '1.2.3.4'}]},
            headers={'Content-Type': 'application/json'},
            timeout=5
        )
        print(f"   Status: {resp.status_code}")
        if resp.status_code == 401:
            print("   ✅ 엔드포인트 등록됨 (인증 필요)")
            return True
        elif resp.status_code == 404:
            print("   ❌ 엔드포인트 미등록 (404)")
            print("   → 서버가 Redis 연결 실패로 view_selection_use_case가 None일 수 있음")
            return False
        else:
            print(f"   ⚠️  예상치 못한 응답: {resp.text[:200]}")
            return False
    except Exception as e:
        print(f"   ❌ 에러: {e}")
        return False

def test_with_token():
    """토큰을 사용한 실제 테스트"""
    print_section("5. 토큰을 사용한 실제 API 테스트")
    
    # Test token 얻기 시도
    print("\n5-1. Test Token 획득 시도...")
    token = None
    try:
        # 여러 형식 시도
        test_data_variants = [
            {'user_id': 1},
            {'user_id': 1, 'keycloak_id': '00000000-0000-0000-0000-000000000001'},
            {'user_id': 1, 'keycloak_id': 'urn:uuid:00000000-0000-0000-0000-000000000001'},
        ]
        
        for test_data in test_data_variants:
            resp = requests.post(
                f'{BASE_URL}/api/auth/test-token',
                json=test_data,
                headers={'Content-Type': 'application/json'},
                timeout=5
            )
            if resp.status_code == 200:
                data = resp.json()
                token = data.get('token') or data.get('access_token')
                if token:
                    print(f"   ✅ Token 획득 성공 (형식: {test_data})")
                    break
            else:
                print(f"   ⚠️  형식 {test_data} 실패: {resp.status_code}")
        
        if not token:
            print("   ❌ Token 획득 실패 - 인증 없이 테스트 진행")
            return False
    except Exception as e:
        print(f"   ⚠️  Token API 에러: {e}")
        return False
    
    # Token으로 Selection 생성
    if token:
        print("\n5-2. View Selection 생성 (Token 사용)")
        try:
            resp = requests.post(
                f'{BASE_URL}/api/v1/view-selections',
                json={
                    'series': [
                        {'study_uid': '1.2.840.113619.2.1.1.123', 'series_uid': '1.2.840.113619.2.1.2.124'},
                        {'study_uid': '1.2.840.113619.2.1.1.125', 'series_uid': '1.2.840.113619.2.1.2.126'}
                    ]
                },
                headers={
                    'Content-Type': 'application/json',
                    'Authorization': f'Bearer {token}'
                },
                timeout=5
            )
            print(f"   Status: {resp.status_code}")
            print(f"   Response: {resp.text[:300]}")
            
            if resp.status_code == 201:
                data = resp.json()
                selection_id = data.get('selection_id')
                print(f"   ✅ Selection 생성 성공: {selection_id}")
                
                # Selection 조회
                print(f"\n5-3. Selection 조회: {selection_id}")
                get_resp = requests.get(
                    f'{BASE_URL}/api/v1/view-selections/{selection_id}',
                    headers={'Authorization': f'Bearer {token}'},
                    timeout=5
                )
                print(f"   Status: {get_resp.status_code}")
                if get_resp.status_code == 200:
                    get_data = get_resp.json()
                    print(f"   ✅ Selection 조회 성공")
                    print(f"   - Series 수: {len(get_data.get('series', []))}")
                    print(f"   - User ID: {get_data.get('user_id')}")
                    
                    # 삭제
                    print(f"\n5-4. Selection 삭제: {selection_id}")
                    del_resp = requests.delete(
                        f'{BASE_URL}/api/v1/view-selections/{selection_id}',
                        headers={'Authorization': f'Bearer {token}'},
                        timeout=5
                    )
                    print(f"   Status: {del_resp.status_code}")
                    if del_resp.status_code == 204:
                        print(f"   ✅ Selection 삭제 성공")
                        return True
                    else:
                        print(f"   ⚠️  삭제 실패: {del_resp.text[:200]}")
                        return False
                else:
                    print(f"   ❌ 조회 실패: {get_resp.text[:200]}")
                    return False
            else:
                print(f"   ❌ 생성 실패")
                return False
        except Exception as e:
            print(f"   ❌ 에러: {e}")
            return False
    
    return False

def main():
    """메인 테스트 실행"""
    print("\n" + "="*60)
    print("🚀 View Selection API 진단 및 테스트")
    print("="*60)
    
    results = {
        'health': False,
        'redis': False,
        'openapi': False,
        'api_endpoint': False,
        'api_test': False,
    }
    
    # 1. Health Check
    results['health'] = test_health()
    if not results['health']:
        print("\n❌ 서버가 실행 중이지 않습니다. 서버를 시작하세요.")
        return 1
    
    # 2. Redis 연결
    results['redis'] = test_redis_connection()
    
    # 3. OpenAPI 엔드포인트 확인
    results['openapi'] = test_openapi_endpoints()
    
    # 4. 실제 API 엔드포인트 테스트
    results['api_endpoint'] = test_api_endpoints()
    
    # 5. 토큰을 사용한 실제 테스트
    if results['api_endpoint']:
        results['api_test'] = test_with_token()
    
    # 결과 요약
    print_section("📊 테스트 결과 요약")
    print(f"서버 Health Check: {'✅' if results['health'] else '❌'}")
    print(f"Redis 연결: {'✅' if results['redis'] else '❌'}")
    print(f"OpenAPI 엔드포인트 등록: {'✅' if results['openapi'] else '❌'}")
    print(f"API 엔드포인트 등록: {'✅' if results['api_endpoint'] else '❌'}")
    print(f"실제 API 테스트: {'✅' if results['api_test'] else '❌'}")
    
    if not results['redis']:
        print("\n⚠️  Redis 연결 실패:")
        print("   1. SSH 터널 확인: lsof -i :6379")
        print("   2. 터널 시작: ./scripts/start-db-tunnels.sh")
        print("   3. .env 파일 확인: APP_REDIS__URL=redis://localhost:6379")
    
    if results['openapi'] and not results['api_endpoint']:
        print("\n⚠️  OpenAPI에는 등록되어 있지만 실제 라우트가 작동하지 않음:")
        print("   → 서버가 Redis 연결 실패로 view_selection_use_case가 None일 수 있음")
        print("   → 서버 로그에서 'Failed to connect to Redis' 메시지 확인")
        print("   → 서버 재시작 필요")
    
    if all([results['health'], results['redis'], results['api_endpoint'], results['api_test']]):
        print("\n🎉 모든 테스트 통과!")
        return 0
    else:
        print("\n⚠️  일부 테스트 실패")
        return 1

if __name__ == "__main__":
    sys.exit(main())


