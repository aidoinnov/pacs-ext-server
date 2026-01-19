"""
E2E 테스트: TimePoint with Studies API (X축 + Y축)

테스트 시나리오:
1. X축 API: Subject의 TimePoint와 Study 목록 조회 (Unassigned 포함)
2. X축 API: Subject의 TimePoint와 Study 목록 조회 (Unassigned 제외)
3. Y축 API: TimePoint의 Annotation 목록 조회
4. Y축 API: Lesion 정보 포함 확인
"""

import pytest
import logging
from typing import Dict, Any
from utils.api_client import APIClient
from config import TestConfig

logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """테스트 사용자 클라이언트 (reader1_user)"""
    client = APIClient(config.base_url, config.timeout)
    client.login("reader1_user", "Qlalfqjsgh1!")
    yield client
    client.close()


@pytest.fixture(scope="module")
def test_data(admin_client) -> Dict[str, Any]:
    """테스트 데이터 준비 - 동적으로 생성"""
    # 1. Project 조회 (첫 번째 프로젝트 사용)
    projects_response = admin_client.get("/api/projects")
    assert projects_response.status_code == 200, f"Failed to get projects: {projects_response.text}"
    projects_data = projects_response.json()
    projects = projects_data.get("projects", [])
    assert len(projects) > 0, "No projects found"
    project_id = projects[0]["id"]
    logger.info(f"Using project_id: {project_id}")

    # 2. Subject 조회 또는 생성
    subjects_response = admin_client.get(f"/api/projects/{project_id}/subjects")
    if subjects_response.status_code == 200:
        subjects = subjects_response.json()
        if len(subjects) > 0:
            subject = subjects[0]
            subject_id = subject["id"]
            subject_code = subject["subject_code"]
            logger.info(f"Using existing subject: {subject_id} ({subject_code})")
        else:
            # Subject 생성
            create_subject_response = admin_client.post(
                f"/api/projects/{project_id}/subjects",
                json={
                    "subject_code": "TEST_TP_STUDIES",
                    "patient_id": "P12345",
                    "patient_name": "Test Patient",
                    "patient_birth_date": "1990-01-01"
                }
            )
            assert create_subject_response.status_code == 201, f"Failed to create subject: {create_subject_response.text}"
            subject = create_subject_response.json()
            subject_id = subject["id"]
            subject_code = subject["subject_code"]
            logger.info(f"Created new subject: {subject_id} ({subject_code})")
    else:
        pytest.skip(f"Cannot access subjects: {subjects_response.status_code}")

    # 3. TimePoint 생성 (없으면)
    tp_response = admin_client.get(f"/api/subjects/{subject_id}/timepoints")
    if tp_response.status_code == 200:
        timepoints = tp_response.json()
        if len(timepoints) == 0:
            # Baseline TimePoint 생성
            baseline_response = admin_client.post(
                f"/api/subjects/{subject_id}/timepoints",
                json={
                    "name": "Baseline",
                    "visit_type": "Baseline",
                    "visit_date": "2026-01-01",
                    "order_index": 0
                }
            )
            if baseline_response.status_code == 201:
                baseline = baseline_response.json()
                logger.info(f"Created Baseline TimePoint: {baseline['id']}")
                timepoints.append(baseline)

            # TP1 TimePoint 생성
            tp1_response = admin_client.post(
                f"/api/subjects/{subject_id}/timepoints",
                json={
                    "name": "TP1",
                    "visit_type": "Visit",
                    "visit_date": "2026-02-01",
                    "order_index": 1
                }
            )
            if tp1_response.status_code == 201:
                tp1 = tp1_response.json()
                logger.info(f"Created TP1 TimePoint: {tp1['id']}")
                timepoints.append(tp1)
    else:
        logger.warning(f"Failed to get timepoints: {tp_response.status_code}")
        timepoints = []

    # 4. Study 데이터 확인 (Unassigned Studies)
    unassigned_response = admin_client.get(f"/api/subjects/{subject_id}/studies/unassigned")
    if unassigned_response.status_code == 200:
        unassigned_studies = unassigned_response.json()
        logger.info(f"Found {len(unassigned_studies)} unassigned studies")
    else:
        unassigned_studies = []
        logger.warning(f"Failed to get unassigned studies: {unassigned_response.status_code}")

    return {
        "project_id": project_id,
        "subject_id": subject_id,
        "subject_code": subject_code,
        "timepoints": timepoints,
        "unassigned_studies": unassigned_studies
    }


class TestTimePointWithStudies:
    """TimePoint with Studies API 테스트"""

    def test_01_get_timepoints_with_studies_include_unassigned(self, admin_client, test_data):
        """X축 API: TimePoint와 Study 목록 조회 (Unassigned 포함)"""
        subject_id = test_data["subject_id"]

        response = admin_client.get(
            f"/api/subjects/{subject_id}/timepoints-with-studies",
            params={"include_unassigned": True}
        )

        assert response.status_code == 200, f"Failed: {response.text}"
        data = response.json()

        # 응답 구조 검증
        assert "subject_id" in data
        assert "subject_code" in data
        assert "timepoints" in data
        assert "unassigned_studies" in data

        assert data["subject_id"] == subject_id
        assert data["subject_code"] == test_data["subject_code"]

        print(f"\n✅ Subject: {data['subject_code']} (ID: {subject_id})")
        print(f"✅ TimePoints: {len(data['timepoints'])}")
        print(f"✅ Unassigned Studies: {len(data['unassigned_studies'])}")

        # TimePoint가 생성되었는지 확인
        assert len(data["timepoints"]) >= 2, "At least 2 TimePoints should be created (Baseline, TP1)"

        # TimePoint 구조 검증
        for tp in data["timepoints"]:
            assert "id" in tp
            assert "name" in tp
            assert "studies" in tp
            print(f"  - {tp['name']}: {len(tp['studies'])} studies")

    def test_02_get_timepoints_with_studies_exclude_unassigned(self, admin_client, test_data):
        """X축 API: TimePoint와 Study 목록 조회 (Unassigned 제외)"""
        subject_id = test_data["subject_id"]
        
        response = admin_client.get(
            f"/api/subjects/{subject_id}/timepoints-with-studies",
            params={"include_unassigned": False}
        )
        
        assert response.status_code == 200, f"Failed: {response.text}"
        data = response.json()
        
        # Unassigned studies가 비어있어야 함
        assert len(data["unassigned_studies"]) == 0
        print(f"\n✅ Unassigned studies excluded: {len(data['unassigned_studies'])}")

    def test_03_get_annotations_by_timepoint(self, admin_client, test_data):
        """Y축 API: TimePoint의 Annotation 목록 조회"""
        timepoints = test_data["timepoints"]
        
        if len(timepoints) == 0:
            pytest.skip("No timepoints available")
        
        timepoint_id = timepoints[0]["id"]
        
        response = admin_client.get(f"/api/timepoints/{timepoint_id}/annotations")
        
        assert response.status_code == 200, f"Failed: {response.text}"
        annotations = response.json()
        
        print(f"\n✅ Annotations in TimePoint {timepoint_id}: {len(annotations)}")
        
        # Annotation 구조 검증
        for ann in annotations:
            assert "id" in ann
            assert "study_instance_uid" in ann
            assert "annotation_data" in ann
            
            # Lesion 정보 확인 (있을 수도, 없을 수도 있음)
            if ann.get("lesion_type"):
                print(f"  - Annotation {ann['id']}: {ann['lesion_type']} #{ann.get('lesion_number', 'N/A')}")

    def test_04_verify_lesion_info_in_annotations(self, admin_client, test_data):
        """Y축 API: Lesion 정보 포함 확인"""
        timepoints = test_data["timepoints"]
        
        if len(timepoints) == 0:
            pytest.skip("No timepoints available")
        
        timepoint_id = timepoints[0]["id"]
        
        response = admin_client.get(f"/api/timepoints/{timepoint_id}/annotations")
        assert response.status_code == 200
        annotations = response.json()
        
        # Lesion 정보가 있는 Annotation 찾기
        lesion_annotations = [a for a in annotations if a.get("lesion_type")]
        
        print(f"\n✅ Total annotations: {len(annotations)}")
        print(f"✅ Lesion annotations: {len(lesion_annotations)}")
        
        # Lesion 정보 검증
        for ann in lesion_annotations:
            assert ann["lesion_type"] in ["TARGET", "NON_TARGET", "TARGET_NEW", "NON_TARGET_NEW"]
            assert isinstance(ann.get("lesion_number"), int)
            assert ann["lesion_number"] >= 1
            print(f"  - {ann['lesion_type']} #{ann['lesion_number']}")

