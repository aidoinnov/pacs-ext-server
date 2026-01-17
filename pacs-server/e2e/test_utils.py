#!/usr/bin/env python3
"""
E2E 테스트 공통 유틸리티 함수
"""

import requests
from typing import List, Optional, Tuple
import time

BASE_URL = "http://localhost:8080"


def login(username: str, password: str) -> str:
    """로그인하여 JWT 토큰 획득"""
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": username, "password": password},
        timeout=5
    )
    
    if response.status_code != 200:
        raise Exception(f"로그인 실패: {response.status_code} - {response.text}")
    
    return response.json()["token"]


def delete_annotation(token: str, annotation_id: int) -> bool:
    """어노테이션 삭제"""
    headers = {"Authorization": f"Bearer {token}"}

    response = requests.delete(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=10
    )

    # 204 No Content 또는 200 OK 모두 성공으로 처리
    return response.status_code in [200, 204]


def cleanup_annotations(token: str, annotation_ids: List[int], verbose: bool = True):
    """생성된 어노테이션들을 정리"""
    if not annotation_ids:
        return
    
    if verbose:
        print(f"\n🧹 Cleanup: {len(annotation_ids)}개 어노테이션 삭제 중...")
    
    success_count = 0
    fail_count = 0
    
    for ann_id in annotation_ids:
        try:
            if delete_annotation(token, ann_id):
                success_count += 1
                if verbose:
                    print(f"   ✓ Deleted annotation ID: {ann_id}")
            else:
                fail_count += 1
                if verbose:
                    print(f"   ✗ Failed to delete annotation ID: {ann_id}")
        except Exception as e:
            fail_count += 1
            if verbose:
                print(f"   ✗ Error deleting annotation ID {ann_id}: {e}")
    
    if verbose:
        print(f"✅ Cleanup 완료: {success_count}개 삭제 성공, {fail_count}개 실패\n")


def get_annotation(token: str, annotation_id: int) -> Optional[dict]:
    """어노테이션 조회"""
    headers = {"Authorization": f"Bearer {token}"}
    
    response = requests.get(
        f"{BASE_URL}/api/annotations/{annotation_id}",
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 200:
        return response.json()
    return None


def create_annotation(token: str, annotation_data: dict) -> Optional[int]:
    """어노테이션 생성"""
    headers = {"Authorization": f"Bearer {token}"}

    response = requests.post(
        f"{BASE_URL}/api/annotations",
        json=annotation_data,
        headers=headers,
        timeout=10
    )

    if response.status_code == 201:
        return response.json()["id"]
    return None


def create_user(username: str, email: str, password: str, full_name: str) -> Optional[Tuple[int, str]]:
    """사용자 생성 (회원가입)

    Returns:
        (user_id, username) 또는 None
    """
    signup_data = {
        "username": username,
        "email": email,
        "password": password,
        "full_name": full_name,
        "organization": "Test Organization",
        "department": "Test Department",
        "phone": "010-0000-0000"
    }

    response = requests.post(
        f"{BASE_URL}/api/auth/signup",
        json=signup_data,
        timeout=10
    )

    if response.status_code == 201:
        data = response.json()
        return (data["user_id"], data["username"])
    return None


def add_user_to_project(admin_token: str, project_id: int, user_id: int, role_id: int = 3) -> bool:
    """프로젝트에 사용자 추가

    Args:
        admin_token: 관리자 토큰
        project_id: 프로젝트 ID
        user_id: 사용자 ID
        role_id: 역할 ID (기본값: 3 - 일반 사용자)

    Returns:
        성공 여부
    """
    headers = {"Authorization": f"Bearer {admin_token}"}

    request_data = {
        "user_id": user_id,
        "role_id": role_id
    }

    response = requests.post(
        f"{BASE_URL}/api/projects/{project_id}/members",
        json=request_data,
        headers=headers,
        timeout=10
    )

    return response.status_code == 200


def approve_user(admin_token: str, user_id: int) -> bool:
    """사용자 승인 (계정 활성화)

    Args:
        admin_token: 관리자 토큰
        user_id: 승인할 사용자 ID

    Returns:
        성공 여부
    """
    headers = {"Authorization": f"Bearer {admin_token}"}

    request_data = {
        "user_id": user_id
    }

    response = requests.post(
        f"{BASE_URL}/api/auth/admin/users/approve",
        json=request_data,
        headers=headers,
        timeout=10
    )

    return response.status_code == 200


def delete_user(admin_token: str, user_id: int) -> bool:
    """사용자 삭제

    Args:
        admin_token: 관리자 토큰
        user_id: 삭제할 사용자 ID

    Returns:
        성공 여부
    """
    headers = {"Authorization": f"Bearer {admin_token}"}

    response = requests.delete(
        f"{BASE_URL}/api/users/{user_id}",
        headers=headers,
        timeout=10
    )

    return response.status_code in [200, 204]


class TestContext:
    """테스트 컨텍스트 관리 클래스"""
    
    def __init__(self, username: str, password: str):
        self.username = username
        self.password = password
        self.token = None
        self.created_annotation_ids = []
    
    def __enter__(self):
        """컨텍스트 진입 시 로그인"""
        self.token = login(self.username, self.password)
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """컨텍스트 종료 시 cleanup"""
        if self.created_annotation_ids:
            cleanup_annotations(self.token, self.created_annotation_ids, verbose=True)
    
    def track_annotation(self, annotation_id: int):
        """생성된 어노테이션 ID 추적"""
        if annotation_id:
            self.created_annotation_ids.append(annotation_id)
    
    def create_and_track(self, annotation_data: dict) -> Optional[int]:
        """어노테이션 생성 및 추적"""
        ann_id = create_annotation(self.token, annotation_data)
        if ann_id:
            self.track_annotation(ann_id)
        return ann_id

