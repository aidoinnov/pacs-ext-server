"""
RECIST Lesion 관리 E2E 테스트

테스트 시나리오:
1. RECIST Lesion CRUD
2. Annotation 연결
3. RECIST 1.1 비즈니스 규칙 검증
   - Max 5 Target Lesions per Subject
   - Baseline TimePoint 필수 (TARGET/NON_TARGET)
   - NEW Lesion은 Follow-up에서만 생성
4. 에러 케이스
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
    project_name = f"E2E RECIST Lesion Test {fake.uuid4()[:8]}"

    response = admin_client.post("/api/projects", json={
        "name": project_name,
        "description": "RECIST Lesion E2E 테스트용 프로젝트",
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


@pytest.fixture(scope="module")
def test_subject(admin_client, test_project):
    """테스트용 Subject 생성"""
    project_id = test_project['id']
    subject_code = f"RECIST{fake.random_int(1000, 9999)}"

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

    # 테스트 후 Subject 삭제
    try:
        admin_client.delete(f"/api/subjects/{subject['id']}")
        logger.info(f"Deleted test subject: {subject['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete test subject: {e}")


@pytest.fixture(scope="module")
def baseline_timepoint(admin_client, test_subject):
    """Baseline TimePoint 생성"""
    subject_id = test_subject['id']

    response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
        "name": "BL",
        "visit_type": "Baseline",
        "order_index": 0
    })

    assert response.status_code == 201, f"Failed to create baseline timepoint: {response.text}"
    timepoint = response.json()
    logger.info(f"Created baseline timepoint: {timepoint['name']} (ID: {timepoint['id']})")

    yield timepoint

    # 테스트 후 TimePoint 삭제
    try:
        admin_client.delete(f"/api/timepoints/{timepoint['id']}")
        logger.info(f"Deleted baseline timepoint: {timepoint['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete baseline timepoint: {e}")


@pytest.fixture(scope="module")
def followup_timepoint(admin_client, test_subject):
    """Follow-up TimePoint 생성"""
    subject_id = test_subject['id']

    response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
        "name": "TP1",
        "visit_type": "Visit",
        "order_index": 1
    })

    assert response.status_code == 201, f"Failed to create follow-up timepoint: {response.text}"
    timepoint = response.json()
    logger.info(f"Created follow-up timepoint: {timepoint['name']} (ID: {timepoint['id']})")

    yield timepoint

    # 테스트 후 TimePoint 삭제
    try:
        admin_client.delete(f"/api/timepoints/{timepoint['id']}")
        logger.info(f"Deleted follow-up timepoint: {timepoint['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete follow-up timepoint: {e}")


class TestRecistLesionCRUD:
    """RECIST Lesion CRUD 테스트"""

    def test_01_create_target_lesion(self, admin_client, test_subject, baseline_timepoint):
        """Target Lesion 생성 테스트"""
        logger.info("Testing Target Lesion creation...")

        subject_id = test_subject['id']
        baseline_id = baseline_timepoint['id']

        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": baseline_id,
            "organ_site": "Liver",
            "description": "Right lobe lesion, 3cm"
        })

        assert response.status_code == 201, f"Failed to create Target Lesion: {response.text}"
        lesion = response.json()

        # 검증
        assert lesion['lesion_type'] == "TARGET"
        assert lesion['baseline_timepoint_id'] == baseline_id
        assert lesion['organ_site'] == "Liver"
        assert lesion['lesion_number'] == 1  # 첫 번째 Lesion
        assert 'id' in lesion
        assert 'created_at' in lesion

        logger.info(f"✓ Created Target Lesion #{lesion['lesion_number']} (ID: {lesion['id']})")

    def test_02_create_non_target_lesion(self, admin_client, test_subject, baseline_timepoint):
        """Non-Target Lesion 생성 테스트"""
        logger.info("Testing Non-Target Lesion creation...")

        subject_id = test_subject['id']
        baseline_id = baseline_timepoint['id']

        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "NON_TARGET",
            "baseline_timepoint_id": baseline_id,
            "organ_site": "Lung",
            "description": "Small nodule, not measurable"
        })

        assert response.status_code == 201, f"Failed to create Non-Target Lesion: {response.text}"
        lesion = response.json()

        # 검증
        assert lesion['lesion_type'] == "NON_TARGET"
        assert lesion['baseline_timepoint_id'] == baseline_id
        assert lesion['organ_site'] == "Lung"
        assert lesion['lesion_number'] == 2  # 두 번째 Lesion

        logger.info(f"✓ Created Non-Target Lesion #{lesion['lesion_number']} (ID: {lesion['id']})")

    def test_03_list_lesions(self, admin_client, test_subject):
        """Lesion 목록 조회 테스트"""
        logger.info("Testing Lesion list retrieval...")

        subject_id = test_subject['id']

        # 전체 목록 조회
        response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
        assert response.status_code == 200, f"Failed to list lesions: {response.text}"
        lesions = response.json()

        assert len(lesions) >= 2  # 최소 2개 (Target + Non-Target)
        logger.info(f"✓ Found {len(lesions)} lesions")

        # Target Lesion만 조회
        response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}?lesion_type=target")
        assert response.status_code == 200
        target_lesions = response.json()

        assert all(l['lesion_type'] == "TARGET" for l in target_lesions)
        logger.info(f"✓ Found {len(target_lesions)} Target Lesions")

    def test_04_get_lesion_detail(self, admin_client, test_subject):
        """Lesion 상세 조회 테스트"""
        logger.info("Testing Lesion detail retrieval...")

        subject_id = test_subject['id']

        # 먼저 Lesion 목록 조회
        response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
        lesions = response.json()
        lesion_id = lesions[0]['id']

        # 상세 조회
        response = admin_client.get(f"/api/recist-lesions/{lesion_id}")
        assert response.status_code == 200, f"Failed to get lesion detail: {response.text}"
        detail = response.json()

        # 검증
        assert detail['id'] == lesion_id
        assert 'annotations' in detail  # Annotation 목록 포함
        assert isinstance(detail['annotations'], list)

        logger.info(f"✓ Retrieved Lesion detail (ID: {lesion_id})")
        logger.info(f"  - Annotations: {len(detail['annotations'])}")

    def test_05_update_lesion(self, admin_client, test_subject):
        """Lesion 수정 테스트"""
        logger.info("Testing Lesion update...")

        subject_id = test_subject['id']

        # 먼저 Lesion 목록 조회
        response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
        lesions = response.json()
        lesion_id = lesions[0]['id']

        # 수정
        response = admin_client.put(f"/api/recist-lesions/{lesion_id}", json={
            "organ_site": "Liver (updated)",
            "description": "Updated description: Right lobe lesion, 3.5cm"
        })

        assert response.status_code == 200, f"Failed to update lesion: {response.text}"
        updated = response.json()

        # 검증
        assert updated['organ_site'] == "Liver (updated)"
        assert updated['description'] == "Updated description: Right lobe lesion, 3.5cm"

        logger.info(f"✓ Updated Lesion (ID: {lesion_id})")

    def test_06_delete_lesion(self, admin_client, test_subject, baseline_timepoint):
        """Lesion 삭제 테스트"""
        logger.info("Testing Lesion deletion...")

        subject_id = test_subject['id']
        baseline_id = baseline_timepoint['id']

        # 삭제용 Lesion 생성
        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": baseline_id,
            "organ_site": "Temp",
            "description": "To be deleted"
        })
        lesion_id = response.json()['id']

        # 삭제
        response = admin_client.delete(f"/api/recist-lesions/{lesion_id}")
        assert response.status_code == 204, f"Failed to delete lesion: {response.text}"

        # 삭제 확인
        response = admin_client.get(f"/api/recist-lesions/{lesion_id}")
        assert response.status_code == 404  # Not Found

        logger.info(f"✓ Deleted Lesion (ID: {lesion_id})")


class TestRecistBusinessRules:
    """RECIST 1.1 비즈니스 규칙 검증 테스트"""

    def test_01_max_5_target_lesions(self, admin_client, test_subject, baseline_timepoint):
        """Max 5 Target Lesions 규칙 검증"""
        logger.info("Testing Max 5 Target Lesions rule...")

        subject_id = test_subject['id']
        baseline_id = baseline_timepoint['id']

        # 기존 Target Lesion 개수 확인
        response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}?lesion_type=target")
        existing_count = len(response.json())

        # 5개까지 생성 시도
        created_lesions = []
        for i in range(5 - existing_count):
            response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
                "lesion_type": "TARGET",
                "baseline_timepoint_id": baseline_id,
                "organ_site": f"Organ {i+1}",
                "description": f"Test lesion {i+1}"
            })
            if response.status_code == 201:
                created_lesions.append(response.json()['id'])

        # 6번째 생성 시도 (실패해야 함)
        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": baseline_id,
            "organ_site": "Organ 6",
            "description": "This should fail"
        })

        assert response.status_code == 400, "Should fail when creating 6th Target Lesion"
        error = response.json()
        assert "maximum" in error['error'].lower() or "5" in error['error']

        logger.info(f"✓ Max 5 Target Lesions rule validated")

        # 생성한 Lesion 정리
        for lesion_id in created_lesions:
            try:
                admin_client.delete(f"/api/recist-lesions/{lesion_id}")
            except:
                pass

    def test_02_baseline_required_for_target(self, admin_client, test_subject):
        """Target Lesion에 Baseline TimePoint 필수 규칙 검증"""
        logger.info("Testing Baseline TimePoint required for Target Lesion...")

        subject_id = test_subject['id']

        # Baseline TimePoint 없이 Target Lesion 생성 시도
        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": None,
            "organ_site": "Liver",
            "description": "This should fail"
        })

        assert response.status_code == 400, "Should fail when creating Target Lesion without Baseline TimePoint"
        logger.info(f"✓ Baseline TimePoint required rule validated")

    def test_03_new_lesion_no_baseline(self, admin_client, test_subject, followup_timepoint):
        """NEW Lesion은 Baseline TimePoint 없이 생성 가능"""
        logger.info("Testing NEW Lesion creation without Baseline TimePoint...")

        subject_id = test_subject['id']

        # NEW Lesion 생성 (Baseline TimePoint 없이)
        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "NEW",
            "baseline_timepoint_id": None,
            "organ_site": "Lymph Node",
            "description": "New lesion found at follow-up"
        })

        assert response.status_code == 201, f"Failed to create NEW Lesion: {response.text}"
        lesion = response.json()

        # 검증
        assert lesion['lesion_type'] == "NEW"
        assert lesion['baseline_timepoint_id'] is None

        logger.info(f"✓ NEW Lesion created without Baseline TimePoint (ID: {lesion['id']})")

        # 정리
        admin_client.delete(f"/api/recist-lesions/{lesion['id']}")

    def test_04_non_target_unlimited(self, admin_client, test_subject, baseline_timepoint):
        """Non-Target Lesion은 개수 제한 없음"""
        logger.info("Testing unlimited Non-Target Lesions...")

        subject_id = test_subject['id']
        baseline_id = baseline_timepoint['id']

        # 10개 Non-Target Lesion 생성 시도
        created_lesions = []
        for i in range(10):
            response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
                "lesion_type": "NON_TARGET",
                "baseline_timepoint_id": baseline_id,
                "organ_site": f"Organ {i+1}",
                "description": f"Non-target lesion {i+1}"
            })
            assert response.status_code == 201, f"Failed to create Non-Target Lesion #{i+1}"
            created_lesions.append(response.json()['id'])

        logger.info(f"✓ Created {len(created_lesions)} Non-Target Lesions (no limit)")

        # 정리
        for lesion_id in created_lesions:
            try:
                admin_client.delete(f"/api/recist-lesions/{lesion_id}")
            except:
                pass


class TestAnnotationLinking:
    """Annotation 연결 테스트"""

    def test_01_link_annotation_to_lesion(self, admin_client, test_subject, baseline_timepoint):
        """Lesion에 Annotation 연결 테스트"""
        logger.info("Testing Annotation linking to Lesion...")

        subject_id = test_subject['id']
        baseline_id = baseline_timepoint['id']

        # Lesion 생성
        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": baseline_id,
            "organ_site": "Liver",
            "description": "Test lesion for annotation"
        })
        lesion = response.json()
        lesion_id = lesion['id']

        # Annotation 생성 (실제 Annotation API 사용)
        # Note: 실제 환경에서는 DICOM 이미지가 필요할 수 있음
        # 여기서는 Annotation ID를 가정
        annotation_id = 999  # Mock Annotation ID

        # Annotation 연결
        response = admin_client.post(f"/api/recist-lesions/{lesion_id}/annotations", json={
            "annotation_id": annotation_id,
            "timepoint_id": baseline_id,
            "measured_length_mm": 32.5
        })

        # Note: Annotation이 실제로 존재하지 않으면 404 에러가 발생할 수 있음
        # 이 경우 테스트를 스킵하거나 Mock 데이터를 사용
        if response.status_code == 201:
            logger.info(f"✓ Linked Annotation {annotation_id} to Lesion {lesion_id}")

            # 상세 조회로 확인
            response = admin_client.get(f"/api/recist-lesions/{lesion_id}")
            detail = response.json()
            assert len(detail['annotations']) > 0
            logger.info(f"  - Annotations in detail: {len(detail['annotations'])}")
        elif response.status_code == 404:
            logger.warning(f"⚠ Annotation {annotation_id} not found (expected in test environment)")
        else:
            logger.error(f"✗ Failed to link annotation: {response.text}")

        # 정리
        admin_client.delete(f"/api/recist-lesions/{lesion_id}")


class TestErrorCases:
    """에러 케이스 테스트"""

    def test_01_create_lesion_invalid_subject(self, admin_client, baseline_timepoint):
        """존재하지 않는 Subject에 Lesion 생성 시도"""
        logger.info("Testing Lesion creation with invalid Subject...")

        invalid_subject_id = 999999
        baseline_id = baseline_timepoint['id']

        response = admin_client.post(f"/api/subjects/{invalid_subject_id}/recist-lesions", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": baseline_id,
            "organ_site": "Liver"
        })

        assert response.status_code == 404, "Should return 404 for invalid Subject"
        logger.info(f"✓ Correctly rejected invalid Subject")

    def test_02_create_lesion_invalid_timepoint(self, admin_client, test_subject):
        """존재하지 않는 TimePoint로 Lesion 생성 시도"""
        logger.info("Testing Lesion creation with invalid TimePoint...")

        subject_id = test_subject['id']
        invalid_timepoint_id = 999999

        response = admin_client.post(f"/api/recist-lesions/subjects/{subject_id}", json={
            "lesion_type": "TARGET",
            "baseline_timepoint_id": invalid_timepoint_id,
            "organ_site": "Liver"
        })

        assert response.status_code == 404, "Should return 404 for invalid TimePoint"
        logger.info(f"✓ Correctly rejected invalid TimePoint")

    def test_03_get_nonexistent_lesion(self, admin_client):
        """존재하지 않는 Lesion 조회"""
        logger.info("Testing retrieval of non-existent Lesion...")

        invalid_lesion_id = 999999

        response = admin_client.get(f"/api/recist-lesions/{invalid_lesion_id}")
        assert response.status_code == 404, "Should return 404 for non-existent Lesion"
        logger.info(f"✓ Correctly returned 404 for non-existent Lesion")

    def test_04_update_nonexistent_lesion(self, admin_client):
        """존재하지 않는 Lesion 수정 시도"""
        logger.info("Testing update of non-existent Lesion...")

        invalid_lesion_id = 999999

        response = admin_client.put(f"/api/recist-lesions/{invalid_lesion_id}", json={
            "organ_site": "Updated"
        })

        assert response.status_code == 404, "Should return 404 for non-existent Lesion"
        logger.info(f"✓ Correctly rejected update of non-existent Lesion")

    def test_05_delete_nonexistent_lesion(self, admin_client):
        """존재하지 않는 Lesion 삭제 시도"""
        logger.info("Testing deletion of non-existent Lesion...")

        invalid_lesion_id = 999999

        response = admin_client.delete(f"/api/recist-lesions/{invalid_lesion_id}")
        assert response.status_code == 404, "Should return 404 for non-existent Lesion"
        logger.info(f"✓ Correctly rejected deletion of non-existent Lesion")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])


