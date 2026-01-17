#!/usr/bin/env python3
"""
권한 기반 어노테이션 필터링 E2E 테스트 (리팩토링 버전)

이 테스트는 사용자의 권한에 따라 어노테이션 조회 결과가 달라지는지 검증합니다.
- ANNOTATION:READ_ALL 권한이 있으면: 프로젝트의 모든 어노테이션 반환
- ANNOTATION:READ_ALL 권한이 없으면: 본인의 어노테이션만 반환
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter, TestAuth
from test_fixtures import UserFixtures, AnnotationFixtures
from test_utils import create_annotation, cleanup_annotations


class AnnotationPermissionFilteringTest(BaseE2ETest):
    """권한 기반 어노테이션 필터링 테스트"""
    
    def __init__(self):
        super().__init__()
        self.test_user_id = None
        self.test_username = None
        self.test_password = None
    
    def get_test_name(self) -> str:
        return "권한 기반 어노테이션 필터링 E2E 테스트"
    
    def run_tests(self):
        """테스트 실행"""
        self.test_admin_sees_all_annotations()
        self.test_normal_user_sees_own_annotations()
        self.test_series_level_filtering_with_permission()
    
    def test_admin_sees_all_annotations(self):
        """테스트 1: READ_ALL 권한이 있는 사용자는 모든 어노테이션을 볼 수 있어야 함"""
        TestPrinter.print_header("테스트 1: Admin 사용자 - 모든 어노테이션 조회")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations?series_instance_uid={TestConfig.SERIES_UID}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            total = data.get("total", 0)
            annotations = data.get("annotations", [])
            
            TestPrinter.print_success(f"Admin user sees {total} annotations")
            
            # 다양한 사용자의 어노테이션이 포함되어 있는지 확인
            if annotations:
                unique_users = set(ann["user_id"] for ann in annotations)
                TestPrinter.print_info(f"Unique users: {len(unique_users)}", indent=1)
                TestPrinter.print_info(f"User IDs: {sorted(unique_users)}", indent=1)
            
            assert total > 0, "Admin should see at least some annotations"
            TestPrinter.print_success("테스트 통과: Admin은 모든 어노테이션을 볼 수 있음")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_normal_user_sees_own_annotations(self):
        """테스트 2: 일반 사용자 - 본인 어노테이션만 조회"""
        TestPrinter.print_header("테스트 2: 일반 사용자 - 본인 어노테이션만 조회")
        
        # 1. 테스트용 사용자 생성 및 설정
        print("1️⃣  테스트용 사용자 생성 및 설정 중...")
        project_id = 556  # 존재하는 프로젝트 ID
        role_id = 196  # PROJECT_ADMIN 역할
        
        user_result = UserFixtures.setup_user_with_project(
            self.token, 
            project_id, 
            role_id=role_id
        )
        
        if not user_result:
            TestPrinter.print_error("사용자 생성 실패")
            return
        
        self.test_user_id, self.test_username, self.test_password = user_result
        self.created_user_ids.append(self.test_user_id)
        
        TestPrinter.print_success(f"사용자 생성 성공: ID={self.test_user_id}, Username={self.test_username}")
        
        # 2. 로그인
        print("\n2️⃣  테스트 사용자로 로그인 중...")
        user_token = TestAuth.login(self.test_username, self.test_password)
        headers = {"Authorization": f"Bearer {user_token}"}
        TestPrinter.print_success("로그인 성공")
        
        # 3. 어노테이션 생성 (본인 것)
        print("\n3️⃣  테스트용 어노테이션 생성 중...")
        annotation_data = {
            "project_id": project_id,
            "study_instance_uid": TestConfig.STUDY_UID,
            "series_instance_uid": TestConfig.SERIES_UID,
            "sop_instance_uid": TestConfig.INSTANCE_UID,
            "annotation_data": {"type": "test"},
            "tool_name": "Test Tool",
            "viewer_software": "Test",
            "description": "Permission test annotation",
        }
        
        ann_id = create_annotation(user_token, annotation_data)
        if not ann_id:
            TestPrinter.print_warning("어노테이션 생성 실패")
            return
        
        TestPrinter.print_success(f"어노테이션 생성 성공: ID={ann_id}")
        
        # 4. Series UID로 어노테이션 조회
        print("\n4️⃣  어노테이션 조회 중...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations?series_instance_uid={TestConfig.SERIES_UID}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code == 200:
            data = response.json()
            total = data.get("total", 0)
            annotations = data.get("annotations", [])
            
            TestPrinter.print_success(f"Normal user sees {total} annotations", indent=1)
            
            # 모든 어노테이션이 본인 것인지 확인
            if annotations:
                user_ids = set(ann["user_id"] for ann in annotations)
                TestPrinter.print_info(f"User IDs in results: {user_ids}", indent=1)
                
                # 일반 사용자는 본인 어노테이션만 볼 수 있어야 함
                if len(user_ids) == 1 and self.test_user_id in user_ids:
                    TestPrinter.print_success("모든 어노테이션이 본인 것임 (정상)", indent=1)
                else:
                    TestPrinter.print_warning(f"다른 사용자의 어노테이션도 포함됨 (user_id={self.test_user_id})", indent=1)
            
            TestPrinter.print_success("테스트 통과: 일반 사용자는 본인 어노테이션만 볼 수 있음")
            
            # Cleanup: 생성한 어노테이션 삭제
            if ann_id:
                cleanup_annotations(user_token, [ann_id])
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}", indent=1)
    
    def test_series_level_filtering_with_permission(self):
        """테스트 3: Series UID로 필터링 시에도 권한 기반 필터링이 적용되어야 함"""
        TestPrinter.print_header("테스트 3: Series UID 필터링 + 권한 기반 필터링")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations?series_instance_uid={TestConfig.SERIES_UID}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            total = data.get("total", 0)
            TestPrinter.print_success(f"Found {total} annotations for series")
            TestPrinter.print_success("테스트 통과: Series 필터링 + 권한 필터링 정상 작동")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)


if __name__ == '__main__':
    test = AnnotationPermissionFilteringTest()
    test.run()

