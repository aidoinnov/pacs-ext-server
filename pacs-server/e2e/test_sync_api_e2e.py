#!/usr/bin/env python3
"""
Sync API E2E Test

동기화 API의 모든 엔드포인트를 테스트합니다:
1. GET /api/sync/status - 동기화 상태 조회
2. POST /api/sync/run - 수동 동기화 실행
3. POST /api/sync/pause - 동기화 일시 정지
4. POST /api/sync/resume - 동기화 재개
5. GET /api/sync/schedule - 스케줄 조회
6. PUT /api/sync/schedule - 스케줄 업데이트
7. GET /api/sync/deps - 의존성 체크
"""

import requests
import time
from test_base import BaseE2ETest, TestConfig, TestPrinter


class SyncAPIE2ETest(BaseE2ETest):
    """Sync API E2E Test"""

    def __init__(self):
        super().__init__()
        self.original_interval = None

    def get_test_name(self) -> str:
        """테스트 이름 반환"""
        return "Sync API E2E Test"

    def setup(self):
        """테스트 환경 설정"""
        # 원래 interval 저장
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/sync/schedule",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        if response.status_code == 200:
            self.original_interval = response.json().get("interval_sec")
            TestPrinter.print_info(f"원래 interval: {self.original_interval}초")

    def cleanup(self):
        """테스트 데이터 정리"""
        # interval 복원
        if self.original_interval:
            try:
                TestPrinter.print_info(f"Interval 복원 중: {self.original_interval}초")
                requests.put(
                    f"{TestConfig.BASE_URL}/api/sync/schedule",
                    json={"interval_sec": self.original_interval},
                    timeout=TestConfig.DEFAULT_TIMEOUT
                )
                TestPrinter.print_success("Interval 복원 완료")
            except Exception as e:
                TestPrinter.print_warning(f"Cleanup 중 에러: {e}")

    def test_sync_status(self):
        """테스트 1: 동기화 상태 조회"""
        TestPrinter.print_header("테스트 1: 동기화 상태 조회")
        
        print("\n1️⃣  GET /api/sync/status 호출...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/sync/status",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"상태 조회 실패: {response.text}"
        
        data = response.json()
        
        # 필수 필드 확인
        assert "is_running" in data, "is_running 필드 없음"
        assert "interval_sec" in data, "interval_sec 필드 없음"
        
        TestPrinter.print_success("동기화 상태 조회 성공")
        TestPrinter.print_info(f"is_running: {data['is_running']}", indent=1)
        TestPrinter.print_info(f"last_run: {data.get('last_run', 'N/A')}", indent=1)
        TestPrinter.print_info(f"next_run: {data.get('next_run', 'N/A')}", indent=1)
        TestPrinter.print_info(f"interval_sec: {data['interval_sec']}", indent=1)

    def test_sync_run(self):
        """테스트 2: 수동 동기화 실행"""
        TestPrinter.print_header("테스트 2: 수동 동기화 실행")
        
        print("\n1️⃣  POST /api/sync/run 호출...")
        print("⏳ 동기화 실행 중 (최대 60초 대기)...")
        
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/sync/run",
            timeout=65  # 서버 타임아웃(60초)보다 약간 길게
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"동기화 실행 실패: {response.text}"
        
        data = response.json()
        
        # 필수 필드 확인
        assert "success" in data, "success 필드 없음"
        assert "processed" in data, "processed 필드 없음"
        assert "duration_ms" in data, "duration_ms 필드 없음"
        
        TestPrinter.print_success("동기화 실행 성공")
        TestPrinter.print_info(f"success: {data['success']}", indent=1)
        TestPrinter.print_info(f"processed: {data['processed']} items", indent=1)
        TestPrinter.print_info(f"duration: {data['duration_ms']} ms", indent=1)
        TestPrinter.print_info(f"error: {data.get('error', 'None')}", indent=1)
        
        # 성공 여부 확인
        assert data['success'] == True, f"동기화 실패: {data.get('error')}"
        
        # 응답 시간 확인 (60초 이내)
        assert data['duration_ms'] < 60000, f"동기화 시간 초과: {data['duration_ms']}ms"

    def test_sync_pause_resume(self):
        """테스트 3: 동기화 일시 정지 및 재개"""
        TestPrinter.print_header("테스트 3: 동기화 일시 정지 및 재개")
        
        print("\n1️⃣  POST /api/sync/pause 호출...")
        response = requests.post(
            f"{TestConfig.BASE_URL}/api/sync/pause",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"일시 정지 실패: {response.text}"
        
        TestPrinter.print_success("동기화 일시 정지 성공")
        
        print("\n2️⃣  POST /api/sync/resume 호출...")
        response2 = requests.post(
            f"{TestConfig.BASE_URL}/api/sync/resume",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )
        
        print(f"Status: {response2.status_code}")
        assert response2.status_code == 200, f"재개 실패: {response2.text}"
        
        TestPrinter.print_success("동기화 재개 성공")

    def test_sync_schedule(self):
        """테스트 4: 스케줄 조회 및 업데이트"""
        TestPrinter.print_header("테스트 4: 스케줄 조회 및 업데이트")

        print("\n1️⃣  GET /api/sync/schedule 호출...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/sync/schedule",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"스케줄 조회 실패: {response.text}"

        data = response.json()
        assert "interval_sec" in data, "interval_sec 필드 없음"

        original_interval = data["interval_sec"]
        TestPrinter.print_success(f"현재 interval: {original_interval}초")

        # 스케줄 업데이트
        new_interval = 600  # 10분
        print(f"\n2️⃣  PUT /api/sync/schedule 호출 (interval: {new_interval}초)...")
        response2 = requests.put(
            f"{TestConfig.BASE_URL}/api/sync/schedule",
            json={"interval_sec": new_interval},
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response2.status_code}")
        assert response2.status_code == 200, f"스케줄 업데이트 실패: {response2.text}"

        TestPrinter.print_success(f"스케줄 업데이트 성공 ({new_interval}초)")

        # 업데이트 확인
        print("\n3️⃣  업데이트 확인...")
        response3 = requests.get(
            f"{TestConfig.BASE_URL}/api/sync/schedule",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        data3 = response3.json()
        assert data3["interval_sec"] == new_interval, f"interval 업데이트 실패: {data3['interval_sec']}"

        TestPrinter.print_success(f"interval 업데이트 확인: {data3['interval_sec']}초")

        # 원래 값으로 복원
        print(f"\n4️⃣  원래 값으로 복원 ({original_interval}초)...")
        requests.put(
            f"{TestConfig.BASE_URL}/api/sync/schedule",
            json={"interval_sec": original_interval},
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        TestPrinter.print_success("원래 값으로 복원 완료")

    def test_sync_deps(self):
        """테스트 5: 의존성 체크"""
        TestPrinter.print_header("테스트 5: 의존성 체크")

        print("\n1️⃣  GET /api/sync/deps 호출...")
        response = requests.get(
            f"{TestConfig.BASE_URL}/api/sync/deps",
            timeout=TestConfig.DEFAULT_TIMEOUT
        )

        print(f"Status: {response.status_code}")
        assert response.status_code == 200, f"의존성 체크 실패: {response.text}"

        data = response.json()

        # 필수 필드 확인
        assert "state" in data, "state 필드 없음"
        assert "svc" in data, "svc 필드 없음"

        TestPrinter.print_success("의존성 체크 성공")
        TestPrinter.print_info(f"state: {data['state']}", indent=1)
        TestPrinter.print_info(f"svc: {data['svc']}", indent=1)
        TestPrinter.print_info(f"svc_arc: {data.get('svc_arc', 'N/A')}", indent=1)
        TestPrinter.print_info(f"svc_direct: {data.get('svc_direct', 'N/A')}", indent=1)

        # 의존성이 모두 주입되었는지 확인
        assert data['state'] == True, "SyncState가 주입되지 않음"
        assert data['svc'] == True, "SyncService가 주입되지 않음"

    def test_sync_multiple_runs(self):
        """테스트 6: 연속 동기화 실행"""
        TestPrinter.print_header("테스트 6: 연속 동기화 실행")

        print("\n1️⃣  첫 번째 동기화 실행...")
        response1 = requests.post(
            f"{TestConfig.BASE_URL}/api/sync/run",
            timeout=65
        )

        print(f"Status: {response1.status_code}")
        assert response1.status_code == 200, f"첫 번째 동기화 실패: {response1.text}"

        data1 = response1.json()
        TestPrinter.print_success(f"첫 번째 동기화 성공 ({data1['duration_ms']}ms)")

        # 잠시 대기
        print("\n⏳ 1초 대기...")
        time.sleep(1)

        print("\n2️⃣  두 번째 동기화 실행...")
        response2 = requests.post(
            f"{TestConfig.BASE_URL}/api/sync/run",
            timeout=65
        )

        print(f"Status: {response2.status_code}")
        assert response2.status_code == 200, f"두 번째 동기화 실패: {response2.text}"

        data2 = response2.json()
        TestPrinter.print_success(f"두 번째 동기화 성공 ({data2['duration_ms']}ms)")

        # 두 번째 실행은 변경사항이 없어서 더 빠를 수 있음
        TestPrinter.print_info(f"첫 번째: {data1['processed']} items, {data1['duration_ms']}ms", indent=1)
        TestPrinter.print_info(f"두 번째: {data2['processed']} items, {data2['duration_ms']}ms", indent=1)

    def run_tests(self):
        """테스트 실행"""
        self.test_sync_status()
        self.test_sync_run()
        self.test_sync_pause_resume()
        self.test_sync_schedule()
        self.test_sync_deps()
        self.test_sync_multiple_runs()


if __name__ == "__main__":
    test = SyncAPIE2ETest()
    test.run()

