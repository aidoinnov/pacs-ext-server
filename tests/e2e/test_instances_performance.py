#!/usr/bin/env python3
"""
Instances API 성능 테스트

V2 배치 쿼리 최적화 효과 확인
"""

import time
import requests
from config import BASE_URL, ADMIN_EMAIL, ADMIN_PASSWORD, TIMEOUT

def login():
    """로그인하여 토큰 획득"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": ADMIN_EMAIL, "password": ADMIN_PASSWORD},
        timeout=TIMEOUT
    )
    response.raise_for_status()
    return response.json()["token"]

def test_instances_api_performance():
    """Instances API 성능 테스트"""
    print("\n" + "="*80)
    print("Instances API 성능 테스트 (V2 배치 쿼리 최적화)")
    print("="*80)
    
    # 로그인
    print("\n🔐 로그인 중...")
    token = login()
    print("✅ 로그인 성공")
    
    # 테스트 파라미터
    study_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781"
    series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
    project_id = 2
    
    # 다양한 limit 값으로 테스트
    limits = [10, 50, 100, 500, 1000]
    
    results = []
    
    for limit in limits:
        print(f"\n📊 테스트: limit={limit}")
        
        url = (
            f"{BASE_URL}/api/dicom/studies/{study_uid}/series/{series_uid}/instances"
            f"?project_id={project_id}"
            f"&includefield=00080018"
            f"&includefield=00200013"
            f"&includefield=00200032"
            f"&includefield=00200037"
            f"&includefield=00201041"
            f"&includefield=00180050"
            f"&includefield=00180088"
            f"&includefield=00281050"
            f"&includefield=00281051"
            f"&limit={limit}"
        )
        
        headers = {"Authorization": f"Bearer {token}"}
        
        # 3회 측정하여 평균 계산
        times = []
        for i in range(3):
            start = time.time()
            response = requests.get(url, headers=headers, timeout=TIMEOUT)
            elapsed = time.time() - start
            times.append(elapsed)
            
            if response.status_code != 200:
                print(f"  ❌ 실패: {response.status_code}")
                print(f"     {response.text[:200]}")
                break
            
            count = len(response.json())
            print(f"  시도 {i+1}: {elapsed:.3f}초 ({count}개 인스턴스)")
        
        if times:
            avg_time = sum(times) / len(times)
            min_time = min(times)
            max_time = max(times)
            
            results.append({
                'limit': limit,
                'avg': avg_time,
                'min': min_time,
                'max': max_time,
                'count': count
            })
            
            print(f"  ✅ 평균: {avg_time:.3f}초 (최소: {min_time:.3f}초, 최대: {max_time:.3f}초)")
    
    # 결과 요약
    print("\n" + "="*80)
    print("📊 성능 테스트 결과 요약")
    print("="*80)
    print(f"{'Limit':<10} {'인스턴스':<12} {'평균 시간':<12} {'최소 시간':<12} {'최대 시간':<12}")
    print("-"*80)
    
    for r in results:
        print(f"{r['limit']:<10} {r['count']:<12} {r['avg']:.3f}초{'':<6} {r['min']:.3f}초{'':<6} {r['max']:.3f}초")
    
    print("\n" + "="*80)
    print("✅ 성능 테스트 완료!")
    print("="*80)
    
    # 성능 평가
    print("\n📈 성능 평가:")
    if results:
        # limit=1000일 때의 평균 시간
        large_result = next((r for r in results if r['limit'] == 1000), None)
        if large_result:
            avg_time = large_result['avg']
            if avg_time < 1.0:
                print(f"  🚀 우수: 1000개 인스턴스를 {avg_time:.3f}초에 조회 (< 1초)")
            elif avg_time < 5.0:
                print(f"  ✅ 양호: 1000개 인스턴스를 {avg_time:.3f}초에 조회 (< 5초)")
            elif avg_time < 30.0:
                print(f"  ⚠️  개선 필요: 1000개 인스턴스를 {avg_time:.3f}초에 조회 (< 30초)")
            else:
                print(f"  ❌ 느림: 1000개 인스턴스를 {avg_time:.3f}초에 조회 (>= 30초)")
            
            # 예상 개선 효과
            print(f"\n💡 V2 배치 쿼리 최적화 효과:")
            print(f"  - V1 (N+1 쿼리): 1000개 × 50ms = ~50초 예상")
            print(f"  - V2 (배치 쿼리): {avg_time:.3f}초 실제")
            if avg_time < 50:
                improvement = ((50 - avg_time) / 50) * 100
                print(f"  - 개선율: {improvement:.1f}% 🎉")

if __name__ == "__main__":
    test_instances_api_performance()

