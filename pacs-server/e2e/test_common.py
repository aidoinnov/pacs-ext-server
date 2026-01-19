#!/usr/bin/env python3
"""
E2E 테스트 공통 유틸리티
모든 E2E 테스트에서 사용하는 공통 함수들
"""

import requests
import time
import uuid
from typing import Optional, Dict, Any
from datetime import date, timedelta

BASE_URL = "http://localhost:8080"

# 기본 관리자 계정 (서버에 이미 존재한다고 가정)
DEFAULT_ADMIN_USERNAME = "iaid-pacs-admin"
DEFAULT_ADMIN_PASSWORD = "Qlalfqjsgh1!"


def get_headers(token: Optional[str] = None) -> Dict[str, str]:
    """요청 헤더 생성"""
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers


def login(username: str, password: str) -> Optional[str]:
    """로그인하여 JWT 토큰 획득"""
    try:
        response = requests.post(
            f"{BASE_URL}/api/auth/login",
            json={"username": username, "password": password},
            headers=get_headers(),
            timeout=5
        )
        if response.status_code == 200:
            result = response.json()
            return result.get("token") or result.get("access_token")
        return None
    except Exception:
        return None


def get_admin_token() -> Optional[str]:
    """기본 관리자 계정으로 로그인"""
    return login(DEFAULT_ADMIN_USERNAME, DEFAULT_ADMIN_PASSWORD)


def create_test_user(username_prefix: str = "testuser") -> Optional[Dict[str, Any]]:
    """테스트 사용자 생성 및 로그인
    
    Returns:
        {
            "user_id": int,
            "username": str,
            "password": str,
            "token": str
        }
    """
    timestamp = int(time.time() * 1000)
    username = f"{username_prefix}_{timestamp}"
    email = f"{username}@example.com"
    password = "TestPassword123!"
    
    user_data = {
        "username": username,
        "email": email,
        "password": password,
        "full_name": f"Test User {timestamp}"
    }
    
    try:
        # 1. 회원가입
        response = requests.post(
            f"{BASE_URL}/api/auth/signup",
            json=user_data,
            headers=get_headers(),
            timeout=5
        )
        
        if response.status_code not in [200, 201]:
            print(f"❌ User creation failed: {response.status_code} - {response.text}")
            return None
        
        signup_result = response.json()
        user_id = signup_result.get("user_id") or signup_result.get("id")
        
        if not user_id:
            print(f"❌ User ID not found in signup response")
            return None
        
        # 2. 사용자 승인 (관리자 권한 필요)
        admin_token = get_admin_token()
        if admin_token:
            approve_response = requests.post(
                f"{BASE_URL}/api/auth/admin/users/approve",
                json={"user_id": user_id},
                headers=get_headers(admin_token),
                timeout=5
            )
            if approve_response.status_code not in [200, 201]:
                print(f"⚠️  User approval failed: {approve_response.status_code}")
        
        # 3. 로그인
        token = login(username, password)
        if not token:
            print(f"❌ Login failed for user: {username}")
            return None
        
        return {
            "user_id": user_id,
            "username": username,
            "password": password,
            "email": email,
            "token": token
        }
    
    except Exception as e:
        print(f"❌ Error creating test user: {e}")
        return None


def create_test_project(token: str, name_prefix: str = "test_project") -> Optional[int]:
    """테스트 프로젝트 생성
    
    Returns:
        project_id (int)
    """
    timestamp = int(time.time() * 1000)
    today = date.today()
    
    project_data = {
        "name": f"{name_prefix}_{timestamp}",
        "description": f"E2E Test Project {timestamp}",
        "sponsor": "Test Sponsor",
        "start_date": str(today),
        "end_date": str(today + timedelta(days=365))
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/projects",
            json=project_data,
            headers=get_headers(token),
            timeout=5
        )
        
        if response.status_code not in [200, 201]:
            print(f"❌ Project creation failed: {response.status_code} - {response.text}")
            return None
        
        project = response.json()
        project_id = project.get("id") or project.get("project_id")
        return project_id
    
    except Exception as e:
        print(f"❌ Error creating test project: {e}")
        return None


def add_user_to_project(user_id: int, project_id: int, token: str, role_name: str = "RESEARCHER") -> bool:
    """사용자를 프로젝트에 추가

    Args:
        user_id: 추가할 사용자 ID
        project_id: 프로젝트 ID
        token: JWT 토큰 (관리자 권한 필요)
        role_name: 역할 이름 (기본값: RESEARCHER)

    Returns:
        성공 여부
    """
    try:
        # 1. 먼저 이미 멤버인지 확인
        response = requests.get(
            f"{BASE_URL}/api/projects/{project_id}/members",
            headers=get_headers(token),
            timeout=5
        )

        if response.status_code == 200:
            members = response.json()
            if isinstance(members, list):
                for member in members:
                    member_user_id = member.get("user_id") or member.get("id")
                    if member_user_id == user_id:
                        print(f"ℹ️  User {user_id} is already a member of project {project_id}")
                        return True

        # 2. Role ID 조회 (관리자 토큰 사용)
        admin_token = get_admin_token()
        if not admin_token:
            print("⚠️  Could not get admin token for role lookup")
            admin_token = token

        role_id = None
        response = requests.get(
            f"{BASE_URL}/api/roles",
            headers=get_headers(admin_token),
            timeout=5
        )

        if response.status_code == 200:
            roles = response.json()
            if isinstance(roles, list):
                for role in roles:
                    if role.get("name") == role_name or role.get("role_name") == role_name:
                        role_id = role.get("id") or role.get("role_id")
                        break
            elif isinstance(roles, dict) and "roles" in roles:
                for role in roles["roles"]:
                    if role.get("name") == role_name or role.get("role_name") == role_name:
                        role_id = role.get("id") or role.get("role_id")
                        break

        if not role_id:
            print(f"⚠️  Could not find role: {role_name}, using default role_id=2")
            role_id = 2  # RESEARCHER role

        # 3. 프로젝트에 멤버 추가 (관리자 토큰 사용)
        member_data = {
            "user_id": user_id,
            "role_id": role_id
        }

        response = requests.post(
            f"{BASE_URL}/api/projects/{project_id}/members",
            json=member_data,
            headers=get_headers(admin_token),
            timeout=5
        )

        if response.status_code in [200, 201]:
            return True
        else:
            print(f"⚠️  Failed to add user to project: {response.status_code}")
            print(f"    Response: {response.text[:200]}")
            return False

    except Exception as e:
        print(f"❌ Error adding user to project: {e}")
        import traceback
        traceback.print_exc()
        return False


def cleanup_project(project_id: int, token: str) -> bool:
    """프로젝트 삭제"""
    try:
        response = requests.delete(
            f"{BASE_URL}/api/projects/{project_id}",
            headers=get_headers(token),
            timeout=5
        )
        return response.status_code in [200, 204]
    except Exception:
        return False


def cleanup_user(user_id: int, admin_token: str) -> bool:
    """사용자 삭제 (관리자 권한 필요)"""
    try:
        response = requests.delete(
            f"{BASE_URL}/api/users/{user_id}",
            headers=get_headers(admin_token),
            timeout=5
        )
        return response.status_code in [200, 204]
    except Exception:
        return False


def health_check() -> bool:
    """서버 헬스 체크"""
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        return response.status_code == 200
    except Exception:
        return False


