#!/usr/bin/env python3
"""
Annotation Snapshot Upload E2E Test (리팩토링 버전)
어노테이션 스냅샷 이미지 업로드 기능 전체 워크플로우 테스트
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import ImageFixtures


class AnnotationSnapshotE2ETest(BaseE2ETest):
    """어노테이션 스냅샷 업로드 테스트"""
    
    def __init__(self):
        super().__init__()
        self.annotation_id = None
    
    def get_test_name(self) -> str:
        return "Annotation Snapshot E2E Test"
    
    def run_tests(self):
        """테스트 실행"""
        self.annotation_id = self.test_annotation_snapshot_workflow()
        self.created_annotation_ids.append(self.annotation_id)
    
    def test_annotation_snapshot_workflow(self) -> int:
        """스냅샷 업로드 전체 워크플로우 테스트"""
        TestPrinter.print_header("📋 Annotation Snapshot Upload E2E Test")
        
        # 개발 모드 헤더 (user_id 포함)
        dev_headers = {
            "Authorization": f"Bearer {self.token}",
            "X-User-ID": "1"  # iaid-pacs-admin의 user_id
        }
        
        # 1. 어노테이션 생성
        annotation_id = self._create_annotation(dev_headers)
        
        # 2. 스냅샷 업로드 URL 요청
        upload_data = self._request_upload_url(annotation_id, dev_headers)
        
        # 3. 테스트 이미지 생성
        image_data = self._create_test_image()
        
        # 4. S3에 이미지 업로드
        self._upload_to_s3(upload_data, image_data)
        
        # 5. 업로드 완료 알림
        self._complete_upload(annotation_id, upload_data, dev_headers)
        
        # 6. 스냅샷 상태 조회
        self._check_snapshot_status(annotation_id)
        
        # 7. 어노테이션 조회하여 스냅샷 정보 확인
        self._verify_snapshot_info(annotation_id, upload_data)
        
        TestPrinter.print_header("🎉 모든 테스트 통과!")
        self._print_summary(annotation_id, upload_data, len(image_data))
        
        return annotation_id
    
    def _create_annotation(self, headers: dict) -> int:
        """1. 어노테이션 생성"""
        print("\n1️⃣  어노테이션 생성 중...")
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": TestConfig.SNAPSHOT_STUDY_UID,
            "series_instance_uid": TestConfig.SNAPSHOT_SERIES_UID,
            "sop_instance_uid": TestConfig.SNAPSHOT_INSTANCE_UID,
            "annotation_data": {
                "type": "circle",
                "x": 300,
                "y": 250,
                "radius": 100,
                "color": "#FF0000",
                "label": "Lesion Area"
            },
            "tool_name": "Circle Tool",
            "tool_version": "2.1.0",
            "viewer_software": "OHIF Viewer",
            "description": "테스트용 병변 영역 어노테이션",
            "label": "Tumor",
            "measurement_values": [
                {"id": "m1", "type": "diameter", "values": [42.3], "unit": "mm"}
            ]
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=30
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code != 201:
            TestPrinter.print_error("어노테이션 생성 실패")
            TestPrinter.print_info(f"Response: {response.text}", indent=1)
            exit(1)
        
        annotation = response.json()
        annotation_id = annotation["id"]
        
        TestPrinter.print_success("어노테이션 생성 성공!", indent=1)
        TestPrinter.print_info(f"Annotation ID: {annotation_id}", indent=1)
        TestPrinter.print_info(f"Label: {annotation.get('label', 'N/A')}", indent=1)
        TestPrinter.print_info(f"Tool: {annotation.get('tool_name', 'N/A')}", indent=1)
        
        return annotation_id
    
    def _request_upload_url(self, annotation_id: int, headers: dict) -> dict:
        """2. 스냅샷 업로드 URL 요청"""
        print("\n2️⃣  스냅샷 업로드 URL 요청 중...")
        
        upload_request = {
            "filename": "snapshot_test_annotation.png",
            "mime_type": "image/png",
            "ttl_seconds": 600
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}/snapshot/upload-url",
            json=upload_request,
            headers=headers,
            timeout=30
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code != 200:
            TestPrinter.print_error("업로드 URL 생성 실패")
            TestPrinter.print_info(f"Response: {response.text}", indent=1)
            exit(1)
        
        upload_data = response.json()
        
        TestPrinter.print_success("업로드 URL 생성 성공!", indent=1)
        TestPrinter.print_info(f"Upload URL: {upload_data['upload_url'][:80]}...", indent=1)
        TestPrinter.print_info(f"Download URL: {upload_data['download_url'][:80]}...", indent=1)
        TestPrinter.print_info(f"Image Key: {upload_data['image_key']}", indent=1)
        TestPrinter.print_info(f"Expires in: {upload_data['expires_in']}s", indent=1)
        
        return upload_data
    
    def _create_test_image(self) -> bytes:
        """3. 테스트 이미지 생성"""
        print("\n3️⃣  테스트 이미지 생성 중...")
        
        image_data = ImageFixtures.create_test_snapshot_image()
        image_size = len(image_data)
        
        TestPrinter.print_success("이미지 생성 완료!", indent=1)
        TestPrinter.print_info(f"Size: {image_size:,} bytes ({image_size / 1024:.2f} KB)", indent=1)
        TestPrinter.print_info("Format: PNG", indent=1)
        TestPrinter.print_info("Dimensions: 800x600", indent=1)
        
        return image_data
    
    def _upload_to_s3(self, upload_data: dict, image_data: bytes):
        """4. S3에 이미지 업로드"""
        print("\n4️⃣  S3에 이미지 업로드 중...")
        
        upload_response = requests.put(
            upload_data["upload_url"],
            data=image_data,
            headers={"Content-Type": "image/png"},
            timeout=30
        )
        
        TestPrinter.print_info(f"Status: {upload_response.status_code}", indent=1)
        
        if upload_response.status_code not in [200, 204]:
            TestPrinter.print_error("S3 업로드 실패")
            TestPrinter.print_info(f"Response: {upload_response.text}", indent=1)
            exit(1)
        
        TestPrinter.print_success("S3 업로드 성공!", indent=1)

    def _complete_upload(self, annotation_id: int, upload_data: dict, headers: dict):
        """5. 업로드 완료 알림"""
        print("\n5️⃣  업로드 완료 알림 중...")

        complete_request = {
            "image_key": upload_data["image_key"],
            "success": True
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}/snapshot/complete-upload",
            json=complete_request,
            headers=headers,
            timeout=60  # S3 다운로드 시간을 고려하여 60초로 증가
        )

        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)

        if response.status_code != 200:
            TestPrinter.print_error("업로드 완료 처리 실패")
            TestPrinter.print_info(f"Response: {response.text}", indent=1)
            exit(1)

        updated_annotation = response.json()

        TestPrinter.print_success("업로드 완료 처리 성공!", indent=1)
        TestPrinter.print_info(f"Snapshot Image Key: {updated_annotation.get('snapshot_image_key', 'N/A')}", indent=1)
        TestPrinter.print_info(f"Snapshot Status: {updated_annotation.get('snapshot_status', 'N/A')}", indent=1)
        TestPrinter.print_info(f"Snapshot Uploaded At: {updated_annotation.get('snapshot_uploaded_at', 'N/A')}", indent=1)

    def _check_snapshot_status(self, annotation_id: int):
        """6. 스냅샷 상태 조회"""
        print("\n6️⃣  스냅샷 상태 조회 중...")

        headers = {"Authorization": f"Bearer {self.token}"}

        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}/snapshot/status",
            headers=headers,
            timeout=30
        )

        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)

        if response.status_code != 200:
            TestPrinter.print_warning("상태 조회 실패 (optional)", indent=1)
            TestPrinter.print_info(f"Response: {response.text}", indent=1)
        else:
            status_data = response.json()
            TestPrinter.print_success("상태 조회 성공!", indent=1)
            TestPrinter.print_info(f"Annotation ID: {status_data.get('annotation_id', 'N/A')}", indent=1)
            TestPrinter.print_info(f"Image Key: {status_data.get('image_key', 'N/A')}", indent=1)
            TestPrinter.print_info(f"Status: {status_data.get('status', 'N/A')}", indent=1)
            TestPrinter.print_info(f"Uploaded At: {status_data.get('uploaded_at', 'N/A')}", indent=1)

    def _verify_snapshot_info(self, annotation_id: int, upload_data: dict):
        """7. 어노테이션 조회하여 스냅샷 정보 확인"""
        print("\n7️⃣  어노테이션 조회하여 스냅샷 정보 확인 중...")

        headers = {"Authorization": f"Bearer {self.token}"}

        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            headers=headers,
            timeout=30
        )

        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)

        if response.status_code != 200:
            TestPrinter.print_error("어노테이션 조회 실패")
            TestPrinter.print_info(f"Response: {response.text}", indent=1)
            exit(1)

        final_annotation = response.json()

        TestPrinter.print_success("어노테이션 조회 성공!", indent=1)
        TestPrinter.print_info(f"ID: {final_annotation.get('id', 'N/A')}", indent=1)
        TestPrinter.print_info(f"Label: {final_annotation.get('label', 'N/A')}", indent=1)
        TestPrinter.print_info(f"Snapshot Image Key: {final_annotation.get('snapshot_image_key', 'N/A')}", indent=1)
        TestPrinter.print_info(f"Snapshot Status: {final_annotation.get('snapshot_status', 'N/A')}", indent=1)

        # 검증
        assert final_annotation.get('snapshot_image_key') == upload_data['image_key'], \
            "스냅샷 이미지 키가 일치하지 않습니다"
        assert final_annotation.get('snapshot_status') == 'completed', \
            f"스냅샷 상태가 'completed'가 아닙니다: {final_annotation.get('snapshot_status')}"

    def _print_summary(self, annotation_id: int, upload_data: dict, image_size: int):
        """테스트 요약 출력"""
        print(f"\n📊 테스트 요약:")
        TestPrinter.print_info(f"Annotation ID: {annotation_id}", indent=1)
        TestPrinter.print_info(f"Image Key: {upload_data['image_key']}", indent=1)
        TestPrinter.print_info(f"Image Size: {image_size:,} bytes", indent=1)
        print()


if __name__ == '__main__':
    test = AnnotationSnapshotE2ETest()
    test.run()

