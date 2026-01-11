#!/usr/bin/env python3
"""
project_data 구조 확인
- project_data에 5개만 있다면 왜 11개 Series가 나오는지 확인
"""
import requests
import json

BASE_URL = "http://localhost:8080"

def get_token():
    """로그인하여 토큰 획득"""
    try:
        resp = requests.post(f'{BASE_URL}/api/auth/login', json={
            'username': 'iaid-pacs-admin',
            'password': 'Qlalfqjsgh1!'
        }, timeout=10)
        
        if resp.status_code == 200:
            data = resp.json()
            return data.get('token') or data.get('access_token')
    except Exception as e:
        print(f"❌ 로그인 에러: {e}")
    return None

def analyze_query_logic():
    """쿼리 로직 분석"""
    print("=" * 60)
    print("🔍 쿼리 로직 분석")
    print("=" * 60)
    
    print("\n현재 get_allowed_series_uids 쿼리:")
    print("-" * 60)
    print("""
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL
""")
    
    print("\n💡 이 쿼리의 동작:")
    print("  1. project_data에서 project_id=2인 레코드 조회")
    print("  2. 각 레코드의 study_id로 project_data_study 조인")
    print("  3. 각 study의 id로 project_data_series 조인")
    print("  4. DISTINCT로 중복 제거")
    
    print("\n📊 가능한 시나리오:")
    print("  - project_data에 5개 레코드가 있음")
    print("  - 각 레코드가 다른 study_id를 가리킴")
    print("  - 각 study가 여러 series를 가지고 있음")
    print("  - 예: 5개 study × 평균 2.2개 series = 11개 series")
    
    print("\n⚠️  문제점:")
    print("  - project_data에 5개만 있다면")
    print("  - 최대 5개 study만 조회 가능")
    print("  - 각 study가 평균 2.2개 series를 가져야 11개가 됨")
    print("  - 또는 일부 study가 많은 series를 가지고 있을 수 있음")

def check_actual_data_structure():
    """실제 데이터 구조 확인 시도"""
    print("\n" + "=" * 60)
    print("🔍 실제 데이터 구조 확인")
    print("=" * 60)
    
    print("\n💡 확인이 필요한 정보:")
    print("  1. project_data 테이블:")
    print("     - project_id=2인 레코드 개수")
    print("     - 각 레코드의 study_id")
    print("     - 각 레코드의 resource_level")
    print("")
    print("  2. project_data_study 테이블:")
    print("     - project_data의 study_id로 조회한 study 개수")
    print("     - 각 study의 id")
    print("")
    print("  3. project_data_series 테이블:")
    print("     - 각 study의 id로 조회한 series 개수")
    print("     - 총 series 개수")
    
    print("\n📝 확인 쿼리:")
    print("-" * 60)
    print("""
-- 1. project_data 개수
SELECT COUNT(*) FROM project_data WHERE project_id = 2;

-- 2. project_data의 study_id 목록
SELECT DISTINCT study_id, resource_level 
FROM project_data 
WHERE project_id = 2;

-- 3. 각 study의 series 개수
SELECT pds.id, pds.study_uid, COUNT(pdser.id) as series_count
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
LEFT JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
GROUP BY pds.id, pds.study_uid
ORDER BY series_count DESC;

-- 4. 총 series 개수 (get_allowed_series_uids와 동일)
SELECT COUNT(DISTINCT pdser.series_uid)
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL;
""")

def suggest_solution():
    """해결 방법 제안"""
    print("\n" + "=" * 60)
    print("💡 해결 방법")
    print("=" * 60)
    
    print("\n1️⃣ DB 직접 확인:")
    print("   - 위의 쿼리를 실행하여 실제 데이터 구조 확인")
    print("   - project_data에 5개가 맞는지 확인")
    print("   - 각 study가 몇 개의 series를 가지는지 확인")
    
    print("\n2️⃣ 가능한 원인:")
    print("   - project_data에 5개 레코드가 있지만")
    print("   - 각 레코드가 다른 study를 가리키고")
    print("   - 각 study가 여러 series를 가지고 있어서")
    print("   - 총 11개 series가 나올 수 있음")
    
    print("\n3️⃣ 확인 방법:")
    print("   - DB 터널 열기: ./scripts/start-db-tunnels.sh")
    print("   - 위의 쿼리 실행")
    print("   - 또는 서버 로그에서 'Found X allowed series UIDs' 확인")

def main():
    print("=" * 60)
    print("🔍 project_data 5개 → Series 11개 원인 분석")
    print("=" * 60)
    
    analyze_query_logic()
    check_actual_data_structure()
    suggest_solution()
    
    print("\n" + "=" * 60)
    print("📊 요약")
    print("=" * 60)
    print("project_data에 5개만 있어도 11개 Series가 나올 수 있습니다:")
    print("  - 5개 project_data 레코드")
    print("  - 각각 다른 study_id를 가리킴")
    print("  - 각 study가 평균 2.2개 series를 가짐")
    print("  - 총 11개 series")
    print("\n정확한 확인을 위해 DB 쿼리를 실행해보세요!")

if __name__ == '__main__':
    main()

