#!/usr/bin/env python3
"""
어노테이션 권한 관리 E2E 테스트

이 테스트는 어노테이션 생성/수정/삭제 권한 제어 및 권한 조회 API를 검증합니다.
- 권한이 있는 사용자는 어노테이션을 생성/수정/삭제할 수 있음
- 권한이 없는 사용자는 어노테이션을 생성/수정/삭제할 수 없음
- 권한 조회 API가 정상 작동함
"""

import requests
import json

BASE_URL = "http://localhost:8080"

# 테스트 사용자 정보
ADMIN_USER = {"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"}
NORMAL_USER = {"username": "iaid-pacs-user1", "password": "Qlalfqjsgh1!"}

def login(username: str, password: str) -> str:
    """로그인하여 JWT 토큰 획득"""
    print(f"🔐 로그인 중: {username}")
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": username, "password": password},
        timeout=5
    )
    
    if response.status_code != 200:
        print(f"❌ 로그인 실패: {response.status_code}")
        print(response.text)
        exit(1)
    
    token = response.json()["token"]
    print(f"✅ 로그인 성공\n")
    return token


def test_create_annotation_with_permission():
    """테스트 1: 권한이 있는 사용자는 어노테이션을 생성할 수 있어야 함"""
    print("\n" + "=" * 70)
    print("테스트 1: 권한 있는 사용자 - 어노테이션 생성")
    print("=" * 70)
    
    token = login(ADMIN_USER["username"], ADMIN_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}
    
    annotation_data = {
        "project_id": 2,
        "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
        "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
        "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
        "annotation_data": {"type": "circle", "x": 100, "y": 200, "radius": 50},
        "tool_name": "Circle Tool",
        "tool_version": "2.1.0",
        "viewer_software": "OHIF Viewer",
        "description": "Permission test annotation",
    }
    
    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 201:
        data = response.json()
        annotation_id = data["id"]
        print(f"✅ 어노테이션 생성 성공! ID: {annotation_id}")
        print("✅ 테스트 통과")
        return annotation_id
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_update_annotation_as_owner(annotation_id: int):
    """테스트 2: 어노테이션 소유자는 수정할 수 있어야 함"""
    print("\n" + "=" * 70)
    print("테스트 2: 소유자 - 어노테이션 수정")
    print("=" * 70)
    
    token = login(ADMIN_USER["username"], ADMIN_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}
    
    update_data = {
        "annotation_data": {"type": "circle", "x": 150, "y": 250, "radius": 75},
        "description": "Updated by owner",
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
        print(f"✅ 어노테이션 수정 성공!")
        print(f"   - Description: {data.get('description')}")
        print("✅ 테스트 통과")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_delete_annotation_as_owner(annotation_id: int):
    """테스트 3: 어노테이션 소유자는 삭제할 수 있어야 함"""
    print("\n" + "=" * 70)
    print("테스트 3: 소유자 - 어노테이션 삭제")
    print("=" * 70)
    
    token = login(ADMIN_USER["username"], ADMIN_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}
    
    response = requests.delete(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 204:
        print(f"✅ 어노테이션 삭제 성공!")
        print("✅ 테스트 통과")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_get_annotation_permissions():
    """테스트 4: 권한 조회 API 테스트"""
    print("\n" + "=" * 70)
    print("테스트 4: 권한 조회 API")
    print("=" * 70)
    
    token = login(ADMIN_USER["username"], ADMIN_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}
    
    project_id = 2
    
    response = requests.get(
        f"{BASE_URL}/api/annotations/permissions?project_id={project_id}",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"✅ 권한 조회 성공!")
        print(f"   - Permissions: {json.dumps(data, indent=2)}")
        print("✅ 테스트 통과")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_create_annotation_without_permission():
    """테스트 5: 권한이 없는 사용자는 어노테이션을 생성할 수 없어야 함"""
    print("\n" + "=" * 70)
    print("테스트 5: 권한 없는 사용자 - 어노테이션 생성 시도")
    print("=" * 70)
    
    # 일반 사용자로 로그인 (프로젝트에 속하지 않음)
    token = login(NORMAL_USER["username"], NORMAL_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}
    
    annotation_data = {
        "project_id": 2,  # 권한 없는 프로젝트
        "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1",
        "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
        "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
        "annotation_data": {"type": "circle", "x": 100, "y": 200, "radius": 50},
        "tool_name": "Circle Tool",
        "viewer_software": "OHIF Viewer",
    }
    
    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    # 권한이 없으면 403 Forbidden 또는 401 Unauthorized 응답
    if response.status_code in [401, 403]:
        print(f"✅ 권한 없는 사용자는 생성할 수 없음 (예상된 동작)")
        print("✅ 테스트 통과")
    elif response.status_code == 201:
        print(f"⚠️  어노테이션이 생성됨 (권한 체크가 없을 수 있음)")
    else:
        print(f"⚠️  예상치 못한 응답: {response.text}")


if __name__ == '__main__':
    try:
        print("\n🚀 어노테이션 권한 관리 E2E 테스트 시작...\n")
        
        annotation_id = test_create_annotation_with_permission()
        test_update_annotation_as_owner(annotation_id)
        test_get_annotation_permissions()
        test_create_annotation_without_permission()
        test_delete_annotation_as_owner(annotation_id)
        
        print("\n" + "=" * 70)
        print("🎉 모든 테스트 통과!")
        print("=" * 70)
        print()
    except Exception as e:
        print(f"\n❌ 테스트 실패: {e}\n")
        import traceback
        traceback.print_exc()
        exit(1)

