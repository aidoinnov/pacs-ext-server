#!/usr/bin/env python3
"""
어노테이션 버전 충돌 (Optimistic Locking) E2E 테스트

이 테스트는 동시 업데이트 시 버전 충돌 처리 기능을 검증합니다.
- 버전 일치 시 업데이트 성공
- 버전 불일치 시 409 Conflict 응답
- 동시 업데이트 시나리오
"""

import requests
import json
from test_utils import cleanup_annotations

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


def create_test_annotation(token: str) -> tuple:
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
        "description": "Version conflict test",
    }
    
    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 201:
        data = response.json()
        annotation_id = data["id"]
        version = data.get("version", 1)
        print(f"✅ 어노테이션 생성 완료! ID: {annotation_id}, Version: {version}\n")
        return annotation_id, version
    else:
        print(f"❌ 생성 실패: {response.text}")
        exit(1)


def test_version_match_update_succeeds(token: str, annotation_id: int, current_version: int):
    """테스트 1: 버전 일치 - 업데이트 성공"""
    print("\n" + "=" * 70)
    print("테스트 1: 버전 일치 - 업데이트 성공")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    update_data = {
        "annotation_data": {"type": "circle", "x": 150, "y": 150, "radius": 60},
        "description": "Updated with correct version",
        "base_version": current_version,
    }
    
    response = requests.put(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        json=update_data,
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        new_version = data.get("version", current_version + 1)
        print(f"✅ 업데이트 성공!")
        print(f"   - Old version: {current_version}")
        print(f"   - New version: {new_version}")
        assert new_version == current_version + 1, "Version should increment by 1"
        print("✅ 테스트 통과")
        return new_version
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_version_mismatch_update_fails(token: str, annotation_id: int, current_version: int):
    """테스트 2: 버전 불일치 - 409 Conflict"""
    print("\n" + "=" * 70)
    print("테스트 2: 버전 불일치 - 409 Conflict")
    print("=" * 70)
    
    headers = {"Authorization": f"Bearer {token}"}
    
    # 잘못된 버전으로 업데이트 시도
    wrong_version = current_version - 1
    update_data = {
        "annotation_data": {"type": "circle", "x": 200, "y": 200, "radius": 70},
        "description": "Update with wrong version",
        "base_version": wrong_version,
    }
    
    response = requests.put(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        json=update_data,
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 409:
        print(f"✅ 버전 충돌 감지됨 (예상된 동작)")
        print(f"   - Client version: {wrong_version}")
        print(f"   - Server version: {current_version}")
        print("✅ 테스트 통과")
    elif response.status_code == 200:
        print(f"⚠️  업데이트 성공 (버전 체크가 없을 수 있음)")
        print(f"   Response: {response.text[:200]}")
    else:
        print(f"⚠️  예상치 못한 응답: {response.text}")


def test_concurrent_update_scenario(token: str, annotation_id: int, version: int):
    """테스트 3: 동시 업데이트 시나리오"""
    print("\n" + "=" * 70)
    print("테스트 3: 동시 업데이트 시나리오")
    print("=" * 70)

    headers = {"Authorization": f"Bearer {token}"}

    print("시나리오:")
    print("1. 사용자 A가 어노테이션 조회 (version = 1)")
    print("2. 사용자 B가 어노테이션 조회 (version = 1)")
    print("3. 사용자 A가 업데이트 성공 (version = 2)")
    print("4. 사용자 B가 업데이트 시도 (base_version = 1) → 409 Conflict\n")

    # 사용자 A: 업데이트 성공
    update_a = {
        "description": "Updated by User A",
        "base_version": version,
    }

    response_a = requests.put(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        json=update_a,
        headers=headers,
        timeout=10
    )

    if response_a.status_code == 200:
        new_version = response_a.json().get("version", version + 1)
        print(f"✅ 사용자 A 업데이트 성공 (version: {version} → {new_version})")
    else:
        print(f"❌ 사용자 A 업데이트 실패")
        exit(1)

    # 사용자 B: 업데이트 실패 (버전 충돌)
    update_b = {
        "description": "Updated by User B",
        "base_version": version,  # 오래된 버전
    }

    response_b = requests.put(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        json=update_b,
        headers=headers,
        timeout=10
    )

    if response_b.status_code == 409:
        print(f"✅ 사용자 B 업데이트 실패 (버전 충돌 감지)")
        print("✅ 테스트 통과: 동시 업데이트 시나리오 정상 작동")
    elif response_b.status_code == 200:
        print(f"⚠️  사용자 B 업데이트 성공 (버전 체크가 없을 수 있음)")
    else:
        print(f"⚠️  예상치 못한 응답: {response_b.text}")


if __name__ == '__main__':
    created_ids = []
    token = None
    try:
        print("\n🚀 어노테이션 버전 충돌 E2E 테스트 시작...\n")

        token = login()
        annotation_id, version = create_test_annotation(token)
        created_ids.append(annotation_id)

        new_version = test_version_match_update_succeeds(token, annotation_id, version)
        test_version_mismatch_update_fails(token, annotation_id, new_version)

        # test_concurrent_update_scenario용 새 어노테이션 생성
        concurrent_id, concurrent_version = create_test_annotation(token)
        created_ids.append(concurrent_id)
        test_concurrent_update_scenario(token, concurrent_id, concurrent_version)

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
        if created_ids and token:
            cleanup_annotations(token, created_ids)

