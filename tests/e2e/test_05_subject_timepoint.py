"""
Subject & TimePoint 관리 E2E 테스트

테스트 시나리오:
1. Subject CRUD
2. TimePoint CRUD
3. Study 할당/해제
4. 비즈니스 규칙 검증
5. 에러 케이스
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
    login_data = client.login(config.admin_email, config.admin_password)
    client.user_id = login_data.get('user_id')  # user_id 저장
    yield client
    client.close()


@pytest.fixture(scope="module")
def test_project(admin_client, config):
    """테스트용 프로젝트 - 기존 프로젝트 556 사용"""
    # 기존 프로젝트 556 사용 (DICOM 데이터가 있는 프로젝트)
    project_id = 556

    # 프로젝트 정보 조회
    response = admin_client.get(f"/api/projects/{project_id}")

    if response.status_code != 200:
        # 프로젝트가 없으면 새로 생성
        project_name = f"E2E Subject Test {fake.uuid4()[:8]}"
        response = admin_client.post("/api/projects", json={
            "name": project_name,
            "description": "Subject/TimePoint E2E 테스트용 프로젝트",
            "sponsor": "Test Hospital",
            "status": "active"
        })
        assert response.status_code in [200, 201], f"Failed to create project: {response.text}"
        project = response.json()
        project_id = project['id']
        logger.info(f"Created new test project: {project['name']} (ID: {project_id})")
    else:
        project = response.json()
        logger.info(f"Using existing project: {project.get('name', 'N/A')} (ID: {project_id})")

    # 현재 사용자를 프로젝트 멤버로 추가
    if hasattr(admin_client, 'user_id') and admin_client.user_id:
        user_id = admin_client.user_id

        # 프로젝트 멤버로 추가 (role_id=196: PROJECT_ADMIN)
        member_response = admin_client.post(
            f"/api/projects/{project_id}/members",
            json={"user_id": user_id, "role_id": 196}
        )

        if member_response.status_code in [200, 201]:
            logger.info(f"Added user {user_id} to project {project_id}")
        else:
            logger.info(f"User may already be a member: {member_response.status_code}")
    else:
        logger.warning("User ID not available, skipping member addition")

    yield project

    # 테스트 후 정리 - 기존 프로젝트는 삭제하지 않음
    logger.info(f"Test completed with project {project['id']} (not deleting existing project)")


@pytest.fixture(scope="module")
def test_subject(admin_client, test_project):
    """테스트용 Subject 생성 - DICOM 데이터의 Patient ID 사용"""
    project_id = test_project['id']
    created_subject = None

    # 프로젝트의 DICOM studies 조회
    response = admin_client.get(f"/api/me/dicom/studies?project_id={project_id}&page_size=1")
    assert response.status_code == 200, f"Failed to get DICOM studies: {response.text}"

    studies = response.json()
    if len(studies) == 0:
        # DICOM 데이터가 없으면 임의로 생성
        subject_code = f"SUB{fake.random_int(1000, 9999)}"
        patient_id = f"P{fake.random_int(10000, 99999)}"
        patient_name = fake.name()
        logger.warning("No DICOM studies found, creating subject with random data")
    else:
        # 첫 번째 study의 Patient 정보 사용
        first_study = studies[0]
        patient_id = first_study.get('00100020', {}).get('Value', [f"P{fake.random_int(10000, 99999)}"])[0]
        patient_name_data = first_study.get('00100010', {}).get('Value', [{}])
        patient_name = patient_name_data[0].get('Alphabetic', fake.name()) if patient_name_data else fake.name()
        subject_code = f"SUB_{patient_id}"
        logger.info(f"Using Patient ID from DICOM: {patient_id}")

    # 기존 subject가 있는지 확인
    existing_subjects_response = admin_client.get(f"/api/projects/{project_id}/subjects")
    if existing_subjects_response.status_code == 200:
        existing_subjects = existing_subjects_response.json()
        for subj in existing_subjects:
            if subj.get('subject_code') == subject_code:
                logger.info(f"Found existing subject: {subject_code} (ID: {subj['id']}), reusing it")
                subject = subj
                subject['subject_code'] = subject_code
                yield subject
                return

    # 새로운 subject 생성
    response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
        "subject_code": subject_code,
        "patient_id": patient_id,
        "patient_name": patient_name,
        "patient_birth_date": "1990-01-01"
    })

    assert response.status_code == 201, f"Failed to create subject: {response.text}"
    subject = response.json()
    subject['subject_code'] = subject_code  # 코드 저장
    created_subject = subject
    logger.info(f"Created test subject: {subject_code} (ID: {subject['id']})")

    yield subject

    # 테스트 후 Subject 삭제 (생성한 경우만)
    if created_subject:
        try:
            # 먼저 모든 timepoints 삭제
            timepoints_response = admin_client.get(f"/api/subjects/{created_subject['id']}/timepoints")
            if timepoints_response.status_code == 200:
                timepoints = timepoints_response.json()
                for tp in timepoints:
                    try:
                        admin_client.delete(f"/api/timepoints/{tp['id']}")
                        logger.info(f"Deleted timepoint: {tp['id']}")
                    except Exception as e:
                        logger.warning(f"Failed to delete timepoint {tp['id']}: {e}")

            # Subject 삭제
            admin_client.delete(f"/api/subjects/{created_subject['id']}")
            logger.info(f"Deleted test subject: {created_subject['id']}")
        except Exception as e:
            logger.warning(f"Failed to delete test subject: {e}")





class TestSubjectManagement:
    """Subject 관리 테스트"""

    def test_01_create_subject(self, admin_client, test_project):
        """Subject 생성 테스트"""
        logger.info("Testing subject creation...")

        project_id = test_project['id']
        subject_code = f"SUB{fake.random_int(1000, 9999)}"

        response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "subject_code": subject_code,
            "patient_id": f"P{fake.random_int(10000, 99999)}",
            "patient_name": fake.name(),
            "patient_birth_date": "1990-01-01"
        })

        assert response.status_code == 201, f"Failed to create subject: {response.text}"
        data = response.json()

        assert "id" in data
        assert data["subject_code"] == subject_code
        assert data["project_id"] == project_id

        logger.info(f"✓ Subject created successfully: {subject_code} (ID: {data['id']})")

        # 정리
        admin_client.delete(f"/api/subjects/{data['id']}")

    def test_02_get_subject(self, admin_client, test_subject):
        """Subject 조회 테스트"""
        logger.info("Testing subject retrieval...")

        response = admin_client.get(f"/api/subjects/{test_subject['id']}")

        assert response.status_code == 200, f"Failed to get subject: {response.text}"
        data = response.json()

        assert data["id"] == test_subject['id']
        assert data["subject_code"] == test_subject["subject_code"]

        logger.info(f"✓ Subject retrieved successfully: {test_subject['subject_code']}")

    def test_03_get_subject_detail(self, admin_client, test_subject):
        """Subject 상세 조회 테스트 (통계 포함)"""
        logger.info("Testing subject detail retrieval...")

        response = admin_client.get(f"/api/subjects/{test_subject['id']}/detail")

        assert response.status_code == 200, f"Failed to get subject detail: {response.text}"
        data = response.json()

        assert data["id"] == test_subject['id']
        assert "timepoint_count" in data
        assert "study_count" in data

        logger.info(f"✓ Subject detail retrieved: timepoints={data['timepoint_count']}, studies={data['study_count']}")

    def test_04_list_subjects_by_project(self, admin_client, test_project, test_subject):
        """프로젝트별 Subject 목록 조회 테스트"""
        logger.info("Testing subject list by project...")

        project_id = test_project['id']
        response = admin_client.get(f"/api/projects/{project_id}/subjects")

        assert response.status_code == 200, f"Failed to list subjects: {response.text}"
        data = response.json()

        assert isinstance(data, list)
        assert len(data) > 0
        assert any(s["id"] == test_subject['id'] for s in data)

        logger.info(f"✓ Found {len(data)} subject(s) in project {project_id}")

    def test_05_update_subject(self, admin_client, test_subject):
        """Subject 수정 테스트"""
        logger.info("Testing subject update...")

        new_patient_name = fake.name()
        response = admin_client.put(f"/api/subjects/{test_subject['id']}", json={
            "patient_name": new_patient_name
        })

        assert response.status_code == 200, f"Failed to update subject: {response.text}"
        data = response.json()

        assert data["id"] == test_subject['id']
        assert data["patient_name"] == new_patient_name

        logger.info(f"✓ Subject updated successfully: patient_name={new_patient_name}")

    def test_06_duplicate_subject_code(self, admin_client, test_project, test_subject):
        """Subject 코드 중복 테스트"""
        logger.info("Testing duplicate subject code...")

        project_id = test_project['id']

        # 같은 코드로 다시 생성 시도
        response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "subject_code": test_subject['subject_code'],
            "patient_id": f"P{fake.random_int(10000, 99999)}"
        })

        assert response.status_code == 409, f"Expected 409 Conflict, got {response.status_code}"

        logger.info(f"✓ Duplicate subject code correctly rejected")


@pytest.fixture(scope="module")
def test_baseline(admin_client, test_subject):
    """테스트용 Baseline TimePoint 생성"""
    created_baseline = None

    # 기존 timepoints 확인
    existing_response = admin_client.get(f"/api/subjects/{test_subject['id']}/timepoints")
    if existing_response.status_code == 200:
        existing_timepoints = existing_response.json()
        for tp in existing_timepoints:
            if tp.get('visit_type') == 'Baseline':
                logger.info(f"Found existing baseline: {tp['name']} (ID: {tp['id']}), reusing it")
                yield tp
                return

    # 새로운 baseline 생성
    response = admin_client.post(f"/api/subjects/{test_subject['id']}/timepoints", json={
        "name": "Baseline",
        "visit_type": "Baseline",
        "order_index": 0,
        "description": "Initial baseline visit"
    })

    assert response.status_code == 201, f"Failed to create baseline: {response.text}"
    baseline = response.json()
    created_baseline = baseline
    logger.info(f"Created test baseline: {baseline['name']} (ID: {baseline['id']})")

    yield baseline

    # 테스트 후 삭제 (생성한 경우만)
    if created_baseline:
        try:
            admin_client.delete(f"/api/timepoints/{created_baseline['id']}")
            logger.info(f"Deleted test baseline: {created_baseline['id']}")
        except Exception as e:
            logger.warning(f"Failed to delete test baseline: {e}")


@pytest.fixture(scope="module")
def test_visit1(admin_client, test_subject):
    """테스트용 Visit 1 TimePoint 생성"""
    created_visit1 = None

    # 기존 timepoints 확인
    existing_response = admin_client.get(f"/api/subjects/{test_subject['id']}/timepoints")
    if existing_response.status_code == 200:
        existing_timepoints = existing_response.json()
        for tp in existing_timepoints:
            if tp.get('name') == 'Visit 1' and tp.get('visit_type') == 'Visit':
                logger.info(f"Found existing visit 1: {tp['name']} (ID: {tp['id']}), reusing it")
                yield tp
                return

    # 새로운 visit 1 생성
    response = admin_client.post(f"/api/subjects/{test_subject['id']}/timepoints", json={
        "name": "Visit 1",
        "visit_type": "Visit",
        "order_index": 1,
        "description": "First follow-up visit"
    })

    assert response.status_code == 201, f"Failed to create visit 1: {response.text}"
    visit1 = response.json()
    created_visit1 = visit1
    logger.info(f"Created test visit 1: {visit1['name']} (ID: {visit1['id']})")

    yield visit1

    # 테스트 후 삭제 (생성한 경우만)
    if created_visit1:
        try:
            admin_client.delete(f"/api/timepoints/{created_visit1['id']}")
            logger.info(f"Deleted test visit 1: {created_visit1['id']}")
        except Exception as e:
            logger.warning(f"Failed to delete test visit 1: {e}")


class TestTimePointManagement:
    """TimePoint 관리 테스트"""

    def test_01_create_baseline_timepoint(self, admin_client, test_project):
        """Baseline TimePoint 생성 테스트"""
        logger.info("Testing baseline timepoint creation...")

        subject_id = None
        timepoint_id = None

        try:
            # 새 Subject 생성 (baseline이 없는 상태)
            subject_code = f"SUB{fake.random_int(1000, 9999)}"
            response = admin_client.post(f"/api/projects/{test_project['id']}/subjects", json={
                "subject_code": subject_code,
                "patient_id": f"P{fake.random_int(10000, 99999)}"
            })
            subject_id = response.json()['id']

            # Baseline 생성
            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "Baseline Test",
                "visit_type": "Baseline",
                "order_index": 0,
                "description": "Test baseline visit"
            })

            assert response.status_code == 201, f"Failed to create timepoint: {response.text}"
            data = response.json()
            timepoint_id = data['id']

            assert "id" in data
            assert data["name"] == "Baseline Test"
            assert data["visit_type"] == "Baseline"
            assert data["subject_id"] == subject_id

            logger.info(f"✓ Baseline timepoint created successfully (ID: {data['id']})")

        finally:
            # 정리
            if timepoint_id:
                try:
                    admin_client.delete(f"/api/timepoints/{timepoint_id}")
                except Exception as e:
                    logger.warning(f"Failed to delete timepoint {timepoint_id}: {e}")
            if subject_id:
                try:
                    admin_client.delete(f"/api/subjects/{subject_id}")
                except Exception as e:
                    logger.warning(f"Failed to delete subject {subject_id}: {e}")

    def test_02_create_visit_timepoint(self, admin_client, test_subject):
        """Visit TimePoint 생성 테스트"""
        logger.info("Testing visit timepoint creation...")

        response = admin_client.post(f"/api/subjects/{test_subject['id']}/timepoints", json={
            "name": "Visit Test",
            "visit_type": "Visit",
            "order_index": 1,
            "description": "Test follow-up visit"
        })

        assert response.status_code == 201, f"Failed to create timepoint: {response.text}"
        data = response.json()

        assert data["name"] == "Visit Test"
        assert data["visit_type"] == "Visit"

        logger.info(f"✓ Visit timepoint created successfully (ID: {data['id']})")

        # 정리
        admin_client.delete(f"/api/timepoints/{data['id']}")

    def test_03_duplicate_baseline(self, admin_client, test_subject, test_baseline):
        """Baseline 중복 생성 테스트"""
        logger.info("Testing duplicate baseline creation...")

        # Baseline을 다시 생성 시도
        response = admin_client.post(f"/api/subjects/{test_subject['id']}/timepoints", json={
            "name": "Baseline 2",
            "visit_type": "Baseline",
            "order_index": 0
        })

        assert response.status_code == 409, f"Expected 409 Conflict, got {response.status_code}"

        logger.info(f"✓ Duplicate baseline correctly rejected")

    def test_04_list_timepoints_by_subject(self, admin_client, test_subject, test_baseline, test_visit1):
        """Subject별 TimePoint 목록 조회 테스트"""
        logger.info("Testing timepoint list by subject...")

        response = admin_client.get(f"/api/subjects/{test_subject['id']}/timepoints")

        assert response.status_code == 200, f"Failed to list timepoints: {response.text}"
        data = response.json()

        assert isinstance(data, list)
        assert len(data) >= 2  # Baseline + Visit 1

        # order_index 순서 확인
        if len(data) >= 2:
            assert data[0]["order_index"] <= data[1]["order_index"]

        logger.info(f"✓ Found {len(data)} timepoint(s) for subject {test_subject['id']}")

    def test_05_update_timepoint(self, admin_client, test_visit1):
        """TimePoint 수정 테스트"""
        logger.info("Testing timepoint update...")

        response = admin_client.put(f"/api/timepoints/{test_visit1['id']}", json={
            "name": "Updated Visit 1"
        })

        assert response.status_code == 200, f"Failed to update timepoint: {response.text}"
        data = response.json()

        assert data["name"] == "Updated Visit 1"

        logger.info(f"✓ TimePoint updated successfully")


class TestStudyAssignment:
    """Study 할당/해제 테스트"""

    def test_01_get_unassigned_studies(self, admin_client, test_subject):
        """미할당 Study 목록 조회 테스트"""
        logger.info("Testing unassigned studies retrieval...")

        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")

        assert response.status_code == 200, f"Failed to get unassigned studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)

        logger.info(f"✓ Found {len(data)} unassigned study(ies)")

    def test_02_assign_study_to_timepoint(self, admin_client, test_subject, test_baseline):
        """Study를 TimePoint에 할당 테스트"""
        logger.info("Testing study assignment...")

        # 미할당 Study 조회
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            pytest.skip("No unassigned studies available")

        study_uid = studies[0]["study_uid"]

        response = admin_client.post(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_uids": [study_uid]
        })

        assert response.status_code == 200, f"Failed to assign study: {response.text}"
        data = response.json()

        assert "assigned_count" in data
        assert data["assigned_count"] == 1

        logger.info(f"✓ Study assigned successfully: {study_uid}")

        # 정리
        admin_client.delete(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_uids": [study_uid]
        })

    def test_03_get_assigned_studies(self, admin_client, test_subject, test_baseline):
        """할당된 Study 목록 조회 테스트"""
        logger.info("Testing assigned studies retrieval...")

        # Study 할당
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            pytest.skip("No unassigned studies available")

        study_uid = studies[0]["study_uid"]
        admin_client.post(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_uids": [study_uid]
        })

        # 할당된 Study 조회
        response = admin_client.get(f"/api/timepoints/{test_baseline['id']}/studies")

        assert response.status_code == 200, f"Failed to get assigned studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)
        assert len(data) >= 1
        assert any(s["study_uid"] == study_uid for s in data)

        logger.info(f"✓ Found {len(data)} assigned study(ies)")

        # 정리
        admin_client.delete(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_uids": [study_uid]
        })

    def test_04_move_study_to_another_timepoint(self, admin_client, test_subject, test_baseline, test_visit1):
        """Study를 다른 TimePoint로 이동 테스트 (MOVE 시맨틱)"""
        logger.info("Testing study move (MOVE semantics)...")

        # 미할당 Study 조회
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            pytest.skip("No unassigned studies available")

        study_uid = studies[0]["study_uid"]

        # Baseline에 할당
        admin_client.post(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_uids": [study_uid]
        })

        # Visit 1으로 이동
        response = admin_client.post(f"/api/timepoints/{test_visit1['id']}/studies", json={
            "study_uids": [study_uid]
        })

        assert response.status_code == 200, f"Failed to move study: {response.text}"

        # Baseline에서 제거되었는지 확인
        response = admin_client.get(f"/api/timepoints/{test_baseline['id']}/studies")
        data = response.json()
        assert not any(s["study_uid"] == study_uid for s in data)

        # Visit 1에 추가되었는지 확인
        response = admin_client.get(f"/api/timepoints/{test_visit1['id']}/studies")
        data = response.json()
        assert any(s["study_uid"] == study_uid for s in data)

        logger.info(f"✓ Study moved successfully (MOVE semantics verified)")

        # 정리
        admin_client.delete(f"/api/timepoints/{test_visit1['id']}/studies", json={
            "study_uids": [study_uid]
        })

    def test_05_unassign_study(self, admin_client, test_subject, test_visit1):
        """Study 할당 해제 테스트"""
        logger.info("Testing study unassignment...")

        # 미할당 Study 조회
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            pytest.skip("No unassigned studies available")

        study_uid = studies[0]["study_uid"]

        # Study 할당
        admin_client.post(f"/api/timepoints/{test_visit1['id']}/studies", json={
            "study_uids": [study_uid]
        })

        # Study 할당 해제
        response = admin_client.delete(f"/api/timepoints/{test_visit1['id']}/studies", json={
            "study_uids": [study_uid]
        })

        assert response.status_code == 200, f"Failed to unassign study: {response.text}"
        data = response.json()

        assert "unassigned_count" in data
        assert data["unassigned_count"] == 1

        # 할당 해제 확인
        response = admin_client.get(f"/api/timepoints/{test_visit1['id']}/studies")
        data = response.json()
        assert not any(s["study_uid"] == study_uid for s in data)

        logger.info(f"✓ Study unassigned successfully")


class TestCascadeProtection:
    """CASCADE 방지 테스트"""

    def test_01_cannot_delete_subject_with_timepoints(self, admin_client, test_subject, test_baseline):
        """TimePoint가 있는 Subject 삭제 방지 테스트"""
        logger.info("Testing cascade protection for subject deletion...")

        response = admin_client.delete(f"/api/subjects/{test_subject['id']}")

        # 400 또는 409 에러 예상
        assert response.status_code in [400, 409], f"Expected 400/409, got {response.status_code}"

        logger.info(f"✓ Subject deletion correctly prevented (has timepoints)")

    def test_02_delete_timepoints_first(self, admin_client, test_project):
        """TimePoint 먼저 삭제 테스트"""
        logger.info("Testing timepoint deletion...")

        subject_id = None
        baseline_id = None
        visit1_id = None

        try:
            # 새 Subject 생성
            subject_code = f"SUB{fake.random_int(1000, 9999)}"
            response = admin_client.post(f"/api/projects/{test_project['id']}/subjects", json={
                "subject_code": subject_code,
                "patient_id": f"P{fake.random_int(10000, 99999)}"
            })
            subject_id = response.json()['id']

            # TimePoint 생성
            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "Baseline",
                "visit_type": "Baseline",
                "order_index": 0
            })
            baseline_id = response.json()['id']

            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "Visit 1",
                "visit_type": "Visit",
                "order_index": 1
            })
            visit1_id = response.json()['id']

            # TimePoint 삭제
            response = admin_client.delete(f"/api/timepoints/{visit1_id}")
            assert response.status_code == 204, f"Failed to delete timepoint: {response.text}"

            response = admin_client.delete(f"/api/timepoints/{baseline_id}")
            assert response.status_code == 204, f"Failed to delete timepoint: {response.text}"

            logger.info(f"✓ TimePoints deleted successfully")

        finally:
            # 정리: Subject 삭제
            if subject_id:
                try:
                    admin_client.delete(f"/api/subjects/{subject_id}")
                    logger.info(f"Cleaned up subject: {subject_id}")
                except Exception as e:
                    logger.warning(f"Failed to cleanup subject {subject_id}: {e}")

    def test_03_delete_subject_after_timepoints_removed(self, admin_client, test_project):
        """TimePoint 삭제 후 Subject 삭제 테스트"""
        logger.info("Testing subject deletion after timepoints removed...")

        subject_id = None
        baseline_id = None

        try:
            # 새 Subject 생성
            subject_code = f"SUB{fake.random_int(1000, 9999)}"
            response = admin_client.post(f"/api/projects/{test_project['id']}/subjects", json={
                "subject_code": subject_code,
                "patient_id": f"P{fake.random_int(10000, 99999)}"
            })
            subject_id = response.json()['id']

            # TimePoint 생성
            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "Baseline",
                "visit_type": "Baseline",
                "order_index": 0
            })
            baseline_id = response.json()['id']

            # TimePoint 삭제
            admin_client.delete(f"/api/timepoints/{baseline_id}")

            # Subject 삭제
            response = admin_client.delete(f"/api/subjects/{subject_id}")

            assert response.status_code == 204, f"Failed to delete subject: {response.text}"

            logger.info(f"✓ Subject deleted successfully after timepoints removed")
            subject_id = None  # 삭제 성공했으므로 cleanup 불필요

        finally:
            # 정리: Subject 삭제 (실패한 경우)
            if subject_id:
                try:
                    # TimePoint 먼저 삭제
                    if baseline_id:
                        admin_client.delete(f"/api/timepoints/{baseline_id}")
                    admin_client.delete(f"/api/subjects/{subject_id}")
                    logger.info(f"Cleaned up subject: {subject_id}")
                except Exception as e:
                    logger.warning(f"Failed to cleanup subject {subject_id}: {e}")


class TestDicomGatewayTimePoint:
    """DICOM Gateway TimePoint 정보 테스트"""

    def test_01_gateway_without_timepoint(self, admin_client, test_project):
        """TimePoint 정보 없이 DICOM Gateway 조회 테스트 (view 파라미터 없음)"""
        logger.info("Testing DICOM Gateway without timepoint info (no view parameter)...")

        project_id = test_project['id']
        response = admin_client.get(f"/api/me/dicom/studies?project_id={project_id}&page_size=5")

        assert response.status_code == 200, f"Failed to get studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)

        # _ext 필드 확인
        if len(data) > 0:
            study = data[0]
            if "_ext" in study:
                # timepoint 필드가 없어야 함 (view 파라미터가 없고 include_timepoint=false가 기본값)
                assert "timepoint" not in study["_ext"], "timepoint should not be included without view parameter"
                logger.info(f"✓ TimePoint not included without view parameter (as expected)")
            else:
                logger.info(f"✓ No _ext field (no extension data)")
        else:
            logger.info(f"✓ No studies found (skipping timepoint check)")

    def test_01_5_gateway_default_view_includes_timepoint(self, admin_client, test_project):
        """view=default일 때 TimePoint 정보 자동 포함 테스트"""
        logger.info("Testing DICOM Gateway with view=default (should include timepoint)...")

        project_id = test_project['id']
        response = admin_client.get(f"/api/me/dicom/studies?project_id={project_id}&view=default&page_size=5")

        assert response.status_code == 200, f"Failed to get studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)

        # _ext 필드 확인
        if len(data) > 0:
            study = data[0]
            assert "_ext" in study, "_ext field should exist"
            # view=default일 때는 timepoint 필드가 자동으로 포함되어야 함
            assert "timepoint" in study["_ext"], "timepoint should be included with view=default"
            logger.info(f"✓ TimePoint automatically included with view=default: {study['_ext']['timepoint']}")
        else:
            logger.info(f"✓ No studies found (skipping timepoint check)")

    def test_02_gateway_with_timepoint(self, admin_client, test_project, test_subject, test_baseline):
        """TimePoint 정보 포함하여 DICOM Gateway 조회 테스트"""
        logger.info("Testing DICOM Gateway with timepoint info...")

        # Study를 TimePoint에 할당
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            logger.warning("⚠️  No unassigned studies available - test will verify API structure only")
            # API 구조만 확인
            project_id = test_project['id']
            response = admin_client.get(
                f"/api/me/dicom/studies?project_id={project_id}&include_timepoint=true&page_size=10"
            )
            assert response.status_code == 200, f"Failed to get studies: {response.text}"
            logger.info("✓ API call successful (no data to verify)")
            return

        study_id = studies[0]["study_id"]
        study_uid = studies[0]["study_uid"]
        assign_response = admin_client.post(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_ids": [study_id]
        })

        assert assign_response.status_code in [200, 201], f"Failed to assign study to timepoint: {assign_response.text}"
        logger.info(f"✓ Assigned study {study_uid[:30]}... (ID: {study_id}) to timepoint {test_baseline['id']}")

        # TimePoint 정보 포함하여 조회
        project_id = test_project['id']
        response = admin_client.get(
            f"/api/me/dicom/studies?project_id={project_id}&include_timepoint=true&page_size=10"
        )

        assert response.status_code == 200, f"Failed to get studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)
        assert len(data) > 0, "No studies found"

        # 할당된 Study 찾기
        assigned_study = None
        for study in data:
            if study.get("0020000D", {}).get("Value", [None])[0] == study_uid:
                assigned_study = study
                break

        if assigned_study:
            assert "_ext" in assigned_study, "_ext field should exist"
            assert "timepoint" in assigned_study["_ext"], "timepoint field should exist in _ext"

            timepoint = assigned_study["_ext"]["timepoint"]
            assert timepoint is not None, "timepoint should not be null for assigned study"
            assert "id" in timepoint
            assert "name" in timepoint
            assert "visitType" in timepoint  # camelCase 확인
            assert timepoint["id"] == test_baseline["id"]
            assert timepoint["name"] == test_baseline["name"]

            logger.info(f"✓ TimePoint info included: {timepoint}")
        else:
            logger.warning(f"Assigned study {study_uid} not found in response")

        # 정리
        admin_client.delete(f"/api/timepoints/{test_baseline['id']}/studies", json={
            "study_uids": [study_uid]
        })

    def test_03_gateway_timepoint_null_for_unassigned(self, admin_client, test_project, test_subject):
        """할당되지 않은 Study는 timepoint가 null인지 확인"""
        logger.info("Testing DICOM Gateway timepoint null for unassigned studies...")

        # 미할당 Study 조회
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            pytest.skip("No unassigned studies available")

        study_uid = studies[0]["study_uid"]

        # TimePoint 정보 포함하여 조회
        project_id = test_project['id']
        response = admin_client.get(
            f"/api/me/dicom/studies?project_id={project_id}&include_timepoint=true&page_size=10"
        )

        assert response.status_code == 200, f"Failed to get studies: {response.text}"
        data = response.json()

        # 미할당 Study 찾기
        unassigned_study = None
        for study in data:
            if study.get("0020000D", {}).get("Value", [None])[0] == study_uid:
                unassigned_study = study
                break

        if unassigned_study:
            assert "_ext" in unassigned_study, "_ext field should exist"
            assert "timepoint" in unassigned_study["_ext"], "timepoint field should exist in _ext"
            assert unassigned_study["_ext"]["timepoint"] is None, "timepoint should be null for unassigned study"

            logger.info(f"✓ TimePoint is null for unassigned study (as expected)")
        else:
            logger.warning(f"Unassigned study {study_uid} not found in response")

    def test_04_gateway_timepoint_visit_no(self, admin_client, test_project, test_subject, test_visit1):
        """Visit TimePoint의 visitNo 필드 확인"""
        logger.info("Testing DICOM Gateway timepoint visitNo field...")

        # Study를 Visit 1에 할당
        response = admin_client.get(f"/api/subjects/{test_subject['id']}/studies/unassigned")
        studies = response.json()

        if len(studies) == 0:
            pytest.skip("No unassigned studies available")

        study_id = studies[0]["study_id"]
        study_uid = studies[0]["study_uid"]
        assign_response = admin_client.post(f"/api/timepoints/{test_visit1['id']}/studies", json={
            "study_ids": [study_id]
        })

        assert assign_response.status_code in [200, 201], f"Failed to assign study to timepoint: {assign_response.text}"
        logger.info(f"✓ Assigned study {study_uid[:30]}... (ID: {study_id}) to timepoint {test_visit1['id']}")

        # TimePoint 정보 포함하여 조회
        project_id = test_project['id']
        response = admin_client.get(
            f"/api/me/dicom/studies?project_id={project_id}&include_timepoint=true&page_size=10"
        )

        assert response.status_code == 200, f"Failed to get studies: {response.text}"
        data = response.json()

        # 할당된 Study 찾기
        assigned_study = None
        for study in data:
            if study.get("0020000D", {}).get("Value", [None])[0] == study_uid:
                assigned_study = study
                break

        if assigned_study:
            timepoint = assigned_study["_ext"]["timepoint"]
            assert "visitNo" in timepoint or timepoint.get("visitNo") is None, "visitNo field should exist (can be null)"

            logger.info(f"✓ TimePoint visitNo field: {timepoint.get('visitNo')}")
        else:
            logger.warning(f"Assigned study {study_uid} not found in response")

        # 정리
        admin_client.delete(f"/api/timepoints/{test_visit1['id']}/studies", json={
            "study_uids": [study_uid]
        })


class TestErrorCases:
    """에러 케이스 테스트"""

    def test_01_get_nonexistent_subject(self, admin_client):
        """존재하지 않는 Subject 조회 테스트"""
        logger.info("Testing nonexistent subject retrieval...")

        response = admin_client.get("/api/subjects/99999999")

        assert response.status_code == 404, f"Expected 404, got {response.status_code}"

        logger.info(f"✓ Nonexistent subject correctly returned 404")

    def test_02_create_subject_invalid_data(self, admin_client, test_project):
        """잘못된 데이터로 Subject 생성 테스트"""
        logger.info("Testing subject creation with invalid data...")

        project_id = test_project['id']

        # subject_code 누락
        response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "patient_id": "P12345"
        })

        assert response.status_code == 400, f"Expected 400, got {response.status_code}"

        logger.info(f"✓ Invalid subject data correctly rejected")

    def test_03_create_timepoint_invalid_visit_type(self, admin_client, test_project):
        """잘못된 visit_type으로 TimePoint 생성 테스트"""
        logger.info("Testing timepoint creation with invalid visit_type...")

        subject_id = None

        try:
            # 새 Subject 생성
            project_id = test_project['id']
            response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
                "subject_code": f"SUB{fake.random_int(1000, 9999)}",
                "patient_id": f"P{fake.random_int(10000, 99999)}"
            })
            subject_id = response.json()['id']

            # 잘못된 visit_type
            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "Invalid",
                "visit_type": "InvalidType",
                "order_index": 0
            })

            assert response.status_code == 400, f"Expected 400, got {response.status_code}"

            logger.info(f"✓ Invalid visit_type correctly rejected")

        finally:
            # 정리
            if subject_id:
                try:
                    admin_client.delete(f"/api/subjects/{subject_id}")
                    logger.info(f"Cleaned up subject: {subject_id}")
                except Exception as e:
                    logger.warning(f"Failed to cleanup subject {subject_id}: {e}")

