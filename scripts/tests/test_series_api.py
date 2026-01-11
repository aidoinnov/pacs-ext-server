#!/usr/bin/env python3
"""Series API 자동 테스트 스크립트"""
import requests
import json
import sys

API_URL = "http://localhost:8080"

def test_login(username, password):
    """로그인 테스트"""
    response = requests.post(
        f"{API_URL}/api/test/login",
        json={"username": username, "password": password}
    )
    if response.status_code != 200:
        print(f"❌ Login failed: {response.status_code}")
        print(response.text)
        return None
    token = response.json().get("access_token")
    if not token:
        print("❌ No access token in response")
        return None
    return token

def test_series_query(token, params):
    """Series 조회 테스트"""
    headers = {"Authorization": f"Bearer {token}"}
    response = requests.get(f"{API_URL}/api/dicom/series", headers=headers, params=params)
    return response

def main():
    print("=== 📋 Series API 자동 테스트 ===\n")
    
    # Test 1: SUPER_ADMIN Login
    print("Test 1: SUPER_ADMIN Login")
    token_admin = test_login("test_super_admin", "TestAdmin123!")
    if not token_admin:
        sys.exit(1)
    print(f"✅ PASSED (token length: {len(token_admin)})\n")
    
    # Test 2: Series 조회
    print("Test 2: Series 전체 조회")
    response = test_series_query(token_admin, {"project_id": 2, "PatientID": "SarcopeniaCase1"})
    if response.status_code != 200:
        print(f"❌ FAILED: {response.status_code}")
        print(response.text)
        sys.exit(1)
    data = response.json()
    if not isinstance(data, list):
        print(f"❌ FAILED: Not an array")
        sys.exit(1)
    print(f"✅ PASSED ({len(data)} series)\n")
    
    # Test 3: 빈 배열 (존재하지 않는 PatientID)
    print("Test 3: 존재하지 않는 PatientID (빈 배열 예상)")
    response = test_series_query(token_admin, {"project_id": 2, "PatientID": "NONEXISTENT_12345"})
    if response.status_code != 200:
        print(f"❌ FAILED: {response.status_code}")
        print(response.text)
        sys.exit(1)
    data = response.json()
    if len(data) != 0:
        print(f"❌ FAILED: Expected 0, got {len(data)}")
        sys.exit(1)
    print(f"✅ PASSED (empty array)\n")
    
    # Test 4: 빈 배열 (존재하지 않는 Modality)
    print("Test 4: 존재하지 않는 Modality (빈 배열 예상)")
    response = test_series_query(token_admin, {"project_id": 2, "PatientID": "SarcopeniaCase1", "Modality": "MR"})
    if response.status_code != 200:
        print(f"❌ FAILED: {response.status_code}")
        print(response.text)
        sys.exit(1)
    data = response.json()
    if len(data) != 0:
        print(f"❌ FAILED: Expected 0, got {len(data)}")
        sys.exit(1)
    print(f"✅ PASSED (empty array)\n")
    
    # Test 5: 일반 USER Login
    print("Test 5: 일반 USER Login")
    token_user = test_login("test_user", "TestUser123!")
    if not token_user:
        sys.exit(1)
    print(f"✅ PASSED (token length: {len(token_user)})\n")
    
    # Test 6: 일반 USER project_id 없이 조회 (400 예상)
    print("Test 6: 일반 USER project_id 없이 조회 (400 예상)")
    response = test_series_query(token_user, {"PatientID": "SarcopeniaCase1"})
    if response.status_code != 400:
        print(f"❌ FAILED: Expected 400, got {response.status_code}")
        print(response.text)
        sys.exit(1)
    print(f"✅ PASSED (400 error)\n")
    
    # Test 7: 일반 USER project_id와 함께 조회
    print("Test 7: 일반 USER project_id와 함께 조회")
    response = test_series_query(token_user, {"project_id": 2, "PatientID": "SarcopeniaCase1"})
    if response.status_code != 200:
        print(f"❌ FAILED: {response.status_code}")
        print(response.text)
        sys.exit(1)
    data = response.json()
    if len(data) == 0:
        print(f"❌ FAILED: Expected data, got empty array")
        sys.exit(1)
    print(f"✅ PASSED ({len(data)} series)\n")
    
    # Test 8: SUPER_ADMIN 전체 접근
    print("Test 8: SUPER_ADMIN project_id 없이 조회 (전체 접근)")
    response = test_series_query(token_admin, {"PatientID": "SarcopeniaCase1"})
    if response.status_code != 200:
        print(f"❌ FAILED: {response.status_code}")
        print(response.text)
        sys.exit(1)
    data = response.json()
    if len(data) == 0:
        print(f"❌ FAILED: SUPER_ADMIN should have global access")
        sys.exit(1)
    print(f"✅ PASSED ({len(data)} series - global access)\n")
    
    # Test 9: thumbnail_url 검증
    print("Test 9: thumbnail_url 필드 검증")
    response = test_series_query(token_admin, {"project_id": 2, "PatientID": "SarcopeniaCase1"})
    data = response.json()
    if len(data) == 0:
        print("❌ FAILED: No data")
        sys.exit(1)
    thumb_url = data[0].get("thumbnail_url")
    if not thumb_url:
        print("❌ FAILED: No thumbnail_url field")
        sys.exit(1)
    if "studies/" not in thumb_url or "series/" not in thumb_url or "thumbnail" not in thumb_url:
        print(f"❌ FAILED: Invalid WADO-RS format: {thumb_url}")
        sys.exit(1)
    print(f"✅ PASSED (WADO-RS format)\n")
    
    print("=" * 40)
    print("🎉 모든 핵심 테스트 통과!")
    print("=" * 40)

if __name__ == "__main__":
    main()

