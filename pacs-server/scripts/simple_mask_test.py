#!/usr/bin/env python3
"""
간단한 마스크 업로드 테스트 스크립트

이 스크립트는 기존 어노테이션을 사용하여 마스크 업로드를 테스트합니다.
"""

import os
import sys
import json
import time
import hashlib
import requests
from pathlib import Path
from typing import Optional

# 서버 설정
PACS_SERVER_URL = os.getenv('PACS_SERVER_URL', 'http://localhost:8080')

class SimpleMaskTester:
    """간단한 마스크 테스트 클래스"""
    
    def __init__(self, server_url: str):
        self.server_url = server_url.rstrip('/')
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
    
    def create_test_annotation_directly(self) -> Optional[int]:
        """데이터베이스에 직접 어노테이션 생성"""
        try:
            # PostgreSQL에 직접 어노테이션 생성
            import subprocess
            result = subprocess.run([
                'psql', os.getenv('DATABASE_URL', 'postgres://admin:admin123@localhost:5432/pacs_db'),
                '-c', """
                INSERT INTO annotation_annotation 
                (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, tool_version, 
                 viewer_software, description, data, is_shared, created_at, updated_at)
                VALUES 
                (1, 1, '1.2.3.4.5.6.7.8.9.10', '1.2.3.4.5.6.7.8.9.11', '1.2.3.4.5.6.7.8.9.12',
                 'manual', '1.0.0', 'OHIF', '테스트 어노테이션', 
                 '{"type": "polygon", "points": [[100, 100], [200, 100], [200, 200], [100, 200]]}',
                 false, NOW(), NOW())
                RETURNING id;
                """
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                # ID 추출
                lines = result.stdout.strip().split('\n')
                for line in lines:
                    if line.strip().isdigit():
                        annotation_id = int(line.strip())
                        self.log(f"테스트 어노테이션 생성 성공: ID {annotation_id}")
                        return annotation_id
                        
            self.log(f"어노테이션 생성 실패: {result.stderr}", "ERROR")
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
            "created_by": 1
        }
        
        try:
            response = self.session.post(
                f"{self.server_url}/api/annotations/{annotation_id}/mask-groups",
                json=mask_group_data,
                headers={"X-User-ID": "1"}
            )
            
            if response.status_code == 201:
                mask_group = response.json()
                mask_group_id = mask_group['id']
                self.log(f"마스크 그룹 생성 성공: ID {mask_group_id}")
                return mask_group_id
            else:
                self.log(f"마스크 그룹 생성 실패: {response.status_code} - {response.text}", "ERROR")
                self.log(f"요청 URL: {self.server_url}/api/annotations/{annotation_id}/mask-groups", "DEBUG")
                self.log(f"요청 데이터: {json.dumps(mask_group_data, indent=2)}", "DEBUG")
                return None
                
        except Exception as e:
            self.log(f"마스크 그룹 생성 중 오류: {e}", "ERROR")
            return None
    
    def upload_test_image(self, annotation_id: int, mask_group_id: int, file_path: Path) -> bool:
        """테스트 이미지 업로드"""
        if not file_path.exists():
            self.log(f"파일을 찾을 수 없습니다: {file_path}", "ERROR")
            return False
            
        try:
            # 파일 정보 수집
            file_size = file_path.stat().st_size
            checksum = hashlib.md5(file_path.read_bytes()).hexdigest()
            
            # 업로드 URL 요청
            upload_request = {
                "mask_group_id": mask_group_id,
                "file_name": file_path.name,
                "mime_type": "image/png",
                "file_size": file_size,
                "checksum": checksum,
                "width": 512,
                "height": 512,
                "slice_index": 1,
                "sop_instance_uid": "1.2.3.4.5.6.7.8.9.12",
                "label_name": "test_label"
            }
            
            response = self.session.post(
                f"{self.server_url}/api/annotations/{annotation_id}/mask-groups/{mask_group_id}/upload-url",
                json=upload_request,
                headers={"X-User-ID": "1"}
            )
            
            if response.status_code == 200:
                upload_info = response.json()
                upload_url = upload_info['upload_url']
                mask_id = upload_info['mask_id']
                
                self.log(f"업로드 URL 획득 성공: Mask ID {mask_id}")
                
                # 실제 파일 업로드
                with open(file_path, 'rb') as f:
                    upload_response = requests.put(upload_url, data=f, headers={'Content-Type': 'image/png'})
                    
                if upload_response.status_code in [200, 204]:
                    self.log(f"파일 업로드 성공: {file_path.name}")
                    return True
                else:
                    self.log(f"파일 업로드 실패: {upload_response.status_code}", "ERROR")
                    return False
            else:
                self.log(f"업로드 URL 요청 실패: {response.status_code} - {response.text}", "ERROR")
                return False
                
        except Exception as e:
            self.log(f"파일 업로드 중 오류: {e}", "ERROR")
            return False
    
    def run_test(self):
        """전체 테스트 실행"""
        self.log("=== 간단한 마스크 업로드 테스트 시작 ===")
        
        # 1. 서버 상태 확인
        if not self.check_server_health():
            self.log("서버가 동작하지 않습니다. 테스트를 중단합니다.", "ERROR")
            return False
        
        # 2. 테스트 어노테이션 생성 (직접 DB에)
        annotation_id = self.create_test_annotation_directly()
        if not annotation_id:
            self.log("어노테이션 생성에 실패했습니다. 테스트를 중단합니다.", "ERROR")
            return False
        
        # 3. 마스크 그룹 생성
        mask_group_id = self.create_mask_group(annotation_id)
        if not mask_group_id:
            self.log("마스크 그룹 생성에 실패했습니다. 테스트를 중단합니다.", "ERROR")
            return False
        
        # 4. 테스트 이미지 업로드
        test_images_dir = Path(__file__).parent.parent / 'test_images'
        test_files = [
            test_images_dir / 'sample_mask_1.png',
            test_images_dir / 'sample_mask_2.png', 
            test_images_dir / 'sample_mask_3.png'
        ]
        
        success_count = 0
        for i, file_path in enumerate(test_files, 1):
            self.log(f"마스크 파일 업로드 중: {file_path.name}")
            if self.upload_test_image(annotation_id, mask_group_id, file_path):
                success_count += 1
                self.log(f"업로드 성공: {file_path.name}")
            else:
                self.log(f"업로드 실패: {file_path.name}", "ERROR")
        
        # 5. 결과 요약
        self.log("=== 테스트 결과 요약 ===")
        self.log(f"총 업로드 시도: {len(test_files)}")
        self.log(f"성공한 업로드: {success_count}")
        self.log(f"실패한 업로드: {len(test_files) - success_count}")
        
        if success_count == len(test_files):
            self.log("모든 테스트가 성공적으로 완료되었습니다!", "SUCCESS")
            return True
        else:
            self.log("일부 테스트가 실패했습니다.", "WARN")
            return False

def main():
    """메인 함수"""
    print("PACS Extension Server - 간단한 마스크 업로드 테스트")
    print("=" * 60)
    
    # 테스트 실행
    tester = SimpleMaskTester(PACS_SERVER_URL)
    success = tester.run_test()
    
    if success:
        print("\n✅ 모든 테스트가 성공적으로 완료되었습니다!")
        sys.exit(0)
    else:
        print("\n❌ 일부 테스트가 실패했습니다.")
        sys.exit(1)

if __name__ == "__main__":
    main()
