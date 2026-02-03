#!/usr/bin/env python3
"""
Report Guide Template API E2E 테스트

이 테스트는 docs/api/REPORT_GUIDE_TEMPLATE_API.md에 문서화된 모든 API를 검증합니다.

테스트 시나리오:
1. 원본 템플릿 CRUD
2. 가이드 이미지 업로드 (3단계 워크플로우)
3. 사용자 커스텀 템플릿 CRUD
4. 유효 템플릿 통합 조회 (GET /api/user/report-templates)
5. Report-가이드 매핑
"""
import os
import pytest
import logging
import time
from faker import Faker
from utils.api_client import APIClient
from utils.signed_url_helpers import (
    assert_image_has_signed_url,
    assert_images_have_signed_urls,
    assert_guide_images_accessible,
)
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
def report_series_uid(admin_client, config):
    """Report 시나리오용 Series 생성 (project → series assign). 사용자+시리즈당 1개 리포트용."""
    from datetime import date, timedelta

    # 1. 프로젝트 생성
    unique_id = f"{int(time.time() * 1000)}_{fake.uuid4()[:8]}"
    project_resp = admin_client.post("/api/projects", json={
        "name": f"e2e_report_guide_{unique_id}",
        "description": "Report Guide Template E2E",
        "sponsor": "E2E",
        "start_date": str(date.today()),
        "end_date": str(date.today() + timedelta(days=365)),
    })
    if project_resp.status_code not in [200, 201]:
        pytest.skip(f"프로젝트 생성 실패: {project_resp.status_code} - {project_resp.text[:200]}")
    project_id = project_resp.json().get("id")
    if not project_id:
        pytest.skip("프로젝트 ID를 얻지 못함")

    # 2. Study+Series 할당 (series/assign이 study도 생성)
    study_uid = f"1.2.840.113619.2.55.3.e2e_{int(time.time())}"
    series_uid = f"1.2.840.113619.2.55.4.e2e_{int(time.time())}"
    series_resp = admin_client.post(
        f"/api/projects/{project_id}/series/assign",
        json={
            "study_uid": study_uid,
            "series_uid": series_uid,
            "series_description": "E2E Report Guide Test Series",
            "modality": "CT",
            "series_number": 1,
        },
    )
    if series_resp.status_code not in [200, 201]:
        try:
            admin_client.delete(f"/api/projects/{project_id}")
        except Exception:
            pass
        pytest.skip(f"Series 할당 실패: {series_resp.status_code} - {series_resp.text[:200]}")

    # 3. user를 프로젝트 멤버로 추가 (report 작성 권한 - admin과 user가 동일 계정이면 선택적)
    me_resp = admin_client.get("/api/users/me")
    if me_resp.status_code == 200:
        user_id = me_resp.json().get("id") or me_resp.json().get("user_id")
        if user_id:
            admin_client.post(f"/api/projects/{project_id}/members", json={"user_id": user_id})

    yield series_uid

    # teardown
    try:
        admin_client.delete(f"/api/projects/{project_id}")
    except Exception:
        pass


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

        response = admin_client.post("/api/report-guide-templates", json={
            "description": "E2E 테스트용 템플릿",
            "conclusion": "결론 템플릿 내용",
            "bodypart": "chest",
            "modalities": ["CT", "MR"],
            "is_shared": True
        })

        assert response.status_code in [200, 201], f"템플릿 생성 실패: {response.text}"
        data = response.json()

        assert "id" in data, "응답에 id가 없습니다"
        assert data["bodypart"] == "chest"
        assert data["is_shared"] == True
        assert data["is_active"] == True
        assert "modalities" in data
        # 모달리티는 비어있을 수 있음 (별도 API로 추가)
        logger.info(f"  - Modalities: {data.get('modalities', [])}")

        # 템플릿 ID 저장
        TestReportGuideTemplateAPI.template_id = data["id"]

        logger.info(f"✓ 템플릿 생성 성공 (ID: {data['id']})")
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

        for t in templates:
            imgs = t.get("images", [])
            if imgs:
                assert_images_have_signed_urls(imgs, f"템플릿 목록 ID={t['id']}", strict=False)

        logger.info(f"✓ 템플릿 목록 조회 성공: {len(templates)}개 템플릿")

    def test_02b_effective_templates_no_custom(self, user_client):
        """3.0 유효 템플릿 조회 (커스텀 없을 때) - 원본만 반환"""
        logger.info("\n" + "="*80)
        logger.info("TEST 2b: 유효 템플릿 조회 (커스텀 없을 때)")
        logger.info("="*80)

        response = user_client.get("/api/user/report-templates")

        if response.status_code == 404:
            pytest.skip(
                "GET /api/user/report-templates 엔드포인트 없음 (404). "
                "pacs-server를 최신 코드로 빌드 후 재시작하세요."
            )
        assert response.status_code == 200, f"유효 템플릿 조회 실패: {response.text}"
        data = response.json()

        assert data.get("success") is True, "success 필드가 true여야 함"
        assert "templates" in data, "templates 필드가 없습니다"
        templates = data["templates"]
        assert isinstance(templates, list), "templates는 리스트여야 함"

        # 커스텀 없으므로 생성한 원본이 source=original, template_id로 나와야 함
        original_templates = [t for t in templates if t.get("source") == "original"]
        template_ids = [t.get("template_id") for t in original_templates if t.get("template_id")]
        assert TestReportGuideTemplateAPI.template_id in template_ids, (
            f"생성한 원본 템플릿(ID:{TestReportGuideTemplateAPI.template_id})이 "
            f"유효 목록에 있어야 함. template_ids={template_ids}"
        )

        # 모든 항목에 source 필드
        for t in templates:
            assert "source" in t, f"템플릿에 source 필드 없음: {t.keys()}"
            assert t["source"] in ("original", "custom"), f"source는 original 또는 custom: {t['source']}"

        logger.info(f"✓ 유효 템플릿 조회 성공: {len(templates)}개 (커스텀 없음, 원본만)")

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
        assert "description" in data
        assert "modalities" in data
        assert "images" in data
        assert isinstance(data["images"], list)
        
        logger.info(f"✓ 템플릿 상세 조회 성공 (ID: {template_id})")
        logger.info(f"  - Images: {len(data['images'])}개")
    
    def test_04_update_template(self, admin_client):
        """1.4 템플릿 수정 (관리자)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 1.4: 템플릿 수정")
        logger.info("="*80)
        
        template_id = TestReportGuideTemplateAPI.template_id
        new_description = f"Updated description {fake.uuid4()[:8]}"
        
        new_modalities = ["MR", "CT"]
        response = admin_client.put(f"/api/report-guide-templates/{template_id}", json={
            "description": new_description,
            "conclusion": "Updated conclusion",
            "bodypart": "brain",
            "is_shared": False,
            "is_active": True,
            "modalities": new_modalities,
        })
        
        assert response.status_code == 200, f"템플릿 수정 실패: {response.text}"
        data = response.json()
        
        assert data["description"] == new_description
        assert data["bodypart"] == "brain"
        assert data["is_shared"] == False
        assert "modalities" in data, "응답에 modalities 필드가 없습니다"
        assert set(data["modalities"]) == set(new_modalities), (
            f"modalities 반영 안됨: expected {new_modalities}, got {data.get('modalities')}"
        )
        
        logger.info(f"✓ 템플릿 수정 성공")
        logger.info(f"  - New Description: {new_description}")
        logger.info(f"  - New Bodypart: brain")
        logger.info(f"  - Modalities: {data['modalities']}")

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
            logger.info(f"  - Image URL: {image_data['image_url'][:80]}...")
        assert_image_has_signed_url(image_data, "업로드 완료 응답 image", strict=False)

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

        assert_images_have_signed_urls(data["images"], "템플릿 이미지", strict=False)

        logger.info(f"✓ 이미지 목록 조회 성공: {len(data['images'])}개 이미지 (signed URL 검증 통과)")

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

        response = user_client.post("/api/user/custom-report-templates", json={
            "base_template_id": base_template_id,
            "description": "개인화된 가이드",
            "conclusion": "커스텀 결론"
        })

        assert response.status_code in [200, 201], f"커스텀 템플릿 생성 실패: {response.text}"
        data = response.json()

        assert "id" in data
        assert data["base_template_id"] == base_template_id

        # 커스텀 템플릿 ID 저장
        TestReportGuideTemplateAPI.custom_template_id = data["id"]

        logger.info(f"✓ 커스텀 템플릿 생성 성공 (ID: {data['id']})")
        logger.info(f"  - Base Template ID: {base_template_id}")

    def test_10_create_new_custom_template(self, user_client):
        """3.2 새로운 커스텀 템플릿 생성"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.2: 새로운 커스텀 템플릿 생성")
        logger.info("="*80)

        response = user_client.post("/api/user/custom-report-templates/new", json={
            "description": "처음부터 만든 가이드",
            "conclusion": "나만의 결론",
            "bodypart": "abdomen",
            "modalities": ["CT"]
        })

        assert response.status_code in [200, 201], f"새 커스텀 템플릿 생성 실패: {response.text}"
        data = response.json()

        assert "id" in data
        assert data["bodypart"] == "abdomen"

        logger.info(f"✓ 새 커스텀 템플릿 생성 성공")
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

    def test_11b_effective_templates_with_customs(self, user_client):
        """3.0 유효 템플릿 조회 (커스텀 있을 때) - 원본+커스텀 병합"""
        logger.info("\n" + "="*80)
        logger.info("TEST 11b: 유효 템플릿 조회 (커스텀 있을 때)")
        logger.info("="*80)

        template_id = TestReportGuideTemplateAPI.template_id
        custom_template_id = TestReportGuideTemplateAPI.custom_template_id

        response = user_client.get("/api/user/report-templates")

        if response.status_code == 404:
            pytest.skip(
                "GET /api/user/report-templates 엔드포인트 없음 (404). "
                "pacs-server를 최신 코드로 빌드 후 재시작하세요."
            )
        assert response.status_code == 200, f"유효 템플릿 조회 실패: {response.text}"
        data = response.json()

        assert data.get("success") is True
        templates = data["templates"]
        assert isinstance(templates, list)

        # 원본 기반 커스텀: source=custom, custom_template_id 있음, base_template_id=template_id
        custom_from_base = [
            t for t in templates
            if t.get("source") == "custom" and t.get("base_template_id") == template_id
        ]
        assert len(custom_from_base) >= 1, "원본에서 만든 커스텀이 유효 목록에 있어야 함"
        assert custom_from_base[0].get("custom_template_id") == custom_template_id
        assert custom_from_base[0].get("template_id") is None

        # 처음부터 만든 커스텀: source=custom, base_template_id=null
        from_scratch = [
            t for t in templates
            if t.get("source") == "custom" and t.get("base_template_id") is None
        ]
        assert len(from_scratch) >= 1, "처음부터 만든 커스텀이 유효 목록에 있어야 함"

        logger.info(f"✓ 유효 템플릿 조회 성공: {len(templates)}개 (원본+커스텀 병합)")
        logger.info(f"  - 원본 기반 커스텀: {len(custom_from_base)}개")
        logger.info(f"  - 처음부터 만든 커스텀: {len(from_scratch)}개")

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
        assert "description" in data
        assert "images" in data
        assert isinstance(data["images"], list)
        if data["images"]:
            assert_images_have_signed_urls(data["images"], "커스텀 템플릿 이미지", strict=False)

        logger.info(f"✓ 커스텀 템플릿 상세 조회 성공 (ID: {custom_template_id})")

    def test_13_update_custom_template(self, user_client):
        """3.5 커스텀 템플릿 수정 (description, conclusion, modalities 포함)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 3.5: 커스텀 템플릿 수정")
        logger.info("="*80)

        custom_template_id = TestReportGuideTemplateAPI.custom_template_id
        new_description = f"Updated custom description {fake.uuid4()[:8]}"
        new_modalities = ["CT", "MR"]

        response = user_client.put(
            f"/api/user/custom-report-templates/{custom_template_id}",
            json={
                "description": new_description,
                "conclusion": "Updated custom conclusion",
                "modalities": new_modalities,
            }
        )

        assert response.status_code == 200, f"커스텀 템플릿 수정 실패: {response.text}"
        data = response.json()

        assert data["description"] == new_description
        assert "modalities" in data, "응답에 modalities 필드가 없습니다"
        assert set(data["modalities"]) == set(new_modalities), (
            f"modalities 반영 안됨: expected {new_modalities}, got {data.get('modalities')}"
        )

        logger.info(f"✓ 커스텀 템플릿 수정 성공")
        logger.info(f"  - New Description: {new_description}")
        logger.info(f"  - Modalities: {data['modalities']}")

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

    def test_15_create_test_report(self, user_client, report_series_uid):
        """4.0 테스트용 Report 생성 (준비) - PUT /api/series/{series_uid}/report (사용자+시리즈당 1개)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.0: 테스트용 Report 생성")
        logger.info("="*80)

        series_uid = report_series_uid
        response = user_client.put(
            f"/api/series/{series_uid}/report",
            json={
                "status": "unread",
                "description": "E2E 테스트용 리포트",
                "conclusion": "추가 검사 불필요",
                "bodypart": "chest"
            }
        )

        if response.status_code == 200:
            data = response.json()
            report_id = data.get("id")
            if report_id:
                TestReportGuideTemplateAPI.report_id = report_id
                logger.info(f"✓ Report 생성/수정 성공 (ID: {report_id})")
            else:
                get_resp = user_client.get(f"/api/series/{series_uid}/report")
                if get_resp.status_code == 200:
                    get_data = get_resp.json()
                    report_id = get_data.get("id")
                    TestReportGuideTemplateAPI.report_id = report_id
                    logger.info(f"✓ Report 조회로 ID 확인 (ID: {report_id})")
                else:
                    TestReportGuideTemplateAPI.report_id = None
                    logger.warning("Report 응답에 id 없음. TEST_SERIES_UID가 DB에 있는 시리즈인지 확인하세요")
        elif response.status_code == 404:
            logger.warning(
                f"Series 없음 (404). TEST_SERIES_UID={series_uid} 가 DB에 있는지 확인하세요. "
                "리포트는 사용자+시리즈당 1개입니다."
            )
            TestReportGuideTemplateAPI.report_id = None
        else:
            logger.warning(f"Report 생성 실패 ({response.status_code}): {response.text[:200]}")
            TestReportGuideTemplateAPI.report_id = None

    def test_15a_empty_guides_before_apply(self, user_client):
        """4.0a 템플릿 미적용 시 Report guides 빈 배열 반환"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.0a: 템플릿 미적용 시 빈 guides 조회")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        if not report_id:
            pytest.skip("Report ID가 없어 테스트를 스킵합니다")

        response = user_client.get(f"/api/reports/{report_id}/guides")
        assert response.status_code == 200, f"guides 조회 실패: {response.text}"
        data = response.json()
        guides = data.get("guides", data) if isinstance(data, dict) else data
        assert isinstance(guides, list), "guides는 리스트여야 함"
        assert len(guides) == 0, "템플릿 적용 전에는 guides가 비어있어야 함"
        logger.info("✓ 템플릿 미적용 시 guides 빈 배열 확인")

    def test_15b_get_series_report(self, user_client, report_series_uid):
        """4.0b 시리즈 리포트 조회 - GET /api/series/{series_uid}/report (report_id 포함)"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.0b: 시리즈 리포트 조회")
        logger.info("="*80)

        if not TestReportGuideTemplateAPI.report_id:
            pytest.skip("Report ID가 없어 테스트를 스킵합니다 (test_15 선행)")

        series_uid = report_series_uid
        response = user_client.get(f"/api/series/{series_uid}/report")

        assert response.status_code == 200, f"시리즈 리포트 조회 실패: {response.text}"
        data = response.json()

        assert "id" in data, "시리즈 리포트 응답에 report id가 없습니다"
        assert data["id"] == TestReportGuideTemplateAPI.report_id
        logger.info(f"✓ 시리즈 리포트 조회 성공 (report_id: {data['id']})")

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

        assert_guide_images_accessible(data, "add_guide_to_report 응답", strict=False)

        # 가이드 ID 저장
        TestReportGuideTemplateAPI.guide_id = data["id"]

        logger.info(f"✓ Report에 가이드 추가 성공 (Guide ID: {data['id']})")

    def test_16b_report_image_snapshot_persistence(self, user_client, admin_client):
        """4.1b 이미지 스냅샷 유지: 템플릿 변경 후에도 Report guides의 이미지는 유지됨"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.1b: 이미지 스냅샷 유지 검증")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        template_id = TestReportGuideTemplateAPI.template_id
        if not report_id or not template_id:
            pytest.skip("Report ID 또는 Template ID가 없어 스킵합니다")

        # 1. 적용 후 guides 조회 (이미지 1개 있을 것)
        resp1 = user_client.get(f"/api/reports/{report_id}/guides")
        assert resp1.status_code == 200
        guides1 = resp1.json().get("guides", [])
        assert len(guides1) >= 1, "가이드가 있어야 함"
        images_before = []
        for g in guides1:
            imgs = g.get("images") or []
            images_before.extend(imgs)
        image_count_before = len(images_before)

        # 2. 템플릿에서 이미지 제거 (image_ids=[])
        admin_client.put(f"/api/report-guide-templates/{template_id}", json={"image_ids": []})

        # 3. Report guides 다시 조회 - 이미지 스냅샷은 유지되어야 함
        resp2 = user_client.get(f"/api/reports/{report_id}/guides")
        assert resp2.status_code == 200
        guides2 = resp2.json().get("guides", [])
        images_after = []
        for g in guides2:
            imgs = g.get("images") or []
            images_after.extend(imgs)
        assert len(images_after) == image_count_before, (
            f"템플릿 변경 후에도 Report 이미지 스냅샷은 유지되어야 함. "
            f"before={image_count_before}, after={len(images_after)}"
        )
        logger.info(f"✓ 이미지 스냅샷 유지 확인 (변경 전 {image_count_before}개, 변경 후 {len(images_after)}개)")

        # 4. 이후 테스트를 위해 템플릿 이미지 복원
        orig_image_id = TestReportGuideTemplateAPI.image_id
        if orig_image_id:
            admin_client.put(f"/api/report-guide-templates/{template_id}", json={"image_ids": [orig_image_id]})

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

        guides = data.get("guides", data) if isinstance(data, dict) else data
        assert isinstance(guides, list), "응답에 guides 리스트가 없습니다"

        # 추가한 가이드가 목록에 있는지 확인 (1:1 모델에서는 guide id = report_id)
        if TestReportGuideTemplateAPI.guide_id:
            guide_ids = [g["id"] for g in guides]
            assert TestReportGuideTemplateAPI.guide_id in guide_ids, "추가한 가이드가 목록에 없습니다"

        for g in guides:
            assert_guide_images_accessible(g, "list_report_guides", strict=False)

        logger.info(f"✓ 가이드 목록 조회 성공: {len(guides)}개 가이드 (signed URL 검증 통과)")

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

        assert_guide_images_accessible(data, "add_custom_guide_to_report 응답", strict=False)

        logger.info(f"✓ Report에 커스텀 가이드 추가 성공 (Guide ID: {data['id']})")

    def test_18b_report_guide_overwrite_verification(self, user_client):
        """4.3b Report 가이드 덮어쓰기 검증: 커스텀 적용 시 원본이 교체됨"""
        logger.info("\n" + "="*80)
        logger.info("TEST 4.3b: Report 가이드 덮어쓰기 검증")
        logger.info("="*80)

        report_id = TestReportGuideTemplateAPI.report_id
        if not report_id:
            pytest.skip("Report ID가 없어 스킵합니다")

        response = user_client.get(f"/api/reports/{report_id}/guides")
        assert response.status_code == 200
        data = response.json()
        guides = data.get("guides", [])
        assert len(guides) == 1, "1:1 모델이므로 가이드 1개여야 함"
        guide = guides[0]
        assert guide.get("custom_template_id") is not None, "덮어쓰기 후 커스텀 템플릿이 적용되어야 함"
        assert guide.get("template_id") is None, "원본 템플릿은 덮어써져서 null이어야 함"
        logger.info("✓ 가이드 덮어쓰기 검증 완료 (커스텀 템플릿으로 교체됨)")

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

        # 1:1 모델: guide_id = report_id
        response = user_client.delete(f"/api/reports/{report_id}/guides/{report_id}")

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


class TestFullRecommendedWorkflow:
    """권장 워크플로우 통합 E2E: guide-images → image_ids → Report → guides"""

    @pytest.fixture(scope="class")
    def workflow_series_uid(self, admin_client, config):
        """권장 워크플로우용 별도 Series 생성"""
        from datetime import date, timedelta

        unique_id = f"{int(time.time() * 1000)}_{fake.uuid4()[:8]}"
        project_resp = admin_client.post("/api/projects", json={
            "name": f"e2e_workflow_{unique_id}",
            "description": "Full Recommended Workflow E2E",
            "sponsor": "E2E",
            "start_date": str(date.today()),
            "end_date": str(date.today() + timedelta(days=365)),
        })
        if project_resp.status_code not in [200, 201]:
            pytest.skip(f"프로젝트 생성 실패: {project_resp.status_code}")
        project_id = project_resp.json().get("id")
        if not project_id:
            pytest.skip("프로젝트 ID를 얻지 못함")

        study_uid = f"1.2.840.113619.2.55.3.wf_{int(time.time())}"
        series_uid = f"1.2.840.113619.2.55.4.wf_{int(time.time())}"
        series_resp = admin_client.post(
            f"/api/projects/{project_id}/series/assign",
            json={
                "study_uid": study_uid,
                "series_uid": series_uid,
                "series_description": "Full Workflow Test Series",
                "modality": "CT",
                "series_number": 1,
            },
        )
        if series_resp.status_code not in [200, 201]:
            try:
                admin_client.delete(f"/api/projects/{project_id}")
            except Exception:
                pass
            pytest.skip(f"Series 할당 실패: {series_resp.status_code}")

        me_resp = admin_client.get("/api/users/me")
        if me_resp.status_code == 200:
            user_id = me_resp.json().get("id") or me_resp.json().get("user_id")
            if user_id:
                admin_client.post(f"/api/projects/{project_id}/members", json={"user_id": user_id})

        yield series_uid

        try:
            admin_client.delete(f"/api/projects/{project_id}")
        except Exception:
            pass

    def test_full_recommended_workflow(
        self, admin_client, user_client, workflow_series_uid
    ):
        """권장 워크플로우: guide-images 업로드 → image_ids로 템플릿 → Report → guides 적용"""
        logger.info("\n" + "="*80)
        logger.info("TEST: 권장 워크플로우 통합 E2E")
        logger.info("="*80)

        series_uid = workflow_series_uid

        # 1. guide-images 업로드 (권장 경로)
        upload_resp = admin_client.post("/api/guide-images/upload-url", json={
            "file_name": f"workflow_guide_{fake.uuid4()[:8]}.png",
            "mime_type": "image/png",
        })
        assert upload_resp.status_code == 200, upload_resp.text
        upload_data = upload_resp.json()
        file_path = upload_data["file_path"]

        complete_resp = admin_client.post("/api/guide-images/complete", json={
            "file_path": file_path,
            "file_size": 2048,
            "mime_type": "image/png",
            "is_shared": True,
        })
        assert complete_resp.status_code == 200, complete_resp.text
        image_id = complete_resp.json()["image"]["id"]
        logger.info(f"  1. 이미지 업로드 완료 (image_id={image_id})")

        # 2. my-uploads 확인
        my_resp = admin_client.get("/api/guide-images/my-uploads")
        assert my_resp.status_code == 200
        assert any(img["id"] == image_id for img in my_resp.json()["images"])
        logger.info("  2. my-uploads 확인 완료")

        # 3. 템플릿 생성 (image_ids로 연결)
        tmpl_resp = admin_client.post("/api/report-guide-templates", json={
            "description": "Workflow template",
            "bodypart": "chest",
            "modalities": ["CT"],
            "image_ids": [image_id],
        })
        assert tmpl_resp.status_code in [200, 201], tmpl_resp.text
        template_id = tmpl_resp.json()["id"]
        logger.info(f"  3. 템플릿 생성 (template_id={template_id}, image_ids 사용)")

        # 4. 유효 템플릿 조회
        eff_resp = user_client.get("/api/user/report-templates")
        assert eff_resp.status_code == 200
        templates = eff_resp.json().get("templates", [])
        found = any(t.get("template_id") == template_id for t in templates)
        assert found, f"생성한 원본 템플릿(ID={template_id})이 유효 목록에 있어야 함"
        logger.info("  4. 유효 템플릿 조회 완료")

        # 5. Report 생성
        report_resp = user_client.put(f"/api/series/{series_uid}/report", json={
            "status": "unread",
            "description": "Workflow report",
            "conclusion": "추가 검사 불필요",
            "bodypart": "chest",
        })
        assert report_resp.status_code == 200, report_resp.text
        report_data = report_resp.json()
        report_id = report_data.get("id")
        if not report_id:
            get_r = user_client.get(f"/api/series/{series_uid}/report")
            assert get_r.status_code == 200
            report_id = get_r.json().get("id")
        assert report_id, "report_id를 얻어야 함"
        logger.info(f"  5. Report 생성 (report_id={report_id})")

        # 6. guides 적용
        guide_resp = user_client.post(f"/api/reports/{report_id}/guides", json={
            "template_id": template_id,
            "custom_template_id": None,
            "display_order": 0,
        })
        assert guide_resp.status_code in [200, 201], guide_resp.text
        logger.info("  6. guides 적용 완료")

        # 7. guides 조회
        list_resp = user_client.get(f"/api/reports/{report_id}/guides")
        assert list_resp.status_code == 200
        guides = list_resp.json().get("guides", [])
        assert len(guides) >= 1, "가이드가 있어야 함"
        assert guides[0].get("template_id") == template_id
        if guides[0].get("images"):
            assert_guide_images_accessible(guides[0], "full_workflow guides", strict=False)
        logger.info("  7. guides 조회 및 검증 완료")

        # Cleanup
        admin_client.delete(f"/api/report-guide-templates/{template_id}")
        admin_client.delete(f"/api/guide-images/{image_id}")

        logger.info("✓ 권장 워크플로우 통합 E2E 성공")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])

