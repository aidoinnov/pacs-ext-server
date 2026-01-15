#!/usr/bin/env python3
"""
/me/dicom 엔드포인트 V2 배치 쿼리 최적화 테스트

V1 vs V2 성능 비교:
- V1: Study마다 2번의 DB 쿼리 (evaluate_study_uid + can_access_study)
- V2: 모든 Study를 1번의 배치 쿼리로 확인

예상 성능 개선:
- Study 100개: 200 쿼리 → 1 쿼리 (99.5% 개선)
"""

import requests
import time
import sys
from pathlib import Path
from typing import Dict, Any

# 프로젝트 루트를 Python 경로에 추가
sys.path.insert(0, str(Path(__file__).parent))

from utils.api_client import APIClient as BaseAPIClient
from config import TestConfig

BASE_URL = "http://localhost:8080"

class APIClient(BaseAPIClient):
    """확장된 API 클라이언트"""

    def get_my_studies(self, **params) -> Dict[str, Any]:
        """GET /api/me/dicom/studies"""
        start = time.time()
        response = self.get("/api/me/dicom/studies", params=params)
        elapsed = time.time() - start

        return {
            "status_code": response.status_code,
            "data": response.json() if response.status_code == 200 else None,
            "elapsed": elapsed
        }

    def get_my_series(self, study_uid: str, **params) -> Dict[str, Any]:
        """GET /api/me/dicom/studies/{study_uid}/series"""
        start = time.time()
        response = self.get(f"/api/me/dicom/studies/{study_uid}/series", params=params)
        elapsed = time.time() - start

        return {
            "status_code": response.status_code,
            "data": response.json() if response.status_code == 200 else None,
            "elapsed": elapsed
        }

    def get_my_instances(self, study_uid: str, series_uid: str, **params) -> Dict[str, Any]:
        """GET /api/me/dicom/studies/{study_uid}/series/{series_uid}/instances"""
        start = time.time()
        response = self.get(
            f"/api/me/dicom/studies/{study_uid}/series/{series_uid}/instances",
            params=params
        )
        elapsed = time.time() - start

        return {
            "status_code": response.status_code,
            "data": response.json() if response.status_code == 200 else None,
            "elapsed": elapsed
        }


def main():
    print("=" * 80)
    print("🧪 /me/dicom V2 Batch Query Optimization Test")
    print("=" * 80)

    # 설정 로드
    config = TestConfig.from_env()
    client = APIClient(config.base_url, timeout=60)

    # 로그인
    print(f"\n🔐 Logging in as {config.admin_email}...")
    try:
        client.login(config.admin_email, config.admin_password)
        print("✅ Login successful")
    except Exception as e:
        print(f"❌ Login failed: {e}")
        return
    
    # 1. /me/studies 테스트
    print("\n" + "=" * 80)
    print("📋 Testing /me/studies endpoint (V2 Batch Query)")
    print("=" * 80)
    
    result = client.get_my_studies(page=1, page_size=50)

    if result["status_code"] == 200:
        data = result["data"]
        if isinstance(data, dict):
            studies = data.get("studies", [])
            total = data.get("total", 0)
        else:
            studies = data if isinstance(data, list) else []
            total = len(studies)

        print(f"✅ Success: {len(studies)} studies returned (total: {total})")
        print(f"⏱️  Response time: {result['elapsed']:.3f}s")
        print(f"🚀 Performance: {'Excellent' if result['elapsed'] < 1.0 else 'Good' if result['elapsed'] < 3.0 else 'Needs improvement'}")

        if studies:
            study = studies[0]
            study_uid = study.get("0020000D", {}).get("Value", [""])[0]
            print(f"\n📄 First study UID: {study_uid[:50]}...")
            
            # 2. /me/series 테스트
            print("\n" + "=" * 80)
            print(f"📋 Testing /me/series endpoint for study")
            print("=" * 80)
            
            series_result = client.get_my_series(study_uid)
            
            if series_result["status_code"] == 200:
                series_list = series_result["data"]
                print(f"✅ Success: {len(series_list)} series returned")
                print(f"⏱️  Response time: {series_result['elapsed']:.3f}s")
                
                if series_list:
                    series = series_list[0]
                    series_uid = series.get("0020000E", {}).get("Value", [""])[0]
                    print(f"\n📄 First series UID: {series_uid[:50]}...")
                    
                    # 3. /me/instances 테스트
                    print("\n" + "=" * 80)
                    print(f"📋 Testing /me/instances endpoint (V2 - Study level access only)")
                    print("=" * 80)
                    
                    instances_result = client.get_my_instances(study_uid, series_uid)
                    
                    if instances_result["status_code"] == 200:
                        instances = instances_result["data"]
                        print(f"✅ Success: {len(instances)} instances returned")
                        print(f"⏱️  Response time: {instances_result['elapsed']:.3f}s")
                        print(f"🚀 Performance: {'Excellent' if instances_result['elapsed'] < 1.0 else 'Good' if instances_result['elapsed'] < 3.0 else 'Needs improvement'}")
                    else:
                        print(f"❌ Failed: {instances_result['status_code']}")
            else:
                print(f"❌ Failed: {series_result['status_code']}")
    else:
        print(f"❌ Failed: {result['status_code']}")
    
    print("\n" + "=" * 80)
    print("✅ All tests completed!")
    print("=" * 80)


if __name__ == "__main__":
    main()

