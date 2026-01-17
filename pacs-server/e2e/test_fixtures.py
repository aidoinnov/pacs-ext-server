#!/usr/bin/env python3
"""
E2E 테스트 픽스처 (테스트 데이터 생성 헬퍼)
"""

import requests
import time
from typing import Optional, Dict, Any, List, Tuple
from test_base import TestConfig


class UserFixtures:
    """사용자 테스트 데이터 생성"""

    @staticmethod
    def create_test_user(
        username: str = None,
        email: str = None,
        password: str = "TestPassword123!",
        full_name: str = "Test User"
    ) -> Optional[Tuple[int, str]]:
        """테스트용 사용자 생성

        Returns:
            (user_id, username) 튜플 또는 None
        """
        from test_utils import create_user

        timestamp = int(time.time())
        username = username or f"test_user_{timestamp}"
        email = email or f"test_{timestamp}@example.com"

        return create_user(username, email, password, full_name)

    @staticmethod
    def setup_user_with_project(
        admin_token: str,
        project_id: int,
        role_id: int = 196,  # PROJECT_ADMIN
        username: str = None,
        password: str = "TestPassword123!"
    ) -> Optional[Tuple[int, str, str]]:
        """사용자 생성 + 승인 + 프로젝트 추가

        Returns:
            (user_id, username, password) 튜플 또는 None
        """
        from test_utils import approve_user, add_user_to_project

        # 사용자 생성
        user_result = UserFixtures.create_test_user(username=username, password=password)
        if not user_result:
            return None

        user_id, username = user_result

        # 사용자 승인
        if not approve_user(admin_token, user_id):
            return None

        # 프로젝트에 추가
        if not add_user_to_project(admin_token, project_id, user_id, role_id=role_id):
            print(f"   ⚠️  프로젝트 추가 실패 (계속 진행)")

        return user_id, username, password


class AnnotationFixtures:
    """어노테이션 테스트 데이터 생성"""
    
    @staticmethod
    def create_basic_annotation(
        token: str,
        description: str = "Test annotation",
        project_id: int = None,
        study_uid: str = None,
        series_uid: str = None,
        instance_uid: str = None,
    ) -> Optional[int]:
        """기본 어노테이션 생성"""
        headers = {"Authorization": f"Bearer {token}"}
        
        annotation_data = {
            "project_id": project_id or TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": study_uid or TestConfig.STUDY_UID,
            "series_instance_uid": series_uid or TestConfig.SERIES_UID,
            "sop_instance_uid": instance_uid or TestConfig.INSTANCE_UID,
            "annotation_data": {"type": "circle", "x": 100, "y": 100, "radius": 50},
            "tool_name": "Circle Tool",
            "viewer_software": "TI-DicomViewer",
            "description": description,
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code == 201:
            return response.json()["id"]
        return None
    
    @staticmethod
    def create_study_level_annotation(token: str, description: str = "Study level test") -> Optional[int]:
        """Study 레벨 어노테이션 생성"""
        headers = {"Authorization": f"Bearer {token}"}
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": TestConfig.STUDY_UID,
            "series_instance_uid": None,
            "sop_instance_uid": None,
            "annotation_data": {"type": "study_note", "text": "Study level annotation"},
            "tool_name": "Note Tool",
            "tool_version": "1.0.0",
            "viewer_software": "TI-DicomViewer",
            "description": description,
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code == 201:
            return response.json()["id"]
        return None
    
    @staticmethod
    def create_series_level_annotation(token: str, description: str = "Series level test") -> Optional[int]:
        """Series 레벨 어노테이션 생성"""
        headers = {"Authorization": f"Bearer {token}"}
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": TestConfig.STUDY_UID,
            "series_instance_uid": TestConfig.SERIES_UID,
            "sop_instance_uid": None,
            "annotation_data": {"type": "series_note", "text": "Series level annotation"},
            "tool_name": "Note Tool",
            "tool_version": "1.0.0",
            "viewer_software": "TI-DicomViewer",
            "description": description,
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code == 201:
            return response.json()["id"]
        return None
    
    @staticmethod
    def create_instance_level_annotation(token: str, description: str = "Instance level test") -> Optional[int]:
        """Instance 레벨 어노테이션 생성"""
        headers = {"Authorization": f"Bearer {token}"}
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": TestConfig.STUDY_UID,
            "series_instance_uid": TestConfig.SERIES_UID,
            "sop_instance_uid": TestConfig.INSTANCE_UID,
            "annotation_data": {"type": "measurement", "value": 10.5},
            "tool_name": "Measurement Tool",
            "tool_version": "1.0.0",
            "viewer_software": "TI-DicomViewer",
            "description": description,
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code == 201:
            return response.json()["id"]
        return None
    
    @staticmethod
    def create_all_level_annotations(token: str) -> List[int]:
        """모든 레벨의 어노테이션 생성"""
        created_ids = []
        
        print("📝 테스트용 어노테이션 생성 중...")
        
        # Study level
        study_id = AnnotationFixtures.create_study_level_annotation(token)
        if study_id:
            created_ids.append(study_id)
            print(f"   ✓ Created annotation ID: {study_id} - Study level test")
        
        # Series level
        series_id = AnnotationFixtures.create_series_level_annotation(token)
        if series_id:
            created_ids.append(series_id)
            print(f"   ✓ Created annotation ID: {series_id} - Series level test")
        
        # Instance level
        instance_id = AnnotationFixtures.create_instance_level_annotation(token)
        if instance_id:
            created_ids.append(instance_id)
            print(f"   ✓ Created annotation ID: {instance_id} - Instance level test")
        
        print(f"✅ {len(created_ids)}개 어노테이션 생성 완료\n")
        return created_ids


class ImageFixtures:
    """이미지 테스트 데이터 생성"""

    @staticmethod
    def create_test_snapshot_image() -> bytes:
        """테스트용 PNG 스냅샷 이미지 생성

        Returns:
            PNG 이미지 바이트 데이터
        """
        import io
        from PIL import Image, ImageDraw, ImageFont

        # 800x600 크기의 이미지 생성
        img = Image.new('RGB', (800, 600), color='white')
        draw = ImageDraw.Draw(img)

        # 배경 그라데이션
        for y in range(600):
            color = int(255 * (1 - y / 600))
            draw.rectangle([(0, y), (800, y+1)], fill=(color, color, 255))

        # 원 그리기 (어노테이션 시뮬레이션)
        draw.ellipse([200, 150, 400, 350], outline='red', width=3)

        # 사각형 그리기
        draw.rectangle([450, 200, 650, 400], outline='green', width=3)

        # 텍스트 추가
        try:
            # 시스템 폰트 사용 시도
            font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 24)
        except:
            # 폰트 로드 실패 시 기본 폰트 사용
            font = ImageFont.load_default()

        draw.text((250, 450), "Test Annotation Snapshot", fill='black', font=font)
        draw.text((250, 480), "Circle: Lesion Area", fill='red', font=font)
        draw.text((250, 510), "Rectangle: ROI", fill='green', font=font)

        # 바이트 스트림으로 변환
        img_byte_arr = io.BytesIO()
        img.save(img_byte_arr, format='PNG')
        img_byte_arr.seek(0)

        return img_byte_arr.getvalue()

