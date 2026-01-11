#!/usr/bin/env python3
import requests

# Login
resp = requests.post('http://localhost:8080/api/auth/login', json={
    'username': 'iaid-pacs-admin',
    'password': 'Qlalfqjsgh1!'
})
token = resp.json()['token']
print(f'✅ Login OK (token length: {len(token)})')

# Test 1: 기본 조회 (view 없음)
print('\n=== Test 1: 기본 조회 (view 없음) ===')
resp = requests.get(
    'http://localhost:8080/api/me/dicom/studies?page=1&page_size=5',
    headers={'Authorization': f'Bearer {token}'}
)
print(f'Status: {resp.status_code}')
print(f'X-Total-Count: {resp.headers.get("X-Total-Count", "N/A")}')
print(f'X-Page: {resp.headers.get("X-Page", "N/A")}')
print(f'X-Page-Size: {resp.headers.get("X-Page-Size", "N/A")}')
print(f'X-Total-Pages: {resp.headers.get("X-Total-Pages", "N/A")}')
print(f'Data count: {len(resp.json())}')
if len(resp.json()) > 0:
    print(f'Has _ext: {"_ext" in resp.json()[0]}')

# Test 2: view=default
print('\n=== Test 2: view=default ===')
resp2 = requests.get(
    'http://localhost:8080/api/me/dicom/studies?view=default&page=1&page_size=5',
    headers={'Authorization': f'Bearer {token}'}
)
print(f'Status: {resp2.status_code}')
print(f'Data count: {len(resp2.json())}')
if len(resp2.json()) > 0:
    has_ext = "_ext" in resp2.json()[0]
    print(f'Has _ext: {has_ext}')
    if has_ext:
        ext_keys = list(resp2.json()[0]["_ext"].keys())
        print(f'_ext keys: {ext_keys}')

# Test 3: project_id=2
print('\n=== Test 3: project_id=2 ===')
resp3 = requests.get(
    'http://localhost:8080/api/me/dicom/studies?project_id=2&page=1&page_size=5',
    headers={'Authorization': f'Bearer {token}'}
)
print(f'Status: {resp3.status_code}')
if resp3.status_code == 200:
    print(f'Data count: {len(resp3.json())}')
else:
    print(f'Error: {resp3.text[:200]}')

