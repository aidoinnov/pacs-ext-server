#!/usr/bin/env python3
"""
마스크 업로드 테스트 스크립트

이 스크립트는 PACS Extension Server에 마스크 이미지를 업로드하는 테스트를 수행합니다.
실제 마스크 업로드 워크플로우를 시뮬레이션합니다.

사용법:
    python3 scripts/test_mask_upload.py

필요한 환경 변수:
    - PACS_SERVER_URL: PACS 서버 URL (기본값: http://localhost:8080)
    - TEST_USER_ID: 테스트 사용자 ID (기본값: 1)
    - TEST_PROJECT_ID: 테스트 프로젝트 ID (기본값: 1)
"""

import os
import sys
import json
import time
import hashlib
import requests
from pathlib import Path
from typing import Dict, Any, Optional

# 서버 설정
PACS_SERVER_URL = os.getenv('PACS_SERVER_URL', 'http://localhost:8080')
TEST_USER_ID = int(os.getenv('TEST_USER_ID', '1'))
TEST_PROJECT_ID = int(os.getenv('TEST_PROJECT_ID', '1'))

# 테스트 이미지 디렉토리
TEST_IMAGES_DIR = Path(__file__).parent.parent / 'test_images'

class MaskUploadTester:
    """마스크 업로드 테스트 클래스"""
    
    def __init__(self, server_url: str, user_id: int, project_id: int):
        self.server_url = server_url.rstrip('/')
        self.user_id = user_id
        self.project_id = project_id
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        })
        
    def log(self, message: str, level: str = "INFO"):
        """로그 메시지 출력"""
        timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
        print(f"[{timestamp}] [{level}] {message}")
        
    def check_server_health(self) -> bool:
        """서버 상태 확인"""
        try:
            response = self.session.get(f"{self.server_url}/health")
            if response.status_code == 200:
                self.log("서버가 정상적으로 동작 중입니다.")
                return True
            else:
                self.log(f"서버 상태 확인 실패: {response.status_code}", "ERROR")
                return False
        except Exception as e:
            self.log(f"서버 연결 실패: {e}", "ERROR")
            return False
    
    def create_test_annotation(self) -> Optional[int]:
        """테스트용 어노테이션 생성"""
        annotation_data = {
            "study_instance_uid": "1.2.3.4.5.6.7.8.9.10",
            "series_instance_uid": "1.2.3.4.5.6.7.8.9.11", 
            "sop_instance_uid": "1.2.3.4.5.6.7.8.9.12",
            "tool_name": "manual",
            "tool_version": "1.0.0",
            "viewer_software": "OHIF",
            "description": "마스크 업로드 테스트용 어노테이션",
            "annotation_data": {
                "type": "polygon",
                "points": [[100, 100], [200, 100], [200, 200], [100, 200]],
                "label": "test_region"
            }
        }
        
        try:
            response = self.session.post(
                f"{self.server_url}/api/annotations",
                json=annotation_data,
                params={"user_id": self.user_id, "project_id": self.project_id}
            )
            
            if response.status_code == 201:
                annotation = response.json()
                annotation_id = annotation['id']
                self.log(f"테스트 어노테이션 생성 성공: ID {annotation_id}")
                return annotation_id
            else:
                self.log(f"어노테이션 생성 실패: {response.status_code} - {response.text}", "ERROR")
                return None
                
        except Exception as e:
            self.log(f"어노테이션 생성 중 오류: {e}", "ERROR")
            return None
    
    def create_mask_group(self, annotation_id: int) -> Optional[int]:
        """마스크 그룹 생성"""
        mask_group_data = {
            "annotation_id": annotation_id,
            "group_name": "Test Mask Group",
            "model_name": "Test Model",
            "version": "1.0.0",
            "modality": "CT",
            "slice_count": 3,
            "mask_type": "segmentation",
            "description": "테스트용 마스크 그룹",
            "created_by": self.user_id
        }
        
        try:
            response = self.session.post(
                f"{self.server_url}/api/mask-groups",
                json=mask_group_data
            )
            
            if response.status_code == 201:
                mask_group = response.json()
                mask_group_id = mask_group['id']
                self.log(f"마스크 그룹 생성 성공: ID {mask_group_id}")
                return mask_group_id
            else:
                self.log(f"마스크 그룹 생성 실패: {response.status_code} - {response.text}", "ERROR")
                return None
                
        except Exception as e:
            self.log(f"마스크 그룹 생성 중 오류: {e}", "ERROR")
            return None
    
    def calculate_file_checksum(self, file_path: Path) -> str:
        """파일 체크섬 계산"""
        hash_md5 = hashlib.md5()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_md5.update(chunk)
        return hash_md5.hexdigest()
    
    def upload_mask_file(self, mask_group_id: int, file_path: Path, slice_index: int) -> Optional[Dict[str, Any]]:
        """마스크 파일 업로드"""
        if not file_path.exists():
            self.log(f"파일을 찾을 수 없습니다: {file_path}", "ERROR")
            return None
            
        # 파일 정보 수집
        file_size = file_path.stat().st_size
        checksum = self.calculate_file_checksum(file_path)
        
        # 업로드 URL 요청
        upload_request = {
            "mask_group_id": mask_group_id,
            "file_name": file_path.name,
            "mime_type": "image/png",
            "file_size": file_size,
            "checksum": checksum,
            "width": 512,
            "height": 512,
            "slice_index": slice_index,
            "sop_instance_uid": f"1.2.3.4.5.6.7.8.9.{12 + slice_index}",
            "label_name": f"test_label_{slice_index}"
        }
        
        try:
            # 1단계: 업로드 URL 요청
            response = self.session.post(
                f"{self.server_url}/api/masks/upload-url",
                json=upload_request
            )
            
            if response.status_code != 200:
                self.log(f"업로드 URL 요청 실패: {response.status_code} - {response.text}", "ERROR")
                return None
                
            upload_info = response.json()
            upload_url = upload_info['upload_url']
            mask_id = upload_info['mask_id']
            
            self.log(f"업로드 URL 획득 성공: Mask ID {mask_id}")
            
            # 2단계: 실제 파일 업로드
            with open(file_path, 'rb') as f:
                upload_response = requests.put(upload_url, data=f, headers={'Content-Type': 'image/png'})
                
            if upload_response.status_code in [200, 204]:
                self.log(f"파일 업로드 성공: {file_path.name}")
                
                # 3단계: 업로드 완료 알림
                complete_request = {
                    "mask_group_id": mask_group_id,
                    "slice_count": 3,
                    "labels": [f"test_label_{i}" for i in range(1, 4)],
                    "uploaded_files": [file_path.name]
                }
                
                complete_response = self.session.post(
                    f"{self.server_url}/api/masks/complete-upload",
                    json=complete_request
                )
                
                if complete_response.status_code == 200:
                    self.log(f"업로드 완료 알림 성공: {file_path.name}")
                    return {
                        'mask_id': mask_id,
                        'file_name': file_path.name,
                        'file_size': file_size,
                        'checksum': checksum
                    }
                else:
                    self.log(f"업로드 완료 알림 실패: {complete_response.status_code}", "WARN")
                    return {
                        'mask_id': mask_id,
                        'file_name': file_path.name,
                        'file_size': file_size,
                        'checksum': checksum
                    }
            else:
                self.log(f"파일 업로드 실패: {upload_response.status_code}", "ERROR")
                return None
                
        except Exception as e:
            self.log(f"파일 업로드 중 오류: {e}", "ERROR")
            return None
    
    def download_mask_file(self, mask_id: int) -> bool:
        """마스크 파일 다운로드 테스트"""
        try:
            download_request = {
                "mask_id": mask_id
            }
            
            response = self.session.post(
                f"{self.server_url}/api/masks/download-url",
                json=download_request
            )
            
            if response.status_code == 200:
                download_info = response.json()
                download_url = download_info['download_url']
                
                # 파일 다운로드
                download_response = requests.get(download_url)
                if download_response.status_code == 200:
                    self.log(f"마스크 파일 다운로드 성공: Mask ID {mask_id}")
                    return True
                else:
                    self.log(f"마스크 파일 다운로드 실패: {download_response.status_code}", "ERROR")
                    return False
            else:
                self.log(f"다운로드 URL 요청 실패: {response.status_code}", "ERROR")
                return False
                
        except Exception as e:
            self.log(f"마스크 파일 다운로드 중 오류: {e}", "ERROR")
            return False
    
    def run_test(self):
        """전체 테스트 실행"""
        self.log("=== 마스크 업로드 테스트 시작 ===")
        
        # 1. 서버 상태 확인
        if not self.check_server_health():
            self.log("서버가 동작하지 않습니다. 테스트를 중단합니다.", "ERROR")
            return False
        
        # 2. 테스트 어노테이션 생성
        annotation_id = self.create_test_annotation()
        if not annotation_id:
            self.log("어노테이션 생성에 실패했습니다. 테스트를 중단합니다.", "ERROR")
            return False
        
        # 3. 마스크 그룹 생성
        mask_group_id = self.create_mask_group(annotation_id)
        if not mask_group_id:
            self.log("마스크 그룹 생성에 실패했습니다. 테스트를 중단합니다.", "ERROR")
            return False
        
        # 4. 테스트 이미지 파일들 업로드
        uploaded_masks = []
        test_files = [
            TEST_IMAGES_DIR / 'sample_mask_1.png',
            TEST_IMAGES_DIR / 'sample_mask_2.png', 
            TEST_IMAGES_DIR / 'sample_mask_3.png'
        ]
        
        for i, file_path in enumerate(test_files, 1):
            self.log(f"마스크 파일 업로드 중: {file_path.name}")
            result = self.upload_mask_file(mask_group_id, file_path, i)
            if result:
                uploaded_masks.append(result)
                self.log(f"업로드 성공: {result['file_name']} (Mask ID: {result['mask_id']})")
            else:
                self.log(f"업로드 실패: {file_path.name}", "ERROR")
        
        # 5. 다운로드 테스트
        if uploaded_masks:
            self.log("다운로드 테스트 시작")
            for mask_info in uploaded_masks:
                self.download_mask_file(mask_info['mask_id'])
        
        # 6. 결과 요약
        self.log("=== 테스트 결과 요약 ===")
        self.log(f"총 업로드 시도: {len(test_files)}")
        self.log(f"성공한 업로드: {len(uploaded_masks)}")
        self.log(f"실패한 업로드: {len(test_files) - len(uploaded_masks)}")
        
        if len(uploaded_masks) == len(test_files):
            self.log("모든 테스트가 성공적으로 완료되었습니다!", "SUCCESS")
            return True
        else:
            self.log("일부 테스트가 실패했습니다.", "WARN")
            return False

def main():
    """메인 함수"""
    print("PACS Extension Server - 마스크 업로드 테스트")
    print("=" * 50)
    
    # 환경 변수 확인
    print(f"서버 URL: {PACS_SERVER_URL}")
    print(f"사용자 ID: {TEST_USER_ID}")
    print(f"프로젝트 ID: {TEST_PROJECT_ID}")
    print(f"테스트 이미지 디렉토리: {TEST_IMAGES_DIR}")
    print()
    
    # 테스트 실행
    tester = MaskUploadTester(PACS_SERVER_URL, TEST_USER_ID, TEST_PROJECT_ID)
    success = tester.run_test()
    
    if success:
        print("\n✅ 모든 테스트가 성공적으로 완료되었습니다!")
        sys.exit(0)
    else:
        print("\n❌ 일부 테스트가 실패했습니다.")
        sys.exit(1)

if __name__ == "__main__":
    main()
