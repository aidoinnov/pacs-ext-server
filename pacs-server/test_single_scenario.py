#!/usr/bin/env python3
"""단일 시나리오만 빠르게 테스트"""

import sys
sys.path.insert(0, '.')

from test_dicom_gateway_report_status_filter_e2e import (
    scenario_1_single_status_filter,
    test_health,
    print_test,
    print_success,
    print_error
)

if __name__ == '__main__':
    print("="*60)
    print("🧪 단일 시나리오 테스트: 단일 status 필터링")
    print("="*60)
    
    if not test_health():
        print_error("Server is not available")
        sys.exit(1)
    
    result = scenario_1_single_status_filter()
    
    if result:
        print_success("시나리오 1 통과!")
        sys.exit(0)
    else:
        print_error("시나리오 1 실패")
        sys.exit(1)



