#!/usr/bin/env python3
"""
RECIST Lesion E2E 테스트 실행 스크립트

Usage:
    python run_recist_lesion.py
    python run_recist_lesion.py --verbose
    python run_recist_lesion.py --test test_01_create_target_lesion
"""

import sys
import os
import subprocess
import argparse
from pathlib import Path

# 현재 디렉토리를 Python 경로에 추가
sys.path.insert(0, str(Path(__file__).parent))


def main():
    parser = argparse.ArgumentParser(description="RECIST Lesion E2E Test Runner")
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Verbose output"
    )
    parser.add_argument(
        "--test", "-t",
        type=str,
        help="Run specific test (e.g., test_01_create_target_lesion)"
    )
    parser.add_argument(
        "--base-url",
        type=str,
        default=os.getenv("BASE_URL", "http://localhost:8080"),
        help="Base URL of the API server"
    )
    parser.add_argument(
        "--admin-email",
        type=str,
        default=os.getenv("ADMIN_EMAIL", "admin@example.com"),
        help="Admin email for authentication"
    )
    parser.add_argument(
        "--admin-password",
        type=str,
        default=os.getenv("ADMIN_PASSWORD", "admin123"),
        help="Admin password for authentication"
    )

    args = parser.parse_args()

    # 환경 변수 설정
    os.environ["BASE_URL"] = args.base_url
    os.environ["ADMIN_EMAIL"] = args.admin_email
    os.environ["ADMIN_PASSWORD"] = args.admin_password

    print("=" * 60)
    print("RECIST Lesion E2E Test Runner")
    print("=" * 60)
    print(f"Base URL: {args.base_url}")
    print(f"Admin Email: {args.admin_email}")
    print("=" * 60)
    print()

    # pytest 명령어 구성
    cmd = ["pytest", "test_07_recist_lesion.py"]

    if args.verbose:
        cmd.extend(["-v", "-s"])
    else:
        cmd.append("-v")

    if args.test:
        cmd.extend(["-k", args.test])

    cmd.append("--tb=short")

    # pytest 실행
    try:
        result = subprocess.run(cmd, check=False)
        sys.exit(result.returncode)
    except KeyboardInterrupt:
        print("\n\nTest interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\nError running tests: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

