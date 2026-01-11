#!/usr/bin/env python3
"""
사용자가 제공한 실제 URL로 성능 테스트
"""

import requests
import time

BASE_URL = "http://localhost:8080"

# 로그인
print("🔐 로그인 중...")
login_resp = requests.post(
    f"{BASE_URL}/api/auth/login",
    json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
    timeout=5
)

if login_resp.status_code != 200:
    print(f"❌ 로그인 실패: {login_resp.status_code}")
    exit(1)

token = login_resp.json().get("token")
headers = {"Authorization": f"Bearer {token}"}
print("✅ 로그인 성공\n")

# 실제 사용자가 제공한 URL
url = "http://localhost:8080/api/me/dicom/series?project_id=2&page=1&page_size=200&user_id=56"

print("=" * 70)
print("실제 사용자 URL 성능 테스트 (10회 실행)")
print("=" * 70)
print(f"URL: {url}\n")

times = []
counts = []
for i in range(10):
    start = time.time()
    try:
        response = requests.get(url, headers=headers, timeout=60)
        elapsed = time.time() - start
        times.append(elapsed)
        
        if response.status_code == 200:
            data = response.json()
            count = len(data) if isinstance(data, list) else 0
            counts.append(count)
            print(f"실행 {i+1:2d}: {elapsed:.3f}초 - Series 수: {count}")
        else:
            print(f"실행 {i+1:2d}: {elapsed:.3f}초 - Status: {response.status_code}")
            counts.append(0)
    except Exception as e:
        elapsed = time.time() - start
        times.append(elapsed)
        counts.append(0)
        print(f"실행 {i+1:2d}: {elapsed:.3f}초 - 에러: {str(e)[:50]}")

print("\n" + "=" * 70)
print("결과 요약")
print("=" * 70)
if times:
    avg = sum(times) / len(times)
    min_t = min(times)
    max_t = max(times)
    print(f"평균 응답 시간: {avg:.3f}초")
    print(f"최소 응답 시간: {min_t:.3f}초")
    print(f"최대 응답 시간: {max_t:.3f}초")
    
    if counts:
        avg_count = sum(counts) / len(counts)
        print(f"\n평균 반환 Series 수: {avg_count:.1f}개")
    
    if max_t > 3:
        print(f"\n⚠️  최대 응답 시간이 3초를 초과했습니다!")
        print(f"   실제 데이터가 많거나 네트워크 지연이 있을 수 있습니다.")
    elif max_t > 1:
        print(f"\n⚠️  최대 응답 시간이 1초를 초과했습니다.")
    else:
        print(f"\n✅ 응답 시간이 양호합니다.")





