#!/usr/bin/env python3
"""
DICOM Data Access Check API E2E 테스트

API: POST /api/v1/dicom/access/check

테스트 시나리오:
1. 사용자가 접근 가능한 Study 확인
2. 사용자가 접근 가능한 Series 확인
3. 여러 프로젝트에 속한 사용자의 접근 권한 확인
4. 접근 불가능한 데이터 확인
5. 잘못된 UID 처리
6. 인증 실패 처리
"""

import requests
import psycopg2
import json
import sys
from typing import List, Dict, Any, Optional
from test_base import TestConfig, TestAuth, TestPrinter

BASE_URL = TestConfig.BASE_URL
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


def check_data_access(
    token: str,
    study_uid: str,
    series_uid: Optional[str] = None,
    project_id: Optional[int] = None
) -> Dict[str, Any]:
    """POST /api/v1/dicom/access/check 호출"""
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    payload = {"study_uid": study_uid}
    if series_uid:
        payload["series_uid"] = series_uid
    if project_id:
        payload["project_id"] = project_id

    response = requests.post(
        f"{BASE_URL}/api/v1/dicom/access/check",
        headers=headers,
        json=payload,
        timeout=10
    )

    return {
        "status_code": response.status_code,
        "data": response.json() if response.status_code == 200 else None,
        "error": response.text if response.status_code != 200 else None
    }


def get_user_accessible_study(user_id: int) -> Optional[Dict[str, Any]]:
    """사용자가 접근 가능한 Study 조회"""
    conn = get_db_connection()
    cursor = conn.cursor()

    cursor.execute("""
        SELECT
            pds.study_uid,
            pds.patient_id,
            pd.project_id,
            p.name as project_name
        FROM project_data pd
        JOIN project_data_study pds ON pd.study_id = pds.id
        JOIN security_project p ON pd.project_id = p.id
        JOIN security_user_project up ON p.id = up.project_id
        WHERE up.user_id = %s
        AND pd.resource_level = 'STUDY'
        LIMIT 1
    """, (user_id,))

    row = cursor.fetchone()
    cursor.close()
    conn.close()

    if row:
        return {
            "study_uid": row[0],
            "patient_id": row[1],
            "project_id": row[2],
            "project_name": row[3]
        }
    return None


def get_user_accessible_series(user_id: int) -> Optional[Dict[str, Any]]:
    """사용자가 접근 가능한 Series 조회"""
    conn = get_db_connection()
    cursor = conn.cursor()

    cursor.execute("""
        SELECT
            pds.series_uid,
            pdst.study_uid,
            pd.project_id,
            p.name as project_name
        FROM project_data pd
        JOIN project_data_series pds ON pd.series_id = pds.id
        JOIN project_data_study pdst ON pds.study_id = pdst.id
        JOIN security_project p ON pd.project_id = p.id
        JOIN security_user_project up ON p.id = up.project_id
        WHERE up.user_id = %s
        AND pd.resource_level = 'SERIES'
        LIMIT 1
    """, (user_id,))

    row = cursor.fetchone()
    cursor.close()
    conn.close()

    if row:
        return {
            "series_uid": row[0],
            "study_uid": row[1],
            "project_id": row[2],
            "project_name": row[3]
        }
    return None


def get_user_id(username: str) -> Optional[int]:
    """사용자 ID 조회"""
    conn = get_db_connection()
    cursor = conn.cursor()

    cursor.execute("SELECT id FROM security_user WHERE username = %s", (username,))
    row = cursor.fetchone()

    cursor.close()
    conn.close()

    return row[0] if row else None


# ===== 테스트 케이스 =====

def test_accessible_study():
    """테스트 1: 사용자가 접근 가능한 Study 확인"""
    print_test("사용자가 접근 가능한 Study 확인")

    # 1. 로그인
    token = TestAuth.login()
    user_id = get_user_id(TestConfig.ADMIN_USER)

    if not user_id:
        print_error("사용자를 찾을 수 없습니다")
        return

    # 2. 접근 가능한 Study 조회
    study_data = get_user_accessible_study(user_id)

    if not study_data:
        print_info("접근 가능한 Study가 없습니다 (테스트 스킵)")
        return

    print_info(f"테스트 Study UID: {study_data['study_uid']}")
    print_info(f"프로젝트: {study_data['project_name']} (ID: {study_data['project_id']})")

    # 3. API 호출
    result = check_data_access(token, study_data['study_uid'])

    # 4. 검증
    if result['status_code'] != 200:
        print_error(f"API 호출 실패: {result['status_code']}")
        print_info(f"에러: {result['error']}")
        return

    data = result['data']
    print_info(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)}")

    # 5. 프로젝트 목록 확인
    if 'projects' not in data:
        print_error("응답에 'projects' 필드가 없습니다")
        return

    projects = data['projects']

    if len(projects) == 0:
        print_error("접근 가능한 프로젝트가 없습니다")
        return

    # 6. 예상 프로젝트가 포함되어 있는지 확인
    project_ids = [p['project_id'] for p in projects]

    if study_data['project_id'] in project_ids:
        print_success(f"Study 접근 권한 확인 성공 (프로젝트 {study_data['project_id']} 포함)")
    else:
        print_error(f"예상 프로젝트 {study_data['project_id']}가 결과에 없습니다")


def test_accessible_series():
    """테스트 2: 사용자가 접근 가능한 Series 확인"""
    print_test("사용자가 접근 가능한 Series 확인")

    # 1. 로그인
    token = TestAuth.login()
    user_id = get_user_id(TestConfig.ADMIN_USER)

    if not user_id:
        print_error("사용자를 찾을 수 없습니다")
        return

    # 2. 접근 가능한 Series 조회
    series_data = get_user_accessible_series(user_id)

    if not series_data:
        print_info("접근 가능한 Series가 없습니다 (테스트 스킵)")
        return

    print_info(f"테스트 Study UID: {series_data['study_uid']}")
    print_info(f"테스트 Series UID: {series_data['series_uid']}")
    print_info(f"프로젝트: {series_data['project_name']} (ID: {series_data['project_id']})")

    # 3. API 호출
    result = check_data_access(
        token,
        series_data['study_uid'],
        series_data['series_uid']
    )

    # 4. 검증
    if result['status_code'] != 200:
        print_error(f"API 호출 실패: {result['status_code']}")
        print_info(f"에러: {result['error']}")
        return

    data = result['data']
    print_info(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)}")

    # 5. 프로젝트 목록 확인
    if 'projects' not in data:
        print_error("응답에 'projects' 필드가 없습니다")
        return

    projects = data['projects']

    if len(projects) == 0:
        print_error("접근 가능한 프로젝트가 없습니다")
        return

    # 6. 예상 프로젝트가 포함되어 있는지 확인
    project_ids = [p['project_id'] for p in projects]

    if series_data['project_id'] in project_ids:
        # 7. access_level이 SERIES인지 확인
        matching_project = next((p for p in projects if p['project_id'] == series_data['project_id']), None)
        if matching_project and matching_project.get('access_level') == 'SERIES':
            print_success(f"Series 접근 권한 확인 성공 (프로젝트 {series_data['project_id']}, SERIES 레벨)")
        else:
            print_success(f"Series 접근 권한 확인 성공 (프로젝트 {series_data['project_id']})")
    else:
        print_error(f"예상 프로젝트 {series_data['project_id']}가 결과에 없습니다")



def test_inaccessible_study():
    """테스트 3: 접근 불가능한 Study 확인"""
    print_test("접근 불가능한 Study 확인")

    # 1. 로그인
    token = TestAuth.login()

    # 2. 존재하지 않는 Study UID 사용
    fake_study_uid = "9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9.9"

    print_info(f"테스트 Study UID: {fake_study_uid}")

    # 3. API 호출
    result = check_data_access(token, fake_study_uid)

    # 4. 검증
    if result['status_code'] != 200:
        print_error(f"API 호출 실패: {result['status_code']}")
        print_info(f"에러: {result['error']}")
        return

    data = result['data']
    print_info(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)}")

    # 5. 프로젝트 목록이 비어있어야 함
    if 'projects' not in data:
        print_error("응답에 'projects' 필드가 없습니다")
        return

    projects = data['projects']

    if len(projects) == 0:
        print_success("접근 불가능한 Study에 대해 빈 프로젝트 목록 반환 성공")
    else:
        print_error(f"접근 불가능한 Study에 대해 프로젝트 목록이 비어있지 않습니다: {len(projects)}개")


def test_invalid_study_uid():
    """테스트 4: 잘못된 Study UID 처리"""
    print_test("잘못된 Study UID 처리")

    # 1. 로그인
    token = TestAuth.login()

    # 2. 잘못된 형식의 Study UID
    invalid_uid = "invalid-uid-format"

    print_info(f"테스트 Study UID: {invalid_uid}")

    # 3. API 호출
    result = check_data_access(token, invalid_uid)

    # 4. 검증 - 400 또는 200 (빈 결과) 모두 허용
    if result['status_code'] == 400:
        print_success("잘못된 UID에 대해 400 Bad Request 반환")
    elif result['status_code'] == 200:
        data = result['data']
        if 'projects' in data and len(data['projects']) == 0:
            print_success("잘못된 UID에 대해 빈 프로젝트 목록 반환")
        else:
            print_error(f"예상치 못한 응답: {data}")
    else:
        print_error(f"예상치 못한 상태 코드: {result['status_code']}")


def test_unauthorized_access():
    """테스트 5: 인증 실패 처리"""
    print_test("인증 실패 처리")

    # 1. 잘못된 토큰 사용
    invalid_token = "invalid.jwt.token"

    # 2. API 호출
    result = check_data_access(invalid_token, TestConfig.STUDY_UID)

    # 3. 검증
    if result['status_code'] == 401:
        print_success("잘못된 토큰에 대해 401 Unauthorized 반환")
    else:
        print_error(f"예상치 못한 상태 코드: {result['status_code']}")
        print_info(f"응답: {result['error']}")


def test_missing_token():
    """테스트 6: 토큰 없이 요청"""
    print_test("토큰 없이 요청")

    # 1. 토큰 없이 API 호출
    headers = {"Content-Type": "application/json"}
    payload = {"study_uid": TestConfig.STUDY_UID}

    response = requests.post(
        f"{BASE_URL}/api/v1/dicom/access/check",
        headers=headers,
        json=payload,
        timeout=10
    )

    # 2. 검증
    if response.status_code == 401:
        print_success("토큰 없이 요청 시 401 Unauthorized 반환")
    else:
        print_error(f"예상치 못한 상태 코드: {response.status_code}")


def test_response_format():
    """테스트 7: 응답 형식 검증"""
    print_test("응답 형식 검증")

    # 1. 로그인
    token = TestAuth.login()
    user_id = get_user_id(TestConfig.ADMIN_USER)

    if not user_id:
        print_error("사용자를 찾을 수 없습니다")
        return

    # 2. 접근 가능한 Study 조회
    study_data = get_user_accessible_study(user_id)

    if not study_data:
        print_info("접근 가능한 Study가 없습니다 (테스트 스킵)")
        return

    # 3. API 호출
    result = check_data_access(token, study_data['study_uid'])

    # 4. 검증
    if result['status_code'] != 200:
        print_error(f"API 호출 실패: {result['status_code']}")
        return

    data = result['data']

    # 5. 필수 필드 확인
    if 'projects' not in data:
        print_error("응답에 'projects' 필드가 없습니다")
        return

    projects = data['projects']

    if len(projects) == 0:
        print_info("프로젝트 목록이 비어있습니다")
        print_success("응답 형식 검증 성공 (빈 목록)")
        return

    # 6. 프로젝트 객체 필드 확인
    required_fields = ['project_id', 'project_name', 'access_level', 'reason']

    for project in projects:
        missing_fields = [field for field in required_fields if field not in project]

        if missing_fields:
            print_error(f"프로젝트 객체에 필수 필드가 없습니다: {missing_fields}")
            return

    print_success(f"응답 형식 검증 성공 ({len(projects)}개 프로젝트)")


def test_specific_project_access():
    """테스트 8: 특정 프로젝트 접근 권한 확인"""
    print_test("특정 프로젝트 접근 권한 확인 (project_id 파라미터)")

    # 1. 로그인
    token = TestAuth.login()
    user_id = get_user_id(TestConfig.ADMIN_USER)

    if not user_id:
        print_error("사용자를 찾을 수 없습니다")
        return

    # 2. 접근 가능한 Study 조회
    study_data = get_user_accessible_study(user_id)

    if not study_data:
        print_info("접근 가능한 Study가 없습니다 (테스트 스킵)")
        return

    project_id = study_data['project_id']
    study_uid = study_data['study_uid']

    print_info(f"테스트 Study UID: {study_uid}")
    print_info(f"테스트 Project ID: {project_id}")

    # 3. 특정 프로젝트로 API 호출
    result = check_data_access(token, study_uid, project_id=project_id)

    # 4. 검증
    if result['status_code'] != 200:
        print_error(f"API 호출 실패: {result['status_code']}")
        print_info(f"응답: {result['error']}")
        return

    data = result['data']
    print_info(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)}")

    # 5. 프로젝트 목록 확인
    if not data.get('accessible'):
        print_error("접근 불가능으로 표시됨")
        return

    projects = data.get('projects', [])

    if len(projects) != 1:
        print_error(f"프로젝트 개수가 1개가 아닙니다: {len(projects)}개")
        return

    if projects[0]['project_id'] != project_id:
        print_error(f"반환된 프로젝트 ID가 다릅니다: {projects[0]['project_id']} != {project_id}")
        return

    print_success(f"특정 프로젝트 접근 권한 확인 성공 (프로젝트 {project_id})")


def test_wrong_project_access():
    """테스트 9: 잘못된 프로젝트 ID로 접근 시도"""
    print_test("잘못된 프로젝트 ID로 접근 시도")

    # 1. 로그인
    token = TestAuth.login()
    user_id = get_user_id(TestConfig.ADMIN_USER)

    if not user_id:
        print_error("사용자를 찾을 수 없습니다")
        return

    # 2. 접근 가능한 Study 조회
    study_data = get_user_accessible_study(user_id)

    if not study_data:
        print_info("접근 가능한 Study가 없습니다 (테스트 스킵)")
        return

    study_uid = study_data['study_uid']
    wrong_project_id = 99999  # 존재하지 않는 프로젝트 ID

    print_info(f"테스트 Study UID: {study_uid}")
    print_info(f"잘못된 Project ID: {wrong_project_id}")

    # 3. 잘못된 프로젝트로 API 호출
    result = check_data_access(token, study_uid, project_id=wrong_project_id)

    # 4. 검증
    if result['status_code'] != 200:
        print_error(f"API 호출 실패: {result['status_code']}")
        return

    data = result['data']
    print_info(f"응답: {json.dumps(data, indent=2, ensure_ascii=False)}")

    # 5. 접근 불가능 확인
    if data.get('accessible'):
        print_error("접근 가능으로 표시됨 (예상: 접근 불가)")
        return

    projects = data.get('projects', [])

    if len(projects) != 0:
        print_error(f"프로젝트 목록이 비어있지 않습니다: {len(projects)}개")
        return

    print_success("잘못된 프로젝트 ID에 대해 빈 프로젝트 목록 반환")


# ===== 메인 실행 =====

def main():
    """모든 테스트 실행"""
    TestPrinter.print_header("🧪 DICOM Data Access Check API E2E 테스트")

    print_info(f"서버 URL: {BASE_URL}")
    print_info(f"테스트 사용자: {TestConfig.ADMIN_USER}")
    print("")

    # 테스트 실행
    test_accessible_study()
    test_accessible_series()
    test_inaccessible_study()
    test_invalid_study_uid()
    test_unauthorized_access()
    test_missing_token()
    test_response_format()
    test_specific_project_access()
    test_wrong_project_access()

    # 결과 출력
    print("\n" + "="*70)
    print("📊 테스트 결과 요약")
    print("="*70)
    print(f"✅ 통과: {test_results['passed']}")
    print(f"❌ 실패: {test_results['failed']}")
    print(f"📝 총계: {test_results['total']}")
    print("")

    if test_results['failed'] == 0:
        print("🎉 모든 테스트 통과!")
        sys.exit(0)
    else:
        print("❌ 일부 테스트 실패")
        sys.exit(1)


if __name__ == "__main__":
    main()


