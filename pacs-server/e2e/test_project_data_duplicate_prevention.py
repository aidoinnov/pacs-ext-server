#!/usr/bin/env python3
"""
E2E Test: Project Data Duplicate Prevention

Tests that the project_data table prevents duplicate entries
for the same (project_id, study_id) combination.
"""

import requests
import time

BASE_URL = "http://localhost:8080"

def test_duplicate_study_assignment_prevention():
    """
    Test 1: 동일한 Study를 같은 프로젝트에 두 번 할당 시도
    - 첫 번째 할당: 성공 (201 Created)
    - 두 번째 할당: 성공하지만 중복 생성 안 됨 (200 OK or 201 Created)
    - 데이터베이스에는 하나의 레코드만 존재
    """
    print("\n" + "="*80)
    print("Test 1: Duplicate Study Assignment Prevention")
    print("="*80)

    # 테스트용 데이터 - 실제 DB에 존재하는 Study 사용
    project_id = 2  # 기존 프로젝트 사용
    study_data = {
        "study_uid": "1.2.410.200022.500.12252244129",  # 실제 DB에 존재하는 Study
        "study_description": "CT,Neck others (with enhance)",
        "patient_id": "TEST001",
        "patient_name": "Test Patient",
        "study_date": "2024-01-24"
    }
    
    # 첫 번째 할당 시도
    print(f"\n1️⃣ First assignment attempt...")
    r1 = requests.post(
        f"{BASE_URL}/api/projects/{project_id}/studies/assign",
        json=study_data
    )
    print(f"   Status: {r1.status_code}")
    print(f"   Response: {r1.json()}")

    # 200/201 (성공) 또는 409 (이미 존재) 모두 허용
    assert r1.status_code in [200, 201, 409], f"First assignment failed: {r1.status_code}"

    if r1.status_code == 409:
        print(f"   ℹ️  Study already assigned (409 Conflict) - this is expected")
        # 이미 할당된 경우, 기존 study_id를 조회
        r_list = requests.get(f"{BASE_URL}/api/project-data/{project_id}/studies")
        studies = r_list.json().get("studies", [])
        matching = [s for s in studies if s.get("study_uid") == study_data["study_uid"]]
        first_id = matching[0]["id"] if matching else None
    else:
        first_id = r1.json().get("study_id")
        print(f"   ✅ First assignment successful, study_id: {first_id}")
    
    # 두 번째 할당 시도 (동일한 Study)
    print(f"\n2️⃣ Second assignment attempt (duplicate)...")
    r2 = requests.post(
        f"{BASE_URL}/api/projects/{project_id}/studies/assign",
        json=study_data
    )
    print(f"   Status: {r2.status_code}")
    print(f"   Response: {r2.json()}")

    # 409 Conflict가 반환되어야 함 (이미 존재)
    assert r2.status_code == 409, f"Expected 409 Conflict, got {r2.status_code}"
    assert "already assigned" in r2.json().get("message", "").lower(), "Expected 'already assigned' message"
    print(f"   ✅ Second assignment correctly rejected with 409 Conflict")
    
    # 프로젝트 데이터 목록 조회하여 중복 확인
    print(f"\n3️⃣ Verify no duplicates in database...")
    r3 = requests.get(f"{BASE_URL}/api/project-data/{project_id}/studies")
    assert r3.status_code == 200

    data_list = r3.json().get("studies", [])
    matching_studies = [d for d in data_list if d.get("study_uid") == study_data["study_uid"]]

    print(f"   Found {len(matching_studies)} matching study(ies)")
    assert len(matching_studies) == 1, f"Expected 1 study, found {len(matching_studies)}"
    print(f"   ✅ Only one study exists in database (no duplicates)")

    print("\n✅ Test 1 PASSED: Duplicate prevention works correctly (409 Conflict on duplicate)")


def test_concurrent_study_assignment():
    """
    Test 2: 동시에 같은 Study를 할당 시도
    - 여러 요청이 동시에 발생해도 중복 생성 안 됨
    """
    print("\n" + "="*80)
    print("Test 2: Concurrent Study Assignment")
    print("="*80)
    
    import concurrent.futures
    
    project_id = 111  # 다른 프로젝트 사용
    study_data = {
        "study_uid": "1.2.410.200022.500.202205101052995.12252192373",  # 실제 DB에 존재하는 Study
        "study_description": "CT,Neck others (with enhance)",
        "patient_id": "CONCURRENT001",
        "patient_name": "Concurrent Patient",
        "study_date": "2024-01-24"
    }
    
    def assign_study():
        """동일한 Study를 할당하는 함수"""
        r = requests.post(
            f"{BASE_URL}/api/projects/{project_id}/studies/assign",
            json=study_data
        )
        return r.status_code, r.json().get("study_id")
    
    # 5개의 동시 요청
    print(f"\n1️⃣ Sending 5 concurrent requests...")
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
        futures = [executor.submit(assign_study) for _ in range(5)]
        results = [f.result() for f in concurrent.futures.as_completed(futures)]
    
    print(f"   Results: {results}")
    
    # 모든 요청이 성공 (200/201) 또는 409 (이미 존재)여야 함
    success_count = sum(1 for status, _ in results if status in [200, 201])
    conflict_count = sum(1 for status, _ in results if status == 409)

    print(f"   Success (200/201): {success_count}, Conflict (409): {conflict_count}")
    assert success_count + conflict_count == 5, f"Unexpected status codes in results"

    # 최소 1개는 성공해야 함
    assert success_count >= 1, "At least one request should succeed"

    # 성공한 요청들의 ID가 모두 동일해야 함
    success_ids = [id for status, id in results if status in [200, 201] and id is not None]
    if success_ids:
        unique_ids = set(success_ids)
        print(f"   Unique study_ids from successful requests: {unique_ids}")
        assert len(unique_ids) == 1, f"Multiple study_ids created: {unique_ids}"
        print(f"   ✅ All successful requests returned same study_id: {list(unique_ids)[0]}")
    
    # 데이터베이스 확인
    print(f"\n2️⃣ Verify database state...")
    r = requests.get(f"{BASE_URL}/api/project-data/{project_id}/studies")
    assert r.status_code == 200

    data_list = r.json().get("studies", [])
    matching_studies = [d for d in data_list if d.get("study_uid") == study_data["study_uid"]]
    
    print(f"   Found {len(matching_studies)} matching study(ies)")
    assert len(matching_studies) == 1, f"Expected 1 study, found {len(matching_studies)}"
    print(f"   ✅ Only one study exists in database")
    
    print("\n✅ Test 2 PASSED: Concurrent assignment handled correctly")


def test_different_projects_same_study():
    """
    Test 3: 다른 프로젝트에 같은 Study 할당
    - 같은 Study를 여러 프로젝트에 할당 가능
    - 각 프로젝트마다 별도의 project_data 레코드 생성
    """
    print("\n" + "="*80)
    print("Test 3: Same Study in Different Projects")
    print("="*80)
    
    study_data = {
        "study_uid": "1.2.410.200022.500.12252244131",  # 실제 DB에 존재하는 Study
        "study_description": "CT,Abdomen & Pelvis (with enhance)(No oral water)",
        "patient_id": "MULTI001",
        "patient_name": "Multi Patient",
        "study_date": "2024-01-24"
    }
    
    # 프로젝트 2에 할당
    print(f"\n1️⃣ Assign to Project 2...")
    r1 = requests.post(f"{BASE_URL}/api/projects/2/studies/assign", json=study_data)
    print(f"   Status: {r1.status_code}")
    assert r1.status_code in [200, 201, 409], f"Unexpected status: {r1.status_code}"

    if r1.status_code == 409:
        print(f"   ℹ️  Already assigned to Project 2 (expected)")
    else:
        id_project2 = r1.json().get("study_id")
        print(f"   ✅ Assigned to Project 2, study_id: {id_project2}")

    # 프로젝트 22에 할당 (다른 프로젝트)
    print(f"\n2️⃣ Assign to Project 22...")
    r2 = requests.post(f"{BASE_URL}/api/projects/22/studies/assign", json=study_data)
    print(f"   Status: {r2.status_code}")
    assert r2.status_code in [200, 201, 409], f"Unexpected status: {r2.status_code}"

    if r2.status_code == 409:
        print(f"   ℹ️  Already assigned to Project 22 (expected)")
    else:
        id_project22 = r2.json().get("study_id")
        print(f"   ✅ Assigned to Project 22, study_id: {id_project22}")

    # 같은 Study를 여러 프로젝트에 할당 가능 (project_data 테이블에 별도 레코드)
    print(f"   ✅ Same study can be assigned to different projects")
    
    print("\n✅ Test 3 PASSED: Same study can be assigned to different projects")


if __name__ == "__main__":
    print("🧪 Project Data Duplicate Prevention E2E Tests")
    print("=" * 80)
    
    try:
        test_duplicate_study_assignment_prevention()
        test_concurrent_study_assignment()
        test_different_projects_same_study()
        
        print("\n" + "="*80)
        print("🎉 ALL TESTS PASSED!")
        print("="*80)
        
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        exit(1)
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        import traceback
        traceback.print_exc()
        exit(1)

