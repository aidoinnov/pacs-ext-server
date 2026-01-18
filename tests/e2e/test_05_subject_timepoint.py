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
    client.login(config.admin_email, config.admin_password)
    yield client
    client.close()


@pytest.fixture(scope="module")
def test_project(admin_client):
    """테스트용 프로젝트 생성"""
    project_name = f"E2E Subject Test {fake.uuid4()[:8]}"
    
    response = admin_client.post("/api/projects", json={
        "name": project_name,
        "description": "Subject/TimePoint E2E 테스트용 프로젝트",
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
        
        # 생성된 Subject를 테스트 컨텍스트에 저장
        self.subject_id = data['id']
        self.subject_code = subject_code
    
    def test_02_get_subject(self, admin_client):
        """Subject 조회 테스트"""
        logger.info("Testing subject retrieval...")
        
        response = admin_client.get(f"/api/subjects/{self.subject_id}")
        
        assert response.status_code == 200, f"Failed to get subject: {response.text}"
        data = response.json()
        
        assert data["id"] == self.subject_id
        assert data["subject_code"] == self.subject_code
        
        logger.info(f"✓ Subject retrieved successfully: {self.subject_code}")
    
    def test_03_get_subject_detail(self, admin_client):
        """Subject 상세 조회 테스트 (통계 포함)"""
        logger.info("Testing subject detail retrieval...")
        
        response = admin_client.get(f"/api/subjects/{self.subject_id}/detail")
        
        assert response.status_code == 200, f"Failed to get subject detail: {response.text}"
        data = response.json()
        
        assert data["id"] == self.subject_id
        assert "timepoint_count" in data
        assert "study_count" in data
        assert data["timepoint_count"] == 0  # 아직 TimePoint 없음
        
        logger.info(f"✓ Subject detail retrieved: timepoints={data['timepoint_count']}, studies={data['study_count']}")
    
    def test_04_list_subjects_by_project(self, admin_client, test_project):
        """프로젝트별 Subject 목록 조회 테스트"""
        logger.info("Testing subject list by project...")
        
        project_id = test_project['id']
        response = admin_client.get(f"/api/projects/{project_id}/subjects")
        
        assert response.status_code == 200, f"Failed to list subjects: {response.text}"
        data = response.json()
        
        assert isinstance(data, list)
        assert len(data) > 0
        assert any(s["id"] == self.subject_id for s in data)

        logger.info(f"✓ Found {len(data)} subject(s) in project {project_id}")

    def test_05_update_subject(self, admin_client):
        """Subject 수정 테스트"""
        logger.info("Testing subject update...")

        new_patient_name = fake.name()
        response = admin_client.put(f"/api/subjects/{self.subject_id}", json={
            "patient_name": new_patient_name
        })

        assert response.status_code == 200, f"Failed to update subject: {response.text}"
        data = response.json()

        assert data["id"] == self.subject_id
        assert data["patient_name"] == new_patient_name

        logger.info(f"✓ Subject updated successfully: patient_name={new_patient_name}")

    def test_06_duplicate_subject_code(self, admin_client, test_project):
        """Subject 코드 중복 테스트"""
        logger.info("Testing duplicate subject code...")

        project_id = test_project['id']

        # 같은 코드로 다시 생성 시도
        response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "subject_code": self.subject_code,
            "patient_id": f"P{fake.random_int(10000, 99999)}"
        })

        assert response.status_code == 409, f"Expected 409 Conflict, got {response.status_code}"

        logger.info(f"✓ Duplicate subject code correctly rejected")


class TestTimePointManagement:
    """TimePoint 관리 테스트"""

    def test_01_create_baseline_timepoint(self, admin_client):
        """Baseline TimePoint 생성 테스트"""
        logger.info("Testing baseline timepoint creation...")

        # Subject ID는 이전 테스트에서 생성된 것 사용
        subject_id = TestSubjectManagement.subject_id

        response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
            "name": "Baseline",
            "visit_type": "Baseline",
            "order_index": 0,
            "description": "Initial baseline visit"
        })

        assert response.status_code == 201, f"Failed to create timepoint: {response.text}"
        data = response.json()

        assert "id" in data
        assert data["name"] == "Baseline"
        assert data["visit_type"] == "Baseline"
        assert data["subject_id"] == subject_id

        logger.info(f"✓ Baseline timepoint created successfully (ID: {data['id']})")

        # TimePoint ID 저장
        self.baseline_id = data['id']

    def test_02_create_visit_timepoint(self, admin_client):
        """Visit TimePoint 생성 테스트"""
        logger.info("Testing visit timepoint creation...")

        subject_id = TestSubjectManagement.subject_id

        response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
            "name": "Visit 1",
            "visit_type": "Visit",
            "order_index": 1,
            "description": "First follow-up visit"
        })

        assert response.status_code == 201, f"Failed to create timepoint: {response.text}"
        data = response.json()

        assert data["name"] == "Visit 1"
        assert data["visit_type"] == "Visit"

        logger.info(f"✓ Visit timepoint created successfully (ID: {data['id']})")

        self.visit1_id = data['id']

    def test_03_duplicate_baseline(self, admin_client):
        """Baseline 중복 생성 테스트"""
        logger.info("Testing duplicate baseline creation...")

        subject_id = TestSubjectManagement.subject_id

        # Baseline을 다시 생성 시도
        response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
            "name": "Baseline 2",
            "visit_type": "Baseline",
            "order_index": 0
        })

        assert response.status_code == 409, f"Expected 409 Conflict, got {response.status_code}"

        logger.info(f"✓ Duplicate baseline correctly rejected")

    def test_04_list_timepoints_by_subject(self, admin_client):
        """Subject별 TimePoint 목록 조회 테스트"""
        logger.info("Testing timepoint list by subject...")

        subject_id = TestSubjectManagement.subject_id
        response = admin_client.get(f"/api/subjects/{subject_id}/timepoints")

        assert response.status_code == 200, f"Failed to list timepoints: {response.text}"
        data = response.json()

        assert isinstance(data, list)
        assert len(data) >= 2  # Baseline + Visit 1

        # order_index 순서 확인
        assert data[0]["order_index"] <= data[1]["order_index"]

        logger.info(f"✓ Found {len(data)} timepoint(s) for subject {subject_id}")

    def test_05_update_timepoint(self, admin_client):
        """TimePoint 수정 테스트"""
        logger.info("Testing timepoint update...")

        response = admin_client.put(f"/api/timepoints/{self.visit1_id}", json={
            "description": "Updated description for Visit 1"
        })

        assert response.status_code == 200, f"Failed to update timepoint: {response.text}"
        data = response.json()

        assert data["description"] == "Updated description for Visit 1"

        logger.info(f"✓ TimePoint updated successfully")


class TestStudyAssignment:
    """Study 할당/해제 테스트"""

    def test_01_get_unassigned_studies(self, admin_client):
        """미할당 Study 목록 조회 테스트"""
        logger.info("Testing unassigned studies retrieval...")

        subject_id = TestSubjectManagement.subject_id
        response = admin_client.get(f"/api/subjects/{subject_id}/studies/unassigned")

        assert response.status_code == 200, f"Failed to get unassigned studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)

        logger.info(f"✓ Found {len(data)} unassigned study(ies)")

        # Study UID 저장 (할당 테스트용)
        if len(data) > 0:
            self.study_uid = data[0]["study_uid"]
            logger.info(f"  Using study UID: {self.study_uid}")
        else:
            logger.warning("  No unassigned studies available for testing")
            self.study_uid = None

    def test_02_assign_study_to_timepoint(self, admin_client):
        """Study를 TimePoint에 할당 테스트"""
        if not hasattr(self, 'study_uid') or self.study_uid is None:
            pytest.skip("No unassigned studies available")

        logger.info("Testing study assignment...")

        timepoint_id = TestTimePointManagement.baseline_id

        response = admin_client.post(f"/api/timepoints/{timepoint_id}/studies", json={
            "study_uids": [self.study_uid]
        })

        assert response.status_code == 200, f"Failed to assign study: {response.text}"
        data = response.json()

        assert "assigned_count" in data
        assert data["assigned_count"] == 1

        logger.info(f"✓ Study assigned successfully: {self.study_uid}")

    def test_03_get_assigned_studies(self, admin_client):
        """할당된 Study 목록 조회 테스트"""
        if not hasattr(self, 'study_uid') or self.study_uid is None:
            pytest.skip("No studies assigned")

        logger.info("Testing assigned studies retrieval...")

        timepoint_id = TestTimePointManagement.baseline_id
        response = admin_client.get(f"/api/timepoints/{timepoint_id}/studies")

        assert response.status_code == 200, f"Failed to get assigned studies: {response.text}"
        data = response.json()

        assert isinstance(data, list)
        assert len(data) >= 1
        assert any(s["study_uid"] == self.study_uid for s in data)

        logger.info(f"✓ Found {len(data)} assigned study(ies)")

    def test_04_move_study_to_another_timepoint(self, admin_client):
        """Study를 다른 TimePoint로 이동 테스트 (MOVE 시맨틱)"""
        if not hasattr(self, 'study_uid') or self.study_uid is None:
            pytest.skip("No studies assigned")

        logger.info("Testing study move (MOVE semantics)...")

        # Visit 1으로 이동
        visit1_id = TestTimePointManagement.visit1_id

        response = admin_client.post(f"/api/timepoints/{visit1_id}/studies", json={
            "study_uids": [self.study_uid]
        })

        assert response.status_code == 200, f"Failed to move study: {response.text}"

        # Baseline에서 제거되었는지 확인
        baseline_id = TestTimePointManagement.baseline_id
        response = admin_client.get(f"/api/timepoints/{baseline_id}/studies")
        data = response.json()
        assert not any(s["study_uid"] == self.study_uid for s in data)

        # Visit 1에 추가되었는지 확인
        response = admin_client.get(f"/api/timepoints/{visit1_id}/studies")
        data = response.json()
        assert any(s["study_uid"] == self.study_uid for s in data)

        logger.info(f"✓ Study moved successfully (MOVE semantics verified)")

    def test_05_unassign_study(self, admin_client):
        """Study 할당 해제 테스트"""
        if not hasattr(self, 'study_uid') or self.study_uid is None:
            pytest.skip("No studies assigned")

        logger.info("Testing study unassignment...")

        visit1_id = TestTimePointManagement.visit1_id

        response = admin_client.delete(f"/api/timepoints/{visit1_id}/studies", json={
            "study_uids": [self.study_uid]
        })

        assert response.status_code == 200, f"Failed to unassign study: {response.text}"
        data = response.json()

        assert "unassigned_count" in data
        assert data["unassigned_count"] == 1

        # 할당 해제 확인
        response = admin_client.get(f"/api/timepoints/{visit1_id}/studies")
        data = response.json()
        assert not any(s["study_uid"] == self.study_uid for s in data)

        logger.info(f"✓ Study unassigned successfully")


class TestCascadeProtection:
    """CASCADE 방지 테스트"""

    def test_01_cannot_delete_subject_with_timepoints(self, admin_client):
        """TimePoint가 있는 Subject 삭제 방지 테스트"""
        logger.info("Testing cascade protection for subject deletion...")

        subject_id = TestSubjectManagement.subject_id

        response = admin_client.delete(f"/api/subjects/{subject_id}")

        # 400 또는 409 에러 예상
        assert response.status_code in [400, 409], f"Expected 400/409, got {response.status_code}"

        logger.info(f"✓ Subject deletion correctly prevented (has timepoints)")

    def test_02_delete_timepoints_first(self, admin_client):
        """TimePoint 먼저 삭제 테스트"""
        logger.info("Testing timepoint deletion...")

        # Visit 1 삭제
        visit1_id = TestTimePointManagement.visit1_id
        response = admin_client.delete(f"/api/timepoints/{visit1_id}")
        assert response.status_code == 204, f"Failed to delete timepoint: {response.text}"

        # Baseline 삭제
        baseline_id = TestTimePointManagement.baseline_id
        response = admin_client.delete(f"/api/timepoints/{baseline_id}")
        assert response.status_code == 204, f"Failed to delete timepoint: {response.text}"

        logger.info(f"✓ TimePoints deleted successfully")

    def test_03_delete_subject_after_timepoints_removed(self, admin_client):
        """TimePoint 삭제 후 Subject 삭제 테스트"""
        logger.info("Testing subject deletion after timepoints removed...")

        subject_id = TestSubjectManagement.subject_id

        response = admin_client.delete(f"/api/subjects/{subject_id}")

        assert response.status_code == 204, f"Failed to delete subject: {response.text}"

        logger.info(f"✓ Subject deleted successfully after timepoints removed")


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

        # 정리
        admin_client.delete(f"/api/subjects/{subject_id}")

        logger.info(f"✓ Invalid visit_type correctly rejected")

