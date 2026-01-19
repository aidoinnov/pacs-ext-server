#!/usr/bin/env python3
import requests
import time

BASE_URL = "http://localhost:8080"
API_PATH = "/api/me/dicom/series"

# 로그인하여 토큰 가져오기
def get_token():
    # user_id 56에 대한 로그인 시도
    # 여러 가능한 사용자명 시도
    possible_users = [
        {"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
        {"username": "test_user_56", "password": "test123"},
    ]
    
    for user in possible_users:
        try:
            response = requests.post(
                f"{BASE_URL}/api/auth/login",
                json={"username": user["username"], "password": user["password"]},
                timeout=5
            )
            if response.status_code == 200:
                data = response.json()
                return data.get("token")
        except:
            continue
    return None

# 토큰 가져오기
token = get_token()
if not token:
    print("⚠️  인증 토큰을 가져올 수 없습니다. 토큰 없이 테스트 진행...")
    headers = {"Content-Type": "application/json"}
else:
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    print(f"✅ 인증 토큰 획득 성공")

# 테스트 파라미터
params_base = {
    "project_id": 2,
    "page": 1
}

print("=" * 60)
print("🚀 Series API 성능 비교 테스트")
print("=" * 60)

# 여러 번 실행하여 평균 측정
NUM_RUNS = 3

# Test 1: page_size=10
print(f"\n📊 Test 1: page_size=10 (평균 {NUM_RUNS}회 실행)")
params_10 = {**params_base, "page_size": 10}
times_10 = []
for i in range(NUM_RUNS):
    start = time.time()
    try:
        response = requests.get(f"{BASE_URL}{API_PATH}", params=params_10, headers=headers, timeout=30)
        elapsed = time.time() - start
        times_10.append(elapsed)
        if i == 0:  # 첫 실행 결과만 출력
            print(f"✅ Status: {response.status_code}")
            if response.status_code == 200:
                data = response.json()
                if isinstance(data, list):
                    print(f"📦 반환된 Series 수: {len(data)}")
    except Exception as e:
        print(f"❌ 에러 발생: {e}")
        break

elapsed_10 = sum(times_10) / len(times_10) if times_10 else None
if elapsed_10:
    print(f"⏱️  평균 응답 시간: {elapsed_10:.3f}초 (최소: {min(times_10):.3f}초, 최대: {max(times_10):.3f}초)")

# Test 2: page_size=200
print(f"\n📊 Test 2: page_size=200 (평균 {NUM_RUNS}회 실행)")
params_200 = {**params_base, "page_size": 200}
times_200 = []
for i in range(NUM_RUNS):
    start = time.time()
    try:
        response = requests.get(f"{BASE_URL}{API_PATH}", params=params_200, headers=headers, timeout=60)
        elapsed = time.time() - start
        times_200.append(elapsed)
        if i == 0:  # 첫 실행 결과만 출력
            print(f"✅ Status: {response.status_code}")
            if response.status_code == 200:
                data = response.json()
                if isinstance(data, list):
                    print(f"📦 반환된 Series 수: {len(data)}")
    except Exception as e:
        print(f"❌ 에러 발생: {e}")
        break

elapsed_200 = sum(times_200) / len(times_200) if times_200 else None
if elapsed_200:
    print(f"⏱️  평균 응답 시간: {elapsed_200:.3f}초 (최소: {min(times_200):.3f}초, 최대: {max(times_200):.3f}초)")

# 비교 결과
print("\n" + "=" * 60)
print("📈 성능 비교 결과")
print("=" * 60)
if elapsed_10 and elapsed_200:
    speedup = elapsed_200 / elapsed_10
    print(f"page_size=10:  {elapsed_10:.3f}초")
    print(f"page_size=200: {elapsed_200:.3f}초")
    if speedup > 1:
        print(f"성능 차이: {speedup:.2f}배 (page_size=10이 {speedup:.2f}배 빠름)")
    else:
        print(f"성능 차이: {1/speedup:.2f}배 (page_size=200이 {1/speedup:.2f}배 빠름)")
elif elapsed_10:
    print(f"page_size=10:  {elapsed_10:.3f}초")
    print(f"page_size=200: 실패")
elif elapsed_200:
    print(f"page_size=10:  실패")
    print(f"page_size=200: {elapsed_200:.3f}초")
else:
    print("❌ 두 테스트 모두 실패")

