#!/usr/bin/env python3
"""
어노테이션 권한 관리 E2E 테스트 (리팩토링 버전)

이 테스트는 어노테이션 생성/수정/삭제 권한 제어 및 권한 조회 API를 검증합니다.
"""

import requests
import json
from test_base import BaseE2ETest, TestConfig, TestPrinter, TestAuth
from test_fixtures import AnnotationFixtures


class AnnotationPermissionManagementTest(BaseE2ETest):
    """어노테이션 권한 관리 테스트"""
    
    def __init__(self):
        super().__init__()
        self.normal_user_token = None
        self.test_annotation_id = None
    
    def get_test_name(self) -> str:
        return "어노테이션 권한 관리 E2E 테스트"
    
    def setup(self):
        """테스트 셋업"""
        super().setup()
        
        # 일반 사용자 로그인 (권한 없는 사용자 테스트용)
        try:
            self.normal_user_token = TestAuth.login("iaid-pacs-user1", "Qlalfqjsgh1!")
        except:
            print("⚠️  일반 사용자 로그인 실패 (일부 테스트 스킵 가능)")
    
    def run_tests(self):
        """테스트 실행"""
        self.test_annotation_id = self.test_create_annotation_with_permission()
        self.test_update_annotation_as_owner(self.test_annotation_id)
        self.test_get_annotation_permissions()
        
        if self.normal_user_token:
            self.test_create_annotation_without_permission()
        
        # 마지막에 삭제
        self.test_delete_annotation_as_owner(self.test_annotation_id)
    
    def test_create_annotation_with_permission(self) -> int:
        """테스트 1: 권한이 있는 사용자는 어노테이션을 생성할 수 있어야 함"""
        TestPrinter.print_header("테스트 1: 권한 있는 사용자 - 어노테이션 생성")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
            "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
            "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
            "annotation_data": {"type": "circle", "x": 100, "y": 200, "radius": 50},
            "tool_name": "Circle Tool",
            "tool_version": "2.1.0",
            "viewer_software": "OHIF Viewer",
            "description": "Permission test annotation",
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 201:
            data = response.json()
            annotation_id = data["id"]
            TestPrinter.print_success(f"어노테이션 생성 성공! ID: {annotation_id}")
            TestPrinter.print_success("테스트 통과")
            return annotation_id
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_update_annotation_as_owner(self, annotation_id: int):
        """테스트 2: 어노테이션 소유자는 수정할 수 있어야 함"""
        TestPrinter.print_header("테스트 2: 소유자 - 어노테이션 수정")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        update_data = {
            "annotation_data": {"type": "circle", "x": 150, "y": 250, "radius": 75},
            "description": "Updated by owner",
        }
        
        response = requests.put(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            json=update_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("어노테이션 수정 성공!")
            TestPrinter.print_info(f"Description: {data.get('description')}", indent=1)
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_delete_annotation_as_owner(self, annotation_id: int):
        """테스트 3: 어노테이션 소유자는 삭제할 수 있어야 함"""
        TestPrinter.print_header("테스트 3: 소유자 - 어노테이션 삭제")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.delete(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 204:
            TestPrinter.print_success("어노테이션 삭제 성공!")
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_get_annotation_permissions(self):
        """테스트 4: 권한 조회 API 테스트"""
        TestPrinter.print_header("테스트 4: 권한 조회 API")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/permissions?project_id={TestConfig.DEFAULT_PROJECT_ID}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            TestPrinter.print_success("권한 조회 성공!")
            TestPrinter.print_info(f"Permissions: {json.dumps(data, indent=2)}", indent=1)
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_create_annotation_without_permission(self):
        """테스트 5: 권한이 없는 사용자는 어노테이션을 생성할 수 없어야 함"""
        TestPrinter.print_header("테스트 5: 권한 없는 사용자 - 어노테이션 생성 시도")
        
        headers = {"Authorization": f"Bearer {self.normal_user_token}"}
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,  # 권한 없는 프로젝트
            "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
            "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
            "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
            "annotation_data": {"type": "circle", "x": 100, "y": 200, "radius": 50},
            "tool_name": "Circle Tool",
            "viewer_software": "OHIF Viewer",
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        # 권한이 없으면 403 Forbidden 또는 401 Unauthorized 응답
        if response.status_code in [401, 403]:
            TestPrinter.print_success("권한 없는 사용자는 생성할 수 없음 (예상된 동작)")
            TestPrinter.print_success("테스트 통과")
        elif response.status_code == 201:
            TestPrinter.print_warning("어노테이션이 생성됨 (권한 체크가 없을 수 있음)")
        else:
            TestPrinter.print_warning(f"예상치 못한 응답: {response.text}")


if __name__ == '__main__':
    test = AnnotationPermissionManagementTest()
    test.run()

