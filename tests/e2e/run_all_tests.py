#!/usr/bin/env python3
"""
전체 E2E 및 성능 테스트 실행
"""
import os
import sys
import subprocess
import logging
import json
from datetime import datetime
from pathlib import Path

# 로깅 설정
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def run_pytest(test_file: str, output_dir: str) -> bool:
    """pytest 실행"""
    logger.info(f"Running {test_file}...")
    
    result_file = os.path.join(output_dir, f"{Path(test_file).stem}_result.json")
    
    cmd = [
        "pytest",
        test_file,
        "-v",
        "-s",
        "--tb=short",
        f"--json-report",
        f"--json-report-file={result_file}"
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode == 0:
            logger.info(f"✓ {test_file} passed")
            return True
        else:
            logger.error(f"✗ {test_file} failed")
            logger.error(result.stdout)
            logger.error(result.stderr)
            return False
    except Exception as e:
        logger.error(f"Error running {test_file}: {e}")
        return False


def main():
    """메인 함수"""
    logger.info("="*80)
    logger.info("PACS Server E2E and Performance Tests")
    logger.info("="*80)
    
    # 출력 디렉토리 생성
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = f"test_results_{timestamp}"
    os.makedirs(output_dir, exist_ok=True)
    
    logger.info(f"Test results will be saved to: {output_dir}")
    
    # 테스트 파일 목록
    test_files = [
        "test_01_auth.py",
        "test_02_project.py",
        "test_03_annotation.py",
        "test_04_snapshot.py",
        "test_11_lesion_assignment.py",
        "test_12_timepoint_with_studies.py",
        "test_performance_01_concurrent.py",
        "test_performance_02_bulk_data.py",
    ]
    
    # 환경 변수 확인
    required_env_vars = ["TEST_BASE_URL"]
    missing_vars = [var for var in required_env_vars if not os.getenv(var)]
    
    if missing_vars:
        logger.warning(f"Missing environment variables: {', '.join(missing_vars)}")
        logger.info("Using default values from config.py")
    
    # 테스트 실행
    results = {}
    passed = 0
    failed = 0
    
    for test_file in test_files:
        if os.path.exists(test_file):
            success = run_pytest(test_file, output_dir)
            results[test_file] = "PASSED" if success else "FAILED"
            
            if success:
                passed += 1
            else:
                failed += 1
        else:
            logger.warning(f"Test file not found: {test_file}")
            results[test_file] = "NOT_FOUND"
    
    # 결과 요약
    logger.info("\n" + "="*80)
    logger.info("Test Summary")
    logger.info("="*80)
    
    for test_file, status in results.items():
        status_symbol = "✓" if status == "PASSED" else "✗"
        logger.info(f"{status_symbol} {test_file}: {status}")
    
    logger.info(f"\nTotal: {len(test_files)}, Passed: {passed}, Failed: {failed}")
    
    # 결과 저장
    summary_file = os.path.join(output_dir, "test_summary.json")
    with open(summary_file, 'w') as f:
        json.dump({
            "timestamp": timestamp,
            "total": len(test_files),
            "passed": passed,
            "failed": failed,
            "results": results
        }, f, indent=2)
    
    logger.info(f"\nTest summary saved to: {summary_file}")
    
    # 리포트 생성 (성능 테스트 결과가 있는 경우)
    try:
        from generate_report import PerformanceReportGenerator
        from utils.performance_metrics import MetricsCollector
        
        # 여기서는 샘플 데이터로 리포트 생성
        # 실제로는 테스트 실행 중 수집된 메트릭을 사용해야 함
        logger.info("\nGenerating performance report...")
        generator = PerformanceReportGenerator(output_dir=output_dir)
        # generator.generate_report(metrics_data)
        
    except Exception as e:
        logger.warning(f"Could not generate performance report: {e}")
    
    # 종료 코드
    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()

