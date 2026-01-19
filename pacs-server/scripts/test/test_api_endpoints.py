#!/usr/bin/env python3
"""API 엔드포인트 테스트 스크립트"""
import requests
import json

BASE_URL = "http://localhost:8080"

# 테스트할 엔드포인트들
endpoints = [
    ("POST", "/api/auth/signup", {"username": "testuser", "email": "test@test.com", "password": "TestPassword123!"}),
    ("POST", "/api/auth/login", {"keycloak_id": "test", "username": "test", "email": "test@test.com"}),
    ("GET", "/api/users", None),
    ("POST", "/api/users", {"keycloak_id": "test", "username": "test", "email": "test@test.com"}),
]

print("Testing API endpoints:\n")
for method, path, data in endpoints:
    try:
        if method == "POST":
            response = requests.request(method, f"{BASE_URL}{path}", json=data, timeout=2)
        else:
            response = requests.request(method, f"{BASE_URL}{path}", timeout=2)
        status = response.status_code
        print(f"{method:6} {path:30} -> {status} ({'OK' if status < 400 else 'ERROR'})")
        if status < 500 and status != 404:
            try:
                result = response.json()
                print(f"         Response: {json.dumps(result, indent=2)[:200]}")
            except:
                print(f"         Response: {response.text[:100]}")
    except Exception as e:
        print(f"{method:6} {path:30} -> ERROR: {e}")
    print()

