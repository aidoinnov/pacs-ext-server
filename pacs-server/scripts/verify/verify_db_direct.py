#!/usr/bin/env python3
"""
DB 직접 연결하여 쿼리 검증
DBeaver나 다른 DB 클라이언트로 연결할 수 없는 경우 사용
"""
import psycopg2
import sys
import os

# DB 연결 정보 (환경 변수에서 가져오기 또는 직접 설정)
DB_CONFIG = {
    'host': os.getenv('APP_DATABASE__HOST', '127.0.0.1'),
    'port': int(os.getenv('APP_DATABASE__PORT', '5456')),
    'user': os.getenv('APP_DATABASE__USERNAME', 'admin'),
    'password': os.getenv('APP_DATABASE__PASSWORD', 'admin123'),
    'database': os.getenv('APP_DATABASE__DATABASE', 'pacs_db')
}

print("=" * 60)
print("🔍 DB 직접 연결 검증")
print("=" * 60)
print(f"연결 정보: {DB_CONFIG['user']}@{DB_CONFIG['host']}:{DB_CONFIG['port']}/{DB_CONFIG['database']}")
print()

try:
    conn = psycopg2.connect(**DB_CONFIG)
    cur = conn.cursor()
    print("✅ DB 연결 성공\n")
    
    # 1. project_data 기본 확인
    print("1️⃣ project_data 테이블 확인")
    print("-" * 60)
    cur.execute("SELECT COUNT(*) FROM project_data WHERE project_id = 2")
    count = cur.fetchone()[0]
    print(f"✅ project_data (project_id=2) 개수: {count}")
    
    if count == 0:
        print("❌ project_data에 데이터가 없습니다!")
        print("   → 데이터 할당이 제대로 되지 않았을 수 있습니다.")
        sys.exit(1)
    
    # 2. get_allowed_series_uids 쿼리 직접 실행
    print("\n2️⃣ get_allowed_series_uids 쿼리 실행")
    print("-" * 60)
    cur.execute("""
        SELECT DISTINCT pdser.series_uid
        FROM project_data pd
        INNER JOIN project_data_study pds ON pd.study_id = pds.id
        INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
        WHERE pd.project_id = 2
          AND pdser.series_uid IS NOT NULL
    """)
    series_uids = cur.fetchall()
    print(f"✅ 허용된 Series UID 개수: {len(series_uids)}")
    
    if len(series_uids) == 0:
        print("❌ 허용된 Series UID가 없습니다!")
        print("\n원인 분석:")
        
        # 원인 1: study_id가 NULL인지 확인
        cur.execute("""
            SELECT COUNT(*) FROM project_data 
            WHERE project_id = 2 AND study_id IS NULL
        """)
        null_study_count = cur.fetchone()[0]
        if null_study_count > 0:
            print(f"  ⚠️  study_id가 NULL인 행: {null_study_count}개")
        
        # 원인 2: project_data_study 조인 실패 확인
        cur.execute("""
            SELECT COUNT(*) FROM project_data pd
            LEFT JOIN project_data_study pds ON pd.study_id = pds.id
            WHERE pd.project_id = 2 AND pds.id IS NULL
        """)
        join_fail_count = cur.fetchone()[0]
        if join_fail_count > 0:
            print(f"  ⚠️  project_data_study 조인 실패: {join_fail_count}개")
            print("     → pd.study_id에 해당하는 project_data_study.id가 없음")
        
        # 원인 3: project_data_series 조인 실패 확인
        cur.execute("""
            SELECT COUNT(*) FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            LEFT JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2 AND pdser.id IS NULL
        """)
        series_join_fail_count = cur.fetchone()[0]
        if series_join_fail_count > 0:
            print(f"  ⚠️  project_data_series 조인 실패: {series_join_fail_count}개")
            print("     → pds.id에 해당하는 project_data_series.study_id가 없음")
        
        # 원인 4: series_uid가 NULL인지 확인
        cur.execute("""
            SELECT COUNT(*) FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2 AND pdser.series_uid IS NULL
        """)
        null_uid_count = cur.fetchone()[0]
        if null_uid_count > 0:
            print(f"  ⚠️  series_uid가 NULL인 Series: {null_uid_count}개")
    else:
        print("✅ 허용된 Series UID 목록 (최대 10개):")
        for i, (uid,) in enumerate(series_uids[:10], 1):
            print(f"  {i}. {uid}")
    
    # 3. 조인 상세 확인
    print("\n3️⃣ 조인 상세 확인")
    print("-" * 60)
    cur.execute("""
        SELECT 
            pd.id as pd_id,
            pd.project_id,
            pd.resource_level,
            pd.study_id as pd_study_id,
            pd.series_id as pd_series_id,
            pds.id as pds_id,
            pds.study_uid,
            pdser.id as pdser_id,
            pdser.series_uid,
            pdser.study_id as pdser_study_id
        FROM project_data pd
        LEFT JOIN project_data_study pds ON pd.study_id = pds.id
        LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
        WHERE pd.project_id = 2
        LIMIT 10
    """)
    rows = cur.fetchall()
    print(f"✅ 조회된 행 수: {len(rows)}")
    for row in rows:
        print(f"  pd_id={row[0]}, project_id={row[1]}, level={row[2]}, pd_study_id={row[3]}, pd_series_id={row[4]}")
        print(f"    pds_id={row[5]}, study_uid={row[6]}")
        print(f"    pdser_id={row[7]}, series_uid={row[8]}, pdser_study_id={row[9]}")
        if row[5] is None:
            print("    ⚠️  project_data_study 조인 실패!")
        if row[7] is None:
            print("    ⚠️  project_data_series 조인 실패!")
        if row[5] is not None and row[9] is not None and row[5] != row[9]:
            print(f"    ⚠️  pds.id ({row[5]}) != pdser.study_id ({row[9]})")
        print()
    
    cur.close()
    conn.close()
    
    print("=" * 60)
    print("✅ 검증 완료")
    print("=" * 60)
    
except psycopg2.OperationalError as e:
    print(f"❌ DB 연결 실패: {e}")
    print("\n해결 방법:")
    print("1. DB 터널이 열려있는지 확인: lsof -i :5456")
    print("2. 연결 정보 확인: .env 파일 또는 환경 변수")
    print("3. DBeaver나 다른 DB 클라이언트로 직접 연결 시도")
    print("4. test_get_allowed_series_uids.sql 파일의 쿼리를 직접 실행")
    sys.exit(1)
except Exception as e:
    print(f"❌ 에러: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)

