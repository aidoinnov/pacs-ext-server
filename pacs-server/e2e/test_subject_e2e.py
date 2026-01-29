#!/usr/bin/env python3
"""
Subject API E2E 테스트

이 테스트는 Subject의 전체 라이프사이클을 검증합니다:
- Subject CRUD
- Subject-Project 관계
- Subject 상세 조회 (통계 포함)
- Subject 코드 및 Patient ID 중복 체크
- TimePoint 관계
"""

import requests
from test_base import BaseE2ETest, TestConfig, TestPrinter


class SubjectE2ETest(BaseE2ETest):
    """Subject E2E 테스트"""
    
    def __init__(self):
        super().__init__()
        self.created_subject_ids = []
        self.test_project_id = 556  # 존재하는 프로젝트 ID
    
    def get_test_name(self) -> str:
        return "Subject E2E 테스트"
    
    def cleanup(self):
        """테스트 정리"""
        TestPrinter.print_header("테스트 정리")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # Subject 삭제
        for subject_id in self.created_subject_ids:
            try:
                response = requests.delete(
                    f"{TestConfig.BASE_URL}/api/subjects/{subject_id}",
                    headers=headers,
                    timeout=TestConfig.DEFAULT_TIMEOUT
                )
                if response.status_code == 204:
                    TestPrinter.print_success(f"Subject {subject_id} 삭제 성공")
            except Exception as e:
                TestPrinter.print_warning(f"Subject {subject_id} 삭제 실패: {e}")
        
        super().cleanup()
    
    def run_tests(self):
        """테스트 실행"""
        self.test_subject_crud()
        self.test_subject_code_validation()
        self.test_subject_duplicate_check()
        self.test_subject_detail_with_stats()
        self.test_subject_list_by_project()
    
    def test_subject_crud(self):
        """테스트 1: Subject CRUD"""
        TestPrinter.print_header("테스트 1: Subject CRUD")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 1. Subject 생성
        print("1️⃣  Subject 생성...")
        create_data = {
            "subject_code": "E2E_TEST_001",
            "patient_id": "PAT_E2E_001",
            "patient_name": "Test Patient",
            "patient_birth_date": "1990-01-01"
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=create_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 201, f"Subject 생성 실패: {response.text}"
        
        subject_data = response.json()
        subject_id = subject_data["id"]
        self.created_subject_ids.append(subject_id)
        TestPrinter.print_success(f"Subject 생성 성공: ID={subject_id}")
        assert subject_data["subject_code"] == "E2E_TEST_001"
        assert subject_data["patient_id"] == "PAT_E2E_001"
        
        # 2. Subject 조회
        print("\n2️⃣  Subject 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/subjects/{subject_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        assert response.status_code == 200, f"Subject 조회 실패: {response.text}"
        retrieved_data = response.json()
        assert retrieved_data["id"] == subject_id
        assert retrieved_data["subject_code"] == "E2E_TEST_001"
        TestPrinter.print_success("Subject 조회 성공")
        
        # 3. Subject 수정
        print("\n3️⃣  Subject 수정...")
        update_data = {
            "subject_code": "E2E_TEST_001_UPDATED",
            "patient_name": "Updated Patient Name"
        }
        
        response = requests.put(
            f"{TestConfig.BASE_URL}/api/subjects/{subject_id}",
            headers=headers,
            json=update_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        assert response.status_code == 200, f"Subject 수정 실패: {response.text}"
        updated_data = response.json()
        assert updated_data["subject_code"] == "E2E_TEST_001_UPDATED"
        assert updated_data["patient_name"] == "Updated Patient Name"
        TestPrinter.print_success("Subject 수정 성공")
        
        # 4. Subject 삭제
        print("\n4️⃣  Subject 삭제...")
        response = requests.delete(
            f"{TestConfig.BASE_URL}/api/subjects/{subject_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        assert response.status_code == 204, f"Subject 삭제 실패: {response.text}"
        TestPrinter.print_success("Subject 삭제 성공")
        self.created_subject_ids.remove(subject_id)
        
        # 5. 삭제 확인
        print("\n5️⃣  삭제 확인...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/subjects/{subject_id}",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        assert response.status_code == 404, "삭제된 Subject가 여전히 조회됨"
        TestPrinter.print_success("삭제 확인 완료")
    
    def test_subject_code_validation(self):
        """테스트 2: Subject 코드 유효성 검증"""
        TestPrinter.print_header("테스트 2: Subject 코드 유효성 검증")
        headers = {"Authorization": f"Bearer {self.token}"}
        
        # 1. 유효하지 않은 Subject 코드 (특수문자 포함)
        print("1️⃣  유효하지 않은 Subject 코드 테스트...")
        invalid_data = {
            "subject_code": "INVALID@CODE#123",
            "patient_id": "PAT_INVALID_001"
        }
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=invalid_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 400, "유효하지 않은 코드가 허용됨"
        TestPrinter.print_success("유효하지 않은 코드 거부됨")

        # 2. 유효한 Subject 코드 (영문, 숫자, 하이픈, 언더스코어)
        print("\n2️⃣  유효한 Subject 코드 테스트...")
        valid_data = {
            "subject_code": "VALID_CODE-123",
            "patient_id": "PAT_VALID_001"
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=valid_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 201, f"유효한 코드가 거부됨: {response.text}"
        subject_id = response.json()["id"]
        self.created_subject_ids.append(subject_id)
        TestPrinter.print_success("유효한 코드 허용됨")

    def test_subject_duplicate_check(self):
        """테스트 3: Subject 코드 및 Patient ID 중복 체크"""
        TestPrinter.print_header("테스트 3: Subject 코드 및 Patient ID 중복 체크")
        headers = {"Authorization": f"Bearer {self.token}"}

        # 1. 첫 번째 Subject 생성
        print("1️⃣  첫 번째 Subject 생성...")
        create_data = {
            "subject_code": "DUP_TEST_001",
            "patient_id": "PAT_DUP_001"
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=create_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        assert response.status_code == 201, f"Subject 생성 실패: {response.text}"
        subject_id = response.json()["id"]
        self.created_subject_ids.append(subject_id)
        TestPrinter.print_success("첫 번째 Subject 생성 성공")

        # 2. 동일한 Subject 코드로 생성 시도 (409 Conflict 예상)
        print("\n2️⃣  동일한 Subject 코드로 생성 시도...")
        duplicate_code_data = {
            "subject_code": "DUP_TEST_001",
            "patient_id": "PAT_DUP_002"
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=duplicate_code_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 409, "중복 Subject 코드가 허용됨"
        TestPrinter.print_success("중복 Subject 코드 거부됨 (409 Conflict)")

        # 3. 동일한 Patient ID로 생성 시도 (409 Conflict 예상)
        print("\n3️⃣  동일한 Patient ID로 생성 시도...")
        duplicate_patient_data = {
            "subject_code": "DUP_TEST_002",
            "patient_id": "PAT_DUP_001"
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=duplicate_patient_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 409, "중복 Patient ID가 허용됨"
        TestPrinter.print_success("중복 Patient ID 거부됨 (409 Conflict)")

    def test_subject_detail_with_stats(self):
        """테스트 4: Subject 상세 조회 (통계 포함)"""
        TestPrinter.print_header("테스트 4: Subject 상세 조회 (통계 포함)")
        headers = {"Authorization": f"Bearer {self.token}"}

        # 1. Subject 생성
        print("1️⃣  Subject 생성...")
        create_data = {
            "subject_code": "DETAIL_TEST_001",
            "patient_id": "PAT_DETAIL_001",
            "patient_name": "Detail Test Patient"
        }

        response = requests.post(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            json=create_data,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        subject_id = response.json()["id"]
        self.created_subject_ids.append(subject_id)
        TestPrinter.print_success(f"Subject 생성 성공: ID={subject_id}")

        # 2. Subject 상세 조회
        print("\n2️⃣  Subject 상세 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/subjects/{subject_id}/detail",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"Subject 상세 조회 실패: {response.text}"

        detail_data = response.json()
        assert "id" in detail_data
        assert "subject_code" in detail_data
        assert "timepoint_count" in detail_data

        TestPrinter.print_success("Subject 상세 조회 성공")
        TestPrinter.print_info(f"Subject Code: {detail_data['subject_code']}", indent=1)
        TestPrinter.print_info(f"TimePoint Count: {detail_data['timepoint_count']}", indent=1)

    def test_subject_list_by_project(self):
        """테스트 5: 프로젝트별 Subject 목록 조회"""
        TestPrinter.print_header("테스트 5: 프로젝트별 Subject 목록 조회")
        headers = {"Authorization": f"Bearer {self.token}"}

        # 1. 여러 개의 Subject 생성
        print("1️⃣  여러 개의 Subject 생성...")
        subject_codes = ["LIST_TEST_001", "LIST_TEST_002", "LIST_TEST_003"]
        created_ids = []

        for code in subject_codes:
            create_data = {
                "subject_code": code,
                "patient_id": f"PAT_{code}"
            }

            response = requests.post(
                f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
                headers=headers,
                json=create_data,
                timeout=TestConfig.DEFAULT_TIMEOUT
            )

            if response.status_code == 201:
                subject_id = response.json()["id"]
                created_ids.append(subject_id)
                self.created_subject_ids.append(subject_id)

        TestPrinter.print_success(f"{len(created_ids)}개 Subject 생성 완료")

        # 2. 프로젝트별 Subject 목록 조회
        print("\n2️⃣  프로젝트별 Subject 목록 조회...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/projects/{self.test_project_id}/subjects",
            headers=headers,
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"Subject 목록 조회 실패: {response.text}"

        subjects = response.json()
        assert isinstance(subjects, list), "응답이 리스트가 아님"
        assert len(subjects) >= len(created_ids), "생성한 Subject가 목록에 없음"

        TestPrinter.print_success(f"Subject 목록 조회 성공: {len(subjects)}개")

        # 3. 생성한 Subject가 목록에 포함되어 있는지 확인
        print("\n3️⃣  생성한 Subject 확인...")
        subject_ids_in_list = [s["id"] for s in subjects]
        for created_id in created_ids:
            assert created_id in subject_ids_in_list, f"Subject {created_id}가 목록에 없음"

        TestPrinter.print_success("모든 생성한 Subject가 목록에 포함됨")


if __name__ == "__main__":
    test = SubjectE2ETest()
    test.run()


