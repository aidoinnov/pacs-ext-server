#!/usr/bin/env python3
"""
데이터베이스에서 직접 Study/Series 조회 후 프로젝트에 할당

사용법:
    python3 assign_all_data_from_db.py --project-id 2
"""

import argparse
import requests
import sys
import time
import os
from typing import List, Dict, Optional

BASE_URL = "http://localhost:8080"
PROJECT_ID = 2
USER_ID = 1

# 데이터베이스 연결 정보 (환경 변수에서 가져오기)
DB_HOST = os.getenv("APP_DATABASE__HOST", "localhost")
DB_PORT = int(os.getenv("APP_DATABASE__PORT", "5456"))
DB_USER = os.getenv("APP_DATABASE__USERNAME", "admin")
DB_PASSWORD = os.getenv("APP_DATABASE__PASSWORD", "admin123")
DB_NAME = os.getenv("APP_DATABASE__DATABASE", "pacs_db")

def get_login_token():
    """로그인 API를 통해 토큰 획득"""
    url = f"{BASE_URL}/api/auth/login"
    data = {
        "username": "iaid-pacs-admin",
        "password": "Qlalfqjsgh1!"
    }
    
    try:
        print("🔐 로그인 중...")
        response = requests.post(url, json=data, timeout=30)
        
        if response.status_code == 200:
            result = response.json()
            token = result.get('token') or result.get('access_token')
            if token:
                print(f"✅ 로그인 성공!")
                print(f"   User ID: {result.get('user_id', 'N/A')}")
                print(f"   Username: {result.get('username', 'N/A')}")
                return token
            else:
                print(f"❌ 토큰을 찾을 수 없습니다")
                print(json.dumps(result, indent=2))
                return None
        else:
            print(f"❌ 로그인 실패: {response.status_code}")
            print(f"Response: {response.text}")
            return None
    except Exception as e:
        print(f"❌ 에러: {e}")
        return None

def get_studies_from_dcm4chee_db():
    """Dcm4chee 데이터베이스에서 직접 Study 조회"""
    try:
        import psycopg2
    except ImportError:
        print("❌ psycopg2가 설치되지 않았습니다.")
        print("   설치: pip install psycopg2-binary")
        return []
    
    print(f"📋 Dcm4chee DB에서 Study 조회 중...")
    print(f"   Host: {DB_HOST}:{DB_PORT}")
    
    try:
        # Dcm4chee DB 연결 (포트 5457)
        conn = psycopg2.connect(
            host=DB_HOST.replace("localhost", "127.0.0.1") if DB_HOST == "localhost" else DB_HOST,
            port=5457,  # Dcm4chee DB 포트
            user="pacsadmin",
            password="HhL}qb(tl}?zJ4}(",
            database="postgres"
        )
        cur = conn.cursor()
        
        # Study 조회 (study 테이블 - patient와 조인)
        # sync_worker.rs를 참고: patient_id는 NULL로 처리
        cur.execute("""
            SELECT DISTINCT 
                st.study_iuid,
                st.study_desc,
                NULL::text AS patient_id,
                st.study_date,
                st.updated_time
            FROM study st
            WHERE st.study_iuid IS NOT NULL
            ORDER BY st.updated_time DESC
            LIMIT 10000
        """)
        
        studies = []
        for row in cur.fetchall():
            study_uid, study_desc, patient_id, study_date, updated_time = row
            studies.append({
                "study_uid": study_uid,
                "study_description": study_desc or "",
                "patient_id": patient_id or "",
                "patient_name": "",
                "study_date": study_date if study_date else None
            })
        
        cur.close()
        conn.close()
        
        print(f"✅ {len(studies)}개 Study 발견")
        return studies
        
    except Exception as e:
        print(f"❌ DB 조회 실패: {e}")
        return []

def get_series_from_dcm4chee_db(study_uid: str):
    """Dcm4chee 데이터베이스에서 특정 Study의 Series 조회"""
    try:
        import psycopg2
    except ImportError:
        return []
    
    try:
        conn = psycopg2.connect(
            host="127.0.0.1",
            port=5457,
            user="pacsadmin",
            password="HhL}qb(tl}?zJ4}(",
            database="postgres"
        )
        cur = conn.cursor()
        
        # Series 조회
        cur.execute("""
            SELECT DISTINCT
                s.series_iuid,
                s.series_desc,
                s.modality,
                s.series_no
            FROM series s
            INNER JOIN study st ON s.study_fk = st.pk
            WHERE st.study_iuid = %s
              AND s.series_iuid IS NOT NULL
            ORDER BY s.series_no ASC NULLS LAST
        """, (study_uid,))
        
        series_list = []
        for row in cur.fetchall():
            series_uid, series_desc, modality, series_no = row
            series_list.append({
                "series_uid": series_uid,
                "series_description": series_desc or "",
                "modality": modality or "",
                "series_number": series_no
            })
        
        cur.close()
        conn.close()
        
        return series_list
        
    except Exception as e:
        print(f"  ⚠️  DB 조회 실패: {e}")
        return []

def assign_series(token: str, study_uid: str, series_data: Dict, assigned_series: set, failed_series: list) -> bool:
    """Series를 프로젝트에 할당"""
    series_uid = series_data.get("series_uid")
    
    if not series_uid:
        return False
        
    if series_uid in assigned_series:
        return True
    
    url = f"{BASE_URL}/api/projects/{PROJECT_ID}/series/assign"
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
    
    payload = {
        "study_uid": study_uid,
        "series_uid": series_uid,
        "series_description": series_data.get("series_description", ""),
        "modality": series_data.get("modality", ""),
        "series_number": series_data.get("series_number")
    }
    
    try:
        response = requests.post(url, json=payload, headers=headers, timeout=30)
        if response.status_code in [200, 201]:
            assigned_series.add(series_uid)
            return True
        elif response.status_code == 409:
            # 이미 할당됨
            assigned_series.add(series_uid)
            return True
        else:
            error_msg = response.text[:200] if response.text else "Unknown error"
            failed_series.append({
                "study_uid": study_uid,
                "series_uid": series_uid,
                "error": f"{response.status_code}: {error_msg}"
            })
            return False
    except requests.exceptions.RequestException as e:
        failed_series.append({
            "study_uid": study_uid,
            "series_uid": series_uid,
            "error": str(e)
        })
        return False

def main():
    print("=" * 60)
    print("🚀 프로젝트에 모든 데이터 할당 (DB 직접 조회)")
    print("=" * 60)
    print(f"프로젝트 ID: {PROJECT_ID}")
    print(f"Base URL: {BASE_URL}")
    print()
    
    # 1. 로그인하여 토큰 획득
    token = get_login_token()
    if not token:
        print("❌ 로그인 실패. 종료합니다.")
        sys.exit(1)
    
    print()
    
    # 2. Dcm4chee DB에서 모든 Study 조회
    studies = get_studies_from_dcm4chee_db()
    
    if not studies:
        print("❌ Study가 없습니다. 종료합니다.")
        sys.exit(1)
    
    print()
    
    # 3. 각 Study와 Series 할당
    assigned_series = set()
    failed_series = []
    total_studies = len(studies)
    total_series = 0
    
    for idx, study_data in enumerate(studies, 1):
        study_uid = study_data.get("study_uid")
        if not study_uid:
            print(f"  [{idx}/{total_studies}] Study UID 없음, 건너뜀")
            continue
        
        print(f"  [{idx}/{total_studies}] Study: {study_uid[:50]}...")
        
        # Study의 모든 Series 조회
        series_list = get_series_from_dcm4chee_db(study_uid)
        total_series += len(series_list)
        
        if not series_list:
            print(f"    ⚠️  Series 없음")
            continue
        
        # 각 Series 할당
        for series_idx, series_data in enumerate(series_list, 1):
            success = assign_series(token, study_uid, series_data, assigned_series, failed_series)
            if series_idx % 10 == 0 or series_idx == len(series_list):
                print(f"    [{series_idx}/{len(series_list)}] Series 할당 중... ({len(assigned_series)}개 성공, {len(failed_series)}개 실패)")
            
            # API 부하 방지를 위한 짧은 대기
            time.sleep(0.05)
        
        print(f"    ✅ {len(series_list)}개 Series 처리 완료")
        print()
    
    # 결과 출력
    print("=" * 60)
    print("📊 할당 결과")
    print("=" * 60)
    print(f"✅ 할당된 Series: {len(assigned_series)}개")
    print(f"❌ 실패한 Series: {len(failed_series)}개")
    print(f"📋 총 처리한 Study: {total_studies}개")
    print(f"📋 총 처리한 Series: {total_series}개")
    
    if failed_series:
        print("\n⚠️  실패한 Series 목록 (최대 10개):")
        for failed in failed_series[:10]:
            study_uid = failed.get('study_uid', 'Unknown')[:50]
            series_uid = failed.get('series_uid', 'Unknown')[:50]
            error = failed.get('error', 'Unknown error')[:100]
            print(f"  - Study: {study_uid}...")
            print(f"    Series: {series_uid}...")
            print(f"    Error: {error}")
    
    print("\n✅ 완료!")

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='DB에서 직접 조회하여 프로젝트에 할당')
    parser.add_argument('--project-id', type=int, default=2, help='프로젝트 ID')
    args = parser.parse_args()
    PROJECT_ID = args.project_id
    main()

