#!/usr/bin/env python3
"""STUDY_ASSIGNMENT_MANAGE, settings.study_assignment 완전 제거 마이그레이션"""
import os
import sys
from pathlib import Path

try:
    import psycopg2
except ImportError:
    print("psycopg2-binary 설치 필요: pip install psycopg2-binary")
    sys.exit(1)


def main():
    db_url = os.getenv(
        "DATABASE_URL",
        "postgresql://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension",
    )
    migration_file = (
        Path(__file__).parent.parent
        / "pacs-server/migrations/20260207_02_remove_study_assignment_manage.sql"
    )

    if not migration_file.exists():
        print(f"마이그레이션 파일 없음: {migration_file}")
        sys.exit(1)

    sql = migration_file.read_text(encoding="utf-8")
    conn = psycopg2.connect(db_url)
    conn.autocommit = True
    cur = conn.cursor()
    cur.execute(sql)
    cur.close()
    conn.close()
    print("STUDY_ASSIGNMENT_MANAGE 제거 마이그레이션 완료")


if __name__ == "__main__":
    main()
