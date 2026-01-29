#!/usr/bin/env python3
"""
수동 DB 삭제 후 ETag 변경 확인 테스트
관리자가 DB에서 직접 삭제한 경우를 시뮬레이션
"""

import requests
import time

BASE_URL = "http://localhost:8080"
ADMIN_USER = "iaid-pacs-admin"
ADMIN_PASSWORD = "Qlalfqjsgh1!"

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
        print(f"Login response: {response.status_code}")
        print(response.text)
        raise Exception(f"Login failed: {response.status_code}")
    return response.json()['token']

def main():
    print("=" * 60)
    print("수동 DB 삭제 후 ETag 변경 확인 테스트")
    print("=" * 60)
    print()
    
    # 로그인
    token = login()
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 현재 프로젝트 목록 조회
    print("1️⃣ 현재 프로젝트 목록 조회")
    response1 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag1 = response1.headers.get('ETag')
    count1 = int(etag1.split('-')[-1].rstrip('"'))
    projects1 = response1.json()['projects']
    
    print(f"   ETag: {etag1}")
    print(f"   전체 프로젝트 수: {count1}")
    print(f"   응답 프로젝트 수: {len(projects1)}")
    print()
    
    # 2. 프로젝트 ID 목록 출력
    print("2️⃣ 프로젝트 ID 목록:")
    for i, project in enumerate(projects1[:10], 1):  # 처음 10개만
        print(f"   {i}. ID={project['id']}, Name={project['name']}")
    if len(projects1) > 10:
        print(f"   ... 외 {len(projects1) - 10}개")
    print()
    
    # 3. 사용자에게 안내
    print("3️⃣ 이제 다음 작업을 수행하세요:")
    print("   1) 다른 터미널에서 psql로 DB 접속")
    print("   2) DELETE FROM security_project WHERE id = <원하는 ID>;")
    print("   3) 삭제 완료 후 이 스크립트로 돌아와서 Enter 키 누르기")
    print()
    input("   삭제 완료 후 Enter를 누르세요... ")
    print()
    
    # 4. 같은 ETag로 재요청 (304 받을 것으로 예상)
    print("4️⃣ 이전 ETag로 재요청 (If-None-Match 사용)")
    response2 = requests.get(
        f"{BASE_URL}/api/projects",
        headers={**headers, "If-None-Match": etag1}
    )
    
    if response2.status_code == 304:
        print(f"   ❌ 304 Not Modified - ETag가 변경되지 않음!")
        print(f"   → 구현에 문제가 있습니다!")
    elif response2.status_code == 200:
        etag2 = response2.headers.get('ETag')
        count2 = int(etag2.split('-')[-1].rstrip('"'))
        projects2 = response2.json()['projects']
        
        print(f"   ✅ 200 OK - ETag가 변경됨!")
        print(f"   이전 ETag: {etag1}")
        print(f"   새 ETag: {etag2}")
        print(f"   이전 개수: {count1}")
        print(f"   새 개수: {count2}")
        print(f"   응답 프로젝트 수: {len(projects2)}")
        print()
        
        if count2 < count1:
            print(f"   ✅ COUNT 감소 감지! ({count1} → {count2})")
            print(f"   → 구현이 정상 작동합니다!")
        else:
            print(f"   ⚠️ COUNT가 감소하지 않음")
    print()
    
    # 5. ETag 없이 재요청 (항상 200 OK)
    print("5️⃣ ETag 없이 재요청 (최신 데이터 확인)")
    response3 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag3 = response3.headers.get('ETag')
    count3 = int(etag3.split('-')[-1].rstrip('"'))
    projects3 = response3.json()['projects']
    
    print(f"   ETag: {etag3}")
    print(f"   전체 프로젝트 수: {count3}")
    print(f"   응답 프로젝트 수: {len(projects3)}")
    print()
    
    # 6. 결과 요약
    print("=" * 60)
    print("결과 요약")
    print("=" * 60)
    print(f"초기 개수: {count1}")
    print(f"최종 개수: {count3}")
    print(f"차이: {count1 - count3}")
    print()
    
    if count3 < count1:
        print("✅ 삭제가 정상적으로 반영되었습니다!")
        print("✅ ETag 캐싱이 정상 작동합니다!")
    else:
        print("⚠️ 개수가 변경되지 않았습니다.")
        print("   DB에서 삭제를 확인하세요.")

if __name__ == "__main__":
    main()

