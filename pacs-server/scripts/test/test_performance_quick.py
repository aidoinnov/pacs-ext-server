#!/usr/bin/env python3
"""
빠른 성능 테스트 - view 파라미터 유무에 따른 응답 시간 비교
"""
import requests
import time

API_URL = "http://localhost:8080"

def login():
    resp = requests.post(f"{API_URL}/api/auth/login", json={
        "username": "iaid-pacs-admin",
        "password": "Qlalfqjsgh1!"
    })
    return resp.json()["token"]

def test_performance(token, params_desc, params):
    start = time.time()
    resp = requests.get(
        f"{API_URL}/api/me/dicom/studies",
        headers={"Authorization": f"Bearer {token}"},
        params=params
    )
    duration = time.time() - start
    
    count = len(resp.json()) if resp.status_code == 200 else 0
    print(f"{params_desc:30s} | {duration:6.2f}s | {count:3d} studies | Status: {resp.status_code}")
    return duration

if __name__ == "__main__":
    print("=" * 80)
    print("성능 테스트: view 파라미터 유무 비교")
    print("=" * 80)
    
    token = login()
    print(f"✅ 로그인 성공\n")
    
    # Test 1: view 없음 (기본)
    print("Test 1: view 파라미터 없음")
    t1 = test_performance(token, "No view", {"page": 1, "page_size": 10})
    
    # Test 2: view=default
    print("\nTest 2: view=default")
    t2 = test_performance(token, "view=default", {"view": "default", "page": 1, "page_size": 10})
    
    # Test 3: view=default + project_id
    print("\nTest 3: view=default + project_id=2")
    t3 = test_performance(token, "view=default + project_id", {"view": "default", "project_id": 2, "page": 1, "page_size": 10})
    
    print("\n" + "=" * 80)
    print(f"결과 요약:")
    print(f"  view 없음:              {t1:.2f}s")
    print(f"  view=default:           {t2:.2f}s  (차이: +{t2-t1:.2f}s)")
    print(f"  view=default+project:   {t3:.2f}s  (차이: +{t3-t1:.2f}s)")
    print("=" * 80)

