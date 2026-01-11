#!/usr/bin/env python3
"""
Series 개수 확인 및 중복 검사
"""
import requests
import json
from collections import Counter

BASE_URL = "http://localhost:8080"

def get_token():
    """로그인하여 토큰 획득"""
    try:
        resp = requests.post(f'{BASE_URL}/api/auth/login', json={
            'username': 'iaid-pacs-admin',
            'password': 'Qlalfqjsgh1!'
        }, timeout=10)
        
        if resp.status_code == 200:
            data = resp.json()
            return data.get('token') or data.get('access_token')
    except Exception as e:
        print(f"❌ 로그인 에러: {e}")
    return None

def check_series_api(token, project_id=2):
    """Series API 호출 및 분석"""
    print("=" * 60)
    print(f"🔍 Series API 분석 (project_id={project_id})")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page=1&page_size=1000'
    print(f"\n📡 요청 URL: {url}")
    print("-" * 60)
    
    try:
        resp = requests.get(url, headers=headers, timeout=60)
        print(f"Status Code: {resp.status_code}")
        
        if resp.status_code == 200:
            data = resp.json()
            
            if isinstance(data, list):
                series_list = data
            elif isinstance(data, dict):
                series_list = data.get('series', [])
            else:
                print(f"❌ 예상치 못한 응답 형식: {type(data)}")
                return
            
            total_count = len(series_list)
            print(f"\n📊 총 Series 개수: {total_count}")
            
            # Series UID 추출 및 중복 확인
            series_uids = []
            study_uids = {}
            
            for series in series_list:
                # Series UID 추출
                series_uid_tag = series.get('0020000E', {})
                series_uid = series_uid_tag.get('Value', [None])[0] if isinstance(series_uid_tag, dict) else None
                
                # Study UID 추출
                study_uid_tag = series.get('0020000D', {})
                study_uid = study_uid_tag.get('Value', [None])[0] if isinstance(study_uid_tag, dict) else None
                
                if series_uid:
                    series_uids.append(series_uid)
                    if study_uid:
                        if study_uid not in study_uids:
                            study_uids[study_uid] = []
                        study_uids[study_uid].append(series_uid)
            
            # 중복 확인
            uid_counter = Counter(series_uids)
            duplicates = {uid: count for uid, count in uid_counter.items() if count > 1}
            
            print(f"\n📈 통계:")
            print(f"   고유 Series UID 개수: {len(set(series_uids))}")
            print(f"   중복된 Series UID: {len(duplicates)}개")
            
            if duplicates:
                print(f"\n⚠️  중복된 Series UID:")
                for uid, count in list(duplicates.items())[:10]:  # 최대 10개만 표시
                    print(f"   - {uid}: {count}회")
                if len(duplicates) > 10:
                    print(f"   ... 외 {len(duplicates) - 10}개")
            
            # Study별 Series 개수
            print(f"\n📚 Study별 Series 개수:")
            study_counts = {study: len(series_list) for study, series_list in study_uids.items()}
            for study_uid, count in list(study_counts.items())[:10]:  # 최대 10개만 표시
                print(f"   Study {study_uid[:50]}...: {count}개 Series")
            if len(study_counts) > 10:
                print(f"   ... 외 {len(study_counts) - 10}개 Study")
            
            print(f"\n   총 Study 개수: {len(study_counts)}")
            print(f"   평균 Series/Study: {total_count / len(study_counts) if study_counts else 0:.2f}")
            
            # 샘플 데이터 출력
            if series_list:
                print(f"\n📋 첫 번째 Series 샘플:")
                sample = series_list[0]
                print(f"   Series UID: {sample.get('0020000E', {}).get('Value', ['N/A'])[0]}")
                print(f"   Study UID: {sample.get('0020000D', {}).get('Value', ['N/A'])[0]}")
                print(f"   Series Description: {sample.get('0008103E', {}).get('Value', ['N/A'])[0]}")
            
        else:
            print(f"❌ 에러: {resp.status_code}")
            print(f"   {resp.text[:500]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
        import traceback
        traceback.print_exc()

def check_db_data():
    """DB에서 직접 데이터 확인"""
    print("\n" + "=" * 60)
    print("🗄️  DB 데이터 확인")
    print("=" * 60)
    
    import os
    import psycopg2
    
    try:
        conn = psycopg2.connect(
            host=os.getenv('APP_DATABASE__HOST', 'localhost'),
            port=int(os.getenv('APP_DATABASE__PORT', '5456')),
            user=os.getenv('APP_DATABASE__USERNAME', 'admin'),
            password=os.getenv('APP_DATABASE__PASSWORD', 'admin'),
            database=os.getenv('APP_DATABASE__DATABASE', 'pacs_rbac')
        )
        
        cur = conn.cursor()
        
        # project_data_series에서 project_id=2인 Series 개수 확인
        cur.execute("""
            SELECT COUNT(DISTINCT pdser.series_uid)
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
        """)
        
        unique_series_count = cur.fetchone()[0]
        print(f"\n📊 DB에서 project_id=2인 고유 Series 개수: {unique_series_count}")
        
        # 중복 확인
        cur.execute("""
            SELECT pdser.series_uid, COUNT(*) as cnt
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
            GROUP BY pdser.series_uid
            HAVING COUNT(*) > 1
            ORDER BY cnt DESC
            LIMIT 10
        """)
        
        duplicates = cur.fetchall()
        if duplicates:
            print(f"\n⚠️  DB에서 중복된 Series UID:")
            for series_uid, count in duplicates:
                print(f"   - {series_uid}: {count}회")
        else:
            print(f"\n✅ DB에 중복 없음")
        
        # Study 개수 확인
        cur.execute("""
            SELECT COUNT(DISTINCT pds.study_uid)
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            WHERE pd.project_id = 2
              AND pds.study_uid IS NOT NULL
        """)
        
        study_count = cur.fetchone()[0]
        print(f"\n📚 DB에서 project_id=2인 Study 개수: {study_count}")
        
        cur.close()
        conn.close()
        
    except Exception as e:
        print(f"❌ DB 연결 에러: {e}")
        print("   환경 변수를 확인하거나 직접 DB에 접근하여 확인하세요")

def main():
    print("=" * 60)
    print("🔍 Series 개수 및 중복 확인")
    print("=" * 60)
    
    # 1. 토큰 획득
    print("\n1️⃣ 토큰 획득")
    print("-" * 60)
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        return
    print("✅ 토큰 획득 성공")
    
    # 2. API 확인
    print("\n2️⃣ API 응답 분석")
    print("-" * 60)
    check_series_api(token, project_id=2)
    
    # 3. DB 확인
    print("\n3️⃣ DB 데이터 확인")
    print("-" * 60)
    check_db_data()
    
    print("\n" + "=" * 60)
    print("✅ 확인 완료")
    print("=" * 60)

if __name__ == '__main__':
    main()

