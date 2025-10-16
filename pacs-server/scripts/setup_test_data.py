#!/usr/bin/env python3
"""
테스트 데이터 설정 스크립트

이 스크립트는 마스크 업로드 테스트를 위한 기본 데이터를 생성합니다.
- 테스트 사용자 생성
- 테스트 프로젝트 생성
- 사용자를 프로젝트에 추가

사용법:
    python3 scripts/setup_test_data.py
"""

import os
import sys
import json
import time
import requests
from typing import Dict, Any, Optional

# 서버 설정
PACS_SERVER_URL = os.getenv('PACS_SERVER_URL', 'http://localhost:8080')

class TestDataSetup:
    """테스트 데이터 설정 클래스"""
    
    def __init__(self, server_url: str):
        self.server_url = server_url.rstrip('/')
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        })
        
    def log(self, message: str, level: str = "INFO"):
        """로그 메시지 출력"""
        timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
        print(f"[{timestamp}] [{level}] {message}")
        
    def create_test_user(self) -> Optional[int]:
        """테스트 사용자 생성"""
        import uuid
        user_data = {
            "keycloak_id": str(uuid.uuid4()),
            "username": "testuser",
            "email": "test@example.com"
        }
        
        try:
            response = self.session.post(
                f"{self.server_url}/api/users",
                json=user_data
            )
            
            if response.status_code == 201:
                user = response.json()
                user_id = user['id']
                self.log(f"테스트 사용자 생성 성공: ID {user_id}")
                return user_id
            elif response.status_code == 409:
                # 사용자가 이미 존재하는 경우, 기존 사용자 조회
                self.log("사용자가 이미 존재합니다. 기존 사용자를 조회합니다.")
                return self.find_user_by_username("testuser")
            else:
                self.log(f"사용자 생성 실패: {response.status_code} - {response.text}", "ERROR")
                return None
                
        except Exception as e:
            self.log(f"사용자 생성 중 오류: {e}", "ERROR")
            return None
    
    def find_user_by_username(self, username: str) -> Optional[int]:
        """사용자명으로 사용자 조회"""
        try:
            response = self.session.get(
                f"{self.server_url}/api/users",
                params={"username": username}
            )
            
            if response.status_code == 200:
                users = response.json()
                if users and len(users) > 0:
                    user_id = users[0]['id']
                    self.log(f"기존 사용자 조회 성공: ID {user_id}")
                    return user_id
            
            self.log(f"사용자를 찾을 수 없습니다: {username}", "ERROR")
            return None
            
        except Exception as e:
            self.log(f"사용자 조회 중 오류: {e}", "ERROR")
            return None
    
    def create_test_project(self) -> Optional[int]:
        """테스트 프로젝트 생성"""
        project_data = {
            "name": "Test Project",
            "description": "마스크 업로드 테스트용 프로젝트",
            "is_active": True
        }
        
        try:
            response = self.session.post(
                f"{self.server_url}/api/projects",
                json=project_data
            )
            
            if response.status_code == 201:
                project = response.json()
                project_id = project['id']
                self.log(f"테스트 프로젝트 생성 성공: ID {project_id}")
                return project_id
            elif response.status_code == 409:
                # 프로젝트가 이미 존재하는 경우, 기존 프로젝트 조회
                self.log("프로젝트가 이미 존재합니다. 기존 프로젝트를 조회합니다.")
                return self.find_project_by_name("Test Project")
            else:
                self.log(f"프로젝트 생성 실패: {response.status_code} - {response.text}", "ERROR")
                return None
                
        except Exception as e:
            self.log(f"프로젝트 생성 중 오류: {e}", "ERROR")
            return None
    
    def find_project_by_name(self, name: str) -> Optional[int]:
        """프로젝트명으로 프로젝트 조회"""
        try:
            response = self.session.get(
                f"{self.server_url}/api/projects"
            )
            
            if response.status_code == 200:
                projects = response.json()
                for project in projects:
                    if project['name'] == name:
                        project_id = project['id']
                        self.log(f"기존 프로젝트 조회 성공: ID {project_id}")
                        return project_id
            
            self.log(f"프로젝트를 찾을 수 없습니다: {name}", "ERROR")
            return None
            
        except Exception as e:
            self.log(f"프로젝트 조회 중 오류: {e}", "ERROR")
            return None
    
    def add_user_to_project(self, user_id: int, project_id: int) -> bool:
        """사용자를 프로젝트에 추가"""
        try:
            response = self.session.post(
                f"{self.server_url}/api/projects/{project_id}/members",
                json={"user_id": user_id}
            )
            
            if response.status_code in [200, 201, 409]:  # 409는 이미 멤버인 경우
                self.log(f"사용자를 프로젝트에 추가 성공: User {user_id} -> Project {project_id}")
                return True
            else:
                self.log(f"사용자 프로젝트 추가 실패: {response.status_code} - {response.text}", "ERROR")
                return False
                
        except Exception as e:
            self.log(f"사용자 프로젝트 추가 중 오류: {e}", "ERROR")
            return False
    
    def run_setup(self):
        """전체 설정 실행"""
        self.log("=== 테스트 데이터 설정 시작 ===")
        
        # 1. 테스트 사용자 생성
        user_id = self.create_test_user()
        if not user_id:
            self.log("사용자 생성에 실패했습니다. 설정을 중단합니다.", "ERROR")
            return False
        
        # 2. 테스트 프로젝트 생성
        project_id = self.create_test_project()
        if not project_id:
            self.log("프로젝트 생성에 실패했습니다. 설정을 중단합니다.", "ERROR")
            return False
        
        # 3. 사용자를 프로젝트에 추가
        if not self.add_user_to_project(user_id, project_id):
            self.log("사용자 프로젝트 추가에 실패했습니다. 설정을 중단합니다.", "ERROR")
            return False
        
        # 4. 결과 출력
        self.log("=== 설정 완료 ===")
        self.log(f"사용자 ID: {user_id}")
        self.log(f"프로젝트 ID: {project_id}")
        self.log("환경 변수 설정:")
        self.log(f"export TEST_USER_ID={user_id}")
        self.log(f"export TEST_PROJECT_ID={project_id}")
        
        return True

def main():
    """메인 함수"""
    print("PACS Extension Server - 테스트 데이터 설정")
    print("=" * 50)
    
    # 설정 실행
    setup = TestDataSetup(PACS_SERVER_URL)
    success = setup.run_setup()
    
    if success:
        print("\n✅ 테스트 데이터 설정이 완료되었습니다!")
        print("\n다음 명령어로 마스크 업로드 테스트를 실행하세요:")
        print("python3 scripts/test_mask_upload.py")
        sys.exit(0)
    else:
        print("\n❌ 테스트 데이터 설정에 실패했습니다.")
        sys.exit(1)

if __name__ == "__main__":
    main()
