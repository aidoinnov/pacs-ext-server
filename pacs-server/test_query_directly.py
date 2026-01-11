#!/usr/bin/env python3
"""
수정된 쿼리를 직접 테스트
"""
import os
from urllib.parse import urlparse
import psycopg2

def test_query():
    """수정된 쿼리 직접 테스트"""
    print("=" * 60)
    print("🔍 수정된 쿼리 직접 테스트")
    print("=" * 60)
    
    # .env 파일에서 DB 연결 정보 가져오기
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
    
    try:
        conn = psycopg2.connect(
            host=host,
            port=port,
            user=user,
            password=password,
            database=database
        )
        
        cur = conn.cursor()
        project_id = 2
        
        # 수정된 쿼리 테스트
        print(f"\n1️⃣ 수정된 쿼리 (UNION 사용)")
        print("-" * 60)
        cur.execute("""
            SELECT DISTINCT combined.series_uid
            FROM (
                SELECT pdser.series_uid
                FROM project_data pd
                INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
                WHERE pd.project_id = %s
                  AND pd.resource_level = 'SERIES'
                  AND pd.series_id IS NOT NULL
                  AND pdser.series_uid IS NOT NULL
                
                UNION
                
                SELECT DISTINCT pdser.series_uid
                FROM project_data pd
                INNER JOIN project_data_study pds ON pd.study_id = pds.id
                INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
                WHERE pd.project_id = %s
                  AND pd.resource_level = 'STUDY'
                  AND pd.study_id IS NOT NULL
                  AND pdser.series_uid IS NOT NULL
            ) AS combined
            ORDER BY combined.series_uid
        """, (project_id, project_id))
        
        result = cur.fetchall()
        print(f"   결과: {len(result)}개")
        for i, (series_uid,) in enumerate(result, 1):
            print(f"     {i}. {series_uid}")
        
        # SERIES 레벨만 테스트
        print(f"\n2️⃣ SERIES 레벨만 테스트")
        print("-" * 60)
        cur.execute("""
            SELECT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
            WHERE pd.project_id = %s
              AND pd.resource_level = 'SERIES'
              AND pd.series_id IS NOT NULL
              AND pdser.series_uid IS NOT NULL
            ORDER BY pdser.series_uid
        """, (project_id,))
        
        result = cur.fetchall()
        print(f"   결과: {len(result)}개")
        for i, (series_uid,) in enumerate(result, 1):
            print(f"     {i}. {series_uid}")
        
        # project_data 확인
        print(f"\n3️⃣ project_data 확인")
        print("-" * 60)
        cur.execute("""
            SELECT id, project_id, resource_level, study_id, series_id
            FROM project_data
            WHERE project_id = %s
            ORDER BY id
        """, (project_id,))
        
        rows = cur.fetchall()
        print(f"   project_data 레코드: {len(rows)}개")
        for row in rows:
            print(f"     ID={row[0]}, resource_level={row[2]}, study_id={row[3]}, series_id={row[4]}")
        
        cur.close()
        conn.close()
        
    except Exception as e:
        print(f"❌ 에러: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    test_query()

