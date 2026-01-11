#!/usr/bin/env python3
"""
View Selection API 테스트 (Mock 데이터 사용)

Redis 연결 문제가 있어도 API 구조를 테스트할 수 있도록
간단한 테스트를 수행합니다.
"""

import requests
import json
import sys

BASE_URL = 'http://localhost:8080'

def print_test(name):
    print(f"\n{'='*60}")
    print(f"🧪 {name}")
    print(f"{'='*60}")

def print_success(msg):
    print(f"✅ {msg}")

def print_error(msg):
    print(f"❌ {msg}")

def print_info(msg):
    print(f"ℹ️  {msg}")

def main():
    print("\n" + "="*60)
    print("🚀 View Selection API 테스트 (Mock)")
    print("="*60)
    
    # 1. Health Check
    print_test("Health Check")
    try:
        resp = requests.get(f'{BASE_URL}/health', timeout=5)
        if resp.status_code == 200:
            print_success(f"서버 정상: {resp.json()}")
        else:
            print_error(f"서버 오류: {resp.status_code}")
            return 1
    except Exception as e:
        print_error(f"서버 연결 실패: {e}")
        return 1
    
    # 2. OpenAPI 확인
    print_test("OpenAPI 엔드포인트 확인")
    try:
        resp = requests.get(f'{BASE_URL}/api-docs/openapi.json', timeout=5)
        if resp.status_code == 200:
            data = resp.json()
            paths = [k for k in data.get('paths', {}).keys() if 'view' in k.lower()]
            if paths:
                print_success(f"View Selection 엔드포인트 {len(paths)}개 발견")
                for p in paths:
                    print(f"   - {p}")
            else:
                print_error("View Selection 엔드포인트 없음")
        else:
            print_error(f"OpenAPI 조회 실패: {resp.status_code}")
    except Exception as e:
        print_error(f"OpenAPI 조회 에러: {e}")
    
    # 3. 실제 API 테스트
    print_test("실제 API 엔드포인트 테스트")
    
    # 인증 없이 테스트
    print_info("인증 없이 POST 요청 (401이 나와야 정상, 404면 라우트 미등록)")
    try:
        resp = requests.post(
            f'{BASE_URL}/api/v1/view-selections',
            json={'series': [{'study_uid': '1.2.3', 'series_uid': '1.2.3.4'}]},
            headers={'Content-Type': 'application/json'},
            timeout=5
        )
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 401:
            print_success("엔드포인트 등록됨 (인증 필요 - 정상)")
            print_info("Redis 연결이 성공했고 API가 정상 등록되었습니다!")
        elif resp.status_code == 404:
            print_error("엔드포인트 미등록 (404)")
            print_info("서버가 Redis 연결 실패로 view_selection_use_case가 None입니다")
            print_info("서버 로그를 확인하세요:")
            print_info("  - 'Failed to connect to Redis' 메시지 확인")
            print_info("  - Redis 터널이 실행 중인지 확인: lsof -i :6379")
            print_info("  - .env 파일 확인: APP_REDIS__URL=redis://localhost:6379")
        elif resp.status_code == 201:
            print_success("Selection 생성 성공 (인증 없이도 성공 - 개발 모드?)")
        else:
            print(f"Response: {resp.text[:200]}")
    except Exception as e:
        print_error(f"API 테스트 에러: {e}")
    
    # 4. GET 테스트
    print_info("인증 없이 GET 요청")
    try:
        resp = requests.get(
            f'{BASE_URL}/api/v1/view-selections/test123',
            headers={'Content-Type': 'application/json'},
            timeout=5
        )
        print(f"Status: {resp.status_code}")
        if resp.status_code == 401:
            print_success("GET 엔드포인트도 등록됨")
        elif resp.status_code == 404:
            print_error("GET 엔드포인트 미등록")
    except Exception as e:
        print_error(f"GET 테스트 에러: {e}")
    
    print("\n" + "="*60)
    print("📊 테스트 완료")
    print("="*60)
    
    return 0

if __name__ == "__main__":
    sys.exit(main())


