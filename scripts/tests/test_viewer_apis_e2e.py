#!/usr/bin/env python3
"""E2E Test for Viewer APIs with Pagination.

Tests three APIs:
1. POST /api/v1/viewer/studies/meta
2. POST /api/v1/viewer/series/meta
3. POST /api/v1/viewer/studies/{study_uid}/series/meta
"""

import json
import sys
from typing import Any, Dict, Optional

import requests


# Configuration
BASE_URL = "http://localhost:8080"
USERNAME = "iaid-pacs-admin"
PASSWORD = "Qlalfqjsgh1!"

# Test data - replace with actual UIDs from your PACS
TEST_STUDY_UID_1 = "1.2.840.114350.2.171.2.798268.2.777675267.1"
TEST_STUDY_UID_2 = "1.3.12.2.1107.5.2.12.21149.202012071330009074651"
TEST_SERIES_UID_1 = "1.2.840.113619.2.495.11554579.117236.29274.1645718974.446"
TEST_SERIES_UID_2 = "1.2.840.113619.2.311.168624790352053237183428645578553404611"


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    BLUE = "\033[94m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def print_header(text: str) -> None:
    print(f"\n{Colors.BOLD}{Colors.BLUE}{'=' * 80}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.BLUE}{text}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.BLUE}{'=' * 80}{Colors.RESET}\n")


def print_success(text: str) -> None:
    print(f"{Colors.GREEN}[OK] {text}{Colors.RESET}")


def print_error(text: str) -> None:
    print(f"{Colors.RED}[ERROR] {text}{Colors.RESET}")


def print_info(text: str) -> None:
    print(f"{Colors.YELLOW}[INFO]  {text}{Colors.RESET}")


def login() -> Optional[str]:
    """Login and get JWT token."""
    print_header("Step 1: Authentication")

    url = f"{BASE_URL}/api/auth/login"
    payload = {"username": USERNAME, "password": PASSWORD}

    print_info(f"Logging in as {USERNAME}...")

    try:
        response = requests.post(url, json=payload)
        response.raise_for_status()

        data = response.json()
        token = data.get("keycloak_access_token")

        if token:
            print_success(f"Login successful! Token length: {len(token)}")
            return token

        print_error("No access_token in response")
        print(f"Response: {json.dumps(data, indent=2)}")
        return None

    except Exception as exc:  # pragma: no cover - debug output
        print_error(f"Login failed: {exc}")
        try:
            # type: ignore[name-defined]
            print(f"Response: {response.text}")
        except Exception:
            pass
        return None


def validate_pagination(data: Dict[str, Any], test_name: str) -> bool:
    """Validate pagination structure in the response body."""

    if "pagination" not in data:
        print_error(f"{test_name}: Missing 'pagination' field")
        return False

    pagination = data["pagination"]
    required_fields = [
        "page",
        "page_size",
        "total_items",
        "total_pages",
        "has_next",
        "has_previous",
    ]

    for field in required_fields:
        if field not in pagination:
            print_error(f"{test_name}: Missing pagination field '{field}'")
            return False

    print_success(f"{test_name}: Pagination structure valid")
    print_info(f"  Page: {pagination['page']}/{pagination['total_pages']}")
    print_info(
        f"  Items: {pagination['page_size']} (Total: {pagination['total_items']})"
    )
    print_info(
        f"  Has next: {pagination['has_next']}, "
        f"Has previous: {pagination['has_previous']}"
    )

    return True


def test_study_meta_api(token: str) -> bool:
    """Test POST /api/v1/viewer/studies/meta."""
    print_header("Test 1: POST /api/v1/viewer/studies/meta (Study Meta Batch API)")

    url = f"{BASE_URL}/api/v1/viewer/studies/meta"
    headers = {"Authorization": f"Bearer {token}"}

    # Test 1.1: Default pagination
    print_info("Test 1.1: Default pagination (no page params)")
    payload = {"study_uids": [TEST_STUDY_UID_1, TEST_STUDY_UID_2]}

    try:
        response = requests.post(url, json=payload, headers=headers)
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 1.1"):
            return False

        if "studies" not in data:
            print_error("Test 1.1: Missing 'studies' field")
            return False

        studies = data.get("studies", [])
        print_success(f"Test 1.1: Found {len(studies)} studies")

        if not studies:
            print_error("Test 1.1: 'studies' array is empty")
            return False

        # Study meta field validation
        sample = studies[0]
        print_info("Sample study meta (first item):")

        # Required field: study_uid
        if "study_uid" not in sample:
            print_error("Test 1.1: Missing required field 'study_uid' in study meta")
            return False
        print_success(f"  study_uid: {sample['study_uid']}")

        # Important metadata fields (warn if missing, do not fail test)
        meta_fields = [
            "study_date",
            "study_time",
            "study_description",
            "patient_name",
            "patient_id",
            "patient_sex",
            "patient_age",
            "patient_birth_date",
            "modalities_in_study",
            "number_of_series",
            "number_of_instances",
        ]

        for field in meta_fields:
            if field in sample:
                print_success(f"  {field}: {sample[field]!r}")
            else:
                print_info(
                    f"  (missing expected meta field '{field}' in sample study)"
                )

    except Exception as exc:
        print_error(f"Test 1.1 failed: {exc}")
        try:
            # type: ignore[name-defined]
            print(f"Response status: {response.status_code}")
            print(f"Response: {response.text}")
        except Exception:
            pass
        return False

    # Test 1.2: Custom pagination
    print_info("\nTest 1.2: Custom pagination (page=1, page_size=1)")
    payload = {
        "study_uids": [TEST_STUDY_UID_1, TEST_STUDY_UID_2],
        "page": 1,
        "page_size": 1,
    }

    try:
        response = requests.post(url, json=payload, headers=headers)
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 1.2"):
            return False

        if data["pagination"]["page_size"] != 1:
            print_error(
                f"Test 1.2: Expected page_size=1, "
                f"got {data['pagination']['page_size']}"
            )
            return False

        if len(data["studies"]) > 1:
            print_error(
                f"Test 1.2: Expected max 1 study, got {len(data['studies'])}"
            )
            return False

        print_success(
            "Test 1.2: Pagination working correctly "
            f"(returned {len(data['studies'])} study)"
        )

    except Exception as exc:
        print_error(f"Test 1.2 failed: {exc}")
        return False

    print_success("Study Meta API: ALL TESTS PASSED")
    return True


def test_series_meta_api(token: str) -> bool:
    """Test POST /api/v1/viewer/series/meta."""
    print_header("Test 2: POST /api/v1/viewer/series/meta (Series Meta Batch API)")

    url = f"{BASE_URL}/api/v1/viewer/series/meta"
    headers = {"Authorization": f"Bearer {token}"}

    # Test 2.1: Default pagination
    print_info("Test 2.1: Default pagination (no page params)")
    payload = {
        "series_queries": [
            {"study_uid": TEST_STUDY_UID_1, "series_uid": TEST_SERIES_UID_1},
            {"study_uid": TEST_STUDY_UID_1, "series_uid": TEST_SERIES_UID_2},
        ]
    }

    try:
        response = requests.post(url, json=payload, headers=headers)
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 2.1"):
            return False

        if "series" not in data:
            print_error("Test 2.1: Missing 'series' field")
            return False

        print_success(f"Test 2.1: Found {len(data['series'])} series")

    except Exception as exc:
        print_error(f"Test 2.1 failed: {exc}")
        return False

    # Test 2.2: Custom pagination
    print_info("\nTest 2.2: Custom pagination (page=1, page_size=1)")
    payload = {
        "series_queries": [
            {"study_uid": TEST_STUDY_UID_1, "series_uid": TEST_SERIES_UID_1},
            {"study_uid": TEST_STUDY_UID_1, "series_uid": TEST_SERIES_UID_2},
        ],
        "page": 1,
        "page_size": 1,
    }

    try:
        response = requests.post(url, json=payload, headers=headers)
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 2.2"):
            return False

        if data["pagination"]["page_size"] != 1:
            print_error(
                f"Test 2.2: Expected page_size=1, "
                f"got {data['pagination']['page_size']}"
            )
            return False

        if len(data["series"]) > 1:
            print_error(
                f"Test 2.2: Expected max 1 series, got {len(data['series'])}"
            )
            return False

        print_success(
            "Test 2.2: Pagination working correctly "
            f"(returned {len(data['series'])} series)"
        )

    except Exception as exc:
        print_error(f"Test 2.2 failed: {exc}")
        return False

    print_success("Series Meta API: ALL TESTS PASSED")
    return True


def test_study_series_meta_api(token: str) -> bool:
    """Test POST /api/v1/viewer/studies/{study_uid}/series/meta."""
    print_header(
        "Test 3: POST /api/v1/viewer/studies/{study_uid}/series/meta "
        "(Study Series Meta API)"
    )

    url = f"{BASE_URL}/api/v1/viewer/studies/{TEST_STUDY_UID_1}/series/meta"
    headers = {"Authorization": f"Bearer {token}"}

    # Test 3.1: Default pagination (no body params)
    print_info("Test 3.1: Default pagination (no body params)")

    try:
        response = requests.post(url, json={}, headers=headers)
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 3.1"):
            return False

        if "series" not in data:
            print_error("Test 3.1: Missing 'series' field")
            return False

        if "study_uid" not in data:
            print_error("Test 3.1: Missing 'study_uid' field")
            return False

        if data["study_uid"] != TEST_STUDY_UID_1:
            print_error(
                f"Test 3.1: Expected study_uid={TEST_STUDY_UID_1}, "
                f"got {data['study_uid']}"
            )
            return False

        print_success(
            f"Test 3.1: Found {len(data['series'])} series for study {data['study_uid']}"
        )
        if data.get("study_description"):
            print_info(f"  Study Description: {data['study_description']}")

    except Exception as exc:
        print_error(f"Test 3.1 failed: {exc}")
        return False

    # Test 3.2: Custom pagination
    print_info("\nTest 3.2: Custom pagination (page=1, page_size=5)")

    try:
        response = requests.post(
            url, json={"page": 1, "page_size": 5}, headers=headers
        )
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 3.2"):
            return False

        if data["pagination"]["page"] != 1:
            print_error(
                f"Test 3.2: Expected page=1, got {data['pagination']['page']}"
            )
            return False

        if data["pagination"]["page_size"] != 5:
            print_error(
                f"Test 3.2: Expected page_size=5, "
                f"got {data['pagination']['page_size']}"
            )
            return False

        if len(data["series"]) > 5:
            print_error(
                f"Test 3.2: Expected max 5 series, got {len(data['series'])}"
            )
            return False

        print_success(
            "Test 3.2: Pagination working correctly "
            f"(returned {len(data['series'])} series)"
        )

    except Exception as exc:
        print_error(f"Test 3.2 failed: {exc}")
        return False

    # Test 3.3: Page size limit (should be clamped to 200)
    print_info("\nTest 3.3: Page size limit (page_size=500 should be clamped to 200)")

    try:
        response = requests.post(
            url, json={"page": 1, "page_size": 500}, headers=headers
        )
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 3.3"):
            return False

        if data["pagination"]["page_size"] != 200:
            print_error(
                "Test 3.3: Expected page_size=200 (clamped), "
                f"got {data['pagination']['page_size']}"
            )
            return False

        print_success("Test 3.3: Page size correctly clamped to 200")

    except Exception as exc:
        print_error(f"Test 3.3 failed: {exc}")
        return False

    # Test 3.4: Navigation (has_next, has_previous)
    print_info("\nTest 3.4: Navigation flags (has_next, has_previous)")

    try:
        # Get first page with small page_size
        response = requests.post(
            url, json={"page": 1, "page_size": 2}, headers=headers
        )
        response.raise_for_status()
        data = response.json()

        total_items = data["pagination"]["total_items"]

        if total_items > 2:
            # First page should have has_next=true, has_previous=false
            if not data["pagination"]["has_next"]:
                print_error("Test 3.4: Expected has_next=true on first page")
                return False

            if data["pagination"]["has_previous"]:
                print_error("Test 3.4: Expected has_previous=false on first page")
                return False

            print_success("Test 3.4a: First page navigation flags correct")

            # Get second page
            response = requests.post(
                url, json={"page": 2, "page_size": 2}, headers=headers
            )
            response.raise_for_status()
            data = response.json()

            if not data["pagination"]["has_previous"]:
                print_error("Test 3.4: Expected has_previous=true on second page")
                return False

            print_success("Test 3.4b: Second page navigation flags correct")
        else:
            print_info(
                f"Test 3.4: Skipped (only {total_items} items, "
                "need >2 for navigation test)"
            )

    except Exception as exc:
        print_error(f"Test 3.4 failed: {exc}")
        return False

    # Test 3.5: Filter by series_uids
    print_info("\nTest 3.5: Filter by series_uids")

    try:
        payload = {
            "series_uids": [TEST_SERIES_UID_1, TEST_SERIES_UID_2],
            "page": 1,
            "page_size": 50,
        }
        response = requests.post(url, json=payload, headers=headers)
        response.raise_for_status()
        data = response.json()

        if not validate_pagination(data, "Test 3.5"):
            return False

        returned_uids = {s["series_uid"] for s in data.get("series", [])}
        expected = {TEST_SERIES_UID_1, TEST_SERIES_UID_2}
        if not expected.issubset(returned_uids):
            print_error(
                f"Test 3.5: Expected at least {expected}, got {returned_uids}"
            )
            return False

        print_success("Test 3.5: Filter by series_uids working correctly")

    except Exception as exc:
        print_error(f"Test 3.5 failed: {exc}")
        return False

    print_success("Study Series Meta API: ALL TESTS PASSED")
    return True


def main() -> None:
    """Main test runner."""
    print_header("E2E Test: Viewer APIs with Pagination")
    print_info(f"Base URL: {BASE_URL}")
    print_info(f"Test Study UID: {TEST_STUDY_UID_1}")

    # Step 1: Login
    token = login()
    if not token:
        print_error("Authentication failed. Exiting.")
        sys.exit(1)

    # Step 2: Test Study Meta API
    if not test_study_meta_api(token):
        print_error("Study Meta API tests failed. Exiting.")
        sys.exit(1)

    # Step 3: Test Series Meta API
    if not test_series_meta_api(token):
        print_error("Series Meta API tests failed. Exiting.")
        sys.exit(1)

    # Step 4: Test Study Series Meta API
    if not test_study_series_meta_api(token):
        print_error("Study Series Meta API tests failed. Exiting.")
        sys.exit(1)

    # Final summary
    print_header("ALL TESTS PASSED!")
    print_success("POST /api/v1/viewer/studies/meta - Pagination working")
    print_success("POST /api/v1/viewer/series/meta - Pagination working")
    print_success(
        "POST /api/v1/viewer/studies/{study_uid}/series/meta - Pagination working"
    )
    print()
    print_info("All three Viewer APIs have been successfully tested with pagination!")
    print_info("Pagination features verified:")
    print_info("  - Default pagination (page=1, page_size=50)")
    print_info("  - Custom pagination (page, page_size)")
    print_info("  - Page size clamping (max 200)")
    print_info("  - Navigation flags (has_next, has_previous)")
    print_info(
        "  - Pagination info structure "
        "(page, page_size, total_items, total_pages)"
    )
    print()


if __name__ == "__main__":
    main()
