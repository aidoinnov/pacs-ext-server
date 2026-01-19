#!/usr/bin/env python3
"""
DB에서 project_id=2에 할당된 Series 개수 확인
"""
import os
import psycopg2
from dotenv import load_dotenv

# .env 파일 로드
load_dotenv()

def check_db():
    """DB에서 직접 확인"""
    print("=" * 60)
    print("🗄️  DB에서 project_id=2에 할당된 Series 확인")
    print("=" * 60)
    
    try:
        # 환경 변수에서 DB 연결 정보 가져오기
        host = os.getenv('APP_DATABASE__HOST', 'localhost')
        port = int(os.getenv('APP_DATABASE__PORT', '5456'))
        user = os.getenv('APP_DATABASE__USERNAME', 'admin')
        password = os.getenv('APP_DATABASE__PASSWORD', 'admin')
        database = os.getenv('APP_DATABASE__DATABASE', 'pacs_rbac')
        
        print(f"\n📡 DB 연결 정보:")
        print(f"   Host: {host}")
        print(f"   Port: {port}")
        print(f"   Database: {database}")
        print(f"   User: {user}")
        print("-" * 60)
        
        conn = psycopg2.connect(
            host=host,
            port=port,
            user=user,
            password=password,
            database=database
        )
        
        cur = conn.cursor()
        
        # 1. project_data에서 project_id=2인 데이터 개수
        cur.execute("""
            SELECT COUNT(DISTINCT pd.id)
            FROM project_data pd
            WHERE pd.project_id = 2
        """)
        project_data_count = cur.fetchone()[0]
        print(f"\n1️⃣ project_data 테이블:")
        print(f"   project_id=2인 레코드: {project_data_count}개")
        
        # 2. project_data_study에서 project_id=2인 Study 개수
        cur.execute("""
            SELECT COUNT(DISTINCT pds.study_uid)
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            WHERE pd.project_id = 2
              AND pds.study_uid IS NOT NULL
        """)
        study_count = cur.fetchone()[0]
        print(f"\n2️⃣ project_data_study 테이블:")
        print(f"   project_id=2인 Study 개수: {study_count}개")
        
        # 3. project_data_series에서 project_id=2인 Series 개수 (고유)
        cur.execute("""
            SELECT COUNT(DISTINCT pdser.series_uid)
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
        """)
        unique_series_count = cur.fetchone()[0]
        print(f"\n3️⃣ project_data_series 테이블:")
        print(f"   project_id=2인 고유 Series 개수: {unique_series_count}개")
        
        # 4. 전체 Series 개수 (중복 포함)
        cur.execute("""
            SELECT COUNT(pdser.id)
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
        """)
        total_series_count = cur.fetchone()[0]
        print(f"   project_id=2인 전체 Series 레코드: {total_series_count}개")
        
        # 5. Study별 Series 개수
        cur.execute("""
            SELECT pds.study_uid, COUNT(DISTINCT pdser.series_uid) as series_count
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pds.study_uid IS NOT NULL
              AND pdser.series_uid IS NOT NULL
            GROUP BY pds.study_uid
            ORDER BY series_count DESC
        """)
        study_series = cur.fetchall()
        print(f"\n4️⃣ Study별 Series 개수:")
        for study_uid, count in study_series:
            print(f"   Study {study_uid[:60]}...: {count}개 Series")
        
        # 6. 다른 project_id와 비교
        cur.execute("""
            SELECT pd.project_id, COUNT(DISTINCT pdser.series_uid) as series_count
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pdser.series_uid IS NOT NULL
            GROUP BY pd.project_id
            ORDER BY pd.project_id
        """)
        project_series = cur.fetchall()
        print(f"\n5️⃣ 프로젝트별 Series 개수:")
        for project_id, count in project_series:
            marker = " ← 현재" if project_id == 2 else ""
            print(f"   project_id={project_id}: {count}개 Series{marker}")
        
        # 7. 최근 할당된 데이터 확인
        cur.execute("""
            SELECT pd.project_id, COUNT(*) as count, MAX(pd.created_at) as last_assigned
            FROM project_data pd
            WHERE pd.project_id = 2
            GROUP BY pd.project_id
        """)
        recent = cur.fetchone()
        if recent:
            print(f"\n6️⃣ 최근 할당 정보:")
            print(f"   마지막 할당 시간: {recent[1]}")
        
        # 8. Series UID 목록 (일부)
        cur.execute("""
            SELECT DISTINCT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
            ORDER BY pdser.series_uid
            LIMIT 15
        """)
        series_uids = cur.fetchall()
        print(f"\n7️⃣ Series UID 샘플 (최대 15개):")
        for (series_uid,) in series_uids:
            print(f"   {series_uid}")
        
        cur.close()
        conn.close()
        
        print("\n" + "=" * 60)
        print("📊 요약")
        print("=" * 60)
        print(f"✅ project_id=2에 할당된 고유 Series: {unique_series_count}개")
        print(f"✅ Study 개수: {study_count}개")
        print(f"✅ API 응답: 11개")
        
        if unique_series_count == 11:
            print("\n✅ DB와 API 응답이 일치합니다!")
        elif unique_series_count > 11:
            print(f"\n⚠️  DB에는 {unique_series_count}개가 있지만 API는 11개만 반환합니다.")
            print("   가능한 원인:")
            print("   1. QIDO에서 일부 Series를 가져오지 못함")
            print("   2. 필터링 로직에서 일부가 제외됨")
            print("   3. Series UID 매칭 실패")
        else:
            print(f"\n⚠️  DB에는 {unique_series_count}개가 있지만 API는 11개를 반환합니다.")
            print("   가능한 원인:")
            print("   1. 다른 프로젝트의 Series가 포함됨")
            print("   2. 필터링 로직 문제")
        
    except psycopg2.OperationalError as e:
        print(f"\n❌ DB 연결 실패: {e}")
        print("\n💡 해결 방법:")
        print("   1. DB 터널이 열려있는지 확인: ./scripts/start-db-tunnels.sh")
        print("   2. 환경 변수 확인: APP_DATABASE__HOST, APP_DATABASE__PORT 등")
        print("   3. 직접 DB에 접근하여 확인")
    except Exception as e:
        print(f"\n❌ 에러: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    check_db()

