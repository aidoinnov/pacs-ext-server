#!/usr/bin/env python3
"""
Mask & Mask Group API E2E 테스트

이 테스트는 Mask와 Mask Group의 전체 라이프사이클을 검증합니다:
- Mask Group CRUD
- Mask CRUD
- Signed URL 생성 (업로드/다운로드)
- 권한 체크
- 통계 조회
"""

import requests
import time
from test_base import BaseE2ETest, TestConfig, TestPrinter


class MaskE2ETest(BaseE2ETest):
    """Mask & Mask Group E2E 테스트"""
    
    def __init__(self):
        super().__init__()
        self.created_mask_group_ids = []
        self.created_mask_ids = []
        self.test_annotation_id = None
    
    def get_test_name(self) -> str:
        return "Mask & Mask Group E2E 테스트"
    
    def setup(self):
        """테스트 설정"""
        super().setup()
        
        # 테스트용 Annotation 생성
        TestPrinter.print_header("테스트용 Annotation 생성")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        annotation_data = {
            "user_id": 1,  # admin user
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": TestConfig.STUDY_UID,
            "series_instance_uid": TestConfig.SERIES_UID,
            "sop_instance_uid": TestConfig.INSTANCE_UID,
            "annotation_data": {"test": "mask_e2e_test"},
            "lesion_number": 1,
            "lesion_type": "TARGET"
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            headers=headers,
            json=annotation_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code in [200, 201]:
            self.test_annotation_id = response.json()["id"]
            TestPrinter.print_success(f"Annotation 생성 성공: ID={self.test_annotation_id}")
        else:
            TestPrinter.print_error(f"Annotation 생성 실패: {response.text}")
            raise Exception("Failed to create test annotation")
    
    def cleanup(self):
        """테스트 정리"""
        TestPrinter.print_header("테스트 정리")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # Mask 삭제
        for mask_id in self.created_mask_ids:
            try:
                # Mask는 Mask Group 삭제 시 자동으로 삭제되므로 별도 삭제 불필요
                pass
            except Exception as e:
                TestPrinter.print_warning(f"Mask {mask_id} 삭제 실패: {e}")
        
        # Mask Group 삭제
        for group_id in self.created_mask_group_ids:
            try:
                response = requests.delete(
                    f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}",
                    headers=headers,
                    timeout=TestConfig.DEFAULT_TIMEOUT
                )
                if response.status_code == 204:
                    TestPrinter.print_success(f"Mask Group {group_id} 삭제 성공")
            except Exception as e:
                TestPrinter.print_warning(f"Mask Group {group_id} 삭제 실패: {e}")
        
        # Annotation 삭제
        if self.test_annotation_id:
            try:
                response = requests.delete(
                    f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}",
                    headers=headers,
                    timeout=TestConfig.DEFAULT_TIMEOUT
                )
                if response.status_code == 204:
                    TestPrinter.print_success(f"Annotation {self.test_annotation_id} 삭제 성공")
            except Exception as e:
                TestPrinter.print_warning(f"Annotation {self.test_annotation_id} 삭제 실패: {e}")
        
        super().cleanup()
    
    def run_tests(self):
        """테스트 실행"""
        self.test_mask_group_crud()
        self.test_mask_crud()
        self.test_signed_url_generation()
        self.test_mask_statistics()
        self.test_mask_list_pagination()
    
    def test_mask_group_crud(self):
        """테스트 1: Mask Group CRUD"""
        TestPrinter.print_header("테스트 1: Mask Group CRUD")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 1. Mask Group 생성
        print("1️⃣  Mask Group 생성...")
        create_data = {
            "group_name": "Test Liver Segmentation",
            "description": "E2E test mask group",
            "modality": "CT",
            "slice_count": 100,
            "mask_type": "segmentation"
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups",
            headers=headers,
            json=create_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 201, f"Mask Group 생성 실패: {response.text}"

        group_data = response.json()
        group_id = group_data["id"]
        self.created_mask_group_ids.append(group_id)
        TestPrinter.print_success(f"Mask Group 생성 성공: ID={group_id}")

        # 2. Mask Group 조회
        print("\n2️⃣  Mask Group 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"Mask Group 조회 실패: {response.text}"
        retrieved_data = response.json()
        assert retrieved_data["id"] == group_id
        assert retrieved_data["group_name"] == "Test Liver Segmentation"
        TestPrinter.print_success("Mask Group 조회 성공")

        # 3. Mask Group 수정
        print("\n3️⃣  Mask Group 수정...")
        update_data = {
            "group_name": "Updated Liver Segmentation",
            "description": "Updated description"
        }

        response = requests.put(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}",
            headers=headers,
            json=update_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"Mask Group 수정 실패: {response.text}"
        updated_data = response.json()
        assert updated_data["group_name"] == "Updated Liver Segmentation"
        TestPrinter.print_success("Mask Group 수정 성공")

        # 4. Mask Group 목록 조회
        print("\n4️⃣  Mask Group 목록 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"Mask Group 목록 조회 실패: {response.text}"
        list_data = response.json()
        assert "mask_groups" in list_data
        assert len(list_data["mask_groups"]) > 0
        TestPrinter.print_success(f"Mask Group 목록 조회 성공: {len(list_data['mask_groups'])}개")

    def test_mask_crud(self):
        """테스트 2: Mask CRUD"""
        TestPrinter.print_header("테스트 2: Mask CRUD")
        headers = {"Authorization": f"Bearer {self.token}"}

        # Mask Group이 없으면 생성
        if not self.created_mask_group_ids:
            create_data = {
                "annotation_id": self.test_annotation_id,
                "group_name": "Test Mask Group for Masks",
                "modality": "CT",
                "creation_method": "MANUAL"
            }
            response = requests.post(
                f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups",
                headers=headers,
                json=create_data,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )
            group_id = response.json()["id"]
            self.created_mask_group_ids.append(group_id)
        else:
            group_id = self.created_mask_group_ids[0]

        # 1. Mask 생성
        print("1️⃣  Mask 생성...")
        mask_data = {
            "mask_group_id": group_id,
            "file_path": f"masks/annotation_{self.test_annotation_id}/group_{group_id}/slice_001_liver.png",
            "mime_type": "image/png",
            "slice_index": 1,
            "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
            "label_name": "liver",
            "file_size": 102400,
            "checksum": "sha256:abc123def456",
            "width": 512,
            "height": 512
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks",
            headers=headers,
            json=mask_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 201, f"Mask 생성 실패: {response.text}"

        created_mask = response.json()
        mask_id = created_mask["id"]
        self.created_mask_ids.append(mask_id)
        TestPrinter.print_success(f"Mask 생성 성공: ID={mask_id}")

        # 2. Mask 조회
        print("\n2️⃣  Mask 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
            headers={"Authorization": f"Bearer {self.token}", "X-User-ID": "1"},
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"Mask 조회 실패: {response.text}"
        retrieved_mask = response.json()
        assert retrieved_mask["id"] == mask_id
        assert retrieved_mask["label_name"] == "liver"
        TestPrinter.print_success("Mask 조회 성공")

        # 3. Mask 수정
        print("\n3️⃣  Mask 수정...")
        update_data = {
            "label_name": "liver_updated",
            "file_size": 204800
        }

        response = requests.put(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
            headers={"Authorization": f"Bearer {self.token}", "X-User-ID": "1"},
            json=update_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"Mask 수정 실패: {response.text}"
        updated_mask = response.json()
        assert updated_mask["label_name"] == "liver_updated"
        TestPrinter.print_success("Mask 수정 성공")

        # 4. Mask 목록 조회
        print("\n4️⃣  Mask 목록 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks",
            headers={"Authorization": f"Bearer {self.token}", "X-User-ID": "1"},
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"Mask 목록 조회 실패: {response.text}"
        list_data = response.json()
        assert "masks" in list_data
        assert len(list_data["masks"]) > 0
        TestPrinter.print_success(f"Mask 목록 조회 성공: {len(list_data['masks'])}개")

        # 5. Mask 삭제
        print("\n5️⃣  Mask 삭제...")
        response = requests.delete(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
            headers={"Authorization": f"Bearer {self.token}", "X-User-ID": "1"},
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 204, f"Mask 삭제 실패: {response.text}"
        TestPrinter.print_success("Mask 삭제 성공")
        self.created_mask_ids.remove(mask_id)

    def test_signed_url_generation(self):
        """테스트 3: Signed URL 생성 (업로드/다운로드)"""
        TestPrinter.print_header("테스트 3: Signed URL 생성")
        headers = {"Authorization": f"Bearer {self.token}"}

        # Mask Group이 없으면 생성
        if not self.created_mask_group_ids:
            create_data = {
                "annotation_id": self.test_annotation_id,
                "group_name": "Test Mask Group for URLs",
                "modality": "CT",
                "creation_method": "MANUAL"
            }
            response = requests.post(
                f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups",
                headers=headers,
                json=create_data,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )
            group_id = response.json()["id"]
            self.created_mask_group_ids.append(group_id)
        else:
            group_id = self.created_mask_group_ids[0]

        # 1. 업로드용 Signed URL 생성
        print("1️⃣  업로드용 Signed URL 생성...")
        upload_request = {
            "mask_group_id": group_id,
            "filename": "test_liver_slice_001.png",
            "mime_type": "image/png",
            "ttl_seconds": 600
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/upload-url",
            headers=headers,
            json=upload_request,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"업로드 URL 생성 실패: {response.text}"

        upload_data = response.json()
        assert "upload_url" in upload_data
        assert "file_path" in upload_data
        TestPrinter.print_success("업로드 URL 생성 성공")
        TestPrinter.print_info(f"File path: {upload_data['file_path']}", indent=1)

        # 2. Mask 생성 (다운로드 URL 테스트용)
        print("\n2️⃣  다운로드 테스트용 Mask 생성...")
        mask_data = {
            "mask_group_id": group_id,
            "file_path": upload_data["file_path"],
            "mime_type": "image/png",
            "slice_index": 1,
            "label_name": "liver",
            "file_size": 102400,
            "checksum": "sha256:test123",
            "width": 512,
            "height": 512
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks",
            headers=headers,
            json=mask_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        mask_id = response.json()["id"]
        self.created_mask_ids.append(mask_id)

        # 3. 다운로드용 Signed URL 생성
        print("\n3️⃣  다운로드용 Signed URL 생성...")
        download_request = {
            "mask_id": mask_id,
            "file_path": upload_data["file_path"],
            "expires_in": 3600
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url",
            headers={"Authorization": f"Bearer {self.token}", "X-User-ID": "1"},
            json=download_request,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"다운로드 URL 생성 실패: {response.text}"

        download_data = response.json()
        assert "download_url" in download_data
        assert "expires_at" in download_data
        TestPrinter.print_success("다운로드 URL 생성 성공")
        TestPrinter.print_info(f"Expires at: {download_data['expires_at']}", indent=1)

    def test_mask_statistics(self):
        """테스트 4: Mask 통계 조회"""
        TestPrinter.print_header("테스트 4: Mask 통계 조회")
        headers = {"Authorization": f"Bearer {self.token}", "X-User-ID": "1"}

        # Mask Group이 없으면 생성
        if not self.created_mask_group_ids:
            create_data = {
                "annotation_id": self.test_annotation_id,
                "group_name": "Test Mask Group for Stats",
                "modality": "CT",
                "creation_method": "MANUAL"
            }
            response = requests.post(
                f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups",
                headers={"Authorization": f"Bearer {self.token}"},
                json=create_data,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )
            group_id = response.json()["id"]
            self.created_mask_group_ids.append(group_id)
        else:
            group_id = self.created_mask_group_ids[0]

        # 여러 개의 Mask 생성
        print("1️⃣  테스트용 Mask 생성...")
        labels = ["liver", "spleen", "kidney"]
        for i, label in enumerate(labels):
            mask_data = {
                "mask_group_id": group_id,
                "file_path": f"masks/annotation_{self.test_annotation_id}/group_{group_id}/slice_{i:03d}_{label}.png",
                "mime_type": "image/png",
                "slice_index": i,
                "label_name": label,
                "file_size": 102400 * (i + 1),
                "checksum": f"sha256:test{i}",
                "width": 512,
                "height": 512
            }

            response = requests.post(
                f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks",
                headers={"Authorization": f"Bearer {self.token}"},
                json=mask_data,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )

            if response.status_code == 201:
                self.created_mask_ids.append(response.json()["id"])

        TestPrinter.print_success(f"{len(labels)}개 Mask 생성 완료")

        # 통계 조회
        print("\n2️⃣  Mask 통계 조회...")
        stats_headers = {**headers, "X-User-ID": "1"}
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks/stats",
            headers=stats_headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"통계 조회 실패: {response.text}"

        stats = response.json()
        assert "total_masks" in stats
        assert "total_size_bytes" in stats
        assert "masks_by_label" in stats
        assert "average_file_size" in stats
        assert "mime_type_distribution" in stats

        TestPrinter.print_success("통계 조회 성공")
        TestPrinter.print_info(f"Total masks: {stats['total_masks']}", indent=1)
        TestPrinter.print_info(f"Total size: {stats['total_size_bytes']} bytes", indent=1)
        TestPrinter.print_info(f"Average size: {stats['average_file_size']} bytes", indent=1)
        TestPrinter.print_info(f"Masks by label: {stats['masks_by_label']}", indent=1)
        TestPrinter.print_info(f"MIME types: {stats['mime_type_distribution']}", indent=1)

    def test_mask_list_pagination(self):
        """테스트 5: Mask 목록 페이지네이션"""
        TestPrinter.print_header("테스트 5: Mask 목록 페이지네이션")
        headers = {"Authorization": f"Bearer {self.token}", "X-User-ID": "1"}

        if not self.created_mask_group_ids:
            TestPrinter.print_warning("Mask Group이 없어 테스트 스킵")
            return

        group_id = self.created_mask_group_ids[0]

        # 페이지네이션 테스트
        print("1️⃣  첫 번째 페이지 조회 (page_size=2)...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}/mask-groups/{group_id}/masks?page=1&page_size=2",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 200, f"페이지네이션 실패: {response.text}"

        page1_data = response.json()
        assert "masks" in page1_data
        assert "total_count" in page1_data
        assert "current_page" in page1_data
        assert page1_data["current_page"] == 1

        TestPrinter.print_success(f"첫 번째 페이지 조회 성공: {len(page1_data['masks'])}개")
        TestPrinter.print_info(f"Total count: {page1_data['total_count']}", indent=1)
        TestPrinter.print_info(f"Total pages: {page1_data.get('total_pages', 'N/A')}", indent=1)


if __name__ == "__main__":
    test = MaskE2ETest()
    test.run()


