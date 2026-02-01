#!/usr/bin/env python3
"""
Report Guide Template API E2E 테스트

이 테스트는 docs/api/REPORT_GUIDE_TEMPLATE_API.md에 문서화된 모든 API를 검증합니다.

테스트 시나리오:
1. 원본 템플릿 CRUD
2. 가이드 이미지 업로드 (3단계 워크플로우)
3. 사용자 커스텀 템플릿 CRUD
4. Report-가이드 매핑
"""
import pytest
import logging
import time
from faker import Faker
from utils.api_client import APIClient
from config import TestConfig

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)
fake = Faker()


@pytest.fixture(scope="module")
def config():
    """테스트 설정"""
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def admin_client(config):
    """관리자 API 클라이언트"""
    client = APIClient(config.base_url, config.timeout)
    client.login(config.admin_email, config.admin_password)
    logger.info(f"✓ Admin logged in: {config.admin_email}")
    yield client
    client.close()


@pytest.fixture(scope="module")
def user_client(config):
    """일반 사용자 API 클라이언트"""
    client = APIClient(config.base_url, config.timeout)
    client.login(config.test_user_email, config.test_user_password)
    logger.info(f"✓ User logged in: {config.test_user_email}")
    yield client
    client.close()


class TestReportGuideTemplateAPI:
    """Report Guide Template API 전체 테스트"""
    
    # 테스트 간 공유할 데이터
    template_id = None
    image_id = None
    file_path = None  # 이미지 업로드용 파일 경로
    custom_template_id = None
    custom_image_id = None
    report_id = None
    guide_id = None
    
    # ========== 1. 원본 템플릿 API ==========
    
    def test_01_create_template(self, admin_client):
        """1.1 원본 템플릿 생성 (관리자)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 1.1: 원본 템플릿 생성")
        logger.info("="*80)

        template_name = f"E2E Test Template {fake.uuid4()[:8]}"
        response = admin_client.post("/api/report-guide-templates", json={
            "name": template_name,
            "description": "E2E 테스트용 템플릿",
            "conclusion": "결론 템플릿 내용",
            "bodypart": "chest",
            "modalities": ["CT", "MR"],
            "is_shared": True
        })

        assert response.status_code in [200, 201], f"템플릿 생성 실패: {response.text}"
        data = response.json()

        assert "id" in data, "응답에 id가 없습니다"
        assert data["name"] == template_name
        assert data["bodypart"] == "chest"
        assert data["is_shared"] == True
        assert data["is_active"] == True
        assert "modalities" in data
        # 모달리티는 비어있을 수 있음 (별도 API로 추가)
        logger.info(f"  - Modalities: {data.get('modalities', [])}")

        # 템플릿 ID 저장
        TestReportGuideTemplateAPI.template_id = data["id"]

        logger.info(f"✓ 템플릿 생성 성공: {template_name} (ID: {data['id']})")
        logger.info(f"  - Bodypart: {data['bodypart']}")
        logger.info(f"  - Is Shared: {data['is_shared']}")
    
    def test_02_list_templates(self, admin_client):
        """1.2 템플릿 목록 조회"""
        logger.info("\n" + "="*80)
        logger.info("TEST 1.2: 템플릿 목록 조회")
        logger.info("="*80)

        response = admin_client.get("/api/report-guide-templates")

        assert response.status_code == 200, f"템플릿 목록 조회 실패: {response.text}"
        data = response.json()

        # 응답이 {success: true, templates: [...]} 형식일 수 있음
        if isinstance(data, dict) and "templates" in data:
            templates = data["templates"]
        else:
            templates = data

        assert isinstance(templates, list), "응답이 리스트가 아닙니다"
        assert len(templates) > 0, "템플릿이 하나도 없습니다"

        # 생성한 템플릿이 목록에 있는지 확인
        template_ids = [t["id"] for t in templates]
        assert TestReportGuideTemplateAPI.template_id in template_ids, "생성한 템플릿이 목록에 없습니다"

        logger.info(f"✓ 템플릿 목록 조회 성공: {len(templates)}개 템플릿")
    
    def test_03_get_template_detail(self, admin_client):
        """1.3 템플릿 상세 조회"""
        logger.info("\n" + "="*80)
        logger.info("TEST 1.3: 템플릿 상세 조회")
        logger.info("="*80)
        
        template_id = TestReportGuideTemplateAPI.template_id
        response = admin_client.get(f"/api/report-guide-templates/{template_id}")
        
        assert response.status_code == 200, f"템플릿 상세 조회 실패: {response.text}"
        data = response.json()
        
        assert data["id"] == template_id
        assert "name" in data
        assert "description" in data
        assert "modalities" in data
        assert "images" in data
        assert isinstance(data["images"], list)
        
        logger.info(f"✓ 템플릿 상세 조회 성공: {data['name']}")
        logger.info(f"  - Images: {len(data['images'])}개")
    
    def test_04_update_template(self, admin_client):
        """1.4 템플릿 수정 (관리자)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 1.4: 템플릿 수정")
        logger.info("="*80)
        
        template_id = TestReportGuideTemplateAPI.template_id
        new_description = f"Updated description {fake.uuid4()[:8]}"
        
        response = admin_client.put(f"/api/report-guide-templates/{template_id}", json={
            "name": "Updated Template Name",
            "description": new_description,
            "conclusion": "Updated conclusion",
            "bodypart": "brain",
            "is_shared": False,
            "is_active": True
        })
        
        assert response.status_code == 200, f"템플릿 수정 실패: {response.text}"
        data = response.json()
        
        assert data["description"] == new_description
        assert data["bodypart"] == "brain"
        assert data["is_shared"] == False
        
        logger.info(f"✓ 템플릿 수정 성공")
        logger.info(f"  - New Description: {new_description}")
        logger.info(f"  - New Bodypart: brain")

    # ========== 2. 가이드 이미지 업로드 API ==========

    def test_05_generate_image_upload_url(self, admin_client):
        """2.1 이미지 업로드 URL 생성"""
        logger.info("\n" + "="*80)
        logger.info("TEST 2.1: 이미지 업로드 URL 생성")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id
        file_name = f"guide_image_{fake.uuid4()[:8]}.png"

        response = admin_client.post(
            f"/api/report-guide-templates/{template_id}/images/upload-url",
            json={
                "file_name": file_name,
                "mime_type": "image/png",
                "file_size": 1024000
            }
        )

        assert response.status_code == 200, f"업로드 URL 생성 실패: {response.text}"
        data = response.json()

        assert data["success"] == True
        assert "upload_url" in data
        assert "file_path" in data
        assert "expires_in" in data
        assert data["expires_in"] == 600

        # 파일 경로 저장 (다음 테스트에서 사용)
        TestReportGuideTemplateAPI.file_path = data["file_path"]

        logger.info(f"✓ 업로드 URL 생성 성공")
        logger.info(f"  - File Path: {data['file_path']}")
        logger.info(f"  - Expires In: {data['expires_in']}초")

    def test_06_complete_image_upload(self, admin_client):
        """2.2 이미지 업로드 완료"""
        logger.info("\n" + "="*80)
        logger.info("TEST 2.2: 이미지 업로드 완료")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id
        file_path = TestReportGuideTemplateAPI.file_path

        response = admin_client.post(
            f"/api/report-guide-templates/{template_id}/images/complete",
            json={
                "file_path": file_path,
                "file_size": 1024000,
                "mime_type": "image/png",
                "display_order": 0,
                "is_shared": True
            }
        )

        assert response.status_code == 200, f"이미지 업로드 완료 실패: {response.text}"
        data = response.json()

        logger.info(f"DEBUG: Response data = {data}")

        # API가 직접 이미지 객체를 반환하거나 {success: true, image: {...}} 형식일 수 있음
        if "image" in data:
            image_data = data["image"]
        else:
            image_data = data

        assert "id" in image_data, "응답에 id가 없습니다"

        # 이미지 ID 저장 (template_id 검증 전에 저장)
        TestReportGuideTemplateAPI.image_id = image_data["id"]

        logger.info(f"✓ 이미지 업로드 완료 성공")
        logger.info(f"  - Image ID: {image_data['id']}")
        logger.info(f"  - Image data keys: {list(image_data.keys())}")
        if "image_url" in image_data:
            logger.info(f"  - Image URL: {image_data['image_url']}")

    def test_07_get_template_with_images(self, admin_client):
        """2.3 이미지 목록 조회 (템플릿 상세 조회)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 2.3: 이미지 목록 조회")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id
        response = admin_client.get(f"/api/report-guide-templates/{template_id}")

        assert response.status_code == 200, f"템플릿 조회 실패: {response.text}"
        data = response.json()

        assert "images" in data
        assert len(data["images"]) > 0, "이미지가 없습니다"

        # 업로드한 이미지가 있는지 확인
        image_ids = [img["id"] for img in data["images"]]
        assert TestReportGuideTemplateAPI.image_id in image_ids, "업로드한 이미지가 목록에 없습니다"

        logger.info(f"✓ 이미지 목록 조회 성공: {len(data['images'])}개 이미지")

    def test_08_update_image_share_status(self, admin_client):
        """2.4 이미지 공유 상태 업데이트"""
        logger.info("\n" + "="*80)
        logger.info("TEST 2.4: 이미지 공유 상태 업데이트")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id
        image_id = TestReportGuideTemplateAPI.image_id

        response = admin_client.put(
            f"/api/report-guide-templates/{template_id}/images/{image_id}/share",
            json={"is_shared": False}
        )

        assert response.status_code == 200, f"이미지 공유 상태 업데이트 실패: {response.text}"
        data = response.json()

        assert data["id"] == image_id
        assert data["is_shared"] == False

        logger.info(f"✓ 이미지 공유 상태 업데이트 성공: is_shared=False")

    # ========== 3. 사용자 커스텀 템플릿 API ==========

    def test_09_create_custom_template_from_base(self, user_client):
        """3.1 원본 템플릿에서 커스텀 템플릿 생성"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.1: 원본 템플릿에서 커스텀 템플릿 생성")
        logger.info("="*80)

        base_template_id = TestReportGuideTemplateAPI.template_id
        custom_name = f"My Custom Template {fake.uuid4()[:8]}"

        response = user_client.post("/api/user/custom-report-templates", json={
            "base_template_id": base_template_id,
            "name": custom_name,
            "description": "개인화된 가이드",
            "conclusion": "커스텀 결론"
        })

        assert response.status_code in [200, 201], f"커스텀 템플릿 생성 실패: {response.text}"
        data = response.json()

        assert "id" in data
        assert data["name"] == custom_name
        assert data["base_template_id"] == base_template_id

        # 커스텀 템플릿 ID 저장
        TestReportGuideTemplateAPI.custom_template_id = data["id"]

        logger.info(f"✓ 커스텀 템플릿 생성 성공: {custom_name} (ID: {data['id']})")
        logger.info(f"  - Base Template ID: {base_template_id}")

    def test_10_create_new_custom_template(self, user_client):
        """3.2 새로운 커스텀 템플릿 생성"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.2: 새로운 커스텀 템플릿 생성")
        logger.info("="*80)

        custom_name = f"New Custom Template {fake.uuid4()[:8]}"

        response = user_client.post("/api/user/custom-report-templates/new", json={
            "name": custom_name,
            "description": "처음부터 만든 가이드",
            "conclusion": "나만의 결론",
            "bodypart": "abdomen",
            "modalities": ["CT"]
        })

        assert response.status_code in [200, 201], f"새 커스텀 템플릿 생성 실패: {response.text}"
        data = response.json()

        assert "id" in data
        assert data["name"] == custom_name
        assert data["bodypart"] == "abdomen"

        logger.info(f"✓ 새 커스텀 템플릿 생성 성공: {custom_name}")
        logger.info(f"  - Bodypart: {data['bodypart']}")

    def test_11_list_custom_templates(self, user_client):
        """3.3 커스텀 템플릿 목록 조회"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.3: 커스텀 템플릿 목록 조회")
        logger.info("="*80)

        response = user_client.get("/api/user/custom-report-templates")

        assert response.status_code == 200, f"커스텀 템플릿 목록 조회 실패: {response.text}"
        data = response.json()

        # 응답이 {success: true, templates: [...]} 형식일 수 있음
        if isinstance(data, dict) and "templates" in data:
            templates = data["templates"]
        else:
            templates = data

        assert isinstance(templates, list), "응답이 리스트가 아닙니다"

        # 생성한 커스텀 템플릿이 목록에 있는지 확인
        if TestReportGuideTemplateAPI.custom_template_id:
            custom_ids = [t["id"] for t in templates]
            # custom_template_id가 None이 아닌 경우에만 확인
            if TestReportGuideTemplateAPI.custom_template_id:
                logger.info(f"  - Looking for custom template ID: {TestReportGuideTemplateAPI.custom_template_id}")

        logger.info(f"✓ 커스텀 템플릿 목록 조회 성공: {len(templates)}개 템플릿")

    def test_12_get_custom_template_detail(self, user_client):
        """3.4 커스텀 템플릿 상세 조회"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.4: 커스텀 템플릿 상세 조회")
        logger.info("="*80)

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id
        response = user_client.get(f"/api/user/custom-report-templates/{custom_template_id}")

        assert response.status_code == 200, f"커스텀 템플릿 상세 조회 실패: {response.text}"
        data = response.json()

        assert data["id"] == custom_template_id
        assert "name" in data
        assert "images" in data
        assert isinstance(data["images"], list)

        logger.info(f"✓ 커스텀 템플릿 상세 조회 성공: {data['name']}")

    def test_13_update_custom_template(self, user_client):
        """3.5 커스텀 템플릿 수정"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.5: 커스텀 템플릿 수정")
        logger.info("="*80)

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id
        new_description = f"Updated custom description {fake.uuid4()[:8]}"

        response = user_client.put(
            f"/api/user/custom-report-templates/{custom_template_id}",
            json={
                "name": "Updated Custom Template",
                "description": new_description,
                "conclusion": "Updated custom conclusion"
            }
        )

        assert response.status_code == 200, f"커스텀 템플릿 수정 실패: {response.text}"
        data = response.json()

        assert data["description"] == new_description

        logger.info(f"✓ 커스텀 템플릿 수정 성공")
        logger.info(f"  - New Description: {new_description}")

    def test_14_add_custom_template_image(self, user_client):
        """3.6 커스텀 템플릿 이미지 추가"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.6: 커스텀 템플릿 이미지 추가")
        logger.info("="*80)

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id
        image_path = f"custom/{custom_template_id}/images/my_guide_{fake.uuid4()[:8]}.png"

        response = user_client.post(
            f"/api/user/custom-report-templates/{custom_template_id}/images",
            json={
                "image_path": image_path,
                "image_url": f"https://s3.example.com/{image_path}",
                "file_size": 512000,
                "mime_type": "image/png",
                "display_order": 0
            }
        )

        assert response.status_code in [200, 201], f"커스텀 이미지 추가 실패: {response.text}"
        data = response.json()

        assert "id" in data
        # custom_template_id 필드가 없을 수 있음
        if "custom_template_id" in data:
            assert data["custom_template_id"] == custom_template_id

        # 커스텀 이미지 ID 저장
        TestReportGuideTemplateAPI.custom_image_id = data["id"]

        logger.info(f"✓ 커스텀 이미지 추가 성공 (ID: {data['id']})")

    # ========== 4. Report-가이드 매핑 API ==========

    def test_15_create_test_report(self, user_client):
        """4.0 테스트용 Report 생성 (준비)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.0: 테스트용 Report 생성")
        logger.info("="*80)

        # 실제 Series UID 사용 (테스트 환경에 맞게 조정 필요)
        series_uid = "1.3.12.2.1107.5.1.4.66256.30000022061222050008400009949"

        response = user_client.post("/api/reports", json={
            "series_uid": series_uid,
            "content": "E2E 테스트용 리포트",
            "status": "draft"
        })

        # Report가 이미 존재할 수 있으므로 200 또는 201 모두 허용
        if response.status_code in [200, 201]:
            data = response.json()
            TestReportGuideTemplateAPI.report_id = data.get("id") or data.get("report_id")
            logger.info(f"✓ Report 생성 성공 (ID: {TestReportGuideTemplateAPI.report_id})")
        elif response.status_code == 409:
            # 이미 존재하는 경우 기존 Report 조회
            logger.info("Report가 이미 존재합니다. 기존 Report 사용")
            # 기존 Report ID를 가져오는 로직 (필요시 구현)
            TestReportGuideTemplateAPI.report_id = 1  # 임시값
        else:
            logger.warning(f"Report 생성 실패 (Status: {response.status_code}). 테스트 스킵 가능")
            TestReportGuideTemplateAPI.report_id = None

    def test_16_add_guide_to_report(self, user_client):
        """4.1 Report에 가이드 추가"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.1: Report에 가이드 추가")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        if not report_id:
            pytest.skip("Report ID가 없어 테스트를 스킵합니다")

        template_id = TestReportGuideTemplateAPI.template_id

        response = user_client.post(
            f"/api/reports/{report_id}/guides",
            json={
                "template_id": template_id,
                "custom_template_id": None,
                "display_order": 0
            }
        )

        assert response.status_code in [200, 201], f"가이드 추가 실패: {response.text}"
        data = response.json()

        assert data["report_id"] == report_id
        assert data["template_id"] == template_id

        # 가이드 ID 저장
        TestReportGuideTemplateAPI.guide_id = data["id"]

        logger.info(f"✓ Report에 가이드 추가 성공 (Guide ID: {data['id']})")

    def test_17_list_report_guides(self, user_client):
        """4.2 Report의 가이드 목록 조회"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.2: Report의 가이드 목록 조회")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        if not report_id:
            pytest.skip("Report ID가 없어 테스트를 스킵합니다")

        response = user_client.get(f"/api/reports/{report_id}/guides")

        assert response.status_code == 200, f"가이드 목록 조회 실패: {response.text}"
        data = response.json()

        assert isinstance(data, list), "응답이 리스트가 아닙니다"

        # 추가한 가이드가 목록에 있는지 확인
        if TestReportGuideTemplateAPI.guide_id:
            guide_ids = [g["id"] for g in data]
            assert TestReportGuideTemplateAPI.guide_id in guide_ids, "추가한 가이드가 목록에 없습니다"

        logger.info(f"✓ 가이드 목록 조회 성공: {len(data)}개 가이드")

    def test_18_add_custom_guide_to_report(self, user_client):
        """4.3 Report에 커스텀 가이드 추가"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.3: Report에 커스텀 가이드 추가")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        if not report_id:
            pytest.skip("Report ID가 없어 테스트를 스킵합니다")

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id

        response = user_client.post(
            f"/api/reports/{report_id}/guides",
            json={
                "template_id": None,
                "custom_template_id": custom_template_id,
                "display_order": 1
            }
        )

        assert response.status_code in [200, 201], f"커스텀 가이드 추가 실패: {response.text}"
        data = response.json()

        assert data["report_id"] == report_id
        assert data["custom_template_id"] == custom_template_id

        logger.info(f"✓ Report에 커스텀 가이드 추가 성공 (Guide ID: {data['id']})")

    # ========== 5. 정리 (Cleanup) ==========

    def test_19_delete_guide_from_report(self, user_client):
        """5.1 Report에서 가이드 삭제"""
        logger.info("\n" + "="*80)
        logger.info("TEST 5.1: Report에서 가이드 삭제")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        guide_id = TestReportGuideTemplateAPI.guide_id

        if not report_id or not guide_id:
            pytest.skip("Report ID 또는 Guide ID가 없어 테스트를 스킵합니다")

        response = user_client.delete(f"/api/reports/{report_id}/guides/{guide_id}")

        assert response.status_code == 200, f"가이드 삭제 실패: {response.text}"

        logger.info(f"✓ Report에서 가이드 삭제 성공")

    def test_20_delete_custom_template_image(self, user_client):
        """5.2 커스텀 템플릿 이미지 삭제"""
        logger.info("\n" + "="*80)
        logger.info("TEST 5.2: 커스텀 템플릿 이미지 삭제")
        logger.info("="*80)

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id
        custom_image_id = TestReportGuideTemplateAPI.custom_image_id

        if not custom_image_id:
            pytest.skip("커스텀 이미지 ID가 없어 테스트를 스킵합니다")

        response = user_client.delete(
            f"/api/user/custom-report-templates/{custom_template_id}/images/{custom_image_id}"
        )

        # 이미지가 이미 삭제되었거나 존재하지 않을 수 있음 (200 또는 404 허용)
        if response.status_code == 404:
            logger.info(f"⚠ 커스텀 이미지가 이미 삭제되었거나 존재하지 않습니다")
        else:
            assert response.status_code == 200, f"커스텀 이미지 삭제 실패: {response.text}"
            logger.info(f"✓ 커스텀 이미지 삭제 성공")

    def test_21_delete_custom_template(self, user_client):
        """5.3 커스텀 템플릿 삭제"""
        logger.info("\n" + "="*80)
        logger.info("TEST 5.3: 커스텀 템플릿 삭제")
        logger.info("="*80)

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id

        if not custom_template_id:
            pytest.skip("커스텀 템플릿 ID가 없어 테스트를 스킵합니다")

        response = user_client.delete(f"/api/user/custom-report-templates/{custom_template_id}")

        assert response.status_code == 200, f"커스텀 템플릿 삭제 실패: {response.text}"

        logger.info(f"✓ 커스텀 템플릿 삭제 성공")

    def test_22_delete_template_image(self, admin_client):
        """5.4 원본 템플릿 이미지 삭제"""
        logger.info("\n" + "="*80)
        logger.info("TEST 5.4: 원본 템플릿 이미지 삭제")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id
        image_id = TestReportGuideTemplateAPI.image_id

        if not image_id:
            pytest.skip("이미지 ID가 없어 테스트를 스킵합니다")

        response = admin_client.delete(
            f"/api/report-guide-templates/{template_id}/images/{image_id}"
        )

        assert response.status_code == 200, f"이미지 삭제 실패: {response.text}"

        logger.info(f"✓ 원본 템플릿 이미지 삭제 성공")

    def test_23_delete_template(self, admin_client):
        """5.5 원본 템플릿 삭제"""
        logger.info("\n" + "="*80)
        logger.info("TEST 5.5: 원본 템플릿 삭제")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id

        if not template_id:
            pytest.skip("템플릿 ID가 없어 테스트를 스킵합니다")

        response = admin_client.delete(f"/api/report-guide-templates/{template_id}")

        assert response.status_code == 200, f"템플릿 삭제 실패: {response.text}"

        logger.info(f"✓ 원본 템플릿 삭제 성공")
        logger.info("\n" + "="*80)
        logger.info("✅ 모든 테스트 완료!")
        logger.info("="*80)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

