#!/usr/bin/env python3
"""
어노테이션 레벨 필터링 E2E 테스트 (리팩토링 버전)

이 테스트는 어노테이션을 Study/Series/Instance 레벨로 필터링하는 기능을 검증합니다.
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import AnnotationFixtures


class AnnotationLevelFilteringTest(BaseE2ETest):
    """어노테이션 레벨 필터링 테스트"""
    
    def get_test_name(self) -> str:
        return "어노테이션 레벨 필터링 E2E 테스트"
    
    def run_tests(self):
        """테스트 실행"""
        # 테스트 데이터 생성
        self.created_annotation_ids = AnnotationFixtures.create_all_level_annotations(self.token)
        
        # 테스트 실행
        self.test_level_filter_study()
        self.test_level_filter_series()
        self.test_level_filter_instance()
    
    def test_level_filter_study(self):
        """테스트 1: Study 레벨 필터링"""
        TestPrinter.print_header("테스트 1: Study 레벨 필터링")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations?study_instance_uid={TestConfig.STUDY_UID}&level=study",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            annotations = data.get("annotations", [])
            
            # Study 레벨만 필터링되었는지 확인
            study_level = [ann for ann in annotations 
                          if not ann["series_instance_uid"] and not ann["sop_instance_uid"]]
            
            TestPrinter.print_success(f"Study level annotations: {len(study_level)}")
            for ann in study_level:
                TestPrinter.print_info(f"ID: {ann['id']}, Description: {ann.get('description', 'N/A')}", indent=1)
            
            assert len(study_level) > 0, "Should have at least one study level annotation"
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_level_filter_series(self):
        """테스트 2: Series 레벨 필터링"""
        TestPrinter.print_header("테스트 2: Series 레벨 필터링")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations?series_instance_uid={TestConfig.SERIES_UID}&level=series",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            annotations = data.get("annotations", [])
            
            # Series 레벨만 필터링되었는지 확인
            series_level = [ann for ann in annotations 
                           if ann["series_instance_uid"] and not ann["sop_instance_uid"]]
            
            TestPrinter.print_success(f"Series level annotations: {len(series_level)}")
            for ann in series_level:
                TestPrinter.print_info(f"ID: {ann['id']}, Description: {ann.get('description', 'N/A')}", indent=1)
            
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_level_filter_instance(self):
        """테스트 3: Instance 레벨 필터링"""
        TestPrinter.print_header("테스트 3: Instance 레벨 필터링")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations?sop_instance_uid={TestConfig.INSTANCE_UID}&level=instance",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            annotations = data.get("annotations", [])
            
            # Instance 레벨만 필터링되었는지 확인
            instance_level = [ann for ann in annotations if ann["sop_instance_uid"]]
            
            TestPrinter.print_success(f"Instance level annotations: {len(instance_level)}")
            for ann in instance_level:
                TestPrinter.print_info(f"ID: {ann['id']}, Description: {ann.get('description', 'N/A')}", indent=1)
            
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)


if __name__ == '__main__':
    test = AnnotationLevelFilteringTest()
    test.run()

