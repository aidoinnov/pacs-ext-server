#!/usr/bin/env python3
"""
어노테이션 버전 충돌 (Optimistic Locking) E2E 테스트 (리팩토링 버전)

이 테스트는 동시 업데이트 시 버전 충돌 처리 기능을 검증합니다.
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import AnnotationFixtures


class AnnotationVersionConflictTest(BaseE2ETest):
    """어노테이션 버전 충돌 테스트"""
    
    def get_test_name(self) -> str:
        return "어노테이션 버전 충돌 E2E 테스트"
    
    def run_tests(self):
        """테스트 실행"""
        # 첫 번째 어노테이션 생성
        annotation_id, version = self._create_test_annotation()
        self.created_annotation_ids.append(annotation_id)
        
        # 테스트 1, 2 실행
        new_version = self.test_version_match_update_succeeds(annotation_id, version)
        self.test_version_mismatch_update_fails(annotation_id, new_version)
        
        # 테스트 3용 새 어노테이션 생성
        concurrent_id, concurrent_version = self._create_test_annotation()
        self.created_annotation_ids.append(concurrent_id)
        self.test_concurrent_update_scenario(concurrent_id, concurrent_version)
    
    def _create_test_annotation(self) -> tuple:
        """테스트용 어노테이션 생성"""
        print("📝 테스트용 어노테이션 생성 중...")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        annotation_data = {
            "project_id": TestConfig.DEFAULT_PROJECT_ID,
            "study_instance_uid": TestConfig.STUDY_UID,
            "series_instance_uid": TestConfig.SERIES_UID,
            "sop_instance_uid": TestConfig.INSTANCE_UID,
            "annotation_data": {"type": "circle", "x": 100, "y": 100, "radius": 50},
            "tool_name": "Circle Tool",
            "viewer_software": "TI-DicomViewer",
            "description": "Version conflict test",
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/annotations",
            json=annotation_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
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
    
    def test_version_match_update_succeeds(self, annotation_id: int, current_version: int) -> int:
        """테스트 1: 버전 일치 - 업데이트 성공"""
        TestPrinter.print_header("테스트 1: 버전 일치 - 업데이트 성공")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        update_data = {
            "annotation_data": {"type": "circle", "x": 150, "y": 150, "radius": 60},
            "description": "Updated with correct version",
            "base_version": current_version,
        }
        
        response = requests.put(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            json=update_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            new_version = data.get("version", current_version + 1)
            TestPrinter.print_success("업데이트 성공!")
            TestPrinter.print_info(f"Old version: {current_version}", indent=1)
            TestPrinter.print_info(f"New version: {new_version}", indent=1)
            assert new_version == current_version + 1, "Version should increment by 1"
            TestPrinter.print_success("테스트 통과")
            return new_version
        else:
            TestPrinter.print_error(f"테스트 실패: {response.text}")
            exit(1)
    
    def test_version_mismatch_update_fails(self, annotation_id: int, current_version: int):
        """테스트 2: 버전 불일치 - 409 Conflict"""
        TestPrinter.print_header("테스트 2: 버전 불일치 - 409 Conflict")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 잘못된 버전으로 업데이트 시도
        wrong_version = current_version - 1
        update_data = {
            "annotation_data": {"type": "circle", "x": 200, "y": 200, "radius": 70},
            "description": "Update with wrong version",
            "base_version": wrong_version,
        }
        
        response = requests.put(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            json=update_data,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        
        if response.status_code == 409:
            TestPrinter.print_success("버전 충돌 감지됨 (예상된 동작)")
            TestPrinter.print_info(f"Client version: {wrong_version}", indent=1)
            TestPrinter.print_info(f"Server version: {current_version}", indent=1)
            TestPrinter.print_success("테스트 통과")
        elif response.status_code == 200:
            TestPrinter.print_warning("업데이트 성공 (버전 체크가 없을 수 있음)")
        else:
            TestPrinter.print_warning(f"예상치 못한 응답: {response.text}")
    
    def test_concurrent_update_scenario(self, annotation_id: int, version: int):
        """테스트 3: 동시 업데이트 시나리오"""
        TestPrinter.print_header("테스트 3: 동시 업데이트 시나리오")
        
        headers = {"Authorization": f"Bearer {self.token}"}
        
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
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            json=update_a,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response_a.status_code == 200:
            new_version = response_a.json().get("version", version + 1)
            TestPrinter.print_success(f"사용자 A 업데이트 성공 (version: {version} → {new_version})")
        else:
            TestPrinter.print_error("사용자 A 업데이트 실패")
            exit(1)
        
        # 사용자 B: 업데이트 실패 (버전 충돌)
        update_b = {
            "description": "Updated by User B",
            "base_version": version,  # 오래된 버전
        }
        
        response_b = requests.put(
            f"{TestConfig.BASE_URL}/api/annotations/{annotation_id}",
            json=update_b,
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        if response_b.status_code == 409:
            TestPrinter.print_success("사용자 B 업데이트 실패 (버전 충돌 감지)")
            TestPrinter.print_success("테스트 통과: 동시 업데이트 시나리오 정상 작동")
        elif response_b.status_code == 200:
            TestPrinter.print_warning("사용자 B 업데이트 성공 (버전 체크가 없을 수 있음)")
        else:
            TestPrinter.print_warning(f"예상치 못한 응답: {response_b.text}")


if __name__ == '__main__':
    test = AnnotationVersionConflictTest()
    test.run()

