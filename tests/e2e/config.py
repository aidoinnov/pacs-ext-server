"""
E2E 테스트 설정
"""
import os
from dataclasses import dataclass
from typing import Optional


# 테스트 계정 정보 (하드코딩)
TEST_ACCOUNTS = {
    'reader': {
        'username': 'reader1_user',
        'email': 'reader1@example.com',
        'password': 'Qlalfqjsgh1!',
    },
    'admin': {
        'username': 'reader1_user',
        'email': 'reader1@example.com',
        'password': 'Qlalfqjsgh1!',
    },
    'user': {
        'username': 'reader1_user',
        'email': 'reader1@example.com',
        'password': 'Qlalfqjsgh1!',
    }
}


@dataclass
class TestConfig:
    """테스트 설정"""
    base_url: str
    admin_email: str
    admin_password: str
    test_user_email: str
    test_user_password: str
    timeout: int = 30

    @classmethod
    def from_env(cls) -> 'TestConfig':
        """환경 변수에서 설정 로드 (기본값은 하드코딩된 테스트 계정 사용)"""
        return cls(
            base_url=os.getenv('TEST_BASE_URL', 'http://localhost:8080'),
            admin_email=os.getenv('TEST_ADMIN_EMAIL', TEST_ACCOUNTS['reader']['username']),
            admin_password=os.getenv('TEST_ADMIN_PASSWORD', TEST_ACCOUNTS['reader']['password']),
            test_user_email=os.getenv('TEST_USER_EMAIL', TEST_ACCOUNTS['user']['username']),
            test_user_password=os.getenv('TEST_USER_PASSWORD', TEST_ACCOUNTS['user']['password']),
            timeout=int(os.getenv('TEST_TIMEOUT', '30')),
        )


@dataclass
class PerformanceConfig:
    """성능 테스트 설정"""
    concurrent_users: int = 10
    requests_per_user: int = 100
    ramp_up_time: int = 5  # seconds
    test_duration: int = 60  # seconds
    
    @classmethod
    def from_env(cls) -> 'PerformanceConfig':
        """환경 변수에서 설정 로드"""
        return cls(
            concurrent_users=int(os.getenv('PERF_CONCURRENT_USERS', '10')),
            requests_per_user=int(os.getenv('PERF_REQUESTS_PER_USER', '100')),
            ramp_up_time=int(os.getenv('PERF_RAMP_UP_TIME', '5')),
            test_duration=int(os.getenv('PERF_TEST_DURATION', '60')),
        )


# 테스트 프로젝트 정보
TEST_PROJECTS = {
    'project1': {
        'id': 1,
        'name': 'Test Project 1',
        'uuid': '550e8400-e29b-41d4-a716-446655440001'
    },
    'project2': {
        'id': 2,
        'name': 'Test Project 2',
        'uuid': '550e8400-e29b-41d4-a716-446655440002'
    }
}

# 테스트 데이터 (실제 DB에 있는 데이터)
TEST_STUDY_UID = "1.2.840.113619.2.55.3.604688119.868.1234567890.1"
TEST_SERIES_UID = "1.2.840.113619.2.55.3.604688119.868.1234567890.2"
TEST_INSTANCE_UID = "1.2.840.113619.2.55.3.604688119.868.1234567890.3"

# 샘플 어노테이션 데이터
SAMPLE_ANNOTATION_DATA = {
    "type": "rectangle",
    "coordinates": {
        "x": 100,
        "y": 100,
        "width": 200,
        "height": 150
    },
    "color": "#FF0000"
}

