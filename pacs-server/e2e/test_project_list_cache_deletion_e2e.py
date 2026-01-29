#!/usr/bin/env python3
"""
E2E 테스트: Project List API ETag 캐싱 - 삭제 감지 테스트
COUNT(*) 포함 ETag로 중간 항목 삭제도 감지하는지 검증
"""

import requests
import time
import json
import sys

BASE_URL = "http://localhost:8080"
ADMIN_USER = "iaid-pacs-admin"
ADMIN_PASSWORD = "Qlalfqjsgh1!"

# 테스트용 토큰 (실제 환경에 맞게 수정)
def get_auth_token():
    """인증 토큰 획득"""
    # PACS 서버 로그인
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={
            "username": ADMIN_USER,
            "password": ADMIN_PASSWORD,
        },
        timeout=5
    )
    if response.status_code == 200:
        return response.json()["token"]  # "token" 필드 사용
    else:
        print(f"❌ Failed to get token: {response.status_code}")
        print(response.text)
        return None


def test_1_project_creation_updates_etag():
    """테스트 1: 프로젝트 생성 시 ETag 변경"""
    print("\n" + "="*60)
    print("테스트 1: 프로젝트 생성 시 ETag 변경")
    print("="*60)
    
    token = get_auth_token()
    if not token:
        print("❌ SKIP: No auth token")
        return False
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 초기 목록 조회
    response1 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    assert response1.status_code == 200
    etag1 = response1.headers.get('ETag')
    # ETag 형식: W/"timestamp-count"에서 count 추출
    count1 = int(etag1.split('-')[-1].rstrip('"'))
    print(f"✅ 초기 ETag: {etag1}, 전체 프로젝트 수: {count1}")

    # 2. 새 프로젝트 생성
    new_project = {
        "name": f"Test Project Delete {int(time.time())}",
        "description": "Test project for deletion",
        "sponsor": "Test Sponsor",
        "start_date": "2026-01-01",
        "auto_complete": False
    }
    response2 = requests.post(f"{BASE_URL}/api/projects", headers=headers, json=new_project)
    if response2.status_code != 201:
        print(f"❌ 프로젝트 생성 실패: {response2.status_code}")
        print(f"Response: {response2.text}")
        raise Exception(f"Failed to create project: {response2.status_code}")
    project_id = response2.json()['id']
    print(f"✅ 프로젝트 생성: ID={project_id}")

    # 3. 목록 재조회 - ETag 변경 확인
    response3 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    assert response3.status_code == 200
    etag2 = response3.headers.get('ETag')
    count2 = int(etag2.split('-')[-1].rstrip('"'))
    print(f"✅ 생성 후 ETag: {etag2}, 전체 프로젝트 수: {count2}")

    assert etag1 != etag2, "ETag should change after creation"
    assert count2 == count1 + 1, f"Project count should increase by 1 (was {count1}, now {count2})"
    
    # 4. 이전 ETag로 요청 - 200 OK 받아야 함
    response4 = requests.get(
        f"{BASE_URL}/api/projects",
        headers={**headers, 'If-None-Match': etag1}
    )
    assert response4.status_code == 200, "Should return 200 with old ETag"
    print(f"✅ 이전 ETag로 요청: 200 OK (새 데이터 반환)")
    
    print("✅ 테스트 1 통과!\n")
    return project_id


def test_2_project_update_changes_etag():
    """테스트 2: 프로젝트 수정 시 ETag 변경"""
    print("\n" + "="*60)
    print("테스트 2: 프로젝트 수정 시 ETag 변경")
    print("="*60)
    
    token = get_auth_token()
    if not token:
        print("❌ SKIP: No auth token")
        return None
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 프로젝트 생성
    new_project = {
        "name": f"Test Project Update {int(time.time())}",
        "description": "Test project for update",
        "sponsor": "Test Sponsor",
        "start_date": "2026-01-01",
        "auto_complete": False
    }
    response1 = requests.post(f"{BASE_URL}/api/projects", headers=headers, json=new_project)
    assert response1.status_code == 201
    project_id = response1.json()['id']
    print(f"✅ 프로젝트 생성: ID={project_id}")
    
    # 2. 초기 ETag 저장
    response2 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag1 = response2.headers.get('ETag')
    print(f"✅ 초기 ETag: {etag1}")

    time.sleep(1.1)  # updated_at 변경 보장 (타임스탬프는 초 단위)

    # 3. 프로젝트 수정
    update_data = {
        "name": new_project["name"],
        "description": "Updated description",
        "sponsor": "Updated Sponsor",
        "start_date": "2026-01-01",
        "end_date": "",  # 빈 문자열로 None 표현
        "auto_complete": False
    }
    response3 = requests.put(f"{BASE_URL}/api/projects/{project_id}", headers=headers, json=update_data)
    if response3.status_code != 200:
        print(f"❌ 프로젝트 수정 실패: {response3.status_code}")
        print(f"Response: {response3.text}")
        raise Exception(f"Failed to update project: {response3.status_code}")
    print(f"✅ 프로젝트 수정 완료")
    
    # 4. ETag 변경 확인
    response4 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag2 = response4.headers.get('ETag')
    print(f"✅ 수정 후 ETag: {etag2}")
    
    assert etag1 != etag2, "ETag should change after update"
    print("✅ 테스트 2 통과!\n")
    
    return project_id


def test_3_middle_project_deletion_detected():
    """테스트 3: 중간 프로젝트 삭제 감지 (핵심 테스트!)"""
    print("\n" + "="*60)
    print("테스트 3: 중간 프로젝트 삭제 감지 (COUNT 포함 ETag)")
    print("="*60)
    
    token = get_auth_token()
    if not token:
        print("❌ SKIP: No auth token")
        return False
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 3개의 프로젝트 생성
    project_ids = []
    for i in range(3):
        new_project = {
            "name": f"Test Project Middle {int(time.time())}-{i}",
            "description": f"Test project {i}",
            "sponsor": "Test Sponsor",
            "start_date": "2026-01-01",
            "auto_complete": False
        }
        response = requests.post(f"{BASE_URL}/api/projects", headers=headers, json=new_project)
        assert response.status_code == 201
        project_ids.append(response.json()['id'])
        time.sleep(0.1)  # updated_at 차이 보장
    
    print(f"✅ 3개 프로젝트 생성: {project_ids}")
    
    # 2. 초기 ETag 저장
    response1 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag1 = response1.headers.get('ETag')
    # ETag 형식: W/"timestamp-count"에서 count 추출
    count1 = int(etag1.split('-')[-1].rstrip('"'))
    print(f"✅ 초기 ETag: {etag1}, 전체 프로젝트 수: {count1}")

    # 3. 중간 프로젝트 삭제 (가장 최근이 아닌 프로젝트)
    middle_project_id = project_ids[1]
    response2 = requests.delete(f"{BASE_URL}/api/projects/{middle_project_id}", headers=headers)
    if response2.status_code not in [200, 204]:
        print(f"❌ 프로젝트 삭제 실패: {response2.status_code}")
        print(f"Response: {response2.text}")
        raise Exception(f"Failed to delete project: {response2.status_code}")
    print(f"✅ 중간 프로젝트 삭제: ID={middle_project_id}")

    # 4. ETag 변경 확인 (COUNT가 변경되므로 ETag도 변경되어야 함!)
    response3 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag2 = response3.headers.get('ETag')
    count2 = int(etag2.split('-')[-1].rstrip('"'))
    print(f"✅ 삭제 후 ETag: {etag2}, 전체 프로젝트 수: {count2}")

    assert etag1 != etag2, "❌ CRITICAL: ETag should change after middle item deletion!"
    assert count2 == count1 - 1, f"Project count should decrease by 1 (was {count1}, now {count2})"
    print(f"✅ ETag 변경 감지 성공! (COUNT 덕분)")
    
    # 5. 이전 ETag로 요청 - 200 OK 받아야 함
    response4 = requests.get(
        f"{BASE_URL}/api/projects",
        headers={**headers, 'If-None-Match': etag1}
    )
    assert response4.status_code == 200, "Should return 200 with old ETag after deletion"
    print(f"✅ 이전 ETag로 요청: 200 OK (삭제 반영된 새 목록 반환)")
    
    # 6. 나머지 프로젝트 정리
    for pid in [project_ids[0], project_ids[2]]:
        requests.delete(f"{BASE_URL}/api/projects/{pid}", headers=headers)
    
    print("✅ 테스트 3 통과! (중간 항목 삭제 감지 성공)\n")
    return True


def test_4_latest_project_deletion_detected():
    """테스트 4: 최신 프로젝트 삭제 감지"""
    print("\n" + "="*60)
    print("테스트 4: 최신 프로젝트 삭제 감지")
    print("="*60)
    
    token = get_auth_token()
    if not token:
        print("❌ SKIP: No auth token")
        return False
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 2개의 프로젝트 생성
    project_ids = []
    for i in range(2):
        new_project = {
            "name": f"Test Project Latest {int(time.time())}-{i}",
            "description": f"Test project {i}",
            "sponsor": "Test Sponsor",
            "start_date": "2026-01-01",
            "auto_complete": False
        }
        response = requests.post(f"{BASE_URL}/api/projects", headers=headers, json=new_project)
        assert response.status_code == 201
        project_ids.append(response.json()['id'])
        time.sleep(0.1)
    
    print(f"✅ 2개 프로젝트 생성: {project_ids}")
    
    # 2. 초기 ETag 저장
    response1 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag1 = response1.headers.get('ETag')
    print(f"✅ 초기 ETag: {etag1}")
    
    # 3. 최신 프로젝트 삭제
    latest_project_id = project_ids[-1]
    response2 = requests.delete(f"{BASE_URL}/api/projects/{latest_project_id}", headers=headers)
    if response2.status_code not in [200, 204]:
        print(f"❌ 프로젝트 삭제 실패: {response2.status_code}")
        print(f"Response: {response2.text}")
        raise Exception(f"Failed to delete project: {response2.status_code}")
    print(f"✅ 최신 프로젝트 삭제: ID={latest_project_id}")
    
    # 4. ETag 변경 확인
    response3 = requests.get(f"{BASE_URL}/api/projects", headers=headers)
    etag2 = response3.headers.get('ETag')
    print(f"✅ 삭제 후 ETag: {etag2}")
    
    assert etag1 != etag2, "ETag should change after latest item deletion"
    print(f"✅ ETag 변경 감지 성공!")
    
    # 5. 정리
    requests.delete(f"{BASE_URL}/api/projects/{project_ids[0]}", headers=headers)
    
    print("✅ 테스트 4 통과!\n")
    return True


def main():
    """모든 테스트 실행"""
    print("\n" + "="*60)
    print("Project List Cache Deletion E2E Tests")
    print("ETag with COUNT(*) - 삭제 감지 테스트")
    print("="*60)
    
    results = []
    
    # 테스트 1: 프로젝트 생성
    try:
        project_id = test_1_project_creation_updates_etag()
        results.append(("테스트 1: 프로젝트 생성", True))
        # 정리
        if project_id:
            token = get_auth_token()
            requests.delete(f"{BASE_URL}/api/projects/{project_id}", 
                          headers={"Authorization": f"Bearer {token}"})
    except Exception as e:
        print(f"❌ 테스트 1 실패: {e}")
        results.append(("테스트 1: 프로젝트 생성", False))
    
    # 테스트 2: 프로젝트 수정
    try:
        project_id = test_2_project_update_changes_etag()
        results.append(("테스트 2: 프로젝트 수정", True))
        # 정리
        if project_id:
            token = get_auth_token()
            requests.delete(f"{BASE_URL}/api/projects/{project_id}",
                          headers={"Authorization": f"Bearer {token}"})
    except Exception as e:
        print(f"❌ 테스트 2 실패: {e}")
        results.append(("테스트 2: 프로젝트 수정", False))
    
    # 테스트 3: 중간 프로젝트 삭제 (핵심!)
    try:
        test_3_middle_project_deletion_detected()
        results.append(("테스트 3: 중간 항목 삭제 감지", True))
    except Exception as e:
        print(f"❌ 테스트 3 실패: {e}")
        results.append(("테스트 3: 중간 항목 삭제 감지", False))
    
    # 테스트 4: 최신 프로젝트 삭제
    try:
        test_4_latest_project_deletion_detected()
        results.append(("테스트 4: 최신 항목 삭제 감지", True))
    except Exception as e:
        print(f"❌ 테스트 4 실패: {e}")
        results.append(("테스트 4: 최신 항목 삭제 감지", False))
    
    # 결과 요약
    print("\n" + "="*60)
    print("테스트 결과 요약")
    print("="*60)
    for name, passed in results:
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{status} - {name}")
    
    total = len(results)
    passed = sum(1 for _, p in results if p)
    print(f"\n총 {total}개 테스트 중 {passed}개 통과 ({passed/total*100:.1f}%)")
    
    if passed == total:
        print("\n🎉 모든 테스트 통과!")
        return 0
    else:
        print(f"\n❌ {total - passed}개 테스트 실패")
        return 1


if __name__ == "__main__":
    sys.exit(main())

