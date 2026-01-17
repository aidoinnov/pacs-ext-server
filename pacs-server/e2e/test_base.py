#!/usr/bin/env python3
"""
E2E 테스트 베이스 클래스 및 공통 설정
"""

import requests
from typing import Optional, List, Dict, Any, Callable
from abc import ABC, abstractmethod
import traceback


# ===== 공통 설정 =====
class TestConfig:
    """테스트 공통 설정"""
    BASE_URL = "http://localhost:8080"
    ADMIN_USER = "iaid-pacs-admin"
    ADMIN_PASSWORD = "Qlalfqjsgh1!"
    DEFAULT_TIMEOUT = 10
    
    # 테스트용 DICOM UIDs
    STUDY_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.307959856517080892181338382781"
    SERIES_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.362217378389574461124736555345"
    INSTANCE_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.238273576775187812804817387920"
    
    # 스냅샷 테스트용 UIDs
    SNAPSHOT_STUDY_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.321111757620390201880556376661"
    SNAPSHOT_SERIES_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.260616660471925521837323152953"
    SNAPSHOT_INSTANCE_UID = "1.3.6.1.4.1.14519.5.2.1.6655.2359.217230834888240455035945707219"
    
    DEFAULT_PROJECT_ID = 2


# ===== 공통 유틸리티 함수 =====
class TestAuth:
    """인증 관련 유틸리티"""
    
    @staticmethod
    def login(username: str = None, password: str = None) -> str:
        """로그인하여 JWT 토큰 획득"""
        username = username or TestConfig.ADMIN_USER
        password = password or TestConfig.ADMIN_PASSWORD
        
        print(f"🔐 로그인 중: {username}")
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/auth/login",
            json={"username": username, "password": password},
            timeout=5
        )
        
        if response.status_code != 200:
            print(f"❌ 로그인 실패: {response.status_code}")
            print(response.text)
            raise Exception(f"Login failed: {response.status_code}")
        
        token = response.json()["token"]
        print(f"✅ 로그인 성공\n")
        return token


class TestPrinter:
    """테스트 출력 유틸리티"""
    
    @staticmethod
    def print_header(title: str):
        """테스트 헤더 출력"""
        print("\n" + "=" * 70)
        print(title)
        print("=" * 70)
    
    @staticmethod
    def print_success(message: str, indent: int = 0):
        """성공 메시지 출력"""
        prefix = "   " * indent
        print(f"{prefix}✅ {message}")

    @staticmethod
    def print_error(message: str, indent: int = 0):
        """에러 메시지 출력"""
        prefix = "   " * indent
        print(f"{prefix}❌ {message}")

    @staticmethod
    def print_warning(message: str, indent: int = 0):
        """경고 메시지 출력"""
        prefix = "   " * indent
        print(f"{prefix}⚠️  {message}")
    
    @staticmethod
    def print_info(message: str, indent: int = 0):
        """정보 메시지 출력"""
        prefix = "   " * indent
        print(f"{prefix}{message}")
    
    @staticmethod
    def print_test_result(passed: bool, test_name: str):
        """테스트 결과 출력"""
        if passed:
            print(f"✅ 테스트 통과: {test_name}")
        else:
            print(f"❌ 테스트 실패: {test_name}")


# ===== 베이스 테스트 클래스 =====
class BaseE2ETest(ABC):
    """E2E 테스트 베이스 클래스"""
    
    def __init__(self):
        self.token: Optional[str] = None
        self.created_annotation_ids: List[int] = []
        self.created_user_ids: List[int] = []
    
    def setup(self):
        """테스트 셋업"""
        print(f"\n🚀 {self.get_test_name()} 시작...\n")
        self.token = TestAuth.login()
    
    def teardown(self):
        """테스트 정리"""
        from test_utils import cleanup_annotations, delete_user
        
        # 어노테이션 정리
        if self.created_annotation_ids and self.token:
            cleanup_annotations(self.token, self.created_annotation_ids)
        
        # 사용자 정리
        if self.created_user_ids and self.token:
            print("\n🧹 Cleanup: 테스트용 사용자 삭제 중...")
            for user_id in self.created_user_ids:
                if delete_user(self.token, user_id):
                    print(f"   ✅ 사용자 ID {user_id} 삭제 성공")
                else:
                    print(f"   ⚠️  사용자 ID {user_id} 삭제 실패")
            print()
    
    def run(self):
        """테스트 실행"""
        try:
            self.setup()
            self.run_tests()
            
            TestPrinter.print_header("🎉 모든 테스트 통과!")
            print()
            
        except Exception as e:
            print(f"\n❌ 테스트 실패: {e}\n")
            traceback.print_exc()
            exit(1)
        finally:
            self.teardown()
    
    @abstractmethod
    def get_test_name(self) -> str:
        """테스트 이름 반환"""
        pass
    
    @abstractmethod
    def run_tests(self):
        """테스트 실행 (서브클래스에서 구현)"""
        pass

