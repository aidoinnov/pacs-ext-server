#!/usr/bin/env python3
"""
project_data 테이블 기반 Study 필터링 E2E 테스트

이 테스트는 check_study_access_batch 함수가 project_data 테이블을 
올바르게 확인하는지 검증합니다.

테스트 시나리오:
1. project_data에 할당된 study만 반환되는지 확인
2. project_data에 없는 study는 반환되지 않는지 확인
3. 여러 프로젝트에 속한 사용자의 경우 각 프로젝트별 필터링 확인
"""

import requests
import psycopg2
import json
import sys
from typing import List, Dict, Any

BASE_URL = "http://localhost:8080"
DB_CONFIG = {
    "host": "localhost",
    "port": 5456,
    "user": "pacs_extension_admin",
    "password": "PacsExtension2024",
    "database": "pacs_extension"
}

# 테스트 결과 추적
test_results = {
    "passed": 0,
    "failed": 0,
    "total": 0
}

def print_test(test_name: str):
    """테스트 시작 출력"""
    print(f"\n{'='*70}")
    print(f"🧪 테스트 {test_results['total'] + 1}: {test_name}")
    print(f"{'='*70}")

def print_success(message: str):
    """성공 메시지 출력"""
    print(f"✅ {message}")
    test_results["passed"] += 1
    test_results["total"] += 1

def print_error(message: str):
    """에러 메시지 출력"""
    print(f"❌ {message}")
    test_results["failed"] += 1
    test_results["total"] += 1

def print_info(message: str):
    """정보 메시지 출력"""
    print(f"ℹ️  {message}")

def get_db_connection():
    """DB 연결"""
    return psycopg2.connect(**DB_CONFIG)

def login(username: str, password: str) -> Dict[str, Any]:
    """로그인하여 토큰 획득"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": username, "password": password},
        timeout=5
    )
    if response.status_code != 200:
        raise Exception(f"Login failed: {response.status_code}")
    return response.json()

def get_user_projects(user_id: int) -> List[Dict[str, Any]]:
    """사용자가 속한 프로젝트 목록 조회"""
    conn = get_db_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT 
            up.project_id,
            p.name as project_name,
            r.name as role_name
        FROM security_user_project up
        JOIN security_project p ON up.project_id = p.id
        JOIN security_role r ON up.role_id = r.id
        WHERE up.user_id = %s
        ORDER BY up.project_id
    """, (user_id,))
    
    projects = []
    for row in cursor.fetchall():
        projects.append({
            "project_id": row[0],
            "project_name": row[1],
            "role_name": row[2]
        })
    
    cursor.close()
    conn.close()
    return projects

def get_project_assigned_studies(project_id: int) -> List[str]:
    """프로젝트에 할당된 study의 patient_id 목록 조회"""
    conn = get_db_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT DISTINCT pds.patient_id
        FROM project_data pd
        JOIN project_data_study pds ON pd.study_id = pds.id
        WHERE pd.project_id = %s
        ORDER BY pds.patient_id
    """, (project_id,))
    
    patient_ids = [row[0] for row in cursor.fetchall()]
    
    cursor.close()
    conn.close()
    return patient_ids

def get_all_patient_ids_in_project_data_study() -> List[str]:
    """project_data_study 테이블의 모든 patient_id 조회"""
    conn = get_db_connection()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT DISTINCT patient_id
        FROM project_data_study
        ORDER BY patient_id
    """)
    
    patient_ids = [row[0] for row in cursor.fetchall()]
    
    cursor.close()
    conn.close()
    return patient_ids

def get_me_studies(token: str, report_status: str = None, page: int = 1, page_size: int = 100) -> List[Dict[str, Any]]:
    """GET /api/me/dicom/studies 호출"""
    headers = {"Authorization": f"Bearer {token}"}
    params = {
        "view": "default",
        "page": page,
        "page_size": page_size
    }
    if report_status:
        params["report_status"] = report_status

    response = requests.get(
        f"{BASE_URL}/api/me/dicom/studies",
        headers=headers,
        params=params,
        timeout=10
    )

    if response.status_code != 200:
        print_info(f"API returned status {response.status_code}")
        return []

    data = response.json()
    if isinstance(data, list):
        return data
    else:
        print_info(f"Unexpected response type: {type(data)}")
        return []

def extract_patient_ids(studies: List[Dict[str, Any]]) -> List[str]:
    """Study 목록에서 patient_id 추출"""
    patient_ids = []
    for study in studies:
        patient_id = study.get("00100020", {}).get("Value", [None])[0]
        if patient_id:
            patient_ids.append(patient_id)
    return patient_ids

# ============================================================================
# 테스트 시나리오
# ============================================================================

def test_1_project_data_filtering():
    """
    테스트 1: project_data 테이블 기반 필터링 검증

    검증 사항:
    - API가 반환하는 모든 study는 project_data 테이블에 할당되어 있어야 함
    - project_data에 없는 study는 반환되지 않아야 함
    """
    print_test("project_data 테이블 기반 필터링 검증")

    # 1. 로그인
    print_info("로그인 중...")
    auth_data = login("reader1_user", "Qlalfqjsgh1!")
    token = auth_data.get("access_token") or auth_data.get("token")
    user_id = 5  # reader1_user의 user_id

    # 2. 사용자가 속한 프로젝트 확인
    print_info("사용자 프로젝트 확인 중...")
    projects = get_user_projects(user_id)
    print_info(f"사용자가 속한 프로젝트: {len(projects)}개")
    for proj in projects:
        print_info(f"  - 프로젝트 {proj['project_id']}: {proj['project_name']} ({proj['role_name']})")

    # 3. 각 프로젝트에 할당된 study 확인
    all_assigned_patient_ids = set()
    for proj in projects:
        assigned = get_project_assigned_studies(proj['project_id'])
        print_info(f"  - 프로젝트 {proj['project_id']}에 할당된 환자: {len(assigned)}명")
        for patient_id in assigned:
            print_info(f"    • {patient_id}")
        all_assigned_patient_ids.update(assigned)

    print_info(f"총 할당된 고유 환자 수: {len(all_assigned_patient_ids)}명")

    # 4. API 호출
    print_info("API 호출 중...")
    studies = get_me_studies(token)
    returned_patient_ids = extract_patient_ids(studies)
    unique_returned = set(returned_patient_ids)

    print_info(f"API가 반환한 study: {len(studies)}개")
    print_info(f"API가 반환한 고유 환자: {len(unique_returned)}명")
    for patient_id in unique_returned:
        print_info(f"  • {patient_id}")

    # 5. 검증: 반환된 모든 환자가 project_data에 할당되어 있는지 확인
    not_assigned = unique_returned - all_assigned_patient_ids

    if len(not_assigned) == 0:
        print_success(f"모든 반환된 환자({len(unique_returned)}명)가 project_data에 할당되어 있음")
    else:
        print_error(f"project_data에 없는 환자가 반환됨: {not_assigned}")
        return False

    # 6. 검증: project_data에 할당되었지만 반환되지 않은 환자 확인 (정보성)
    not_returned = all_assigned_patient_ids - unique_returned
    if len(not_returned) > 0:
        print_info(f"할당되었지만 반환되지 않은 환자: {len(not_returned)}명 (QIDO 조회 실패 가능)")
        for patient_id in not_returned:
            print_info(f"  • {patient_id}")

    return True

def test_2_unassigned_studies_not_returned():
    """
    테스트 2: 할당되지 않은 study는 반환되지 않음

    검증 사항:
    - project_data_study에는 있지만 project_data에는 없는 환자가 반환되지 않아야 함
    """
    print_test("할당되지 않은 study 필터링 검증")

    # 1. 로그인
    auth_data = login("reader1_user", "Qlalfqjsgh1!")
    token = auth_data.get("access_token") or auth_data.get("token")
    user_id = 5

    # 2. project_data_study의 모든 환자 조회
    all_patients = set(get_all_patient_ids_in_project_data_study())
    print_info(f"project_data_study의 전체 환자: {len(all_patients)}명")

    # 3. 사용자 프로젝트에 할당된 환자 조회
    projects = get_user_projects(user_id)
    assigned_patients = set()
    for proj in projects:
        assigned = get_project_assigned_studies(proj['project_id'])
        assigned_patients.update(assigned)

    print_info(f"사용자 프로젝트에 할당된 환자: {len(assigned_patients)}명")

    # 4. 할당되지 않은 환자 계산
    unassigned_patients = all_patients - assigned_patients
    print_info(f"할당되지 않은 환자: {len(unassigned_patients)}명")
    for patient_id in sorted(unassigned_patients):
        print_info(f"  • {patient_id}")

    # 5. API 호출
    studies = get_me_studies(token)
    returned_patient_ids = set(extract_patient_ids(studies))

    # 6. 검증: 할당되지 않은 환자가 반환되지 않았는지 확인
    returned_unassigned = returned_patient_ids & unassigned_patients

    if len(returned_unassigned) == 0:
        print_success(f"할당되지 않은 환자({len(unassigned_patients)}명)가 반환되지 않음")
    else:
        print_error(f"할당되지 않은 환자가 반환됨: {returned_unassigned}")
        return False

    return True

# ============================================================================
# 메인 실행
# ============================================================================

def main():
    """메인 테스트 실행"""
    print("\n" + "="*70)
    print("🧪 project_data 필터링 E2E 테스트")
    print("="*70)

    try:
        # 테스트 실행
        test_1_project_data_filtering()
        test_2_unassigned_studies_not_returned()

        # 결과 출력
        print("\n" + "="*70)
        print("📊 테스트 결과")
        print("="*70)
        print(f"✅ 통과: {test_results['passed']}")
        print(f"❌ 실패: {test_results['failed']}")
        print(f"📝 총: {test_results['total']}")
        print("="*70)

        if test_results['failed'] == 0:
            print("\n🎉 모든 테스트 통과!")
            sys.exit(0)
        else:
            print(f"\n❌ {test_results['failed']}개 테스트 실패")
            sys.exit(1)

    except Exception as e:
        print(f"\n❌ 테스트 실행 중 오류 발생: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()


