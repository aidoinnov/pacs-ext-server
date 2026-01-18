#!/usr/bin/env python3
"""
RECIST Lesion 시나리오 테스트 실행 스크립트
"""

import sys
import subprocess
import argparse
from pathlib import Path


def print_header():
    """헤더 출력"""
    print("=" * 60)
    print("RECIST Lesion Scenario Test Runner")
    print("=" * 60)
    print(f"Base URL: http://localhost:8080")
    print(f"Admin Email: admin@example.com")
    print("=" * 60)
    print()


def run_tests(verbose: bool = False, test_name: str = None):
    """테스트 실행"""
    cmd = ["pytest", "test_08_recist_scenario.py"]
    
    if verbose:
        cmd.extend(["-v", "-s"])
    else:
        cmd.append("-v")
    
    if test_name:
        cmd.append(f"-k {test_name}")
    
    # 실행
    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description="Run RECIST Lesion scenario tests")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--test", "-t", help="Run specific test")
    
    args = parser.parse_args()
    
    print_header()
    
    exit_code = run_tests(verbose=args.verbose, test_name=args.test)
    
    sys.exit(exit_code)


if __name__ == "__main__":
    main()

