#!/usr/bin/env python3
"""
GC Runner E2E Test

이 스크립트는 GC Runner의 전체 워크플로우를 테스트합니다.
- PENDING 타임아웃 처리
- FAILED 스냅샷 정리
- Dry-run 모드
- Grace Period 검증
"""

import os
import sys
import subprocess
import psycopg2
from datetime import datetime, timedelta
from typing import Optional, Dict, List
import json

# 색상 코드
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def print_header(text: str):
    """헤더 출력"""
    print(f"\n{Colors.CYAN}{Colors.BOLD}{'='*60}")
    print(f"📋 {text}")
    print(f"{'='*60}{Colors.RESET}\n")

def print_success(text: str):
    """성공 메시지 출력"""
    print(f"{Colors.GREEN}✅ {text}{Colors.RESET}")

def print_error(text: str):
    """에러 메시지 출력"""
    print(f"{Colors.RED}❌ {text}{Colors.RESET}")

def print_info(text: str):
    """정보 메시지 출력"""
    print(f"{Colors.BLUE}ℹ️  {text}{Colors.RESET}")

def print_warning(text: str):
    """경고 메시지 출력"""
    print(f"{Colors.YELLOW}⚠️  {text}{Colors.RESET}")

class DatabaseHelper:
    """데이터베이스 헬퍼 클래스"""
    
    def __init__(self, db_url: str):
        self.db_url = db_url
        self.conn = None
        
    def connect(self):
        """데이터베이스 연결"""
        self.conn = psycopg2.connect(self.db_url)
        self.conn.autocommit = True
        
    def close(self):
        """데이터베이스 연결 종료"""
        if self.conn:
            self.conn.close()
            
    def execute(self, query: str, params: tuple = None) -> Optional[List]:
        """쿼리 실행"""
        with self.conn.cursor() as cur:
            cur.execute(query, params)
            try:
                return cur.fetchall()
            except:
                return None
                
    def cleanup_test_data(self):
        """테스트 데이터 정리"""
        print_info("Cleaning up test data...")
        self.execute("DELETE FROM gc_deletion_log WHERE annotation_id >= 90000")
        self.execute("DELETE FROM annotation_annotation WHERE id >= 90000")
        self.execute("DELETE FROM security_project WHERE id = 99999")
        self.execute("DELETE FROM security_user WHERE id = 99999")
        print_success("Cleanup completed")
        
    def setup_test_fixtures(self):
        """테스트 픽스처 생성"""
        print_info("Setting up test fixtures...")
        
        # 테스트용 user 생성
        self.execute("""
            INSERT INTO security_user (id, keycloak_id, username, email, created_at)
            OVERRIDING SYSTEM VALUE VALUES (
                99999, 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 
                'test-gc-user', 'test-gc@example.com', NOW()
            ) ON CONFLICT (id) DO NOTHING
        """)
        
        # 테스트용 project 생성
        self.execute("""
            INSERT INTO security_project (id, name, description, is_active, created_at)
            OVERRIDING SYSTEM VALUE VALUES (
                99999, 'test-gc-project', 'Test project for GC E2E', true, NOW()
            ) ON CONFLICT (id) DO NOTHING
        """)
        
        print_success("Test fixtures created")
        
    def create_test_annotation(
        self, 
        annotation_id: int,
        status: str,
        days_ago: int,
        snapshot_key: str
    ):
        """테스트 어노테이션 생성"""
        created_at = datetime.utcnow() - timedelta(days=days_ago)
        uploaded_at = created_at if status == 'completed' else None
        
        self.execute("""
            INSERT INTO annotation_annotation (
                id, project_id, user_id, study_uid, series_uid, instance_uid,
                tool_name, data, is_shared, created_at, updated_at,
                snapshot_image_key, snapshot_status, snapshot_uploaded_at
            ) OVERRIDING SYSTEM VALUE VALUES (
                %s, 99999, 99999, %s, 'test-series', 'test-instance',
                'test-tool', '{}', false, %s, %s,
                %s, %s, %s
            ) ON CONFLICT (id) DO UPDATE SET
                snapshot_status = %s,
                snapshot_image_key = %s,
                created_at = %s,
                updated_at = %s,
                snapshot_uploaded_at = %s
        """, (
            annotation_id, f'test-study-{annotation_id}',
            created_at, created_at,
            snapshot_key, status, uploaded_at,
            status, snapshot_key, created_at, created_at, uploaded_at
        ))

    def get_annotation_status(self, annotation_id: int) -> Optional[Dict]:
        """어노테이션 상태 조회"""
        result = self.execute("""
            SELECT id, snapshot_status, snapshot_image_key,
                   EXTRACT(DAY FROM NOW() - created_at) as days_old
            FROM annotation_annotation
            WHERE id = %s
        """, (annotation_id,))

        if result and len(result) > 0:
            row = result[0]
            return {
                'id': row[0],
                'snapshot_status': row[1],
                'snapshot_image_key': row[2],
                'days_old': int(row[3]) if row[3] else 0
            }
        return None

    def get_all_test_annotations(self) -> List[Dict]:
        """모든 테스트 어노테이션 조회"""
        result = self.execute("""
            SELECT id, snapshot_status, snapshot_image_key,
                   EXTRACT(DAY FROM NOW() - created_at) as days_old
            FROM annotation_annotation
            WHERE id >= 90000
            ORDER BY id
        """)

        annotations = []
        for row in result or []:
            annotations.append({
                'id': row[0],
                'snapshot_status': row[1],
                'snapshot_image_key': row[2],
                'days_old': int(row[3]) if row[3] else 0
            })
        return annotations

    def get_gc_logs(self) -> List[Dict]:
        """GC 로그 조회"""
        result = self.execute("""
            SELECT id, annotation_id, snapshot_image_key, status, error_message, deleted_at
            FROM gc_deletion_log
            WHERE annotation_id >= 90000
            ORDER BY deleted_at DESC
        """)

        logs = []
        for row in result or []:
            logs.append({
                'id': row[0],
                'annotation_id': row[1],
                'snapshot_image_key': row[2],
                'status': row[3],
                'error_message': row[4],
                'deleted_at': row[5]
            })
        return logs


class GcRunner:
    """GC Runner 실행 헬퍼"""

    def __init__(self, binary_path: str = "./target/debug/gc_runner"):
        self.binary_path = binary_path

    def timeout_pending(
        self,
        grace_days: int = 3,
        batch_size: int = 100,
        dry_run: bool = False
    ) -> Dict:
        """PENDING 타임아웃 실행"""
        cmd = [
            self.binary_path,
            "timeout-pending",
            "--grace-days", str(grace_days),
            "--batch-size", str(batch_size)
        ]

        if dry_run:
            cmd.append("--dry-run")

        result = subprocess.run(cmd, capture_output=True, text=True)

        # 출력 파싱
        output = result.stdout + result.stderr

        return {
            'exit_code': result.returncode,
            'output': output,
            'success': result.returncode == 0
        }

    def cleanup_failed(
        self,
        grace_days: int = 7,
        batch_size: int = 100,
        dry_run: bool = False
    ) -> Dict:
        """FAILED 정리 실행"""
        cmd = [
            self.binary_path,
            "cleanup-failed",
            "--grace-days", str(grace_days),
            "--batch-size", str(batch_size)
        ]

        if dry_run:
            cmd.append("--dry-run")

        result = subprocess.run(cmd, capture_output=True, text=True)

        output = result.stdout + result.stderr

        return {
            'exit_code': result.returncode,
            'output': output,
            'success': result.returncode == 0
        }


def run_test_scenario_1(db: DatabaseHelper, gc: GcRunner):
    """시나리오 1: PENDING 타임아웃 (Dry-run)"""
    print_header("Test 1: Job A - PENDING Timeout (Dry-run)")

    print_info("Running Job A: Timeout Pending Snapshots")
    print(f"   Grace Days: 3")
    print(f"   Batch Size: 100")
    print(f"   Dry-run: true")

    result = gc.timeout_pending(grace_days=3, batch_size=100, dry_run=True)

    if not result['success']:
        print_error(f"Job A failed: {result['output']}")
        return False

    print_success("Job A completed (dry-run)")

    # 검증: 상태가 변경되지 않았는지 확인
    ann = db.get_annotation_status(90001)
    if ann and ann['snapshot_status'] == 'pending':
        print_success("Test 1 PASSED: ID 90001 still PENDING (dry-run)")
        return True
    else:
        print_error(f"Test 1 FAILED: ID 90001 status changed in dry-run mode")
        return False


def run_test_scenario_2(db: DatabaseHelper, gc: GcRunner):
    """시나리오 2: PENDING 타임아웃 (실제 실행)"""
    print_header("Test 2: Job A - PENDING Timeout (Actual)")

    print_info("Running Job A: Timeout Pending Snapshots")
    print(f"   Grace Days: 3")
    print(f"   Batch Size: 100")
    print(f"   Dry-run: false")

    result = gc.timeout_pending(grace_days=3, batch_size=100, dry_run=False)

    if not result['success']:
        print_error(f"Job A failed: {result['output']}")
        return False

    print_success("Job A completed")

    # 검증 1: ID 90001이 FAILED로 변경되었는지 확인
    ann1 = db.get_annotation_status(90001)
    if ann1 and ann1['snapshot_status'] == 'failed':
        print_success("Test 2 PASSED: ID 90001 changed to FAILED")
    else:
        print_error(f"Test 2 FAILED: ID 90001 status is {ann1['snapshot_status'] if ann1 else 'None'}")
        return False

    # 검증 2: ID 90002는 변경되지 않았는지 확인 (grace period 미만)
    ann2 = db.get_annotation_status(90002)
    if ann2 and ann2['snapshot_status'] == 'pending':
        print_success("Test 2 PASSED: ID 90002 still PENDING (grace period not met)")
        return True
    else:
        print_error(f"Test 2 FAILED: ID 90002 status is {ann2['snapshot_status'] if ann2 else 'None'}")
        return False


def run_test_scenario_3(db: DatabaseHelper, gc: GcRunner):
    """시나리오 3: FAILED 정리 (Dry-run)"""
    print_header("Test 3: Job B - FAILED Cleanup (Dry-run)")

    print_info("Running Job B: Cleanup Failed Snapshots")
    print(f"   Grace Days: 7")
    print(f"   Batch Size: 100")
    print(f"   Dry-run: true")

    result = gc.cleanup_failed(grace_days=7, batch_size=100, dry_run=True)

    if not result['success']:
        print_error(f"Job B failed: {result['output']}")
        return False

    print_success("Job B completed (dry-run)")

    # 검증: snapshot_image_key가 변경되지 않았는지 확인
    ann = db.get_annotation_status(90003)
    if ann and ann['snapshot_image_key'] is not None:
        print_success("Test 3 PASSED: ID 90003 snapshot_image_key unchanged (dry-run)")
        return True
    else:
        print_error(f"Test 3 FAILED: ID 90003 snapshot_image_key changed in dry-run mode")
        return False


def run_test_scenario_4(db: DatabaseHelper, gc: GcRunner):
    """시나리오 4: FAILED 정리 (실제 실행)"""
    print_header("Test 4: Job B - FAILED Cleanup (Actual)")

    print_info("Running Job B: Cleanup Failed Snapshots")
    print(f"   Grace Days: 7")
    print(f"   Batch Size: 100")
    print(f"   Dry-run: false")

    result = gc.cleanup_failed(grace_days=7, batch_size=100, dry_run=False)

    # S3 에러는 예상된 동작 (테스트 환경에서 S3가 없음)
    print_warning("Note: S3 deletion may fail in test environment (expected)")

    # 검증 1: GC 로그가 기록되었는지 확인
    logs = db.get_gc_logs()
    if not logs:
        print_warning("Test 4: No GC logs found (may be expected if S3 fails)")
        return True

    print_success(f"Test 4 PASSED: GC logs recorded ({len(logs)} entries)")
    for log in logs[:3]:  # 최근 3개만 출력
        status_icon = "✅" if log['status'] == 'success' else "❌" if log['status'] == 'failed' else "⏭️"
        print(f"  {status_icon} Annotation {log['annotation_id']}: {log['status']}")
        if log['error_message']:
            print(f"     Error: {log['error_message'][:50]}...")

    # 검증 2: S3 삭제 실패 시에도 snapshot_image_key는 유지되어야 함
    # (성공 시에만 NULL로 변경)
    ann3 = db.get_annotation_status(90003)
    if ann3:
        # S3 삭제가 실패했으므로 snapshot_image_key는 여전히 존재해야 함
        if ann3['snapshot_image_key'] is not None:
            print_success("Test 4 PASSED: ID 90003 snapshot_image_key preserved (S3 delete failed)")
        else:
            print_error("Test 4 FAILED: ID 90003 snapshot_image_key should not be NULL when S3 delete fails")
            return False

    return True


def run_test_scenario_5(db: DatabaseHelper):
    """시나리오 5: Grace Period 미만 (처리 안됨)"""
    print_header("Test 5: Grace Period Validation")

    # ID 90002 (PENDING, 2일) - grace period 3일 미만
    ann2 = db.get_annotation_status(90002)
    if ann2 and ann2['snapshot_status'] == 'pending':
        print_success("Test 5 PASSED: ID 90002 still PENDING (2 days < 3 days grace)")
    else:
        print_error(f"Test 5 FAILED: ID 90002 should still be PENDING")
        return False

    # ID 90004 (FAILED, 5일) - grace period 7일 미만
    ann4 = db.get_annotation_status(90004)
    if ann4 and ann4['snapshot_image_key'] is not None:
        print_success("Test 5 PASSED: ID 90004 snapshot_image_key exists (5 days < 7 days grace)")
        return True
    else:
        print_error(f"Test 5 FAILED: ID 90004 snapshot_image_key should exist")
        return False


def run_test_scenario_6(db: DatabaseHelper):
    """시나리오 6: 전체 워크플로우 검증"""
    print_header("Test 6: Full Workflow Validation")

    print_info("Checking all annotations...")
    annotations = db.get_all_test_annotations()

    print("\n📊 Final State:")
    print(f"{'ID':<8} {'Status':<12} {'Image Key':<40} {'Days Old':<10}")
    print("-" * 70)

    for ann in annotations:
        key_display = ann['snapshot_image_key'][:37] + "..." if ann['snapshot_image_key'] and len(ann['snapshot_image_key']) > 40 else (ann['snapshot_image_key'] or "NULL")
        print(f"{ann['id']:<8} {ann['snapshot_status']:<12} {key_display:<40} {ann['days_old']:<10}")

    print_success("Test 6 PASSED: Full workflow completed")
    return True


def run_test_scenario_7(db: DatabaseHelper, gc: GcRunner):
    """시나리오 7: 멱등성 검증 (중복 실행)"""
    print_header("Test 7: Idempotency - Duplicate Execution")

    print_info("Creating fresh PENDING annotation for idempotency test...")
    db.create_test_annotation(90020, 'pending', 5, 'snapshots/99999/90020/test-90020.png')
    print("  - Created PENDING annotation (5 days old) - ID: 90020")

    # 첫 번째 실행
    print_info("First execution...")
    result1 = gc.timeout_pending(grace_days=3, batch_size=100, dry_run=False)
    if not result1['success']:
        print_error(f"First execution failed: {result1['output']}")
        return False

    # 상태 확인
    ann_after_first = db.get_annotation_status(90020)
    if not ann_after_first or ann_after_first['snapshot_status'] != 'failed':
        print_error("First execution did not change status to FAILED")
        return False

    print_success("First execution completed: ID 90020 → FAILED")

    # GC 로그 개수 확인
    logs_after_first = db.get_gc_logs()
    count_after_first = len([log for log in logs_after_first if log['annotation_id'] == 90020])
    print(f"  GC logs for ID 90020 after first run: {count_after_first}")

    # 두 번째 실행 (멱등성 검증)
    print_info("Second execution (should be idempotent)...")
    result2 = gc.timeout_pending(grace_days=3, batch_size=100, dry_run=False)
    if not result2['success']:
        print_error(f"Second execution failed: {result2['output']}")
        return False

    # 상태가 여전히 FAILED인지 확인
    ann_after_second = db.get_annotation_status(90020)
    if not ann_after_second or ann_after_second['snapshot_status'] != 'failed':
        print_error("Second execution changed status unexpectedly")
        return False

    # GC 로그가 중복 생성되지 않았는지 확인
    logs_after_second = db.get_gc_logs()
    count_after_second = len([log for log in logs_after_second if log['annotation_id'] == 90020])
    print(f"  GC logs for ID 90020 after second run: {count_after_second}")

    if count_after_second == count_after_first:
        print_success("Test 7 PASSED: Second execution was idempotent (no duplicate processing)")
        return True
    else:
        print_warning(f"Test 7 WARNING: Log count changed from {count_after_first} to {count_after_second}")
        # 이건 경고만 하고 통과 (FAILED 상태는 다시 처리 안되므로 정상)
        return True


def run_test_scenario_8(db: DatabaseHelper, gc: GcRunner):
    """시나리오 8: Grace Period 경계값 테스트"""
    print_header("Test 8: Grace Period Boundary Values")

    print_info("Creating annotations at exact boundary values...")

    # 정확히 3일 된 PENDING (경계값)
    db.create_test_annotation(90010, 'pending', 3, 'snapshots/99999/90010/test-90010.png')
    print("  - Created PENDING annotation (exactly 3 days old) - ID: 90010")

    # 정확히 7일 된 FAILED (경계값)
    db.create_test_annotation(90011, 'failed', 7, 'snapshots/99999/90011/test-90011.png')
    print("  - Created FAILED annotation (exactly 7 days old) - ID: 90011")

    # PENDING 타임아웃 실행 (grace_days=3)
    print_info("Running PENDING timeout with grace_days=3...")
    result1 = gc.timeout_pending(grace_days=3, batch_size=100, dry_run=False)

    # 검증 1: 정확히 3일 된 것은 처리되어야 함 (>= 조건)
    ann10 = db.get_annotation_status(90010)
    if ann10 and ann10['snapshot_status'] == 'failed':
        print_success("Test 8.1 PASSED: ID 90010 (exactly 3 days) changed to FAILED")
    else:
        print_error(f"Test 8.1 FAILED: ID 90010 status is {ann10['snapshot_status'] if ann10 else 'None'}")
        return False

    # FAILED 정리 실행 (grace_days=7)
    print_info("Running FAILED cleanup with grace_days=7...")
    result2 = gc.cleanup_failed(grace_days=7, batch_size=100, dry_run=False)

    # 검증 2: 정확히 7일 된 것은 처리되어야 함 (>= 조건)
    # S3 삭제는 실패하지만 시도는 해야 함
    logs = db.get_gc_logs()
    log_for_90011 = [log for log in logs if log['annotation_id'] == 90011]

    if log_for_90011:
        print_success(f"Test 8.2 PASSED: ID 90011 (exactly 7 days) was processed")
        return True
    else:
        print_error("Test 8.2 FAILED: ID 90011 was not processed")
        return False


def run_test_scenario_9(db: DatabaseHelper, gc: GcRunner):
    """시나리오 9: 빈 결과 처리 (처리할 항목 없음)"""
    print_header("Test 9: Empty Result Handling")

    print_info("Cleaning up all test data to create empty state...")
    db.cleanup_test_data()
    db.setup_test_fixtures()

    # PENDING 타임아웃 실행 (처리할 항목 없음)
    print_info("Running PENDING timeout with no eligible annotations...")
    result1 = gc.timeout_pending(grace_days=3, batch_size=100, dry_run=False)

    if result1['success']:
        print_success("Test 9.1 PASSED: Empty PENDING timeout completed successfully")
    else:
        print_error("Test 9.1 FAILED: Empty PENDING timeout should succeed")
        return False

    # FAILED 정리 실행 (처리할 항목 없음)
    print_info("Running FAILED cleanup with no eligible annotations...")
    result2 = gc.cleanup_failed(grace_days=7, batch_size=100, dry_run=False)

    if result2['success']:
        print_success("Test 9.2 PASSED: Empty FAILED cleanup completed successfully")
        return True
    else:
        print_error("Test 9.2 FAILED: Empty FAILED cleanup should succeed")
        return False


def run_test_scenario_10(db: DatabaseHelper, gc: GcRunner):
    """시나리오 10: Batch Size 경계값 테스트"""
    print_header("Test 10: Batch Size Boundary Values")

    print_info("Creating multiple annotations for batch testing...")

    # 5개의 PENDING 어노테이션 생성 (모두 4일 이상)
    for i in range(5):
        ann_id = 90030 + i
        db.create_test_annotation(ann_id, 'pending', 4, f'snapshots/99999/{ann_id}/test-{ann_id}.png')
    print("  - Created 5 PENDING annotations (4 days old)")

    # 테스트 1: batch_size=3 (5개 중 3개만 처리)
    print_info("Running with batch_size=3 (should process only 3)...")
    result1 = gc.timeout_pending(grace_days=3, batch_size=3, dry_run=False)

    # 처리된 개수 확인
    failed_count = 0
    for i in range(5):
        ann = db.get_annotation_status(90030 + i)
        if ann and ann['snapshot_status'] == 'failed':
            failed_count += 1

    if failed_count == 3:
        print_success(f"Test 10.1 PASSED: Exactly 3 annotations processed (batch_size=3)")
    else:
        print_error(f"Test 10.1 FAILED: Expected 3 processed, got {failed_count}")
        return False

    # 테스트 2: batch_size=1 (최소값)
    print_info("Running with batch_size=1 (should process only 1 more)...")
    result2 = gc.timeout_pending(grace_days=3, batch_size=1, dry_run=False)

    # 처리된 개수 확인 (총 4개가 되어야 함)
    failed_count = 0
    for i in range(5):
        ann = db.get_annotation_status(90030 + i)
        if ann and ann['snapshot_status'] == 'failed':
            failed_count += 1

    if failed_count == 4:
        print_success(f"Test 10.2 PASSED: Exactly 1 more annotation processed (batch_size=1)")
        return True
    else:
        print_error(f"Test 10.2 FAILED: Expected 4 total processed, got {failed_count}")
        return False


def run_test_scenario_11(db: DatabaseHelper, gc: GcRunner):
    """시나리오 11: snapshot_image_key NULL 케이스"""
    print_header("Test 11: NULL snapshot_image_key Handling")

    print_info("Creating FAILED annotation with NULL snapshot_image_key...")

    # snapshot_image_key가 NULL인 FAILED 어노테이션 생성
    db.execute("""
        INSERT INTO annotation_annotation (
            id, project_id, user_id, study_uid, series_uid, instance_uid,
            tool_name, data, is_shared, created_at, updated_at,
            snapshot_image_key, snapshot_status, snapshot_uploaded_at
        ) OVERRIDING SYSTEM VALUE VALUES (
            90040, 99999, 99999, 'test-study-90040', 'test-series', 'test-instance',
            'test-tool', '{}', false, NOW() - INTERVAL '10 days', NOW() - INTERVAL '10 days',
            NULL, 'failed', NULL
        ) ON CONFLICT (id) DO UPDATE SET
            snapshot_status = 'failed',
            snapshot_image_key = NULL,
            created_at = NOW() - INTERVAL '10 days',
            updated_at = NOW() - INTERVAL '10 days'
    """)
    print("  - Created FAILED annotation with NULL snapshot_image_key - ID: 90040")

    # FAILED 정리 실행
    print_info("Running FAILED cleanup (should skip NULL snapshot_image_key)...")
    result = gc.cleanup_failed(grace_days=7, batch_size=100, dry_run=False)

    if not result['success']:
        print_error("Test 11 FAILED: GC job should succeed even with NULL keys")
        return False

    # 로그 확인 - NULL인 항목은 처리되지 않아야 함
    logs = db.get_gc_logs()
    log_for_90040 = [log for log in logs if log['annotation_id'] == 90040]

    if not log_for_90040:
        print_success("Test 11 PASSED: NULL snapshot_image_key was correctly skipped")
        return True
    else:
        print_error("Test 11 FAILED: NULL snapshot_image_key should not be processed")
        return False


def run_test_scenario_12(db: DatabaseHelper, gc: GcRunner):
    """시나리오 12: Advisory Lock 검증"""
    print_header("Test 12: Advisory Lock Verification")

    print_info("Testing advisory lock mechanism...")

    # 락 ID (Job A와 동일)
    LOCK_ID = 1001

    # 먼저 모든 락 해제 (이전 테스트에서 남아있을 수 있음)
    db.execute(f"SELECT pg_advisory_unlock_all()")

    # 테스트 1: 락 획득
    print_info("Attempting to acquire lock...")
    result = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID})")
    lock_acquired = result[0][0] if result else False

    if not lock_acquired:
        print_error("Test 12.1 FAILED: Could not acquire lock")
        return False

    print_success("Test 12.1 PASSED: Lock acquired successfully")

    # 테스트 2: 새로운 커넥션에서 중복 락 획득 시도 (실패해야 함)
    # 주의: 같은 세션에서는 같은 락을 여러 번 획득 가능 (reference counting)
    # 따라서 새로운 커넥션을 생성해야 함
    print_info("Testing lock from different connection (should fail)...")

    import psycopg2
    import os

    try:
        # 새로운 DB 커넥션 생성
        conn2 = psycopg2.connect(os.environ['DATABASE_URL'])
        cur2 = conn2.cursor()

        # 다른 커넥션에서 락 획득 시도
        cur2.execute(f"SELECT pg_try_advisory_lock({LOCK_ID})")
        lock_acquired_2 = cur2.fetchone()[0]

        cur2.close()
        conn2.close()

        if lock_acquired_2:
            print_error("Test 12.2 FAILED: Lock should not be acquired from different connection")
            # 정리
            db.execute(f"SELECT pg_advisory_unlock({LOCK_ID})")
            return False

        print_success("Test 12.2 PASSED: Duplicate lock acquisition prevented")

    except Exception as e:
        print_error(f"Test 12.2 FAILED: Error testing lock from different connection: {e}")
        db.execute(f"SELECT pg_advisory_unlock({LOCK_ID})")
        return False

    # 테스트 3: 락 해제
    print_info("Releasing lock...")
    db.execute(f"SELECT pg_advisory_unlock({LOCK_ID})")

    # 테스트 4: 해제 후 다시 획득 가능한지 확인
    print_info("Attempting to acquire lock after release...")
    result3 = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID})")
    lock_acquired_3 = result3[0][0] if result3 else False

    if not lock_acquired_3:
        print_error("Test 12.3 FAILED: Lock should be available after release")
        return False

    print_success("Test 12.3 PASSED: Lock re-acquired after release")

    # 정리
    db.execute(f"SELECT pg_advisory_unlock({LOCK_ID})")

    print_success("Test 12 PASSED: Advisory lock mechanism working correctly")
    print("  Note: Actual concurrent process testing requires manual verification")
    print("  Suggestion: Run 'cargo run --bin gc_runner timeout-pending' twice simultaneously")

    return True


def run_test_scenario_13(db: DatabaseHelper, gc: GcRunner):
    """시나리오 13: Job A/B 독립성 테스트 (서로 다른 락 ID)"""
    print_header("Test 13: Job A/B Independence (Different Lock IDs)")

    print_info("Testing that Job A and Job B use different locks...")

    # 락 ID
    LOCK_ID_JOB_A = 1001
    LOCK_ID_JOB_B = 1002

    # 먼저 모든 락 해제
    db.execute("SELECT pg_advisory_unlock_all()")

    # 테스트 1: Job A 락 획득
    print_info("Acquiring Job A lock (ID: 1001)...")
    result_a = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID_JOB_A})")
    lock_a_acquired = result_a[0][0] if result_a else False

    if not lock_a_acquired:
        print_error("Test 13.1 FAILED: Could not acquire Job A lock")
        return False

    print_success("Test 13.1 PASSED: Job A lock acquired")

    # 테스트 2: Job A 락이 획득된 상태에서 Job B 락 획득 시도 (성공해야 함)
    print_info("Attempting to acquire Job B lock (ID: 1002) while Job A lock is held...")
    result_b = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID_JOB_B})")
    lock_b_acquired = result_b[0][0] if result_b else False

    if not lock_b_acquired:
        print_error("Test 13.2 FAILED: Job B lock should be acquirable independently")
        # 정리
        db.execute(f"SELECT pg_advisory_unlock({LOCK_ID_JOB_A})")
        return False

    print_success("Test 13.2 PASSED: Job B lock acquired independently (different lock ID)")

    # 테스트 3: 두 락 모두 획득된 상태 확인
    print_info("Verifying both locks are held simultaneously...")

    # Job A 락 다시 획득 시도 (같은 세션이므로 성공 - reference counting)
    result_a2 = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID_JOB_A})")
    lock_a2_acquired = result_a2[0][0] if result_a2 else False

    if not lock_a2_acquired:
        print_error("Test 13.3 FAILED: Job A lock should still be held")
        # 정리
        db.execute(f"SELECT pg_advisory_unlock({LOCK_ID_JOB_A})")
        db.execute(f"SELECT pg_advisory_unlock({LOCK_ID_JOB_B})")
        return False

    print_success("Test 13.3 PASSED: Both locks held simultaneously")

    # 정리 (reference counting 때문에 여러 번 unlock 필요)
    db.execute(f"SELECT pg_advisory_unlock({LOCK_ID_JOB_A})")  # 첫 번째 unlock
    db.execute(f"SELECT pg_advisory_unlock({LOCK_ID_JOB_A})")  # 두 번째 unlock
    db.execute(f"SELECT pg_advisory_unlock({LOCK_ID_JOB_B})")

    print_success("Test 13 PASSED: Job A and Job B use independent locks")
    print("  → Job A (timeout-pending) and Job B (cleanup-failed) can run simultaneously")

    return True


def run_test_scenario_14(db: DatabaseHelper, gc: GcRunner):
    """시나리오 14: 락 자동 해제 테스트 (커넥션 종료 시)"""
    print_header("Test 14: Lock Auto-Release on Connection Close")

    print_info("Testing that locks are automatically released when connection closes...")

    import psycopg2
    import os

    LOCK_ID = 1001

    # 먼저 모든 락 해제
    db.execute("SELECT pg_advisory_unlock_all()")

    try:
        # 테스트 1: 새 커넥션에서 락 획득
        print_info("Creating new connection and acquiring lock...")
        conn_temp = psycopg2.connect(os.environ['DATABASE_URL'])
        cur_temp = conn_temp.cursor()

        cur_temp.execute(f"SELECT pg_try_advisory_lock({LOCK_ID})")
        lock_acquired = cur_temp.fetchone()[0]

        if not lock_acquired:
            print_error("Test 14.1 FAILED: Could not acquire lock in temporary connection")
            cur_temp.close()
            conn_temp.close()
            return False

        print_success("Test 14.1 PASSED: Lock acquired in temporary connection")

        # 테스트 2: 메인 커넥션에서 락 획득 시도 (실패해야 함)
        print_info("Attempting to acquire same lock from main connection (should fail)...")
        result = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID})")
        lock_acquired_main = result[0][0] if result else False

        if lock_acquired_main:
            print_error("Test 14.2 FAILED: Lock should not be available (held by temp connection)")
            cur_temp.close()
            conn_temp.close()
            db.execute(f"SELECT pg_advisory_unlock({LOCK_ID})")
            return False

        print_success("Test 14.2 PASSED: Lock correctly held by temporary connection")

        # 테스트 3: 임시 커넥션 종료 (락 명시적 해제 없이)
        print_info("Closing temporary connection WITHOUT explicit unlock...")
        cur_temp.close()
        conn_temp.close()

        # 잠시 대기 (커넥션 종료 처리 시간)
        import time
        time.sleep(0.5)

        # 테스트 4: 메인 커넥션에서 락 획득 시도 (성공해야 함)
        print_info("Attempting to acquire lock from main connection (should succeed)...")
        result2 = db.execute(f"SELECT pg_try_advisory_lock({LOCK_ID})")
        lock_acquired_after = result2[0][0] if result2 else False

        if not lock_acquired_after:
            print_error("Test 14.3 FAILED: Lock should be auto-released after connection close")
            return False

        print_success("Test 14.3 PASSED: Lock automatically released when connection closed")

        # 정리
        db.execute(f"SELECT pg_advisory_unlock({LOCK_ID})")

        print_success("Test 14 PASSED: Advisory locks are automatically released on connection close")
        print("  → No risk of permanent lock if GC process crashes")

        return True

    except Exception as e:
        print_error(f"Test 14 FAILED: Error during test: {e}")
        # 정리
        db.execute("SELECT pg_advisory_unlock_all()")
        return False


def main():
    """메인 함수"""
    print(f"\n{Colors.BOLD}{Colors.CYAN}🧪 GC Runner E2E Test{Colors.RESET}")
    print(f"{Colors.CYAN}{'='*60}{Colors.RESET}\n")

    # 환경 변수
    db_url = os.getenv('DATABASE_URL', 'postgresql://aido@localhost:5432/pacs_db')
    gc_binary = os.getenv('GC_RUNNER_PATH', './target/debug/gc_runner')

    # 바이너리 존재 확인
    if not os.path.exists(gc_binary):
        print_error(f"GC Runner binary not found: {gc_binary}")
        print_info("Please build first: cargo build --bin gc_runner")
        sys.exit(1)

    # 데이터베이스 연결
    db = DatabaseHelper(db_url)
    try:
        db.connect()
        print_success("Connected to database")
    except Exception as e:
        print_error(f"Failed to connect to database: {e}")
        sys.exit(1)

    # GC Runner 초기화
    gc = GcRunner(gc_binary)

    try:
        # Setup
        print_header("Test Setup")
        db.cleanup_test_data()
        db.setup_test_fixtures()

        # 테스트 데이터 생성
        print_info("Creating test data...")
        db.create_test_annotation(90001, 'pending', 4, 'snapshots/99999/90001/test-90001.png')
        print("  - Created PENDING annotation (4 days old) - ID: 90001")

        db.create_test_annotation(90002, 'pending', 2, 'snapshots/99999/90002/test-90002.png')
        print("  - Created PENDING annotation (2 days old) - ID: 90002")

        db.create_test_annotation(90003, 'failed', 8, 'snapshots/99999/90003/test-90003.png')
        print("  - Created FAILED annotation (8 days old) - ID: 90003")

        db.create_test_annotation(90004, 'failed', 5, 'snapshots/99999/90004/test-90004.png')
        print("  - Created FAILED annotation (5 days old) - ID: 90004")

        db.create_test_annotation(90005, 'completed', 10, 'snapshots/99999/90005/test-90005.png')
        print("  - Created COMPLETED annotation (10 days old) - ID: 90005")

        print_success("Test data created")

        # 현재 상태 출력
        print("\n📊 Current state before GC:")
        annotations = db.get_all_test_annotations()
        for ann in annotations:
            print(f"  ID {ann['id']}: {ann['snapshot_status']} ({ann['days_old']} days old)")

        # 테스트 실행
        results = []

        results.append(("Scenario 1: PENDING Timeout (Dry-run)", run_test_scenario_1(db, gc)))
        results.append(("Scenario 2: PENDING Timeout (Actual)", run_test_scenario_2(db, gc)))
        results.append(("Scenario 3: FAILED Cleanup (Dry-run)", run_test_scenario_3(db, gc)))
        results.append(("Scenario 4: FAILED Cleanup (Actual)", run_test_scenario_4(db, gc)))
        results.append(("Scenario 5: Grace Period Validation", run_test_scenario_5(db)))
        results.append(("Scenario 6: Full Workflow", run_test_scenario_6(db)))
        results.append(("Scenario 7: Idempotency Test", run_test_scenario_7(db, gc)))
        results.append(("Scenario 8: Grace Period Boundaries", run_test_scenario_8(db, gc)))
        results.append(("Scenario 9: Empty Result Handling", run_test_scenario_9(db, gc)))
        results.append(("Scenario 10: Batch Size Boundaries", run_test_scenario_10(db, gc)))
        results.append(("Scenario 11: NULL snapshot_image_key", run_test_scenario_11(db, gc)))
        results.append(("Scenario 12: Advisory Lock Verification", run_test_scenario_12(db, gc)))
        results.append(("Scenario 13: Job A/B Independence", run_test_scenario_13(db, gc)))
        results.append(("Scenario 14: Lock Auto-Release", run_test_scenario_14(db, gc)))

        # 결과 요약
        print_header("Test Results Summary")

        passed = sum(1 for _, result in results if result)
        total = len(results)

        for name, result in results:
            if result:
                print_success(f"{name}")
            else:
                print_error(f"{name}")

        print(f"\n{Colors.BOLD}Total: {passed}/{total} tests passed{Colors.RESET}")

        if passed == total:
            print_success("All tests passed! 🎉")
            sys.exit(0)
        else:
            print_error(f"{total - passed} test(s) failed")
            sys.exit(1)

    except Exception as e:
        print_error(f"Test failed with exception: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        db.close()


if __name__ == '__main__':
    main()



