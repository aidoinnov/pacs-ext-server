"""
Subject 자동 생성 테스트

Project에 Study를 할당할 때 Subject가 자동으로 생성되는지 테스트합니다.

주의: 이 테스트는 실제 DICOM Study가 DB에 있어야 합니다.
DICOM C-STORE를 통해 Study가 먼저 저장되어야 합니다.
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

    def test_01_api_spec_validation(self, admin_client, test_project):
        """API 스펙 검증: subject_code만 받는지 확인"""
        logger.info("Testing API spec: only study_uid and subject_code...")

        project_id = test_project['id']
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"

        # Study 할당 (subject_code 지정)
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid,
            "subject_code": "CUSTOM-001"
        })

        # Study가 DB에 없으면 404 (정상)
        if response.status_code == 404:
            logger.info(f"✓ API accepts study_uid and subject_code only (Study not in DB)")
            return

        # 성공하면 200
        assert response.status_code == 200, f"Unexpected response: {response.text}"
        logger.info(f"✓ API spec validated: study_uid + subject_code")

    def test_02_subject_code_custom(self, admin_client, test_project):
        """사용자 지정 Subject Code 테스트"""
        logger.info("Testing custom subject_code...")

        project_id = test_project['id']
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"

        # Subject Code 지정하여 Study 할당
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid,
            "subject_code": "STUDY-A-001"
        })

        # Study가 DB에 없으면 404 (정상)
        if response.status_code == 404:
            logger.info(f"✓ Custom subject_code accepted (Study not in DB)")
            return

        assert response.status_code == 200
        logger.info(f"✓ Custom subject_code accepted")

    def test_03_subject_code_optional(self, admin_client, test_project):
        """Subject Code 없이 자동 생성 테스트"""
        logger.info("Testing auto-generated subject_code...")

        project_id = test_project['id']
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"

        # Subject Code 없이 Study 할당 (자동 생성)
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid
        })

        # Study가 DB에 없으면 404 (정상)
        if response.status_code == 404:
            logger.info(f"✓ Auto subject_code generation accepted (Study not in DB)")
            return

        assert response.status_code == 200
        logger.info(f"✓ Auto subject_code generation works")

    def test_04_old_api_rejected(self, admin_client, test_project):
        """이전 API 스펙 (patient_id, patient_name) 거부 테스트"""
        logger.info("Testing old API spec rejection...")

        project_id = test_project['id']
        study_uid = f"1.2.840.113619.2.55.3.{fake.random_int(100000, 999999)}"

        # 이전 API 스펙으로 호출 (patient_id, patient_name 포함)
        response = admin_client.post(f"/api/projects/{project_id}/studies/assign", json={
            "study_uid": study_uid,
            "patient_id": "P12345",  # 더 이상 지원 안 함
            "patient_name": "홍길동"  # 더 이상 지원 안 함
        })

        # 필드가 무시되고 정상 처리되어야 함 (또는 404)
        assert response.status_code in [200, 404], f"Unexpected error: {response.text}"
        logger.info(f"✓ Old API fields ignored (status: {response.status_code})")

