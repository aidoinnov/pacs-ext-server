"""
Guide Image Independent Management E2E Test
===========================================
새로운 독립적인 가이드 이미지 관리 워크플로우 테스트

워크플로우:
1. 이미지를 독립적으로 업로드 (템플릿 없이)
2. 업로드된 이미지 목록 조회
3. 템플릿 생성 시 이미지 ID로 연결
4. 이미지 재사용 (여러 템플릿에서 동일 이미지 사용)
5. 이미지 삭제
"""

import pytest
import requests
import json
import io
from typing import Dict, List
from config import TestConfig
from utils.signed_url_helpers import assert_image_has_signed_url, assert_images_have_signed_urls

# 테스트 설정
config = TestConfig.from_env()
BASE_URL = config.base_url
ADMIN_EMAIL = config.admin_email
ADMIN_PASSWORD = config.admin_password


class TestGuideImageWorkflow:
    """독립적인 가이드 이미지 관리 워크플로우 테스트"""

    @pytest.fixture(scope="class")
    def auth_token(self) -> str:
        """인증 토큰 획득"""
        response = requests.post(
            f"{BASE_URL}/api/auth/login",
            json={"username": ADMIN_EMAIL, "password": ADMIN_PASSWORD}
        )
        assert response.status_code == 200, f"Login failed: {response.text}"
        data = response.json()
        return data.get("access_token") or data.get("token")

    @pytest.fixture(scope="class")
    def headers(self, auth_token: str) -> Dict[str, str]:
        """인증 헤더"""
        return {"Authorization": f"Bearer {auth_token}"}

    @pytest.fixture(scope="class")
    def uploaded_images(self, headers: Dict[str, str]) -> List[int]:
        """테스트용 이미지 업로드 (3개)"""
        image_ids = []
        
        for i in range(3):
            # 1. 업로드 URL 생성
            upload_url_response = requests.post(
                f"{BASE_URL}/api/guide-images/upload-url",
                headers=headers,
                json={
                    "file_name": f"test_image_{i+1}.png",
                    "mime_type": "image/png"
                }
            )
            assert upload_url_response.status_code == 200, f"Upload URL generation failed: {upload_url_response.text}"
            upload_data = upload_url_response.json()
            
            # 2. S3 업로드 시뮬레이션 (실제로는 presigned URL로 업로드)
            # 여기서는 업로드 완료만 처리
            
            # 3. 업로드 완료 처리
            complete_response = requests.post(
                f"{BASE_URL}/api/guide-images/complete",
                headers=headers,
                json={
                    "file_path": upload_data["file_path"],
                    "file_size": 1024 * (i + 1),  # 1KB, 2KB, 3KB
                    "mime_type": "image/png",
                    "is_shared": True
                }
            )
            assert complete_response.status_code == 200, f"Upload complete failed: {complete_response.text}"
            complete_data = complete_response.json()
            
            assert complete_data["success"] is True
            assert "image" in complete_data
            image_ids.append(complete_data["image"]["id"])
        
        yield image_ids
        
        # Cleanup: 테스트 후 이미지 삭제
        for image_id in image_ids:
            requests.delete(
                f"{BASE_URL}/api/guide-images/{image_id}",
                headers=headers
            )

    def test_01_upload_guide_image(self, headers: Dict[str, str]):
        """테스트 1: 독립적인 가이드 이미지 업로드"""
        import uuid
        file_name = f"test_standalone_{uuid.uuid4().hex[:8]}.png"
        # 1. 업로드 URL 생성
        response = requests.post(
            f"{BASE_URL}/api/guide-images/upload-url",
            headers=headers,
            json={
                "file_name": file_name,
                "mime_type": "image/png"
            }
        )

        assert response.status_code == 200, f"upload-url 실패: {response.text}"
        data = response.json()
        assert data["success"] is True
        assert "upload_url" in data
        assert "file_path" in data
        assert "guide-images/user" in data["file_path"]  # 사용자별 경로

        # 2. 업로드 완료 처리
        complete_response = requests.post(
            f"{BASE_URL}/api/guide-images/complete",
            headers=headers,
            json={
                "file_path": data["file_path"],
                "file_size": 2048,
                "mime_type": "image/png",
                "is_shared": True
            }
        )

        assert complete_response.status_code == 200, (
            f"complete 실패 ({complete_response.status_code}): {complete_response.text[:500]}"
        )
        complete_data = complete_response.json()
        assert complete_data["success"] is True
        assert "image" in complete_data
        
        image = complete_data["image"]
        assert image["id"] > 0
        assert image["image_path"] == data["file_path"]
        assert image["file_size"] == 2048
        assert image["is_shared"] is True
        assert_image_has_signed_url(image, "upload complete 응답", strict=False)

        # Cleanup
        image_id = image["id"]
        delete_response = requests.delete(
            f"{BASE_URL}/api/guide-images/{image_id}",
            headers=headers
        )
        assert delete_response.status_code == 200

    def test_02_get_my_uploaded_images(self, headers: Dict[str, str], uploaded_images: List[int]):
        """테스트 2: 내가 업로드한 이미지 목록 조회"""
        response = requests.get(
            f"{BASE_URL}/api/guide-images/my-uploads",
            headers=headers
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True
        assert "images" in data
        assert "total_count" in data
        assert data["total_count"] >= 3  # 최소 3개 (fixture에서 업로드한 것)
        
        # 업로드한 이미지 ID가 목록에 있는지 확인
        image_ids_in_response = [img["id"] for img in data["images"]]
        for uploaded_id in uploaded_images:
            assert uploaded_id in image_ids_in_response

        assert_images_have_signed_urls(data["images"], "my-uploads", strict=False)

    def test_03_create_template_with_image_ids(self, headers: Dict[str, str], uploaded_images: List[int]):
        """테스트 3: 이미지 ID를 사용하여 템플릿 생성"""
        # 템플릿 생성 (이미지 ID 2개 사용)
        response = requests.post(
            f"{BASE_URL}/api/report-guide-templates",
            headers=headers,
            json={
                "description": "Template using pre-uploaded images",
                "modalities": ["CT"],
                "image_ids": uploaded_images[:2]  # 처음 2개 이미지 사용
            }
        )

        assert response.status_code == 200
        data = response.json()
        assert data["id"] > 0
        assert data["id"] is not None

        template_id = data["id"]

        # 템플릿 조회하여 이미지가 연결되었는지 확인
        get_response = requests.get(
            f"{BASE_URL}/api/report-guide-templates/{template_id}",
            headers=headers
        )
        assert get_response.status_code == 200
        template_data = get_response.json()

        # 이미지 목록 확인 및 signed URL 검증
        assert "images" in template_data, "템플릿 응답에 images 필드가 없습니다"
        assert len(template_data["images"]) >= 2, "최소 2개 이미지가 연결되어 있어야 함"
        assert_images_have_signed_urls(template_data["images"], "템플릿 이미지", strict=False)

        # Cleanup
        delete_response = requests.delete(
            f"{BASE_URL}/api/report-guide-templates/{template_id}",
            headers=headers
        )
        assert delete_response.status_code == 200

    def test_04_update_template_image_ids(self, headers: Dict[str, str], uploaded_images: List[int]):
        """테스트 4: 템플릿 이미지 ID 업데이트"""
        # 템플릿 생성 (이미지 1개)
        create_response = requests.post(
            f"{BASE_URL}/api/report-guide-templates",
            headers=headers,
            json={
                "modalities": ["MR"],
                "image_ids": [uploaded_images[0]]
            }
        )
        assert create_response.status_code == 200
        template_id = create_response.json()["id"]

        # 템플릿 업데이트 (다른 이미지로 변경)
        update_response = requests.put(
            f"{BASE_URL}/api/report-guide-templates/{template_id}",
            headers=headers,
            json={
                "image_ids": uploaded_images[1:3]  # 2번째, 3번째 이미지로 변경
            }
        )
        assert update_response.status_code == 200

        # Cleanup
        requests.delete(
            f"{BASE_URL}/api/report-guide-templates/{template_id}",
            headers=headers
        )

    def test_05_image_reusability(self, headers: Dict[str, str], uploaded_images: List[int]):
        """테스트 5: 이미지 재사용 (여러 템플릿에서 동일 이미지 사용)"""
        # 템플릿 1 생성 (이미지 1, 2 사용)
        template1_response = requests.post(
            f"{BASE_URL}/api/report-guide-templates",
            headers=headers,
            json={
                "modalities": ["CT"],
                "image_ids": uploaded_images[:2]
            }
        )
        assert template1_response.status_code == 200
        template1_id = template1_response.json()["id"]

        # 템플릿 2 생성 (이미지 2, 3 사용 - 이미지 2 재사용!)
        template2_response = requests.post(
            f"{BASE_URL}/api/report-guide-templates",
            headers=headers,
            json={
                "modalities": ["MR"],
                "image_ids": uploaded_images[1:3]
            }
        )
        assert template2_response.status_code == 200
        template2_id = template2_response.json()["id"]

        # 두 템플릿 모두 성공적으로 생성됨 (이미지 재사용 성공)

        # Cleanup
        requests.delete(f"{BASE_URL}/api/report-guide-templates/{template1_id}", headers=headers)
        requests.delete(f"{BASE_URL}/api/report-guide-templates/{template2_id}", headers=headers)

    def test_06_delete_guide_image(self, headers: Dict[str, str]):
        """테스트 6: 가이드 이미지 삭제"""
        # 이미지 업로드
        upload_url_response = requests.post(
            f"{BASE_URL}/api/guide-images/upload-url",
            headers=headers,
            json={"file_name": "to_be_deleted.png", "mime_type": "image/png"}
        )
        assert upload_url_response.status_code == 200

        complete_response = requests.post(
            f"{BASE_URL}/api/guide-images/complete",
            headers=headers,
            json={
                "file_path": upload_url_response.json()["file_path"],
                "file_size": 1024,
                "mime_type": "image/png"
            }
        )
        assert complete_response.status_code == 200
        image_id = complete_response.json()["image"]["id"]

        # 이미지 삭제
        delete_response = requests.delete(
            f"{BASE_URL}/api/guide-images/{image_id}",
            headers=headers
        )
        assert delete_response.status_code == 200
        data = delete_response.json()
        assert data["success"] is True

        # 삭제 확인 (목록에서 사라졌는지)
        list_response = requests.get(
            f"{BASE_URL}/api/guide-images/my-uploads",
            headers=headers
        )
        assert list_response.status_code == 200
        image_ids = [img["id"] for img in list_response.json()["images"]]
        assert image_id not in image_ids

    def test_07_filter_by_is_shared(self, headers: Dict[str, str]):
        """테스트 7: is_shared 필터링"""
        # 공유 이미지 업로드
        shared_upload = requests.post(
            f"{BASE_URL}/api/guide-images/upload-url",
            headers=headers,
            json={"file_name": "shared.png", "mime_type": "image/png"}
        )
        shared_complete = requests.post(
            f"{BASE_URL}/api/guide-images/complete",
            headers=headers,
            json={
                "file_path": shared_upload.json()["file_path"],
                "file_size": 1024,
                "is_shared": True
            }
        )
        shared_id = shared_complete.json()["image"]["id"]

        # 비공유 이미지 업로드
        private_upload = requests.post(
            f"{BASE_URL}/api/guide-images/upload-url",
            headers=headers,
            json={"file_name": "private.png", "mime_type": "image/png"}
        )
        private_complete = requests.post(
            f"{BASE_URL}/api/guide-images/complete",
            headers=headers,
            json={
                "file_path": private_upload.json()["file_path"],
                "file_size": 1024,
                "is_shared": False
            }
        )
        private_id = private_complete.json()["image"]["id"]

        # 공유 이미지만 조회
        shared_list = requests.get(
            f"{BASE_URL}/api/guide-images/my-uploads?is_shared=true",
            headers=headers
        )
        assert shared_list.status_code == 200
        shared_ids = [img["id"] for img in shared_list.json()["images"]]
        assert shared_id in shared_ids

        # Cleanup
        requests.delete(f"{BASE_URL}/api/guide-images/{shared_id}", headers=headers)
        requests.delete(f"{BASE_URL}/api/guide-images/{private_id}", headers=headers)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])


