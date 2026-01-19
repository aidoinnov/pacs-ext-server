"""
RECIST Lesion Assignment E2E 테스트 (방안 2: 하이브리드)

테스트 시나리오:
1. Annotation에 lesion_type + lesion_number 직접 할당
2. 서버가 자동으로 recist_lesion 테이블 관리 (향후 구현)
3. 여러 TimePoint에 걸쳐 동일 Lesion 추적
4. NEW Lesion 발견 시나리오
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
    """테스트 사용자 클라이언트 (reader1_user)"""
    client = APIClient(config.base_url, config.timeout)
    # reader1_user 계정 사용
    client.login("reader1_user", "Qlalfqjsgh1!")
    yield client
    client.close()


@pytest.fixture(scope="module")
def test_project(admin_client):
    """테스트용 프로젝트 생성"""
    project_name = f"E2E Lesion Assignment Test {fake.uuid4()[:8]}"

    response = admin_client.post("/api/projects", json={
        "name": project_name,
        "description": "Lesion Assignment E2E 테스트용 프로젝트",
        "sponsor": "Test Hospital",
        "status": "active"
    })

    assert response.status_code in [200, 201], f"Failed to create project: {response.text}"
    project = response.json()
    logger.info(f"Created test project: {project['name']} (ID: {project['id']})")

    # reader1_user를 프로젝트 멤버로 추가
    # 먼저 사용자 정보 조회
    me_response = admin_client.get("/api/users/me")
    if me_response.status_code == 200:
        user = me_response.json()
        user_id = user.get('id')

        # 프로젝트 멤버로 추가 (role_id=196: PROJECT_ADMIN)
        member_response = admin_client.post(f"/api/projects/{project['id']}/members", json={
            "user_id": user_id,
            "role_id": 196  # PROJECT_ADMIN role
        })

        if member_response.status_code in [200, 201]:
            logger.info(f"Added user {user_id} to project {project['id']}")
        else:
            logger.warning(f"Failed to add user to project: {member_response.text}")

    yield project

    # 테스트 후 프로젝트 삭제
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
        logger.info(f"Deleted test project: {project['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete test project: {e}")


@pytest.fixture(scope="module")
def test_subject(admin_client, test_project):
    """테스트용 Subject 생성"""
    project_id = test_project['id']
    subject_code = f"LESION{fake.random_int(1000, 9999)}"

    response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
        "subject_code": subject_code,
        "patient_id": f"P{fake.random_int(10000, 99999)}",
        "patient_name": fake.name(),
        "patient_birth_date": "1990-01-01"
    })

    assert response.status_code == 201, f"Failed to create subject: {response.text}"
    subject = response.json()
    logger.info(f"Created test subject: {subject_code} (ID: {subject['id']})")

    yield subject


class TestLesionAssignment:
    """Lesion Assignment 테스트"""

    def test_01_create_annotation_with_lesion_type(self, admin_client, test_project):
        """Annotation 생성 시 lesion_type과 lesion_number 할당"""
        project_id = test_project['id']
        
        # Annotation 생성 (Target Lesion 1)
        response = admin_client.post("/api/annotations", json={
            "project_id": project_id,
            "study_instance_uid": f"1.2.840.{fake.random_int(100000, 999999)}",
            "annotation_data": {
                "type": "Length",
                "points": [[100, 100], [200, 200]],
                "length": 141.42
            },
            "tool_name": "Length",
            "lesion_type": "TARGET",
            "lesion_number": 1,
            "label": "Liver lesion"
        })

        assert response.status_code == 201, f"Failed to create annotation: {response.text}"
        annotation = response.json()
        
        # 검증
        assert annotation['lesion_type'] == "TARGET"
        assert annotation['lesion_number'] == 1
        assert annotation['label'] == "Liver lesion"

        logger.info(f"✅ Created annotation with Target Lesion 1 (ID: {annotation['id']})")

    def test_02_update_annotation_lesion_type(self, admin_client, test_project):
        """Annotation 업데이트로 lesion_type 변경"""
        project_id = test_project['id']
        
        # 1. Annotation 생성 (lesion 없이)
        response = admin_client.post("/api/annotations", json={
            "project_id": project_id,
            "study_instance_uid": f"1.2.840.{fake.random_int(100000, 999999)}",
            "annotation_data": {
                "type": "Length",
                "points": [[50, 50], [150, 150]]
            },
            "tool_name": "Length"
        })

        assert response.status_code == 201
        annotation = response.json()
        annotation_id = annotation['id']
        
        # 2. Lesion 할당 (Target Lesion 2)
        response = admin_client.put(f"/api/annotations/{annotation_id}", json={
            "lesion_type": "TARGET",
            "lesion_number": 2,
            "label": "Lung lesion"
        })

        assert response.status_code == 200, f"Failed to update annotation: {response.text}"
        updated = response.json()

        # 검증
        assert updated['lesion_type'] == "TARGET"
        assert updated['lesion_number'] == 2
        assert updated['label'] == "Lung lesion"

        logger.info(f"✅ Updated annotation to Target Lesion 2 (ID: {annotation_id})")

    def test_03_non_target_lesion(self, admin_client, test_project):
        """Non-target Lesion 할당"""
        project_id = test_project['id']

        response = admin_client.post("/api/annotations", json={
            "project_id": project_id,
            "study_instance_uid": f"1.2.840.{fake.random_int(100000, 999999)}",
            "annotation_data": {
                "type": "Point",
                "points": [[300, 300]]
            },
            "tool_name": "Point",
            "lesion_type": "NON_TARGET",
            "lesion_number": 1,
            "label": "Bone metastasis"
        })

        assert response.status_code == 201
        annotation = response.json()

        assert annotation['lesion_type'] == "NON_TARGET"
        assert annotation['lesion_number'] == 1

        logger.info(f"✅ Created Non-target Lesion 1 (ID: {annotation['id']})")

    def test_04_new_lesion(self, admin_client, test_project):
        """NEW Lesion 할당 (Follow-up에서 발견)"""
        project_id = test_project['id']

        response = admin_client.post("/api/annotations", json={
            "project_id": project_id,
            "study_instance_uid": f"1.2.840.{fake.random_int(100000, 999999)}",
            "annotation_data": {
                "type": "Length",
                "points": [[400, 400], [500, 500]]
            },
            "tool_name": "Length",
            "lesion_type": "TARGET_NEW",
            "lesion_number": 1,
            "label": "New liver lesion"
        })

        assert response.status_code == 201
        annotation = response.json()

        assert annotation['lesion_type'] == "TARGET_NEW"
        assert annotation['lesion_number'] == 1

        logger.info(f"✅ Created Target New Lesion 1 (ID: {annotation['id']})")

    def test_05_query_annotations_by_lesion_type(self, admin_client, test_project):
        """Lesion type별 Annotation 조회"""
        project_id = test_project['id']
        study_uid = f"1.2.840.{fake.random_int(100000, 999999)}"

        # 여러 Annotation 생성
        for i in range(3):
            admin_client.post("/api/annotations", json={
                "project_id": project_id,
                "study_instance_uid": study_uid,
                "annotation_data": {"type": "Point", "points": [[i*100, i*100]]},
                "tool_name": "Point",
                "lesion_type": "TARGET",
                "lesion_number": i + 1
            })

        # Study의 모든 Annotation 조회 (올바른 API 엔드포인트 사용)
        response = admin_client.get("/api/annotations", params={
            "study_instance_uid": study_uid,
            "project_id": project_id
        })
        assert response.status_code == 200

        data = response.json()
        annotations = data.get('annotations', []) if isinstance(data, dict) else data
        target_lesions = [a for a in annotations if a.get('lesion_type') == 'TARGET']

        assert len(target_lesions) == 3
        assert sorted([a['lesion_number'] for a in target_lesions]) == [1, 2, 3]

        logger.info(f"✅ Queried {len(target_lesions)} Target Lesions")

    def test_06_remove_lesion_assignment(self, admin_client, test_project):
        """Lesion 할당 변경 테스트"""
        project_id = test_project['id']

        # 1. Annotation 생성 (Target Lesion 5)
        response = admin_client.post("/api/annotations", json={
            "project_id": project_id,
            "study_instance_uid": f"1.2.840.{fake.random_int(100000, 999999)}",
            "annotation_data": {"type": "Point", "points": [[100, 100]]},
            "tool_name": "Point",
            "lesion_type": "TARGET",
            "lesion_number": 5
        })

        assert response.status_code == 201
        annotation = response.json()
        annotation_id = annotation['id']

        assert annotation['lesion_type'] == "TARGET"
        assert annotation['lesion_number'] == 5

        # 2. Lesion 타입 변경 (Target → Non-target)
        response = admin_client.put(f"/api/annotations/{annotation_id}", json={
            "lesion_type": "NON_TARGET",
            "lesion_number": 1
        })

        assert response.status_code == 200
        updated = response.json()

        assert updated['lesion_type'] == "NON_TARGET"
        assert updated['lesion_number'] == 1

        logger.info(f"✅ Changed lesion assignment: TARGET #5 → NON_TARGET #1")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])


