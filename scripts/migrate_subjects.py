#!/usr/bin/env python3
"""
Subject 자동 생성 마이그레이션 도구

기존 프로젝트에 할당된 Study들에 대해 Subject를 자동으로 생성합니다.

Usage:
    python scripts/migrate_subjects.py --project-id 1
    python scripts/migrate_subjects.py --all-projects
    python scripts/migrate_subjects.py --project-id 1 --dry-run
"""

import argparse
import logging
import sys
import os
from typing import List, Dict, Optional
import psycopg2
from psycopg2.extras import RealDictCursor

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class SubjectMigrator:
    """Subject 자동 생성 마이그레이션"""

    def __init__(self, db_url: str, dry_run: bool = False):
        self.db_url = db_url
        self.dry_run = dry_run
        self.conn = None
        self.subject_counter = {}  # project_id -> count (dry-run용)

    def connect(self):
        """데이터베이스 연결"""
        self.conn = psycopg2.connect(self.db_url)
        logger.info("✓ Database connected")

    def close(self):
        """데이터베이스 연결 종료"""
        if self.conn:
            self.conn.close()
            logger.info("✓ Database connection closed")

    def get_projects(self, project_id: Optional[int] = None) -> List[Dict]:
        """프로젝트 목록 조회"""
        with self.conn.cursor(cursor_factory=RealDictCursor) as cur:
            if project_id:
                cur.execute("SELECT id, name FROM security_project WHERE id = %s", (project_id,))
            else:
                cur.execute("SELECT id, name FROM security_project WHERE is_active = true")
            return cur.fetchall()

    def get_assigned_studies(self, project_id: int) -> List[Dict]:
        """프로젝트에 할당된 Study 목록 조회 (patient_id 유무 무관)"""
        with self.conn.cursor(cursor_factory=RealDictCursor) as cur:
            cur.execute("""
                SELECT DISTINCT s.id, s.study_uid, s.patient_id, s.patient_name, s.patient_birth_date
                FROM project_data pd
                JOIN project_data_study s ON pd.study_id = s.id
                WHERE pd.project_id = %s
                  AND pd.resource_level = 'STUDY'
                ORDER BY s.id
            """, (project_id,))
            return cur.fetchall()

    def get_existing_subject(self, project_id: int, patient_id: Optional[str]) -> Optional[Dict]:
        """기존 Subject 조회 (patient_id 기반)"""
        if not patient_id:
            return None

        with self.conn.cursor(cursor_factory=RealDictCursor) as cur:
            cur.execute("""
                SELECT id, subject_code, patient_id
                FROM project_subject
                WHERE project_id = %s AND patient_id = %s
            """, (project_id, patient_id))
            return cur.fetchone()

    def generate_subject_code(self, project_id: int, patient_id: Optional[str]) -> str:
        """
        유일한 Subject Code 생성

        - patient_id가 있으면: patient_id 기반 (예: P12345)
        - patient_id가 없으면: A-001, A-002, ..., A-999, B-001, ... 형식
        """
        if patient_id:
            # Patient ID 기반 생성
            base_code = patient_id[:50]
            candidate = base_code
            suffix = 0

            with self.conn.cursor() as cur:
                while True:
                    cur.execute("""
                        SELECT EXISTS(
                            SELECT 1 FROM project_subject
                            WHERE project_id = %s AND subject_code = %s
                        )
                    """, (project_id, candidate))
                    exists = cur.fetchone()[0]

                    if not exists:
                        return candidate

                    suffix += 1
                    candidate = f"{base_code}_{suffix}"

                    if suffix > 100:
                        break

        # patient_id가 없거나 충돌 시: A-001, A-002, ... 형식
        with self.conn.cursor() as cur:
            # 기존 Subject 수 조회
            cur.execute("""
                SELECT COUNT(*) FROM project_subject WHERE project_id = %s
            """, (project_id,))
            count = cur.fetchone()[0]

            # Dry-run 모드에서는 카운터 사용
            if self.dry_run:
                if project_id not in self.subject_counter:
                    self.subject_counter[project_id] = count
                count = self.subject_counter[project_id]

            offset = 0
            while True:
                # A-001 ~ A-999, B-001 ~ B-999, ...
                total_num = count + offset
                prefix = chr(65 + (total_num // 999))  # A, B, C, ...
                number = (total_num % 999) + 1
                candidate = f"{prefix}-{number:03d}"

                cur.execute("""
                    SELECT EXISTS(
                        SELECT 1 FROM project_subject
                        WHERE project_id = %s AND subject_code = %s
                    )
                """, (project_id, candidate))
                exists = cur.fetchone()[0]

                if not exists:
                    # Dry-run 모드에서는 카운터 증가
                    if self.dry_run:
                        self.subject_counter[project_id] += 1
                    return candidate

                offset += 1
                if offset > 10000:
                    raise Exception("Failed to generate unique subject code")

    def create_subject(self, project_id: int, subject_code: str, study: Dict) -> int:
        """Subject 생성"""
        with self.conn.cursor() as cur:
            cur.execute("""
                INSERT INTO project_subject (project_id, subject_code, patient_id, patient_name, patient_birth_date)
                VALUES (%s, %s, %s, %s, %s)
                RETURNING id
            """, (project_id, subject_code, study['patient_id'], study['patient_name'], study['patient_birth_date']))
            subject_id = cur.fetchone()[0]
            self.conn.commit()
            return subject_id

    def migrate_project(self, project_id: int, project_name: str):
        """프로젝트의 Subject 마이그레이션"""
        logger.info(f"\n{'='*60}")
        logger.info(f"Project: {project_name} (ID: {project_id})")
        logger.info(f"{'='*60}")

        # 할당된 Study 조회
        studies = self.get_assigned_studies(project_id)
        studies_with_patient = [s for s in studies if s['patient_id']]
        studies_without_patient = [s for s in studies if not s['patient_id']]

        logger.info(f"Found {len(studies)} studies total")
        logger.info(f"  - With patient_id: {len(studies_with_patient)}")
        logger.info(f"  - Without patient_id: {len(studies_without_patient)}")

        if not studies:
            logger.info("No studies to migrate")
            return

        created_count = 0
        reused_count = 0

        for study in studies:
            patient_id = study['patient_id']

            # 기존 Subject 확인
            existing = self.get_existing_subject(project_id, patient_id)

            if existing:
                patient_info = f"Patient: {patient_id}" if patient_id else "No patient_id"
                logger.info(f"  ✓ Reuse Subject: {existing['subject_code']} ({patient_info})")
                reused_count += 1
                continue

            # Subject Code 생성
            subject_code = self.generate_subject_code(project_id, patient_id)

            patient_info = f"Patient: {patient_id}" if patient_id else "No patient_id"

            if self.dry_run:
                logger.info(f"  [DRY-RUN] Would create Subject: {subject_code} ({patient_info})")
            else:
                subject_id = self.create_subject(project_id, subject_code, study)
                logger.info(f"  ✓ Created Subject: {subject_code} (ID: {subject_id}, {patient_info})")

            created_count += 1

        logger.info(f"\nSummary:")
        logger.info(f"  - Created: {created_count}")
        logger.info(f"  - Reused: {reused_count}")
        logger.info(f"  - Total: {created_count + reused_count}")

    def run(self, project_id: Optional[int] = None):
        """마이그레이션 실행"""
        try:
            self.connect()

            # 프로젝트 목록 조회
            projects = self.get_projects(project_id)

            if not projects:
                logger.error("No projects found")
                return

            logger.info(f"Found {len(projects)} project(s) to migrate")

            # 각 프로젝트 마이그레이션
            for project in projects:
                self.migrate_project(project['id'], project['name'])

            logger.info(f"\n{'='*60}")
            logger.info("✓ Migration completed successfully")
            logger.info(f"{'='*60}")

        except Exception as e:
            logger.error(f"Migration failed: {e}", exc_info=True)
            if self.conn:
                self.conn.rollback()
            raise
        finally:
            self.close()


def get_default_db_url() -> str:
    """기본 DB URL 가져오기 (.env 파일 또는 환경 변수)"""
    # 1순위: APP_DATABASE_URL (실제 사용 중인 URL)
    db_url = os.getenv('APP_DATABASE_URL')
    if db_url:
        # postgres:// -> postgresql:// 변환
        if db_url.startswith('postgres://'):
            db_url = db_url.replace('postgres://', 'postgresql://', 1)
        return db_url

    # 2순위: DATABASE_URL
    db_url = os.getenv('DATABASE_URL')
    if db_url:
        if db_url.startswith('postgres://'):
            db_url = db_url.replace('postgres://', 'postgresql://', 1)
        return db_url

    # 3순위: 개별 환경 변수에서 조합
    db_user = os.getenv('POSTGRES_USER', 'postgres')
    db_pass = os.getenv('POSTGRES_PASSWORD', 'postgres')
    db_host = os.getenv('DATABASE_HOST', 'localhost')
    db_port = os.getenv('POSTGRES_PORT', '5432')
    db_name = os.getenv('POSTGRES_DB', 'pacs_extension')

    return f"postgresql://{db_user}:{db_pass}@{db_host}:{db_port}/{db_name}"


def main():
    # .env 파일 로드 시도
    env_path = os.path.join(os.path.dirname(__file__), '..', 'pacs-server', '.env')
    if os.path.exists(env_path):
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ.setdefault(key, value)

    parser = argparse.ArgumentParser(
        description='Subject 자동 생성 마이그레이션 도구',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # 특정 프로젝트만 마이그레이션
  python scripts/migrate_subjects.py --project-id 1

  # 모든 활성 프로젝트 마이그레이션
  python scripts/migrate_subjects.py --all-projects

  # Dry-run (실제 생성 안 함)
  python scripts/migrate_subjects.py --project-id 1 --dry-run

  # 커스텀 DB URL
  python scripts/migrate_subjects.py --all-projects --db-url "postgresql://user:pass@localhost/pacs"
        """
    )

    parser.add_argument(
        '--project-id',
        type=int,
        help='마이그레이션할 프로젝트 ID'
    )
    parser.add_argument(
        '--all-projects',
        action='store_true',
        help='모든 활성 프로젝트 마이그레이션'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='실제 생성하지 않고 시뮬레이션만 수행'
    )
    parser.add_argument(
        '--db-url',
        default=None,
        help='데이터베이스 URL (기본값: .env 파일 또는 환경 변수에서 읽음)'
    )

    args = parser.parse_args()

    # 인자 검증
    if not args.project_id and not args.all_projects:
        parser.error("--project-id 또는 --all-projects 중 하나를 지정해야 합니다")

    if args.project_id and args.all_projects:
        parser.error("--project-id와 --all-projects를 동시에 사용할 수 없습니다")

    # DB URL 결정
    db_url = args.db_url or get_default_db_url()
    logger.info(f"Database: {db_url.split('@')[1] if '@' in db_url else db_url}")

    # 마이그레이션 실행
    migrator = SubjectMigrator(db_url, dry_run=args.dry_run)

    if args.dry_run:
        logger.info("🔍 DRY-RUN MODE: 실제 생성하지 않습니다")

    migrator.run(project_id=args.project_id)


if __name__ == '__main__':
    main()

