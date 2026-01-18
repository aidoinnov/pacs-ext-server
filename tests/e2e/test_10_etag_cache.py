"""
ETag 기반 캐시 전략 테스트

DICOM Gateway API의 ETag 동작을 검증합니다:
1. 첫 요청 시 ETag 반환
2. If-None-Match 헤더로 재요청 시 304 응답
3. 데이터 변경 후 새 ETag 반환
"""
import pytest
from utils.api_client import APIClient
from config import TEST_ACCOUNTS


class TestETagCache:
    """ETag 캐시 테스트"""

    @classmethod
    def setup_class(cls):
        """테스트 클래스 초기화"""
        cls.client = APIClient(base_url="http://localhost:8080")
        reader = TEST_ACCOUNTS['reader']
        cls.client.login(reader['username'], reader['password'])
        
        # 테스트용 프로젝트 ID
        cls.project_id = 2
        cls.patient_id = "Lung_Dx-A0011"

    def test_01_first_request_returns_etag(self):
        """첫 요청 시 ETag가 반환되는지 확인"""
        response = self.client.get(
            "/api/me/dicom/studies",
            params={
                "view": "default",
                "project_id": self.project_id,
                "patient_id": self.patient_id,
                "page": 1,
                "page_size": 10,
            }
        )
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        # ETag 헤더 확인
        etag = response.headers.get('ETag')
        assert etag is not None, "ETag header not found"
        assert etag.startswith('"') and etag.endswith('"'), "ETag should be quoted"
        
        # Cache-Control 헤더 확인
        cache_control = response.headers.get('Cache-Control')
        assert cache_control is not None, "Cache-Control header not found"
        assert 'no-cache' in cache_control, "Cache-Control should contain 'no-cache'"
        assert 'must-revalidate' in cache_control, "Cache-Control should contain 'must-revalidate'"
        
        # 데이터 확인
        studies = response.json()
        assert isinstance(studies, list), "Response should be a list"
        
        print(f"✅ First request: ETag={etag}, {len(studies)} studies")
        
        # 다음 테스트를 위해 ETag 저장
        self.__class__.first_etag = etag
        self.__class__.first_data = studies

    def test_02_if_none_match_returns_304(self):
        """If-None-Match 헤더로 재요청 시 304 응답 확인"""
        # 이전 테스트에서 받은 ETag 사용
        etag = self.__class__.first_etag

        # If-None-Match 헤더와 함께 요청
        response = self.client.get(
            "/api/me/dicom/studies",
            params={
                "view": "default",
                "project_id": self.project_id,
                "patient_id": self.patient_id,
                "page": 1,
                "page_size": 10,
            },
            extra_headers={
                "If-None-Match": etag
            }
        )
        
        # 304 Not Modified 응답 확인
        assert response.status_code == 304, f"Expected 304, got {response.status_code}"
        
        # ETag 헤더는 여전히 반환되어야 함
        response_etag = response.headers.get('ETag')
        assert response_etag == etag, f"ETag mismatch: {response_etag} != {etag}"
        
        # 304 응답은 body가 없어야 함
        assert len(response.content) == 0, "304 response should have no body"
        
        print(f"✅ If-None-Match request: 304 Not Modified, no body")

    def test_03_data_change_returns_new_etag(self):
        """데이터 변경 후 새 ETag가 반환되는지 확인"""
        # Subject 조회
        subjects_response = self.client.get(
            f"/api/projects/{self.project_id}/subjects"
        )
        assert subjects_response.status_code == 200
        subjects = subjects_response.json()
        
        # Patient ID로 Subject 찾기
        subject = None
        for s in subjects:
            if s.get('patient_id') == self.patient_id:
                subject = s
                break
        
        assert subject is not None, f"Subject with patient_id={self.patient_id} not found"
        subject_id = subject['id']
        
        # TimePoint 조회
        timepoints_response = self.client.get(
            f"/api/subjects/{subject_id}/timepoints"
        )
        assert timepoints_response.status_code == 200
        timepoints = timepoints_response.json()
        assert len(timepoints) > 0, "No timepoints found"
        
        timepoint_id = timepoints[0]['id']
        
        # 미할당 Study 조회
        unassigned_response = self.client.get(
            f"/api/subjects/{subject_id}/studies/unassigned"
        )
        assert unassigned_response.status_code == 200
        unassigned_studies = unassigned_response.json()
        
        if len(unassigned_studies) == 0:
            print("⚠️  No unassigned studies, skipping data change test")
            pytest.skip("No unassigned studies available")
        
        study_id = unassigned_studies[0]['study_id']
        
        # TimePoint에 Study 할당
        assign_response = self.client.post(
            f"/api/timepoints/{timepoint_id}/studies",
            json={"study_ids": [study_id]}
        )
        assert assign_response.status_code == 200
        
        # X-Cache-Invalidate 헤더 확인
        cache_invalidate = assign_response.headers.get('X-Cache-Invalidate')
        assert cache_invalidate == 'dicom-studies', "X-Cache-Invalidate header should be 'dicom-studies'"
        
        print(f"✅ Study {study_id} assigned to TimePoint {timepoint_id}")
        
        # 이전 ETag로 다시 요청
        old_etag = self.__class__.first_etag
        response = self.client.get(
            "/api/me/dicom/studies",
            params={
                "view": "default",
                "project_id": self.project_id,
                "patient_id": self.patient_id,
                "page": 1,
                "page_size": 10,
            },
            extra_headers={
                "If-None-Match": old_etag
            }
        )
        
        # 데이터가 변경되었으므로 200 응답과 새 ETag 반환
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        new_etag = response.headers.get('ETag')
        assert new_etag is not None, "New ETag not found"
        assert new_etag != old_etag, f"ETag should change after data modification: {new_etag} == {old_etag}"
        
        print(f"✅ Data changed: Old ETag={old_etag}, New ETag={new_etag}")
        
        # Study 할당 해제 (cleanup)
        unassign_response = self.client.delete(
            f"/api/timepoints/{timepoint_id}/studies",
            json={"study_ids": [study_id]}
        )
        assert unassign_response.status_code == 200
        print(f"✅ Cleanup: Study {study_id} unassigned")

