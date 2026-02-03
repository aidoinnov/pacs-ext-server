#!/usr/bin/env python3
"""
SW Information API E2E 테스트

의료영상저장장치 소프트웨어 정보(SW Information) 조회 API 검증.
- GET /api/sw-information (목록)
- GET /api/sw-information/{id} (상세)
"""
import logging
import pytest
from config import TestConfig
from utils.api_client import APIClient

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


@pytest.fixture(scope="module")
def config():
    return TestConfig.from_env()


@pytest.fixture(scope="module")
def client(config):
    c = APIClient(config.base_url, config.timeout)
    c.login(config.admin_email, config.admin_password)
    logger.info("✓ Admin logged in for SW Information tests")
    yield c
    c.close()


class TestSwInformationAPI:
    """SW Information API E2E 테스트"""

    def test_01_list_sw_information(self, client):
        """GET /api/sw-information - 목록 조회"""
        logger.info("\n" + "=" * 80)
        logger.info("TEST: SW Information 목록 조회")
        logger.info("=" * 80)

        response = client.get("/api/sw-information")

        assert response.status_code == 200, f"목록 조회 실패: {response.text}"
        data = response.json()

        assert "success" in data
        assert data["success"] is True
        assert "items" in data
        assert "total_count" in data
        assert isinstance(data["items"], list)

        if data["items"]:
            item = data["items"][0]
            assert "id" in item
            assert "product_item" in item
            assert "model_name" in item
            assert "manufacturer" in item
            assert "address" in item
            assert "manufacturing_permit_number" in item

        logger.info(f"✓ SW Information 목록 조회 성공: {data['total_count']}건")

    def test_02_get_sw_information_by_id(self, client):
        """GET /api/sw-information/{id} - 상세 조회"""
        logger.info("\n" + "=" * 80)
        logger.info("TEST: SW Information 상세 조회")
        logger.info("=" * 80)

        # 먼저 목록에서 id 획득
        list_resp = client.get("/api/sw-information")
        assert list_resp.status_code == 200
        data = list_resp.json()
        items = data.get("items", [])

        if not items:
            pytest.skip("SW Information 데이터가 없어 상세 조회 스킵")

        sw_id = items[0]["id"]

        response = client.get(f"/api/sw-information/{sw_id}")

        assert response.status_code == 200, f"상세 조회 실패: {response.text}"
        item = response.json()

        assert "id" in item
        assert item["id"] == sw_id
        assert "product_item" in item
        assert "model_name" in item
        assert "manufacturer" in item
        assert "address" in item

        logger.info(f"✓ SW Information 상세 조회 성공: id={sw_id}")

    def test_03_get_sw_information_not_found(self, client):
        """GET /api/sw-information/99999 - 존재하지 않는 ID는 404"""
        response = client.get("/api/sw-information/99999")
        assert response.status_code == 404
