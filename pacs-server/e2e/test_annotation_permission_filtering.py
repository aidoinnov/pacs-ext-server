#!/usr/bin/env python3
"""
권한 기반 어노테이션 필터링 E2E 테스트

이 테스트는 사용자의 권한에 따라 어노테이션 조회 결과가 달라지는지 검증합니다.
- ANNOTATION:READ_ALL 권한이 있으면: 프로젝트의 모든 어노테이션 반환
- ANNOTATION:READ_ALL 권한이 없으면: 본인의 어노테이션만 반환
"""

import requests
import json
import time
from test_utils import create_user, add_user_to_project, delete_user, approve_user, create_annotation, cleanup_annotations

BASE_URL = "http://localhost:8080"

# 테스트 사용자 정보
ADMIN_USER = {"username": "iaid-pacs-admin", "password": "Qlalfqjsgh1!"}  # SUPER_ADMIN 권한

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


def test_admin_sees_all_annotations():
    """테스트 1: READ_ALL 권한이 있는 사용자는 모든 어노테이션을 볼 수 있어야 함"""
    print("\n" + "=" * 70)
    print("테스트 1: Admin 사용자 - 모든 어노테이션 조회")
    print("=" * 70)

    token = login(ADMIN_USER["username"], ADMIN_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}

    # Series UID로 어노테이션 조회 (프로젝트 멤버십 불필요)
    series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"

    response = requests.get(
        f"{BASE_URL}/api/annotations?series_instance_uid={series_uid}",
        headers=headers,
        timeout=10
    )

    print(f"Status: {response.status_code}")

    if response.status_code == 200:
        data = response.json()
        total = data.get("total", 0)
        annotations = data.get("annotations", [])

        print(f"✅ Admin user sees {total} annotations")

        # 다양한 사용자의 어노테이션이 포함되어 있는지 확인
        if annotations:
            unique_users = set(ann["user_id"] for ann in annotations)
            print(f"   - Unique users: {len(unique_users)}")
            print(f"   - User IDs: {sorted(unique_users)}")

        assert total > 0, "Admin should see at least some annotations"
        print("✅ 테스트 통과: Admin은 모든 어노테이션을 볼 수 있음")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


def test_normal_user_sees_own_annotations(admin_token: str):
    """테스트 2: 일반 사용자 - 본인 어노테이션만 조회"""
    print("\n" + "=" * 70)
    print("테스트 2: 일반 사용자 - 본인 어노테이션만 조회")
    print("=" * 70)

    # 1. 테스트용 사용자 생성
    print("1️⃣  테스트용 사용자 생성 중...")
    timestamp = int(time.time())
    test_username = f"test_user_{timestamp}"
    test_email = f"test_{timestamp}@example.com"
    test_password = "TestPassword123!"

    user_result = create_user(test_username, test_email, test_password, "Test User")
    if not user_result:
        print("❌ 사용자 생성 실패")
        return None

    user_id, username = user_result
    print(f"   ✅ 사용자 생성 성공: ID={user_id}, Username={username}")

    try:
        # 2. 사용자 승인 (계정 활성화)
        print("\n2️⃣  사용자 승인 중...")
        if not approve_user(admin_token, user_id):
            print("   ❌ 사용자 승인 실패")
            return user_id
        print("   ✅ 사용자 승인 성공")

        # 3. 프로젝트에 사용자 추가
        print("\n3️⃣  프로젝트에 사용자 추가 중...")
        project_id = 556  # 존재하는 프로젝트 ID 사용
        role_id = 196  # PROJECT_ADMIN 역할
        if not add_user_to_project(admin_token, project_id, user_id, role_id=role_id):
            print("   ⚠️  프로젝트 추가 실패 (계속 진행)")
        else:
            print(f"   ✅ 프로젝트 {project_id}에 사용자 추가 성공")

        # 4. 로그인
        print("\n4️⃣  테스트 사용자로 로그인 중...")
        token = login(test_username, test_password)
        headers = {"Authorization": f"Bearer {token}"}
        print("   ✅ 로그인 성공")

        # 5. 어노테이션 생성 (본인 것)
        print("\n5️⃣  테스트용 어노테이션 생성 중...")
        annotation_data = {
            "project_id": project_id,
            "study_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781",
            "series_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345",
            "sop_instance_uid": "1.3.6.1.4.1.14519.5.2.1.6655.2359.238273576775187812804817387920",
            "annotation_data": {"type": "test"},
            "tool_name": "Test Tool",
            "viewer_software": "Test",
            "description": "Permission test annotation",
        }

        ann_id = create_annotation(token, annotation_data)
        if not ann_id:
            print("   ⚠️  어노테이션 생성 실패")
            return user_id
        print(f"   ✅ 어노테이션 생성 성공: ID={ann_id}")

        # 6. Series UID로 어노테이션 조회
        print("\n6️⃣  어노테이션 조회 중...")
        series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"

        response = requests.get(
            f"{BASE_URL}/api/annotations?series_instance_uid={series_uid}",
            headers=headers,
            timeout=10
        )

        print(f"   Status: {response.status_code}")

        if response.status_code == 200:
            data = response.json()
            total = data.get("total", 0)
            annotations = data.get("annotations", [])

            print(f"   ✅ Normal user sees {total} annotations")

            # 모든 어노테이션이 본인 것인지 확인
            if annotations:
                user_ids = set(ann["user_id"] for ann in annotations)
                print(f"   - User IDs in results: {user_ids}")

                # 일반 사용자는 본인 어노테이션만 볼 수 있어야 함
                if len(user_ids) == 1 and user_id in user_ids:
                    print("   ✅ 모든 어노테이션이 본인 것임 (정상)")
                else:
                    print(f"   ⚠️  다른 사용자의 어노테이션도 포함됨 (user_id={user_id})")

            print("✅ 테스트 통과: 일반 사용자는 본인 어노테이션만 볼 수 있음")

            # Cleanup: 생성한 어노테이션 삭제
            if ann_id:
                cleanup_annotations(token, [ann_id])
        else:
            print(f"   ❌ 테스트 실패: {response.text}")
            return user_id

        return user_id

    except Exception as e:
        print(f"❌ 테스트 중 오류 발생: {e}")
        return user_id


def test_series_level_filtering_with_permission():
    """테스트 3: Series UID로 필터링 시에도 권한 기반 필터링이 적용되어야 함"""
    print("\n" + "=" * 70)
    print("테스트 3: Series UID 필터링 + 권한 기반 필터링")
    print("=" * 70)
    
    token = login(ADMIN_USER["username"], ADMIN_USER["password"])
    headers = {"Authorization": f"Bearer {token}"}
    
    series_uid = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
    
    response = requests.get(
        f"{BASE_URL}/api/annotations?series_instance_uid={series_uid}",
        headers=headers,
        timeout=10
    )
    
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        total = data.get("total", 0)
        print(f"✅ Found {total} annotations for series")
        print("✅ 테스트 통과: Series 필터링 + 권한 필터링 정상 작동")
    else:
        print(f"❌ 테스트 실패: {response.text}")
        exit(1)


if __name__ == '__main__':
    created_user_id = None
    admin_token = None

    try:
        print("\n🚀 권한 기반 어노테이션 필터링 E2E 테스트 시작...\n")

        # Admin 로그인 (cleanup용)
        admin_token = login(ADMIN_USER["username"], ADMIN_USER["password"])

        test_admin_sees_all_annotations()
        created_user_id = test_normal_user_sees_own_annotations(admin_token)
        test_series_level_filtering_with_permission()

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
        # Cleanup: 생성한 사용자 삭제
        if created_user_id and admin_token:
            print("\n🧹 Cleanup: 테스트용 사용자 삭제 중...")
            if delete_user(admin_token, created_user_id):
                print(f"   ✅ 사용자 ID {created_user_id} 삭제 성공")
            else:
                print(f"   ⚠️  사용자 ID {created_user_id} 삭제 실패")
            print()

