#!/usr/bin/env python3
"""
QIDO Studies 빈 응답 처리 테스트

문제: dcm4chee가 빈 응답을 반환할 때 "EOF while parsing a value at line 1 column 0" 에러 발생
원인: qido_studies_with_bearer() 함수가 빈 응답을 처리하지 않고 바로 JSON 파싱 시도
해결: 빈 응답일 때 빈 배열 [] 반환하도록 수정
"""

import sys
from pathlib import Path

# 프로젝트 루트를 Python 경로에 추가
sys.path.insert(0, str(Path(__file__).parent))

from utils.api_client import APIClient
from config import TestConfig


def test_studies_with_wildcard_search():
    """와일드카드 검색으로 Studies 조회 (빈 응답 가능성 있음)"""
    print("=" * 80)
    print("🧪 Testing /api/dicom/studies with wildcard search")
    print("=" * 80)
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=30)
    
    # 로그인
    print(f"\n🔐 Logging in as {config.admin_email}...")
    try:
        client.login(config.admin_email, config.admin_password)
        print("✅ Login successful")
    except Exception as e:
        print(f"❌ Login failed: {e}")
        return False
    
    # 와일드카드 검색 (결과가 없을 수 있음)
    print("\n" + "=" * 80)
    print("📋 Testing GET /api/dicom/studies?project_id=2&00100010=*ddf*")
    print("=" * 80)
    
    try:
        response = client.get(
            "/api/dicom/studies",
            params={
                "project_id": 2,
                "sqlLikeMatching": "true",
                "limit": 10,
                "00100010": "*ddf*"  # 존재하지 않을 가능성이 높은 환자명
            }
        )
        
        if response.status_code == 200:
            studies = response.json()
            print(f"✅ Success! Received response (type: {type(studies).__name__})")
            
            if isinstance(studies, list):
                print(f"   - Number of studies: {len(studies)}")
                if len(studies) == 0:
                    print("   - ✅ Empty result handled correctly (returned [])")
                else:
                    print(f"   - Found {len(studies)} studies")
                    for i, study in enumerate(studies[:3]):
                        patient_name = study.get("00100010", {}).get("Value", [{}])[0].get("Alphabetic", "N/A")
                        print(f"   - Study {i+1}: {patient_name}")
                return True
            else:
                print(f"❌ Expected list, got {type(studies).__name__}")
                return False
                
        elif response.status_code == 500:
            error_text = response.text
            if "EOF while parsing" in error_text:
                print(f"❌ FAILED: Still getting EOF parsing error")
                print(f"   Response: {error_text}")
                return False
            else:
                print(f"❌ Server error: {error_text}")
                return False
        else:
            print(f"❌ Unexpected status code: {response.status_code}")
            print(f"   Response: {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False


def test_studies_normal_search():
    """일반 검색으로 Studies 조회 (결과가 있을 것으로 예상)"""
    print("\n" + "=" * 80)
    print("📋 Testing GET /api/dicom/studies (normal search)")
    print("=" * 80)
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=30)
    
    try:
        client.login(config.admin_email, config.admin_password)
        response = client.get(
            "/api/dicom/studies",
            params={
                "project_id": 2,
                "limit": 5
            }
        )
        
        if response.status_code == 200:
            studies = response.json()
            print(f"✅ Success! Found {len(studies)} studies")
            return True
        else:
            print(f"❌ Failed with status {response.status_code}")
            return False
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False


def main():
    print("\n" + "=" * 80)
    print("🔧 QIDO Studies Empty Response Fix Verification")
    print("=" * 80)
    print("\n📝 Issue: EOF while parsing a value at line 1 column 0")
    print("🔨 Fix: Added empty response handling in qido_studies_with_bearer()")
    print("✨ Result: Returns [] when dcm4chee returns empty response")
    
    results = []
    
    # 테스트 1: 와일드카드 검색 (빈 응답 가능성)
    results.append(("Wildcard search (empty response)", test_studies_with_wildcard_search()))
    
    # 테스트 2: 일반 검색
    results.append(("Normal search", test_studies_normal_search()))
    
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

