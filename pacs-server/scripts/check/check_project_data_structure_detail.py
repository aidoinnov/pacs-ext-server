#!/usr/bin/env python3
"""
project_data 테이블 구조 상세 확인
- resource_level=SERIES일 때 series_id가 있는지 확인
"""
import os
from urllib.parse import urlparse
import psycopg2

def check_project_data_structure():
    """project_data 테이블 구조 확인"""
    print("=" * 60)
    print("🗄️  project_data 테이블 구조 확인")
    print("=" * 60)
    
    # .env 파일에서 APP_DATABASE_URL 파싱
    env_path = os.path.join(os.path.dirname(__file__), '.env')
    host = 'localhost'
    port = 5456
    user = 'admin'
    password = 'admin'
    database = 'pacs_rbac'
    
    try:
        with open(env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line.startswith('APP_DATABASE_URL='):
                    db_url = line.split('=', 1)[1].strip('"\'')
                    parsed = urlparse(db_url)
                    user = parsed.username or 'admin'
                    password = parsed.password or 'admin'
                    host = parsed.hostname or 'localhost'
                    port = parsed.port or 5456
                    database = parsed.path.lstrip('/') or 'pacs_rbac'
                    break
    except Exception as e:
        print(f"⚠️  .env 파일 읽기 실패: {e}")
    
    print(f"\n📡 DB 연결 정보:")
    print(f"   Host: {host}:{port}")
    print(f"   Database: {database}")
    print(f"   User: {user}")
    print("-" * 60)
    
    try:
        conn = psycopg2.connect(
            host=host,
            port=port,
            user=user,
            password=password,
            database=database
        )
        
        cur = conn.cursor()
        
        # 1. project_data 테이블 구조 확인
        print("\n1️⃣ project_data 테이블 구조")
        print("-" * 60)
        cur.execute("""
            SELECT column_name, data_type, is_nullable
            FROM information_schema.columns
            WHERE table_name = 'project_data'
            ORDER BY ordinal_position
        """)
        columns = cur.fetchall()
        print("   컬럼 목록:")
        for col_name, col_type, is_nullable in columns:
            nullable = "NULL" if is_nullable == 'YES' else "NOT NULL"
            print(f"     - {col_name}: {col_type} ({nullable})")
        
        # 2. project_id=2인 레코드 상세 확인
        print("\n2️⃣ project_id=2인 레코드 상세")
        print("-" * 60)
        cur.execute("""
            SELECT id, project_id, study_id, series_id, resource_level, created_at
            FROM project_data
            WHERE project_id = 2
            ORDER BY id
        """)
        rows = cur.fetchall()
        print(f"   총 {len(rows)}개 레코드:")
        for row in rows:
            pd_id, proj_id, study_id, series_id, resource_level, created_at = row
            print(f"     ID={pd_id}, project_id={proj_id}, study_id={study_id}, "
                  f"series_id={series_id}, resource_level={resource_level}")
        
        # 3. series_id가 있는 경우 해당 series만 조회
        print("\n3️⃣ series_id가 있는 레코드의 Series 확인")
        print("-" * 60)
        cur.execute("""
            SELECT 
                pd.id as project_data_id,
                pd.series_id,
                pd.resource_level,
                pdser.series_uid
            FROM project_data pd
            LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
            WHERE pd.project_id = 2
              AND pd.series_id IS NOT NULL
            ORDER BY pd.id
        """)
        series_rows = cur.fetchall()
        if series_rows:
            print(f"   series_id가 있는 레코드: {len(series_rows)}개")
            for row in series_rows:
                pd_id, series_id, resource_level, series_uid = row
                print(f"     project_data_id={pd_id}, series_id={series_id}, "
                      f"series_uid={series_uid}")
        else:
            print("   ⚠️  series_id가 있는 레코드가 없습니다")
        
        # 4. 올바른 쿼리 (resource_level에 따라)
        print("\n4️⃣ 올바른 쿼리 로직")
        print("-" * 60)
        print("   resource_level='STUDY'인 경우:")
        print("     - study_id로 해당 study의 모든 series 조회")
        print("   resource_level='SERIES'인 경우:")
        print("     - series_id로 해당 series만 조회")
        
        # 5. 수정된 쿼리 테스트
        print("\n5️⃣ 수정된 쿼리 결과 (resource_level별)")
        print("-" * 60)
        
        # SERIES 레벨인 경우
        cur.execute("""
            SELECT DISTINCT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
            WHERE pd.project_id = 2
              AND pd.resource_level = 'SERIES'
              AND pd.series_id IS NOT NULL
              AND pdser.series_uid IS NOT NULL
            ORDER BY pdser.series_uid
        """)
        series_level_series = cur.fetchall()
        print(f"   resource_level='SERIES'인 경우: {len(series_level_series)}개")
        for i, (series_uid,) in enumerate(series_level_series, 1):
            print(f"     {i}. {series_uid}")
        
        # STUDY 레벨인 경우
        cur.execute("""
            SELECT DISTINCT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pd.resource_level = 'STUDY'
              AND pd.study_id IS NOT NULL
              AND pdser.series_uid IS NOT NULL
            ORDER BY pdser.series_uid
        """)
        study_level_series = cur.fetchall()
        print(f"\n   resource_level='STUDY'인 경우: {len(study_level_series)}개")
        
        # 통합 (현재 잘못된 쿼리)
        print("\n6️⃣ 현재 쿼리 결과 (잘못된 방식)")
        print("-" * 60)
        cur.execute("""
            SELECT DISTINCT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
        """)
        current_series = cur.fetchall()
        print(f"   현재 쿼리 결과: {len(current_series)}개 (잘못됨)")
        
        cur.close()
        conn.close()
        
        print("\n" + "=" * 60)
        print("📊 결론")
        print("=" * 60)
        print(f"   project_data 레코드: {len(rows)}개")
        print(f"   resource_level='SERIES'인 경우 올바른 Series: {len(series_level_series)}개")
        print(f"   현재 잘못된 쿼리 결과: {len(current_series)}개")
        print("\n💡 문제:")
        print("   - resource_level='SERIES'일 때 series_id로 직접 조회해야 함")
        print("   - 현재는 study_id로 조인해서 study의 모든 series를 가져옴")
        print("\n💡 해결:")
        print("   - get_allowed_series_uids 함수를 수정해야 함")
        print("   - resource_level에 따라 다른 쿼리 사용")
        
    except Exception as e:
        print(f"\n❌ 에러: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    check_project_data_structure()

