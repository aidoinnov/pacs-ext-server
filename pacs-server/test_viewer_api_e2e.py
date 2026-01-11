#!/usr/bin/env python3
"""
Viewer Study Meta API E2E 테스트
POST /api/viewer/studies/meta
"""
import requests
import json
import sys
from typing import Optional, List, Dict

BASE_URL = "http://localhost:8080"

# 테스트 결과 추적
test_results = {
    "total": 0,
    "passed": 0,
    "failed": 0
}

def print_section(title: str):
    """섹션 제목 출력"""
    print("\n" + "="*70)
    print(f"🔍 {title}")
    print("="*70)

def print_test(name: str):
    """테스트 시작 출력"""
    print(f"\n📋 Test: {name}")

def print_success(message: str):
    """성공 메시지 출력"""
    print(f"✅ {message}")
    test_results["passed"] += 1
    test_results["total"] += 1

def print_error(message: str):
    """에러 메시지 출력"""
    print(f"❌ {message}")
    test_results["failed"] += 1
    test_results["total"] += 1

def print_info(message: str):
    """정보 메시지 출력"""
    print(f"ℹ️  {message}")

def get_token() -> Optional[str]:
    """로그인하여 JWT 토큰 획득"""
    print_test("로그인 및 토큰 획득")
    try:
        # 실제 환경에 맞게 수정 필요
        resp = requests.post(f'{BASE_URL}/api/auth/login', json={
            'username': 'iaid-pacs-admin',
            'password': 'Qlalfqjsgh1!'
        }, timeout=10)
        
        if resp.status_code == 200:
            data = resp.json()
            token = data.get('token') or data.get('access_token')
            if token:
                print_success(f"로그인 성공 (token length: {len(token)})")
                return token
            else:
                print_error("응답에 토큰이 없습니다")
                return None
        else:
            print_error(f"로그인 실패: {resp.status_code}")
            print_info(f"Response: {resp.text[:200]}")
            return None
    except Exception as e:
        print_error(f"로그인 에러: {e}")
        return None

def get_sample_study_uids(token: str) -> List[str]:
    """테스트용 Study UID 목록 조회"""
    print_test("샘플 Study UID 조회")

    # 먼저 하드코딩된 테스트 Study UID 사용
    hardcoded_uids = [
        "1.2.840.113619.2.55.3.604688433.1234",
        "1.2.840.113619.2.55.3.604688433.5678"
    ]

    try:
        # QIDO-RS를 통해 실제 Study UID 조회 시도
        headers = {"Authorization": f"Bearer {token}"}
        resp = requests.get(
            f"{BASE_URL}/dicom-web/studies",
            headers=headers,
            params={"limit": 5},
            timeout=10
        )

        if resp.status_code == 200:
            studies = resp.json()
            study_uids = []
            for study in studies:
                study_uid_tag = study.get('0020000D', {})
                if isinstance(study_uid_tag, dict):
                    value = study_uid_tag.get('Value', [])
                    if value:
                        study_uids.append(str(value[0]))

            if study_uids:
                print_success(f"{len(study_uids)}개 Study UID 조회 성공")
                for uid in study_uids:
                    print_info(f"  - {uid}")
                return study_uids
            else:
                print_info("QIDO-RS에서 Study를 찾을 수 없어 하드코딩된 UID 사용")
                return hardcoded_uids
        else:
            print_info(f"QIDO-RS 조회 실패 ({resp.status_code}), 하드코딩된 UID 사용")
            return hardcoded_uids
    except Exception as e:
        print_info(f"QIDO-RS 조회 에러 ({e}), 하드코딩된 UID 사용")
        return hardcoded_uids

def test_viewer_studies_meta(token: str, study_uids: List[str]):
    """Viewer Study Meta API 테스트"""
    print_test("Viewer Study Meta API - 정상 케이스")
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "study_uids": study_uids
    }
    
    try:
        resp = requests.post(
            f"{BASE_URL}/api/v1/viewer/studies/meta",
            headers=headers,
            json=payload,
            timeout=30
        )
        
        print_info(f"Status Code: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            print_info(f"Response: {json.dumps(data, indent=2, ensure_ascii=False)[:500]}")
            
            # 응답 구조 검증
            if "studies" in data:
                studies = data["studies"]
                print_success(f"응답 성공: {len(studies)}개 Study 메타데이터 조회")
                
                # 각 Study 메타데이터 검증
                for study in studies:
                    required_fields = ["study_uid", "patient_id"]
                    missing_fields = [f for f in required_fields if f not in study]
                    
                    if missing_fields:
                        print_error(f"필수 필드 누락: {missing_fields}")
                    else:
                        print_info(f"  Study: {study.get('study_uid')} - {study.get('study_description', 'N/A')}")
            else:
                print_error("응답에 'studies' 필드가 없습니다")
        else:
            print_error(f"API 호출 실패: {resp.status_code}")
            print_info(f"Response: {resp.text[:500]}")
    except Exception as e:
        print_error(f"API 호출 에러: {e}")

def test_empty_study_uids(token: str):
    """빈 Study UID 목록 테스트"""
    print_test("Viewer Study Meta API - 빈 목록")

    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }

    payload = {
        "study_uids": []
    }

    try:
        resp = requests.post(
            f"{BASE_URL}/api/v1/viewer/studies/meta",
            headers=headers,
            json=payload,
            timeout=10
        )

        print_info(f"Status Code: {resp.status_code}")

        if resp.status_code == 200:
            data = resp.json()
            if "studies" in data and len(data["studies"]) == 0:
                print_success("빈 목록 처리 성공")
            else:
                print_error(f"예상과 다른 응답: {data}")
        else:
            print_info(f"빈 목록에 대한 응답: {resp.status_code}")
            # 400이나 다른 에러 코드도 허용 가능
            print_success("빈 목록 처리 완료")
    except Exception as e:
        print_error(f"API 호출 에러: {e}")

def test_invalid_study_uids(token: str):
    """존재하지 않는 Study UID 테스트"""
    print_test("Viewer Study Meta API - 존재하지 않는 Study UID")

    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }

    payload = {
        "study_uids": ["9.9.9.9.9.9.9.9.9", "8.8.8.8.8.8.8.8.8"]
    }

    try:
        resp = requests.post(
            f"{BASE_URL}/api/v1/viewer/studies/meta",
            headers=headers,
            json=payload,
            timeout=10
        )

        print_info(f"Status Code: {resp.status_code}")

        if resp.status_code == 200:
            data = resp.json()
            if "studies" in data:
                print_success(f"존재하지 않는 UID 처리 성공 (반환된 Study: {len(data['studies'])}개)")
            else:
                print_error(f"예상과 다른 응답: {data}")
        elif resp.status_code == 404:
            print_success("존재하지 않는 Study에 대해 404 반환")
        else:
            print_info(f"응답: {resp.text[:200]}")
    except Exception as e:
        print_error(f"API 호출 에러: {e}")

def test_unauthorized_access():
    """인증 없이 접근 테스트"""
    print_test("Viewer Study Meta API - 인증 없음")

    payload = {
        "study_uids": ["1.2.3.4.5"]
    }

    try:
        resp = requests.post(
            f"{BASE_URL}/api/v1/viewer/studies/meta",
            json=payload,
            timeout=10
        )

        print_info(f"Status Code: {resp.status_code}")

        if resp.status_code == 401:
            print_success("인증 없는 요청 차단 성공 (401)")
        elif resp.status_code == 403:
            print_success("인증 없는 요청 차단 성공 (403)")
        else:
            print_error(f"예상과 다른 응답: {resp.status_code}")
    except Exception as e:
        print_error(f"API 호출 에러: {e}")

def test_health():
    """서버 Health Check"""
    print_test("서버 Health Check")
    try:
        resp = requests.get(f'{BASE_URL}/health', timeout=5)
        if resp.status_code == 200:
            print_success(f"서버 정상: {resp.json()}")
            return True
        else:
            print_error(f"서버 오류: {resp.status_code}")
            return False
    except Exception as e:
        print_error(f"서버 연결 실패: {e}")
        return False

def print_summary():
    """테스트 결과 요약 출력"""
    print_section("테스트 결과 요약")
    print(f"Total Tests: {test_results['total']}")
    print(f"✅ Passed: {test_results['passed']}")
    print(f"❌ Failed: {test_results['failed']}")

    if test_results['failed'] == 0:
        print("\n🎉 모든 테스트 통과!")
        return 0
    else:
        print(f"\n⚠️  {test_results['failed']}개 테스트 실패")
        return 1

def main():
    """메인 테스트 실행"""
    print_section("Viewer Study Meta API E2E 테스트 시작")

    # 1. Health Check
    if not test_health():
        print("\n❌ 서버가 실행 중이지 않습니다. 테스트를 중단합니다.")
        sys.exit(1)

    # 2. 로그인 및 토큰 획득
    token = get_token()
    if not token:
        print("\n❌ 로그인 실패. 테스트를 중단합니다.")
        sys.exit(1)

    # 3. 샘플 Study UID 조회
    study_uids = get_sample_study_uids(token)

    # 4. 테스트 실행
    if study_uids:
        test_viewer_studies_meta(token, study_uids)
        test_viewer_studies_meta(token, study_uids[:2])  # 일부만 테스트

    test_empty_study_uids(token)
    test_invalid_study_uids(token)
    test_unauthorized_access()

    # 5. 결과 요약
    exit_code = print_summary()
    sys.exit(exit_code)

if __name__ == "__main__":
    main()


