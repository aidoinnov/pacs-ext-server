#!/usr/bin/env python3
"""
DB 쿼리 직접 실행하여 문제 검증
"""
import psycopg2
import sys

# DB 연결 정보
DB_CONFIG = {
    'host': '127.0.0.1',
    'port': 5456,
    'user': 'admin',
    'password': 'admin123',
    'database': 'pacs_db'
}

print("=" * 60)
print("🔍 DB 쿼리 검증")
print("=" * 60)

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
        sys.exit(1)
    
    # 2. project_data 상세 확인
    print("\n2️⃣ project_data 상세 확인")
    print("-" * 60)
    cur.execute("""
        SELECT 
            pd.id,
            pd.project_id,
            pd.resource_level,
            pd.study_id,
            pd.series_id,
            pd.created_at
        FROM project_data pd
        WHERE pd.project_id = 2
        LIMIT 10
    """)
    rows = cur.fetchall()
    print(f"✅ 조회된 행 수: {len(rows)}")
    for row in rows:
        print(f"  id={row[0]}, project_id={row[1]}, level={row[2]}, study_id={row[3]}, series_id={row[4]}")
    
    # 3. project_data_study 확인
    print("\n3️⃣ project_data_study 확인")
    print("-" * 60)
    cur.execute("SELECT COUNT(*) FROM project_data_study")
    study_count = cur.fetchone()[0]
    print(f"✅ project_data_study 개수: {study_count}")
    
    # 4. project_data_series 확인
    print("\n4️⃣ project_data_series 확인")
    print("-" * 60)
    cur.execute("SELECT COUNT(*) FROM project_data_series")
    series_count = cur.fetchone()[0]
    print(f"✅ project_data_series 개수: {series_count}")
    
    # 5. 조인 테스트 (LEFT JOIN으로 확인)
    print("\n5️⃣ 조인 테스트 (LEFT JOIN)")
    print("-" * 60)
    cur.execute("""
        SELECT 
            pd.id as pd_id,
            pd.project_id,
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
        print(f"  pd_id={row[0]}, project_id={row[1]}, pd_study_id={row[2]}, pd_series_id={row[3]}")
        print(f"    pds_id={row[4]}, study_uid={row[5]}")
        print(f"    pdser_id={row[6]}, series_uid={row[7]}, pdser_study_id={row[8]}")
        if row[4] is None:
            print("    ⚠️  project_data_study 조인 실패!")
        if row[6] is None:
            print("    ⚠️  project_data_series 조인 실패!")
        print()
    
    # 6. get_allowed_series_uids 쿼리 직접 실행
    print("6️⃣ get_allowed_series_uids 쿼리 실행")
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
    
    # 7. 조인 조건 확인 (pds.id = pdser.study_id)
    print("\n7️⃣ 조인 조건 확인 (pds.id = pdser.study_id)")
    print("-" * 60)
    cur.execute("""
        SELECT 
            pd.id as pd_id,
            pd.study_id as pd_study_id,
            pds.id as pds_id,
            pdser.id as pdser_id,
            pdser.study_id as pdser_study_id,
            CASE 
                WHEN pds.id = pdser.study_id THEN '✅ 일치'
                ELSE '❌ 불일치'
            END as match_status
        FROM project_data pd
        INNER JOIN project_data_study pds ON pd.study_id = pds.id
        LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
        WHERE pd.project_id = 2
        LIMIT 10
    """)
    rows = cur.fetchall()
    print(f"✅ 조회된 행 수: {len(rows)}")
    for row in rows:
        print(f"  pd_id={row[0]}, pd_study_id={row[1]}, pds_id={row[2]}, pdser_id={row[3]}, pdser_study_id={row[4]}, {row[5]}")
        if row[2] != row[4]:
            print(f"    ⚠️  pds.id ({row[2]}) != pdser.study_id ({row[4]})")
    
    cur.close()
    conn.close()
    
    print("\n" + "=" * 60)
    print("✅ 검증 완료")
    print("=" * 60)
    
except psycopg2.OperationalError as e:
    print(f"❌ DB 연결 실패: {e}")
    print("\n다른 연결 정보로 시도 중...")
    # 다른 포트/사용자로 시도
    for port in [5456, 5432]:
        for user, pwd in [('admin', 'admin123'), ('postgres', 'postgres')]:
            try:
                test_config = DB_CONFIG.copy()
                test_config['port'] = port
                test_config['user'] = user
                test_config['password'] = pwd
                test_conn = psycopg2.connect(**test_config)
                test_conn.close()
                print(f"✅ 연결 성공: {user}@{port}")
                print(f"   설정: {test_config}")
                break
            except:
                pass
except Exception as e:
    print(f"❌ 에러: {e}")
    import traceback
    traceback.print_exc()

