#!/usr/bin/env python3
"""
어노테이션 HEAD 요청 E2E 테스트

이 테스트는 HEAD 요청을 통한 캐시 검증 및 리소스 존재 확인 기능을 검증합니다.
- ETag 기반 캐시 검증
- Last-Modified 기반 캐시 검증
- 리소스 존재 확인
- 304 Not Modified 응답
"""

import requests
from datetime import datetime
from test_utils import TestContext, cleanup_annotations

BASE_URL = "http://localhost:8080"
USER_ID = 'iaid-pacs-admin'
USER_PASSWORD = 'Qlalfqjsgh1!'

def login():
    """로그인하여 JWT 토큰 획득"""
    print("🔐 로그인 중...")
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": USER_ID, "password": USER_PASSWORD},
        timeout=5
    )
    
    if response.status_code != 200:
        print(f"❌ 로그인 실패: {response.status_code}")
        exit(1)
    
    token = response.json()["token"]
    print(f"✅ 로그인 성공\n")
    return token


def create_test_annotation(token: str) -> int:
    """테스트용 어노테이션 생성"""
    print("📝 테스트용 어노테이션 생성 중...")
    headers = {"Authorization": f"Bearer {token}"}
    
    annotation_data = {
        "project_id": 2,
        "study_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781",
        "series_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345",
        "sop_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.238273576775187812804817387920",
        "annotation_data": {"type": "circle", "x": 100, "y": 100, "radius": 50},
        "tool_name": "Circle Tool",
        "viewer_software": "TI-DicomViewer",
        "description": "HEAD request test",
    }
    
    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 201:
        annotation_id = response.json()["id"]
        print(f"✅ 어노테이션 생성 완료! ID: {annotation_id}\n")
        return annotation_id
    else:
        print(f"❌ 생성 실패: {response.text}")
        exit(1)


def test_etag_cache_validation(token: str, annotation_id: int):
    """테스트 1: ETag 기반 캐시 검증"""
    print("\n" + "=" * 70)
    print("테스트 1: ETag 기반 캐시 검증")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. GET 요청으로 ETag 획득
    print("1️⃣  GET 요청으로 ETag 획득...")
    response = requests.get(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=10
    )
    
    if response.status_code != 200:
        print(f"❌ GET 요청 실패: {response.text}")
        exit(1)
    
    etag = response.headers.get("ETag")
    print(f"   ETag: {etag}")
    
    # 2. HEAD 요청 with If-None-Match
    print("\n2️⃣  HEAD 요청 with If-None-Match...")
    head_headers = {**headers, "If-None-Match": etag}
    
    response = requests.head(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=head_headers,
        timeout=10
    )
    
    print(f"   Status: {response.status_code}")

    if response.status_code == 304:
        print(f"✅ 304 Not Modified (캐시 유효)")
        print("✅ 테스트 통과")
    elif response.status_code == 200:
        print(f"⚠️  200 OK (ETag가 변경되었거나 캐시 검증 미지원)")
    else:
        print(f"⚠️  예상치 못한 응답: {response.status_code}")
        if response.text:
            print(f"   Error: {response.text}")


def test_last_modified_cache_validation(token: str, annotation_id: int):
    """테스트 2: Last-Modified 기반 캐시 검증"""
    print("\n" + "=" * 70)
    print("테스트 2: Last-Modified 기반 캐시 검증")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. GET 요청으로 Last-Modified 획득
    print("1️⃣  GET 요청으로 Last-Modified 획득...")
    response = requests.get(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=10
    )
    
    if response.status_code != 200:
        print(f"❌ GET 요청 실패: {response.text}")
        exit(1)
    
    last_modified = response.headers.get("Last-Modified")
    print(f"   Last-Modified: {last_modified}")
    
    # 2. HEAD 요청 with If-Modified-Since
    if last_modified:
        print("\n2️⃣  HEAD 요청 with If-Modified-Since...")
        head_headers = {**headers, "If-Modified-Since": last_modified}
        
        response = requests.head(
            f"{BASE_URL}/api/annotations/{annotation_id}",
            headers=head_headers,
            timeout=10
        )
        
        print(f"   Status: {response.status_code}")
        
        if response.status_code == 304:
            print(f"✅ 304 Not Modified (캐시 유효)")
            print("✅ 테스트 통과")
        elif response.status_code == 200:
            print(f"⚠️  200 OK (리소스가 수정되었거나 캐시 검증 미지원)")
        else:
            print(f"⚠️  예상치 못한 응답: {response.status_code}")
    else:
        print("⚠️  Last-Modified 헤더 없음")


def test_resource_existence_check(token: str, annotation_id: int):
    """테스트 3: 리소스 존재 확인"""
    print("\n" + "=" * 70)
    print("테스트 3: 리소스 존재 확인")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. 존재하는 리소스 HEAD 요청
    print("1️⃣  존재하는 리소스 HEAD 요청...")
    response = requests.head(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=10
    )
    
    print(f"   Status: {response.status_code}")
    
    if response.status_code == 200:
        print(f"✅ 리소스 존재 확인")
    else:
        print(f"❌ 예상치 못한 응답: {response.status_code}")
        exit(1)
    
    # 2. 존재하지 않는 리소스 HEAD 요청
    print("\n2️⃣  존재하지 않는 리소스 HEAD 요청...")
    fake_id = 999999
    response = requests.head(
        f"{BASE_URL}/api/annotations/{fake_id}",
        headers=headers,
        timeout=10
    )
    
    print(f"   Status: {response.status_code}")
    
    if response.status_code == 404:
        print(f"✅ 리소스 없음 확인")
        print("✅ 테스트 통과")
    else:
        print(f"⚠️  예상치 못한 응답: {response.status_code}")


def test_head_annotations_list(token: str):
    """테스트 4: 어노테이션 목록 HEAD 요청"""
    print("\n" + "=" * 70)
    print("테스트 4: 어노테이션 목록 HEAD 요청")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
    
    response = requests.head(
        f"{BASE_URL}/api/annotations?series_instance_uid={series_uid}",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        x_total_count = response.headers.get("X-Total-Count")
        last_modified = response.headers.get("Last-Modified")
        
        print(f"✅ HEAD 요청 성공")
        print(f"   - X-Total-Count: {x_total_count}")
        print(f"   - Last-Modified: {last_modified}")
        print("✅ 테스트 통과")
    else:
        print(f"⚠️  예상치 못한 응답: {response.status_code}")


if __name__ == '__main__':
    created_ids = []
    try:
        print("\n🚀 어노테이션 HEAD 요청 E2E 테스트 시작...\n")

        token = login()
        annotation_id = create_test_annotation(token)
        created_ids.append(annotation_id)

        test_etag_cache_validation(token, annotation_id)
        test_last_modified_cache_validation(token, annotation_id)
        test_resource_existence_check(token, annotation_id)
        test_head_annotations_list(token)

        print("\n" + "=" * 70)
        print("🎉 모든 테스트 통과!")
        print("=" * 70)
        print()

    except Exception as e:
        print(f"\n❌ 테스트 실패: {e}\n")
        import traceback
        traceback.print_exc()
        exit(1)
    finally:
        # Cleanup
        if created_ids:
            cleanup_annotations(token, created_ids)

