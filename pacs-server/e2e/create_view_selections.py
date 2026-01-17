#!/usr/bin/env python3
"""
View Selection 생성 스크립트
지정된 Study/Series UID로 View Selection을 생성합니다.
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter


class CreateViewSelections(BaseE2ETest):
    """View Selection 생성"""
    
    def __init__(self):
        super().__init__()
        self.created_selection_ids = []
    
    def get_test_name(self) -> str:
        return "View Selection 생성"
    
    def run_tests(self):
        """View Selection 생성"""
        TestPrinter.print_header("View Selection 생성")
        
        # 데이터 정의
        selections = [
            {
                'name': '첫 번째 데이터',
                'series': [
                    {
                        'study_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781',
                        'series_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345'
                    }
                ]
            },
            {
                'name': '두 번째 데이터',
                'series': [
                    {
                        'study_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661',
                        'series_uid': '1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953'
                    }
                ]
            },
            {
                'name': '두 개 모두 포함',
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
        ]
        
        # 각 Selection 생성
        for idx, selection_data in enumerate(selections, 1):
            print(f"\n{idx}. {selection_data['name']} 생성 중...")
            
            response = requests.post(
                f"{TestConfig.BASE_URL}/api/v1/view-selections",
                json={'series': selection_data['series']},
                headers={"Authorization": f"Bearer {self.token}"},
                timeout=30
            )
            
            TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
            
            if response.status_code == 201:
                result = response.json()
                selection_id = result.get('selection_id')
                if selection_id:
                    self.created_selection_ids.append(selection_id)
                    TestPrinter.print_success(f"생성 성공! ID: {selection_id}", indent=1)
                    TestPrinter.print_info(f"Series 수: {len(selection_data['series'])}", indent=1)
                    
                    # Series 정보 출력
                    for series_idx, series in enumerate(selection_data['series'], 1):
                        TestPrinter.print_info(f"Series {series_idx}:", indent=1)
                        TestPrinter.print_info(f"  Study UID: {series['study_uid']}", indent=2)
                        TestPrinter.print_info(f"  Series UID: {series['series_uid']}", indent=2)
                else:
                    TestPrinter.print_error("Selection ID 없음", indent=1)
            else:
                TestPrinter.print_error(f"생성 실패: {response.text[:200]}", indent=1)
        
        # 요약
        print()
        TestPrinter.print_header("생성 요약")
        TestPrinter.print_success(f"총 {len(self.created_selection_ids)}개 View Selection 생성 완료!")
        
        if self.created_selection_ids:
            print("\n생성된 Selection ID 목록:")
            for idx, selection_id in enumerate(self.created_selection_ids, 1):
                TestPrinter.print_info(f"{idx}. {selection_id}", indent=1)
        
        print()


if __name__ == '__main__':
    test = CreateViewSelections()
    test.run()

