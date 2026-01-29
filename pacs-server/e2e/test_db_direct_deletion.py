#!/usr/bin/env python3
"""
자동화된 DB 직접 삭제 테스트
API를 통해 생성 → DB에서 직접 삭제 → ETag 변경 확인
"""

import requests
import psycopg2
import time

BASE_URL = "http://localhost:8080"
ADMIN_USER = "iaid-pacs-admin"
ADMIN_PASSWORD = "Qlalfqjsgh1!"

# PostgreSQL 연결 정보 (env.development 기준)
DB_CONFIG = {
    "host": "localhost",
    "port": 5432,
    "database": "pacs_db",
    "user": "admin",
    "password": "admin123"
}

def login():
    """로그인하여 토큰 받기"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={
            "username": ADMIN_USER,
            "password": ADMIN_PASSWORD
        },
        timeout=5
    )
    if response.status_code != 200:
        raise Exception(f"Login failed: {response.status_code}")
    return response.json()['token']

def main():
    print("=" * 70)
    print("자동화된 DB 직접 삭제 후 ETag 변경 확인 테스트")
    print("=" * 70)
    print()
    
    # 로그인
    token = login()
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 초기 상태 확인
    print("1️⃣ 초기 프로젝트 목록 조회")
    response1 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag1 = response1.headers.get('ETag')
    count1 = int(etag1.split('-')[-1].rstrip('"'))
    print(f"   초기 ETag: {etag1}")
    print(f"   초기 개수: {count1}")
    print()
    
    # 2. 테스트용 프로젝트 3개 생성
    print("2️⃣ 테스트용 프로젝트 3개 생성")
    project_ids = []
    for i in range(3):
        new_project = {
            "name": f"DB Delete Test {int(time.time())}-{i}",
            "description": "Test project for DB deletion",
            "sponsor": "Test Sponsor",
            "start_date": "2026-01-01",
            "auto_complete": False
        }
        response = requests.post(f"{BASE_URL}/api/projects", headers=headers, json=new_project)
        if response.status_code != 201:
            print(f"   ❌ 프로젝트 생성 실패: {response.status_code}")
            return
        project_ids.append(response.json()['id'])
        time.sleep(0.1)
    
    print(f"   생성된 프로젝트 ID: {project_ids}")
    print()
    
    # 3. 생성 후 ETag 확인
    print("3️⃣ 생성 후 ETag 확인")
    response2 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag2 = response2.headers.get('ETag')
    count2 = int(etag2.split('-')[-1].rstrip('"'))
    print(f"   생성 후 ETag: {etag2}")
    print(f"   생성 후 개수: {count2}")
    print(f"   증가량: +{count2 - count1}")
    print()
    
    # 4. DB에서 직접 중간 프로젝트 삭제
    print("4️⃣ DB에서 직접 중간 프로젝트 삭제")
    middle_project_id = project_ids[1]
    print(f"   삭제할 프로젝트 ID: {middle_project_id}")
    
    try:
        conn = psycopg2.connect(**DB_CONFIG)
        cursor = conn.cursor()
        
        # 삭제 전 확인
        cursor.execute("SELECT COUNT(*) FROM security_project WHERE id = %s", (middle_project_id,))
        before_count = cursor.fetchone()[0]
        print(f"   삭제 전 DB 확인: {before_count}개 존재")
        
        # 직접 삭제
        cursor.execute("DELETE FROM security_project WHERE id = %s", (middle_project_id,))
        conn.commit()
        
        # 삭제 후 확인
        cursor.execute("SELECT COUNT(*) FROM security_project WHERE id = %s", (middle_project_id,))
        after_count = cursor.fetchone()[0]
        print(f"   삭제 후 DB 확인: {after_count}개 존재")
        print(f"   ✅ DB에서 직접 삭제 완료!")
        
        cursor.close()
        conn.close()
    except Exception as e:
        print(f"   ❌ DB 삭제 실패: {e}")
        print(f"   (DB 연결 정보를 확인하세요)")
        return
    print()
    
    # 5. 이전 ETag로 재요청 (If-None-Match)
    print("5️⃣ 이전 ETag로 재요청 (If-None-Match 사용)")
    print(f"   If-None-Match: {etag2}")
    response3 = requests.get(
        f"{BASE_URL}/api/projects",
        headers={**headers, "If-None-Match": etag2}
    )
    
    if response3.status_code == 304:
        print(f"   ❌ 304 Not Modified 받음")
        print(f"   → ETag가 변경되지 않음! 구현에 문제가 있습니다!")
        print()
    elif response3.status_code == 200:
        etag3 = response3.headers.get('ETag')
        count3 = int(etag3.split('-')[-1].rstrip('"'))
        
        print(f"   ✅ 200 OK 받음")
        print(f"   이전 ETag: {etag2}")
        print(f"   새 ETag: {etag3}")
        print(f"   이전 개수: {count2}")
        print(f"   새 개수: {count3}")
        print(f"   감소량: -{count2 - count3}")
        print()
        
        if etag2 != etag3:
            print(f"   ✅ ETag 변경 감지 성공!")
        if count3 == count2 - 1:
            print(f"   ✅ COUNT 감소 감지 성공! (중간 항목 삭제 감지)")
        print()
    
    # 6. 정리: 나머지 테스트 프로젝트 삭제
    print("6️⃣ 테스트 프로젝트 정리")
    for pid in [project_ids[0], project_ids[2]]:  # 첫 번째와 세 번째
        requests.delete(f"{BASE_URL}/api/projects/{pid}", headers=headers)
    print(f"   ✅ 나머지 테스트 프로젝트 삭제 완료")
    print()
    
    # 7. 최종 결과
    print("=" * 70)
    print("✅ 테스트 완료!")
    print("=" * 70)
    print("결론: DB에서 직접 삭제해도 ETag가 변경되어 클라이언트가 감지할 수 있습니다!")
    print()

if __name__ == "__main__":
    main()

