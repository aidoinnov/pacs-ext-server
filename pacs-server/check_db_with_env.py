#!/usr/bin/env python3
"""
.env 파일의 DB 연결 정보로 project_data 구조 확인
"""
import os
import psycopg2
from dotenv import load_dotenv

# .env 파일 로드
load_dotenv()

def check_db_structure():
    """DB 구조 확인"""
    print("=" * 60)
    print("🗄️  DB 구조 확인 (project_id=2)")
    print("=" * 60)
    
    # .env 파일에서 APP_DATABASE_URL 파싱
    from urllib.parse import urlparse
    import urllib.parse
    
    host = 'localhost'
    port = 5456
    user = 'admin'
    password = 'admin'
    database = 'pacs_rbac'
    
    # .env 파일 직접 읽기
    try:
        env_path = os.path.join(os.path.dirname(__file__), '.env')
        if os.path.exists(env_path):
            with open(env_path, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line.startswith('APP_DATABASE_URL='):
                        db_url = line.split('=', 1)[1].strip('"\'')
                        # postgres://user:password@host:port/database 형식 파싱
                        parsed = urlparse(db_url)
                        user = parsed.username or 'admin'
                        password = parsed.password or 'admin'
                        host = parsed.hostname or 'localhost'
                        port = parsed.port or 5456
                        database = parsed.path.lstrip('/') or 'pacs_rbac'
                        break
    except Exception as e:
        print(f"⚠️  .env 파일 읽기 실패: {e}")
        print("   기본값 사용")
    
    print(f"\n📡 DB 연결 정보:")
    print(f"   Host: {host}")
    print(f"   Port: {port}")
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
        
        # 1. project_data 개수
        print("\n1️⃣ project_data 테이블")
        print("-" * 60)
        cur.execute("SELECT COUNT(*) FROM project_data WHERE project_id = 2")
        project_data_count = cur.fetchone()[0]
        print(f"   project_id=2인 레코드: {project_data_count}개")
        
        # 2. project_data 상세 정보
        cur.execute("""
            SELECT id, study_id, resource_level, created_at
            FROM project_data 
            WHERE project_id = 2
            ORDER BY id
        """)
        project_data_rows = cur.fetchall()
        print(f"\n   상세 정보:")
        for row in project_data_rows:
            print(f"     ID={row[0]}, study_id={row[1]}, resource_level={row[2]}, created_at={row[3]}")
        
        # 3. 각 study의 series 개수
        print("\n2️⃣ Study별 Series 개수")
        print("-" * 60)
        cur.execute("""
            SELECT 
                pd.id as project_data_id,
                pd.study_id,
                pds.study_uid,
                pd.resource_level,
                COUNT(pdser.id) as series_count
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            LEFT JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
            GROUP BY pd.id, pd.study_id, pds.study_uid, pd.resource_level
            ORDER BY series_count DESC
        """)
        study_series = cur.fetchall()
        
        total_series = 0
        for row in study_series:
            project_data_id, study_id, study_uid, resource_level, series_count = row
            total_series += series_count
            print(f"   project_data_id={project_data_id}, study_id={study_id}")
            print(f"     study_uid: {study_uid[:60]}...")
            print(f"     resource_level: {resource_level}")
            print(f"     series_count: {series_count}개")
            print()
        
        print(f"   총 Series 개수: {total_series}개")
        
        # 4. get_allowed_series_uids 쿼리와 동일한 결과
        print("\n3️⃣ get_allowed_series_uids 쿼리 결과")
        print("-" * 60)
        cur.execute("""
            SELECT DISTINCT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
            ORDER BY pdser.series_uid
        """)
        allowed_series = cur.fetchall()
        print(f"   고유 Series UID 개수: {len(allowed_series)}개")
        
        if len(allowed_series) <= 15:
            print(f"\n   Series UID 목록:")
            for i, (series_uid,) in enumerate(allowed_series, 1):
                print(f"     {i:2d}. {series_uid}")
        else:
            print(f"\n   Series UID 목록 (최대 15개):")
            for i, (series_uid,) in enumerate(allowed_series[:15], 1):
                print(f"     {i:2d}. {series_uid}")
            print(f"     ... 외 {len(allowed_series) - 15}개")
        
        # 5. 중복 확인
        print("\n4️⃣ 중복 확인")
        print("-" * 60)
        cur.execute("""
            SELECT pdser.series_uid, COUNT(*) as cnt
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
            GROUP BY pdser.series_uid
            HAVING COUNT(*) > 1
        """)
        duplicates = cur.fetchall()
        if duplicates:
            print(f"   ⚠️  중복된 Series UID: {len(duplicates)}개")
            for series_uid, count in duplicates:
                print(f"     {series_uid}: {count}회")
        else:
            print(f"   ✅ 중복 없음")
        
        # 6. 요약
        print("\n" + "=" * 60)
        print("📊 요약")
        print("=" * 60)
        print(f"   project_data 레코드: {project_data_count}개")
        print(f"   Study 개수: {len(study_series)}개")
        print(f"   총 Series 개수 (중복 포함): {total_series}개")
        print(f"   고유 Series UID: {len(allowed_series)}개")
        
        if project_data_count == 5 and len(allowed_series) == 11:
            print("\n✅ 정상:")
            print(f"   - project_data에 5개 레코드가 있음")
            print(f"   - 각 레코드가 다른 study를 가리킴")
            print(f"   - 각 study가 여러 series를 가져서 총 11개가 됨")
            print(f"   - 평균 {len(allowed_series) / project_data_count:.1f}개 series per project_data")
        
        cur.close()
        conn.close()
        
    except psycopg2.OperationalError as e:
        print(f"\n❌ DB 연결 실패: {e}")
        print("\n💡 해결 방법:")
        print("   1. DB 터널이 열려있는지 확인: ./scripts/start-db-tunnels.sh")
        print("   2. .env 파일의 연결 정보 확인")
        print("   3. DB 서버가 실행 중인지 확인")
    except Exception as e:
        print(f"\n❌ 에러: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    check_db_structure()

