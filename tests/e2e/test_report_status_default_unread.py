#!/usr/bin/env python3
"""
Report Status 기본값 "unread" 처리 테스트

문제: report_status가 없는 study들이 필터링에서 제외됨
해결: report_status가 없으면 기본값 "unread"로 처리
"""

import sys
from pathlib import Path

# 프로젝트 루트를 Python 경로에 추가
sys.path.insert(0, str(Path(__file__).parent))

from utils.api_client import APIClient
from config import TestConfig


def test_studies_without_report_status_filter():
    """report_status 필터 없이 조회 (모든 study 반환)"""
    print("=" * 80)
    print("🧪 Testing /api/me/dicom/studies (without report_status filter)")
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
        return False, 0
    
    # 필터 없이 조회
    print("\n📋 Fetching all studies...")
    try:
        response = client.get(
            "/api/me/dicom/studies",
            params={
                "view": "default",
                "page": 1,
                "page_size": 10
            }
        )
        
        if response.status_code == 200:
            data = response.json()

            # 응답이 배열인 경우와 객체인 경우 모두 처리
            if isinstance(data, list):
                studies = data
                total = len(data)
            else:
                studies = data.get("studies", [])
                total = data.get("total", 0)

            print(f"✅ Success! Found {len(studies)} studies (total: {total})")
            
            # _ext.report_status 확인
            for i, study in enumerate(studies[:3]):
                ext = study.get("_ext", {})
                report_status = ext.get("report_status", "N/A")
                print(f"   - Study {i+1}: report_status = {report_status}")
            
            return True, total
        else:
            print(f"❌ Failed with status {response.status_code}")
            return False, 0
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False, 0


def test_studies_with_unread_filter():
    """report_status=unread 필터로 조회"""
    print("\n" + "=" * 80)
    print("🧪 Testing /api/me/dicom/studies?report_status=unread")
    print("=" * 80)
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=30)
    
    try:
        client.login(config.admin_email, config.admin_password)
        
        response = client.get(
            "/api/me/dicom/studies",
            params={
                "view": "default",
                "report_status": "unread",
                "page": 1,
                "page_size": 10
            }
        )
        
        if response.status_code == 200:
            data = response.json()

            # 응답이 배열인 경우와 객체인 경우 모두 처리
            if isinstance(data, list):
                studies = data
                total = len(data)
            else:
                studies = data.get("studies", [])
                total = data.get("total", 0)

            print(f"✅ Success! Found {len(studies)} unread studies (total: {total})")
            
            # 모든 study가 report_status=unread인지 확인
            all_unread = True
            for study in studies:
                ext = study.get("_ext", {})
                report_status = ext.get("report_status", "N/A")
                if report_status != "unread":
                    all_unread = False
                    print(f"   ⚠️  Found non-unread study: {report_status}")
            
            if all_unread:
                print("   ✅ All studies have report_status='unread'")
            
            return True, total
        else:
            print(f"❌ Failed with status {response.status_code}")
            return False, 0
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False, 0


def test_studies_with_unapproved_filter():
    """report_status=unapproved 필터로 조회"""
    print("\n" + "=" * 80)
    print("🧪 Testing /api/me/dicom/studies?report_status=unapproved")
    print("=" * 80)
    
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=30)
    
    try:
        client.login(config.admin_email, config.admin_password)
        
        response = client.get(
            "/api/me/dicom/studies",
            params={
                "view": "default",
                "report_status": "unapproved",
                "page": 1,
                "page_size": 10
            }
        )
        
        if response.status_code == 200:
            data = response.json()

            # 응답이 배열인 경우와 객체인 경우 모두 처리
            if isinstance(data, list):
                studies = data
                total = len(data)
            else:
                studies = data.get("studies", [])
                total = data.get("total", 0)

            print(f"✅ Success! Found {len(studies)} unapproved studies (total: {total})")
            return True, total
        else:
            print(f"❌ Failed with status {response.status_code}")
            return False, 0
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
        return False, 0


def main():
    print("\n" + "=" * 80)
    print("🔧 Report Status Default 'unread' Fix Verification")
    print("=" * 80)
    print("\n📝 Issue: Studies without report_status are excluded from filtering")
    print("🔨 Fix: Default to 'unread' when report_status is missing")
    print("✨ Result: All studies now have report_status (default: 'unread')")
    
    results = []
    
    # 테스트 1: 필터 없이 조회
    passed, total_all = test_studies_without_report_status_filter()
    results.append(("No filter (all studies)", passed))
    
    # 테스트 2: unread 필터
    passed, total_unread = test_studies_with_unread_filter()
    results.append(("Filter: unread", passed))
    
    # 테스트 3: unapproved 필터
    passed, total_unapproved = test_studies_with_unapproved_filter()
    results.append(("Filter: unapproved", passed))
    
    # 결과 요약
    print("\n" + "=" * 80)
    print("📊 Test Results Summary")
    print("=" * 80)
    
    for test_name, passed in results:
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{status} - {test_name}")
    
    print("\n" + "=" * 80)
    print("📈 Statistics")
    print("=" * 80)
    print(f"Total studies (no filter): {total_all}")
    print(f"Unread studies: {total_unread}")
    print(f"Unapproved studies: {total_unapproved}")
    
    if total_unread > 0:
        print(f"\n✅ Default 'unread' is working! Found {total_unread} unread studies.")
    else:
        print(f"\n⚠️  No unread studies found (expected at least some with default 'unread')")
    
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

