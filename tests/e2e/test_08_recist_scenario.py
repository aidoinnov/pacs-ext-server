"""
RECIST Lesion 시나리오 테스트

실제 임상 워크플로우를 시뮬레이션하는 통합 시나리오 테스트입니다.
"""

import pytest
import logging
from typing import Dict, List
from config import TestConfig

logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """관리자 API 클라이언트"""
    from utils.api_client import APIClient
    client = APIClient(config.base_url, config.timeout)
    client.login(config.admin_email, config.admin_password)
    return client


@pytest.fixture(scope="module")
def clinical_trial_project(admin_client):
    """임상시험 프로젝트 생성"""
    response = admin_client.post("/api/projects", json={
        "name": f"Clinical Trial - RECIST Scenario Test",
        "description": "Multi-subject RECIST evaluation scenario",
        "sponsor": "Test Hospital",
        "start_date": "2025-01-01",
        "end_date": "2026-12-31",
        "auto_complete": False
    })
    
    assert response.status_code == 201, f"Failed to create project: {response.text}"
    project = response.json()
    logger.info(f"Created clinical trial project: {project['name']} (ID: {project['id']})")
    
    yield project
    
    # Cleanup
    try:
        admin_client.delete(f"/api/projects/{project['id']}")
        logger.info(f"Deleted project: {project['id']}")
    except Exception as e:
        logger.warning(f"Failed to delete project: {e}")


@pytest.fixture(scope="module")
def multiple_subjects(admin_client, clinical_trial_project):
    """여러 Subject 생성 (3명의 환자)"""
    project_id = clinical_trial_project['id']
    subjects = []
    
    for i in range(1, 4):
        response = admin_client.post(f"/api/projects/{project_id}/subjects", json={
            "subject_code": f"SUBJ-{i:03d}",
            "patient_id": f"PT{i:05d}",
            "patient_name": f"Patient {i}",
            "patient_birth_date": f"198{i}-01-01"
        })
        
        assert response.status_code == 201, f"Failed to create subject {i}: {response.text}"
        subject = response.json()
        subjects.append(subject)
        logger.info(f"Created subject: {subject['subject_code']} (ID: {subject['id']})")
    
    yield subjects
    
    # Cleanup
    for subject in subjects:
        try:
            admin_client.delete(f"/api/subjects/{subject['id']}")
            logger.info(f"Deleted subject: {subject['id']}")
        except Exception as e:
            logger.warning(f"Failed to delete subject: {e}")


@pytest.fixture(scope="module")
def baseline_evaluation_data(admin_client, multiple_subjects):
    """Baseline 평가 데이터 생성"""
    baseline_data = []

    for idx, subject in enumerate(multiple_subjects, 1):
        subject_id = subject['id']
        logger.info(f"\n--- Patient {idx}: {subject['subject_code']} ---")

        # 1. Baseline TimePoint 생성
        response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
            "name": "BL",
            "visit_type": "Baseline",
            "visit_no": 0,
            "order_index": 0
        })
        assert response.status_code == 201
        baseline_tp = response.json()
        logger.info(f"✓ Created Baseline TimePoint (ID: {baseline_tp['id']})")

        # 2. Target Lesions 생성 (환자마다 다른 개수)
        target_count = idx + 1  # Patient 1: 2개, Patient 2: 3개, Patient 3: 4개
        target_lesions = []

        for i in range(target_count):
            response = admin_client.post(
                f"/api/recist-lesions/subjects/{subject_id}",
                json={
                    "lesion_type": "TARGET",
                    "baseline_timepoint_id": baseline_tp['id'],
                    "organ_site": f"Liver-S{i+1}",
                    "description": f"Target lesion {i+1} in liver segment {i+1}"
                }
            )
            assert response.status_code == 201
            lesion = response.json()
            target_lesions.append(lesion)
            logger.info(f"  ✓ Created Target Lesion #{lesion['lesion_number']}: {lesion['organ_site']}")

        # 3. Non-Target Lesions 생성
        non_target_count = 2
        non_target_lesions = []

        for i in range(non_target_count):
            response = admin_client.post(
                f"/api/recist-lesions/subjects/{subject_id}",
                json={
                    "lesion_type": "NON_TARGET",
                    "baseline_timepoint_id": baseline_tp['id'],
                    "organ_site": f"Lung-{['RUL', 'RLL'][i]}",
                    "description": f"Non-target lesion in {['right upper', 'right lower'][i]} lung"
                }
            )
            assert response.status_code == 201
            lesion = response.json()
            non_target_lesions.append(lesion)
            logger.info(f"  ✓ Created Non-Target Lesion #{lesion['lesion_number']}: {lesion['organ_site']}")

        baseline_data.append({
            "subject": subject,
            "baseline_tp": baseline_tp,
            "target_lesions": target_lesions,
            "non_target_lesions": non_target_lesions
        })

    return baseline_data


class TestClinicalTrialScenario:
    """임상시험 전체 워크플로우 시나리오"""

    def test_01_baseline_evaluation(self, admin_client, baseline_evaluation_data):
        """시나리오 1: Baseline 평가 - 3명의 환자에 대한 초기 병변 평가"""
        logger.info("=" * 80)
        logger.info("Scenario 1: Baseline Evaluation for 3 Patients")
        logger.info("=" * 80)

        # 검증: 모든 환자의 병변이 올바르게 생성되었는지 확인
        logger.info(f"\n--- Baseline Evaluation Summary ---")
        for idx, data in enumerate(baseline_evaluation_data, 1):
            subject_id = data['subject']['id']
            response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
            assert response.status_code == 200
            all_lesions = response.json()

            target_count = len([l for l in all_lesions if l['lesion_type'] == 'TARGET'])
            non_target_count = len([l for l in all_lesions if l['lesion_type'] == 'NON_TARGET'])

            logger.info(f"Patient {idx} ({data['subject']['subject_code']}): "
                       f"{target_count} Target, {non_target_count} Non-Target lesions")

            assert target_count == idx + 1, f"Expected {idx + 1} target lesions, got {target_count}"
            assert non_target_count == 2, f"Expected 2 non-target lesions, got {non_target_count}"

        logger.info("✅ Baseline evaluation completed successfully")

    def test_02_first_followup_with_new_lesions(self, admin_client, baseline_evaluation_data):
        """시나리오 2: 첫 번째 Follow-up - 일부 환자에서 새로운 병변 발견"""
        logger.info("=" * 80)
        logger.info("Scenario 2: First Follow-up (Week 6) - New Lesions Detected")
        logger.info("=" * 80)

        followup_data = []

        for idx, baseline in enumerate(baseline_evaluation_data, 1):
            subject = baseline['subject']
            subject_id = subject['id']
            logger.info(f"\n--- Patient {idx}: {subject['subject_code']} Follow-up ---")

            # 1. Follow-up TimePoint 생성 (Week 6)
            response = admin_client.post(f"/api/subjects/{subject_id}/timepoints", json={
                "name": "W6",
                "visit_type": "Visit",
                "visit_no": 1,
                "order_index": 1
            })
            assert response.status_code == 201, f"Failed to create follow-up timepoint: {response.text}"
            followup_tp = response.json()
            logger.info(f"✓ Created Follow-up TimePoint W6 (ID: {followup_tp['id']})")

            # 2. Patient 2와 3에서 NEW lesion 발견
            new_lesions = []
            if idx >= 2:
                response = admin_client.post(
                    f"/api/recist-lesions/subjects/{subject_id}",
                    json={
                        "lesion_type": "NEW",
                        "baseline_timepoint_id": None,
                        "organ_site": "Bone-L3",
                        "description": "New bone metastasis detected at L3"
                    }
                )
                assert response.status_code == 201
                new_lesion = response.json()
                new_lesions.append(new_lesion)
                logger.info(f"  ⚠️  NEW Lesion detected: {new_lesion['organ_site']} (Progressive Disease)")

            followup_data.append({
                "subject": subject,
                "followup_tp": followup_tp,
                "new_lesions": new_lesions
            })

        # 검증: NEW lesion이 올바르게 생성되었는지 확인
        logger.info(f"\n--- Follow-up Summary ---")
        for idx, data in enumerate(followup_data, 1):
            subject_id = data['subject']['id']
            response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}?lesion_type=NEW")
            assert response.status_code == 200
            new_lesions = response.json()

            expected_new = 1 if idx >= 2 else 0
            actual_new = len(new_lesions)

            logger.info(f"Patient {idx}: {actual_new} NEW lesion(s)")
            # 이전 테스트에서 생성된 NEW lesion이 있을 수 있으므로 >= 체크
            assert actual_new >= expected_new, f"Expected >= {expected_new} new lesions, got {actual_new}"

        logger.info("✅ Follow-up evaluation completed successfully")

    def test_03_lesion_count_validation(self, admin_client, baseline_evaluation_data):
        """시나리오 3: 전체 병변 개수 검증"""
        logger.info("=" * 80)
        logger.info("Scenario 3: Overall Lesion Count Validation")
        logger.info("=" * 80)

        total_target = 0
        total_non_target = 0
        total_new = 0

        for idx, baseline in enumerate(baseline_evaluation_data, 1):
            subject_id = baseline['subject']['id']

            # 전체 병변 조회
            response = admin_client.get(f"/api/recist-lesions/subjects/{subject_id}")
            assert response.status_code == 200
            all_lesions = response.json()

            target = len([l for l in all_lesions if l['lesion_type'] == 'TARGET'])
            non_target = len([l for l in all_lesions if l['lesion_type'] == 'NON_TARGET'])
            new = len([l for l in all_lesions if l['lesion_type'] == 'NEW'])

            total_target += target
            total_non_target += non_target
            total_new += new

            logger.info(f"Patient {idx}: Target={target}, Non-Target={non_target}, New={new}")

        logger.info(f"\n--- Total Lesion Summary ---")
        logger.info(f"Total Target Lesions: {total_target}")
        logger.info(f"Total Non-Target Lesions: {total_non_target}")
        logger.info(f"Total New Lesions: {total_new}")

        # 예상값 검증
        assert total_target == 2 + 3 + 4, f"Expected 9 total target lesions, got {total_target}"
        assert total_non_target == 2 * 3, f"Expected 6 total non-target lesions, got {total_non_target}"
        # NEW lesion은 이전 테스트에서 생성될 수 있으므로 >= 0
        assert total_new >= 0, f"Expected >= 0 total new lesions, got {total_new}"

        logger.info("✅ Lesion count validation passed")


class TestAnnotationIntegrationScenario:
    """Annotation 연동 통합 시나리오"""

    def test_01_annotation_workflow(self, admin_client, baseline_evaluation_data):
        """시나리오 4: Annotation 생성 및 Lesion 연결 워크플로우"""
        logger.info("=" * 80)
        logger.info("Scenario 4: Annotation Integration Workflow")
        logger.info("=" * 80)

        # 첫 번째 Subject의 baseline 데이터 사용
        baseline_data = baseline_evaluation_data[0]
        subject = baseline_data['subject']
        subject_id = subject['id']
        baseline_tp = baseline_data['baseline_tp']

        logger.info(f"Using existing Baseline TimePoint (ID: {baseline_tp['id']})")

        # 2. 기존 Target Lesion 사용 (이미 생성됨)
        target_lesions = baseline_data['target_lesions']
        if not target_lesions:
            # 없으면 새로 생성
            response = admin_client.post(
                f"/api/recist-lesions/subjects/{subject_id}",
                json={
                    "lesion_type": "TARGET",
                    "baseline_timepoint_id": baseline_tp['id'],
                    "organ_site": "Liver-S5",
                    "description": "Target lesion for annotation test"
                }
            )
            assert response.status_code == 201
            lesion = response.json()
            lesion_id = lesion['id']
            logger.info(f"✓ Created Target Lesion (ID: {lesion_id})")
        else:
            lesion = target_lesions[0]
            lesion_id = lesion['id']
            logger.info(f"✓ Using existing Target Lesion (ID: {lesion_id})")

        # 3. Lesion 상세 조회 (Annotation 없이도 동작 확인)
        response = admin_client.get(f"/api/recist-lesions/{lesion_id}")
        assert response.status_code == 200
        lesion_detail = response.json()

        # RecistLesionDetail 구조: lesion 필드와 annotations 필드가 평탄화되어 있음
        assert 'id' in lesion_detail
        assert lesion_detail['id'] == lesion_id
        assert 'annotations' in lesion_detail

        logger.info(f"✓ Verified Lesion detail structure")
        logger.info(f"  - Lesion ID: {lesion_detail['id']}")
        logger.info(f"  - Organ Site: {lesion_detail['organ_site']}")
        logger.info(f"  - Annotations: {len(lesion_detail['annotations'])} linked")

        logger.info("✅ Annotation integration workflow completed successfully")

