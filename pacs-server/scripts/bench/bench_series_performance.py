#!/usr/bin/env python3
"""
Series API 성능 비교 스크립트
page_size=10 vs page_size=200의 응답 시간을 비교합니다.
"""

import requests
import time
import sys

BASE_URL = "http://localhost:8080"

def login():
    """로그인하여 토큰 획득"""
    print("🔐 로그인 중...")
    try:
        response = requests.post(
            f"{BASE_URL}/api/auth/login",
            json={"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"},
            timeout=5
        )
        
        if response.status_code != 200:
            print(f"❌ 로그인 실패: {response.status_code}")
            print(f"응답: {response.text[:200]}")
            return None
        
        token = response.json().get("token")
        if not token:
            print("❌ 토큰을 받을 수 없습니다")
            return None
        
        print("✅ 로그인 성공\n")
        return token
    except Exception as e:
        print(f"❌ 로그인 에러: {e}")
        return None

def test_api(url, headers, timeout=30):
    """API 호출 및 시간 측정"""
    start = time.time()
    try:
        response = requests.get(url, headers=headers, timeout=timeout)
        elapsed = time.time() - start
        
        result = {
            "status": response.status_code,
            "elapsed": elapsed,
            "success": response.status_code == 200
        }
        
        if response.status_code == 200:
            data = response.json()
            if isinstance(data, list):
                result["count"] = len(data)
            else:
                result["data_type"] = type(data).__name__
        
        return result
    except Exception as e:
        return {
            "status": 0,
            "elapsed": time.time() - start,
            "success": False,
            "error": str(e)
        }

def main():
    print("=" * 70)
    print("🚀 Series API 성능 비교 테스트")
    print("=" * 70)
    
    # 로그인
    token = login()
    if not token:
        print("❌ 로그인 실패로 테스트를 중단합니다.")
        sys.exit(1)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 여러 번 실행하여 평균 측정
    NUM_RUNS = 5
    print(f"\n각 테스트를 {NUM_RUNS}회 실행하여 평균을 계산합니다...\n")
    
    # Test 1: page_size=10
    print("=" * 70)
    print(f"📊 Test 1: page_size=10 ({NUM_RUNS}회 실행)")
    print("=" * 70)
    url_10 = f"{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=10"
    
    times_10 = []
    count_10 = None
    for i in range(NUM_RUNS):
        result = test_api(url_10, headers, timeout=30)
        if result["success"]:
            times_10.append(result["elapsed"])
            if count_10 is None and result.get("count") is not None:
                count_10 = result["count"]
        else:
            print(f"❌ 실행 {i+1} 실패: {result.get('error', 'Unknown')}")
    
    if times_10:
        avg_10 = sum(times_10) / len(times_10)
        min_10 = min(times_10)
        max_10 = max(times_10)
        print(f"⏱️  평균 응답 시간: {avg_10:.3f}초")
        print(f"   최소: {min_10:.3f}초, 최대: {max_10:.3f}초")
        if count_10 is not None:
            print(f"📦 반환된 Series 수: {count_10}")
    else:
        print("❌ 모든 실행 실패")
        avg_10 = None
    
    # Test 2: page_size=200
    print("\n" + "=" * 70)
    print(f"📊 Test 2: page_size=200 ({NUM_RUNS}회 실행)")
    print("=" * 70)
    url_200 = f"{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=200"
    
    times_200 = []
    count_200 = None
    for i in range(NUM_RUNS):
        result = test_api(url_200, headers, timeout=60)
        if result["success"]:
            times_200.append(result["elapsed"])
            if count_200 is None and result.get("count") is not None:
                count_200 = result["count"]
        else:
            print(f"❌ 실행 {i+1} 실패: {result.get('error', 'Unknown')}")
    
    if times_200:
        avg_200 = sum(times_200) / len(times_200)
        min_200 = min(times_200)
        max_200 = max(times_200)
        print(f"⏱️  평균 응답 시간: {avg_200:.3f}초")
        print(f"   최소: {min_200:.3f}초, 최대: {max_200:.3f}초")
        if count_200 is not None:
            print(f"📦 반환된 Series 수: {count_200}")
    else:
        print("❌ 모든 실행 실패")
        avg_200 = None
    
    result_10 = {"elapsed": avg_10, "count": count_10, "success": avg_10 is not None}
    result_200 = {"elapsed": avg_200, "count": count_200, "success": avg_200 is not None}
    
    # 비교 결과
    print("\n" + "=" * 70)
    print("📈 성능 비교 결과 (평균)")
    print("=" * 70)
    
    if result_10["success"] and result_200["success"]:
        elapsed_10 = result_10["elapsed"]
        elapsed_200 = result_200["elapsed"]
        
        print(f"page_size=10:  {elapsed_10:.3f}초 (평균)")
        print(f"page_size=200: {elapsed_200:.3f}초 (평균)")
        
        if elapsed_10 > 0:
            speedup = elapsed_200 / elapsed_10
            if speedup > 1.1:  # 10% 이상 차이
                print(f"\n✅ page_size=10이 {speedup:.2f}배 빠릅니다!")
                print(f"   시간 절약: {elapsed_200 - elapsed_10:.3f}초 ({(elapsed_200 - elapsed_10) / elapsed_200 * 100:.1f}%)")
            elif speedup < 0.9:  # 10% 이상 차이
                print(f"\n⚠️  page_size=200이 {1/speedup:.2f}배 빠릅니다")
                print(f"   (데이터가 적어서 오차일 수 있습니다)")
            else:
                print(f"\n➡️  성능 차이가 거의 없습니다 (오차 범위 내)")
        
        # 데이터 수 비교
        count_10 = result_10.get("count", 0)
        count_200 = result_200.get("count", 0)
        print(f"\n📊 데이터 수 비교:")
        print(f"   page_size=10:  {count_10}개")
        print(f"   page_size=200: {count_200}개")
        
        if count_10 == 0 and count_200 == 0:
            print("\n⚠️  주의: 반환된 데이터가 없습니다.")
            print("   실제 데이터가 많을 때는 page_size=10이 훨씬 빠를 것으로 예상됩니다.")
    else:
        print("❌ 일부 테스트가 실패했습니다.")
        if not result_10["success"]:
            print(f"   page_size=10 실패")
        if not result_200["success"]:
            print(f"   page_size=200 실패")

if __name__ == "__main__":
    main()

