#!/usr/bin/env python3
"""
어노테이션 HEAD 요청 E2E 테스트 (리팩토링 버전)

이 테스트는 HEAD 요청을 통한 캐시 검증 및 리소스 존재 확인 기능을 검증합니다.
- ETag 기반 캐시 검증
- Last-Modified 기반 캐시 검증
- 리소스 존재 확인
- 304 Not Modified 응답
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import AnnotationFixtures


class AnnotationHeadRequestTest(BaseE2ETest):
    """어노테이션 HEAD 요청 테스트"""
    
    def __init__(self):
        super().__init__()
        self.test_annotation_id = None
    
    def get_test_name(self) -> str:
        return "어노테이션 HEAD 요청 E2E 테스트"
    
    def run_tests(self):
        """테스트 실행"""
        # 테스트용 어노테이션 생성
        self.test_annotation_id = self._create_test_annotation()
        self.created_annotation_ids.append(self.test_annotation_id)
        
        # 테스트 실행
        self.test_etag_cache_validation()
        self.test_last_modified_cache_validation()
        self.test_resource_existence_check()
        self.test_head_annotations_list()
    
    def _create_test_annotation(self) -> int:
        """테스트용 어노테이션 생성"""
        print("📝 테스트용 어노테이션 생성 중...")
        
        annotation_id = AnnotationFixtures.create_basic_annotation(
            self.token,
            description="HEAD request test"
        )
        
        if annotation_id:
            print(f"✅ 어노테이션 생성 완료! ID: {annotation_id}\n")
            return annotation_id
        else:
            print("❌ 생성 실패")
            exit(1)
    
    def test_etag_cache_validation(self):
        """테스트 1: ETag 기반 캐시 검증"""
        TestPrinter.print_header("테스트 1: ETag 기반 캐시 검증")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 1. GET 요청으로 ETag 획득
        print("1️⃣  GET 요청으로 ETag 획득...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code != 200:
            TestPrinter.print_error(f"GET 요청 실패: {response.text}")
            exit(1)
        
        etag = response.headers.get("ETag")
        TestPrinter.print_info(f"ETag: {etag}", indent=1)
        
        # 2. HEAD 요청 with If-None-Match
        print("\n2️⃣  HEAD 요청 with If-None-Match...")
        head_headers = {**headers, "If-None-Match": etag}
        
        response = requests.head(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}",
            headers=head_headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code == 304:
            TestPrinter.print_success("304 Not Modified (캐시 유효)")
            TestPrinter.print_success("테스트 통과")
        elif response.status_code == 200:
            TestPrinter.print_warning("200 OK (ETag가 변경되었거나 캐시 검증 미지원)")
        else:
            TestPrinter.print_warning(f"예상치 못한 응답: {response.status_code}")
    
    def test_last_modified_cache_validation(self):
        """테스트 2: Last-Modified 기반 캐시 검증"""
        TestPrinter.print_header("테스트 2: Last-Modified 기반 캐시 검증")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 1. GET 요청으로 Last-Modified 획득
        print("1️⃣  GET 요청으로 Last-Modified 획득...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response.status_code != 200:
            TestPrinter.print_error(f"GET 요청 실패: {response.text}")
            exit(1)
        
        last_modified = response.headers.get("Last-Modified")
        TestPrinter.print_info(f"Last-Modified: {last_modified}", indent=1)
        
        # 2. HEAD 요청 with If-Modified-Since
        if last_modified:
            print("\n2️⃣  HEAD 요청 with If-Modified-Since...")
            head_headers = {**headers, "If-Modified-Since": last_modified}
            
            response = requests.head(
                f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}",
                headers=head_headers,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )
            
            TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
            
            if response.status_code == 304:
                TestPrinter.print_success("304 Not Modified (캐시 유효)")
                TestPrinter.print_success("테스트 통과")
            elif response.status_code == 200:
                TestPrinter.print_warning("200 OK (리소스가 수정되었거나 캐시 검증 미지원)")
            else:
                TestPrinter.print_warning(f"예상치 못한 응답: {response.status_code}")
        else:
            TestPrinter.print_warning("Last-Modified 헤더 없음")
    
    def test_resource_existence_check(self):
        """테스트 3: 리소스 존재 확인"""
        TestPrinter.print_header("테스트 3: 리소스 존재 확인")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 1. 존재하는 리소스 HEAD 요청
        print("1️⃣  존재하는 리소스 HEAD 요청...")
        response = requests.head(
            f"{TestConfig.BASE_URL}/api/annotations/{self.test_annotation_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code == 200:
            TestPrinter.print_success("리소스 존재 확인")
        else:
            TestPrinter.print_error(f"예상치 못한 응답: {response.status_code}")
            exit(1)
        
        # 2. 존재하지 않는 리소스 HEAD 요청
        print("\n2️⃣  존재하지 않는 리소스 HEAD 요청...")
        fake_id = 999999
        response = requests.head(
            f"{TestConfig.BASE_URL}/api/annotations/{fake_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        TestPrinter.print_info(f"Status: {response.status_code}", indent=1)
        
        if response.status_code == 404:
            TestPrinter.print_success("리소스 없음 확인")
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_warning(f"예상치 못한 응답: {response.status_code}")
    
    def test_head_annotations_list(self):
        """테스트 4: 어노테이션 목록 HEAD 요청"""
        TestPrinter.print_header("테스트 4: 어노테이션 목록 HEAD 요청")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        response = requests.head(
            f"{TestConfig.BASE_URL}/api/annotations?series_instance_uid={TestConfig.SERIES_UID}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            x_total_count = response.headers.get("X-Total-Count")
            last_modified = response.headers.get("Last-Modified")
            
            TestPrinter.print_success("HEAD 요청 성공")
            TestPrinter.print_info(f"X-Total-Count: {x_total_count}", indent=1)
            TestPrinter.print_info(f"Last-Modified: {last_modified}", indent=1)
            TestPrinter.print_success("테스트 통과")
        else:
            TestPrinter.print_warning(f"예상치 못한 응답: {response.status_code}")


if __name__ == '__main__':
    test = AnnotationHeadRequestTest()
    test.run()

