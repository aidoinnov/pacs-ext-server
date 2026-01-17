#!/usr/bin/env python3
"""
View Selection E2E Test (리팩토링 버전)
뷰 선택 기능 전체 워크플로우 테스트
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter


class ViewSelectionE2ETest(BaseE2ETest):
    """View Selection 테스트"""
    
    def __init__(self):
        super().__init__()
        self.created_selection_ids = []
    
    def get_test_name(self) -> str:
        return "View Selection E2E Test"
    
    def run_tests(self):
        """테스트 실행"""
        self.test_create_view_selection()
        self.test_retrieve_view_selection()
        self.test_delete_view_selection()
    
    def test_create_view_selection(self):
        """테스트 1: View Selection 생성"""
        TestPrinter.print_header("테스트 1: View Selection 생성")
        
        # 첫 번째 데이터
        selection_data_1 = {
            'series': [
                {
                    'study_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781',
                    'series_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345'
                }
            ]
        }
        
        selection_id_1 = self._create_selection(selection_data_1, "첫 번째 Selection")
        if selection_id_1:
            self.created_selection_ids.append(selection_id_1)
        
        # 두 번째 데이터
        selection_data_2 = {
            'series': [
                {
                    'study_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661',
                    'series_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953'
                }
            ]
        }
        
        selection_id_2 = self._create_selection(selection_data_2, "두 번째 Selection")
        if selection_id_2:
            self.created_selection_ids.append(selection_id_2)
        
        # 두 개 모두 포함하는 Selection
        selection_data_both = {
            'series': [
                {
                    'study_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781',
                    'series_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345'
                },
                {
                    'study_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661',
                    'series_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953'
                }
            ]
        }
        
        selection_id_both = self._create_selection(selection_data_both, "두 개 모두 포함")
        if selection_id_both:
            self.created_selection_ids.append(selection_id_both)
        
        TestPrinter.print_success(f"총 {len(self.created_selection_ids)}개 Selection 생성 완료!", indent=1)
        print()
    
    def test_retrieve_view_selection(self):
        """테스트 2: View Selection 조회"""
        TestPrinter.print_header("테스트 2: View Selection 조회")
        
        if not self.created_selection_ids:
            TestPrinter.print_warning("생성된 Selection이 없어 조회 테스트 스킵", indent=1)
            print()
            return
        
        for idx, selection_id in enumerate(self.created_selection_ids, 1):
            print(f"\n{idx}. Selection 조회: {selection_id}")
            
            response = requests.get(
                f"{TestConfig.BASE_URL}/api/v1/view-selections/{selection_id}",
                headers={"Authorization": f"Bearer {self.token}"},
                timeout=30
            )
            
            TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
            
            if response.status_code == 200:
                result = response.json()
                TestPrinter.print_success("조회 성공!", indent=1)
                TestPrinter.print_info(f"Selection ID: {result.get('selection_id')}", indent=1)
                TestPrinter.print_info(f"Series 수: {len(result.get('series', []))}", indent=1)
                TestPrinter.print_info(f"User ID: {result.get('user_id')}", indent=1)
                
                # Series 정보 출력
                for series_idx, series in enumerate(result.get('series', []), 1):
                    TestPrinter.print_info(f"Series {series_idx}:", indent=1)
                    TestPrinter.print_info(f"  Study UID: {series.get('study_uid')}", indent=2)
                    TestPrinter.print_info(f"  Series UID: {series.get('series_uid')}", indent=2)
            else:
                TestPrinter.print_error(f"조회 실패: {response.text[:200]}", indent=1)
        
        print()
    
    def test_delete_view_selection(self):
        """테스트 3: View Selection 삭제"""
        TestPrinter.print_header("테스트 3: View Selection 삭제")
        
        if not self.created_selection_ids:
            TestPrinter.print_warning("삭제할 Selection이 없음", indent=1)
            print()
            return
        
        deleted_count = 0
        for idx, selection_id in enumerate(self.created_selection_ids, 1):
            print(f"\n{idx}. Selection 삭제: {selection_id}")
            
            response = requests.delete(
                f"{TestConfig.BASE_URL}/api/v1/view-selections/{selection_id}",
                headers={"Authorization": f"Bearer {self.token}"},
                timeout=30
            )
            
            TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
            
            if response.status_code == 204:
                TestPrinter.print_success("삭제 성공!", indent=1)
                deleted_count += 1
            else:
                TestPrinter.print_error(f"삭제 실패: {response.text[:200]}", indent=1)
        
        TestPrinter.print_success(f"총 {deleted_count}/{len(self.created_selection_ids)}개 삭제 완료!", indent=1)
        
        # 삭제 완료된 것은 cleanup에서 제외
        self.created_selection_ids.clear()
        print()
    
    def _create_selection(self, selection_data: dict, description: str) -> str:
        """View Selection 생성 헬퍼"""
        print(f"\n📝 {description} 생성 중...")
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/v1/view-selections",
            json=selection_data,
            headers={"Authorization": f"Bearer {self.token}"},
            timeout=30
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code == 201:
            result = response.json()
            selection_id = result.get('selection_id')
            if selection_id:
                TestPrinter.print_success(f"생성 성공! ID: {selection_id}", indent=1)
                TestPrinter.print_info(f"Series 수: {len(selection_data['series'])}", indent=1)
                return selection_id
            else:
                TestPrinter.print_error("Selection ID 없음", indent=1)
                return None
        else:
            TestPrinter.print_error(f"생성 실패: {response.text[:200]}", indent=1)
            return None


if __name__ == '__main__':
    test = ViewSelectionE2ETest()
    test.run()

