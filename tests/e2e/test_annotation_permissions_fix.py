#!/usr/bin/env python3
"""
Annotation Permissions 엔드포인트 401 에러 수정 테스트

문제: Bearer 토큰을 보내도 401 Unauthorized 에러 발생
원인: extract_user_id_or_unauthorized() 함수가 개발 모드 전용이고 JWT 토큰을 처리하지 않음
해결: extract_user_id_with_auth() 함수로 변경 (개발 모드 + JWT 인증 지원)
"""

import sys
from pathlib import Path

# 프로젝트 루트를 Python 경로에 추가
sys.path.insert(0, str(Path(__file__).parent))

from utils.api_client import APIClient
from config import TestConfig


def test_annotation_permissions_with_jwt():
    """JWT 토큰으로 권한 조회 테스트"""
    print("=" * 80)
    print("🧪 Testing /api/annotations/permissions with JWT Token")
    print("=" * 80)
    
    # 설정 로드
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=30)
    
    # 로그인
    print(f"\n🔐 Logging in as {config.admin_email}...")
    try:
        client.login(config.admin_email, config.admin_password)
        print("✅ Login successful")
        print(f"🎫 Token: {client.token[:50]}...")
    except Exception as e:
        print(f"❌ Login failed: {e}")
        return False
    
    # 권한 조회 테스트
    print("\n" + "=" * 80)
    print("📋 Testing GET /api/annotations/permissions?project_id=2")
    print("=" * 80)
    
    try:
        response = client.get("/api/annotations/permissions", params={"project_id": 2})
        
        if response.status_code == 200:
            permissions = response.json()
            print("✅ Success! Permissions retrieved:")
            print(f"   - can_read_own: {permissions.get('can_read_own')}")
            print(f"   - can_read_all: {permissions.get('can_read_all')}")
            print(f"   - can_write: {permissions.get('can_write')}")
            print(f"   - can_delete: {permissions.get('can_delete')}")
            print(f"   - can_share: {permissions.get('can_share')}")
            return True
        elif response.status_code == 401:
            print(f"❌ FAILED: Still getting 401 Unauthorized")
            print(f"   Response: {response.text}")
            return False
        else:
            print(f"❌ Unexpected status code: {response.status_code}")
            print(f"   Response: {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False


def test_annotation_permissions_without_project_id():
    """project_id 없이 요청 (400 에러 예상)"""
    print("\n" + "=" * 80)
    print("📋 Testing GET /api/annotations/permissions (without project_id)")
    print("=" * 80)
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=30)
    
    try:
        client.login(config.admin_email, config.admin_password)
        response = client.get("/api/annotations/permissions")
        
        if response.status_code == 400:
            print("✅ Correctly returned 400 Bad Request (project_id required)")
            return True
        else:
            print(f"❌ Expected 400, got {response.status_code}")
            return False
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False


def main():
    print("\n" + "=" * 80)
    print("🔧 Annotation Permissions Endpoint Fix Verification")
    print("=" * 80)
    print("\n📝 Issue: Bearer token returns 401 Unauthorized")
    print("🔨 Fix: Changed extract_user_id_or_unauthorized() → extract_user_id_with_auth()")
    print("✨ Result: Now supports both dev mode and JWT authentication")
    
    results = []
    
    # 테스트 1: JWT 토큰으로 권한 조회
    results.append(("JWT Authentication", test_annotation_permissions_with_jwt()))
    
    # 테스트 2: project_id 없이 요청
    results.append(("Missing project_id", test_annotation_permissions_without_project_id()))
    
    # 결과 요약
    print("\n" + "=" * 80)
    print("📊 Test Results Summary")
    print("=" * 80)
    
    for test_name, passed in results:
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{status} - {test_name}")
    
    all_passed = all(result[1] for result in results)
    
    print("\n" + "=" * 80)
    if all_passed:
        print("🎉 All tests passed! The fix is working correctly.")
    else:
        print("⚠️  Some tests failed. Please review the output above.")
    print("=" * 80)
    
    return all_passed


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)

