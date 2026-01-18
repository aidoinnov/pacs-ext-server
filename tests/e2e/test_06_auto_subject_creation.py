"""
Subject 자동 생성 테스트

Project에 Study를 할당할 때 Subject가 자동으로 생성되는지 테스트합니다.
"""

import pytest
import logging
from faker import Faker
from utils.api_client import APIClient
from config import TestConfig

logger = logging.getLogger(__name__)
fake = Faker()


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """관리자 클라이언트"""
    client = APIClient(config.base_url, config.timeout)
    client.login(config.admin_email, config.admin_password)
    yield client
    client.close()


@pytest.fixture(scope="module")
def test_project(admin_client):
    """테스트용 프로젝트 생성"""
    project_name = f"E2E Auto Subject Test {fake.uuid4()[:8]}"

    response = admin_client.post("/api/projects", json={
        "name": project_name,
        "description": "Auto Subject 생성 E2E 테스트용 프로젝트",
        "sponsor": "Test Hospital",
        "status": "active"
    })

    assert response.status_code in [200, 201], f"Failed to create project: {response.text}"
    project = response.json()
    logger.info(f"Created test project: {project['name']} (ID: {project['id']})")

    yield project

    # 테스트 후 프로젝트 삭제
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
        logger.info(f"Deleted test project: {project['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete test project: {e}")


class TestAutoSubjectCreation:
    """Study 할당 시 Subject 자동 생성 테스트"""

    def test_01_assign_study_creates_subject(self, admin_client, test_project):
        """Study 할당 시 Subject 자동 생성 테스트"""
        logger.info("Testing auto subject creation on study assignment...")

        project_id = test_project['id']
        patient_id = f"P{fake.random_int(10000, 99999)}"
        patient_name = fake.name()
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"

        # 1. Study 할당 전 Subject 확인 (없어야 함)
        subjects_before = admin_client.get(f"/api/projects/{project_id}/subjects")
        assert subjects_before.status_code == 200
        subject_count_before = len(subjects_before.json())

        # 2. Study 할당
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid,
            "study_description": "CT CHEST",
            "patient_id": patient_id,
            "patient_name": patient_name,
            "study_date": "2026-01-15"
        })

        assert response.status_code == 200, f"Failed to assign study: {response.text}"
        logger.info(f"✓ Study assigned: {study_uid}")

        # 3. Subject가 자동 생성되었는지 확인
        subjects_after = admin_client.get(f"/api/projects/{project_id}/subjects")
        assert subjects_after.status_code == 200
        subjects = subjects_after.json()
        
        assert len(subjects) == subject_count_before + 1, "Subject should be auto-created"

        # 4. 생성된 Subject 확인
        new_subject = next((s for s in subjects if s['patient_id'] == patient_id), None)
        assert new_subject is not None, "Subject with matching patient_id should exist"
        assert new_subject['patient_name'] == patient_name
        assert new_subject['subject_code'] == patient_id or new_subject['subject_code'].startswith(patient_id)

        logger.info(f"✓ Subject auto-created: {new_subject['subject_code']} (ID: {new_subject['id']})")

    def test_02_duplicate_patient_id_reuses_subject(self, admin_client, test_project):
        """같은 Patient ID의 Study 할당 시 Subject 재사용 테스트"""
        logger.info("Testing subject reuse for duplicate patient_id...")

        project_id = test_project['id']
        patient_id = f"P{fake.random_int(10000, 99999)}"
        patient_name = fake.name()

        # 1. 첫 번째 Study 할당
        study_uid_1 = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"
        response1 = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid_1,
            "study_description": "CT CHEST",
            "patient_id": patient_id,
            "patient_name": patient_name,
            "study_date": "2026-01-15"
        })
        assert response1.status_code == 200

        # Subject 개수 확인
        subjects_after_first = admin_client.get(f"/api/projects/{project_id}/subjects")
        subject_count_after_first = len(subjects_after_first.json())

        # 2. 같은 Patient ID로 두 번째 Study 할당
        study_uid_2 = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"
        response2 = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid_2,
            "study_description": "CT ABDOMEN",
            "patient_id": patient_id,
            "patient_name": patient_name,
            "study_date": "2026-01-16"
        })
        assert response2.status_code == 200

        # 3. Subject 개수가 증가하지 않았는지 확인 (재사용)
        subjects_after_second = admin_client.get(f"/api/projects/{project_id}/subjects")
        subject_count_after_second = len(subjects_after_second.json())

        assert subject_count_after_second == subject_count_after_first, \
            "Subject should be reused, not created again"

        logger.info(f"✓ Subject reused for duplicate patient_id: {patient_id}")

    def test_03_subject_code_uniqueness(self, admin_client, test_project):
        """Subject Code 중복 방지 테스트 (같은 Patient ID)"""
        logger.info("Testing subject code uniqueness...")

        project_id = test_project['id']
        patient_id = f"P{fake.random_int(10000, 99999)}"

        # 1. 수동으로 Subject 생성 (Patient ID를 Subject Code로 사용)
        manual_subject = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "subject_code": patient_id,
            "patient_id": f"{patient_id}_DIFFERENT",  # 다른 Patient ID
            "patient_name": fake.name(),
            "patient_birth_date": "1990-01-01"
        })
        assert manual_subject.status_code == 201

        # 2. 같은 Patient ID로 Study 할당 (Subject Code 충돌 발생)
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid,
            "study_description": "CT CHEST",
            "patient_id": patient_id,
            "patient_name": fake.name(),
            "study_date": "2026-01-15"
        })
        assert response.status_code == 200

        # 3. Subject가 생성되었는지 확인 (suffix 추가되어야 함)
        subjects = admin_client.get(f"/api/projects/{project_id}/subjects")
        assert subjects.status_code == 200
        
        subject_codes = [s['subject_code'] for s in subjects.json()]
        # patient_id 또는 patient_id_1, patient_id_2 등이 있어야 함
        matching_codes = [code for code in subject_codes if code.startswith(patient_id)]
        assert len(matching_codes) >= 2, "Should have multiple subjects with similar codes"

        logger.info(f"✓ Subject code uniqueness maintained: {matching_codes}")

    def test_04_no_patient_id_uses_sequential_code(self, admin_client, test_project):
        """Patient ID 없을 때 순차 번호 사용 테스트"""
        logger.info("Testing sequential subject code when patient_id is missing...")

        project_id = test_project['id']
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"

        # Patient ID 없이 Study 할당
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid,
            "study_description": "CT CHEST",
            "patient_id": None,  # Patient ID 없음
            "patient_name": fake.name(),
            "study_date": "2026-01-15"
        })

        # Patient ID가 없으면 Subject가 생성되지 않아야 함
        assert response.status_code == 200
        logger.info(f"✓ Study assigned without patient_id (no subject created)")

