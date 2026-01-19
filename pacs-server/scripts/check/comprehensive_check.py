#!/usr/bin/env python3
"""
종합 검사: DB 데이터와 Dcm4chee 통신 확인
"""
import requests
import psycopg2
import os
import sys

BASE_URL = "http://localhost:8080"

# DB 연결 정보
DB_CONFIG = {
    'host': os.getenv('APP_DATABASE__HOST', '127.0.0.1'),
    'port': int(os.getenv('APP_DATABASE__PORT', '5456')),
    'user': os.getenv('APP_DATABASE__USERNAME', 'admin'),
    'password': os.getenv('APP_DATABASE__PASSWORD', 'admin123'),
    'database': os.getenv('APP_DATABASE__DATABASE', 'pacs_db')
}

print("=" * 60)
print("🔍 종합 검사: DB 데이터 vs Dcm4chee 통신")
print("=" * 60)

# ============================================================================
# 1. DB 데이터 확인
# ============================================================================
print("\n1️⃣ DB 데이터 확인")
print("-" * 60)

db_ok = False
allowed_series_uids = []

try:
    conn = psycopg2.connect(**DB_CONFIG)
    cur = conn.cursor()
    print("✅ DB 연결 성공")
    
    # project_data 확인
    cur.execute("SELECT COUNT(*) FROM project_data WHERE project_id = 2")
    count = cur.fetchone()[0]
    print(f"   project_data (project_id=2) 개수: {count}")
    
    if count == 0:
        print("   ❌ project_data에 데이터가 없습니다!")
    else:
        # get_allowed_series_uids 쿼리 실행
        cur.execute("""
            SELECT DISTINCT pdser.series_uid
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
            WHERE pd.project_id = 2
              AND pdser.series_uid IS NOT NULL
        """)
        series_uids = cur.fetchall()
        allowed_series_uids = [uid[0] for uid in series_uids]
        print(f"   ✅ 허용된 Series UID 개수: {len(allowed_series_uids)}")
        
        if len(allowed_series_uids) == 0:
            print("   ❌ 허용된 Series UID가 없습니다!")
            print("   → 조인 문제 또는 데이터 누락 가능성")
            
            # 원인 분석
            cur.execute("""
                SELECT COUNT(*) FROM project_data pd
                LEFT JOIN project_data_study pds ON pd.study_id = pds.id
                WHERE pd.project_id = 2 AND pds.id IS NULL
            """)
            study_join_fail = cur.fetchone()[0]
            
            cur.execute("""
                SELECT COUNT(*) FROM project_data pd
                INNER JOIN project_data_study pds ON pd.study_id = pds.id
                LEFT JOIN project_data_series pdser ON pds.id = pdser.study_id
                WHERE pd.project_id = 2 AND pdser.id IS NULL
            """)
            series_join_fail = cur.fetchone()[0]
            
            if study_join_fail > 0:
                print(f"      ⚠️  project_data_study 조인 실패: {study_join_fail}개")
            if series_join_fail > 0:
                print(f"      ⚠️  project_data_series 조인 실패: {series_join_fail}개")
        else:
            print(f"   ✅ 허용된 Series UID 샘플 (최대 5개):")
            for uid in allowed_series_uids[:5]:
                print(f"      - {uid}")
            db_ok = True
    
    cur.close()
    conn.close()
    
except psycopg2.OperationalError as e:
    print(f"   ❌ DB 연결 실패: {e}")
    print("   → DB 터널이 열려있는지 확인하세요")
except Exception as e:
    print(f"   ❌ DB 에러: {e}")

# ============================================================================
# 2. Dcm4chee 통신 확인
# ============================================================================
print("\n2️⃣ Dcm4chee 통신 확인")
print("-" * 60)

# 로그인
try:
    login_resp = requests.post(f'{BASE_URL}/api/auth/login', json={
        'username': 'iaid-pacs-admin',
        'password': 'Qlalfqjsgh1!'
    }, timeout=10)
    
    if login_resp.status_code != 200:
        print(f"   ❌ 로그인 실패: {login_resp.status_code}")
        print(f"      {login_resp.text[:200]}")
        sys.exit(1)
    
    token = login_resp.json().get('token')
    headers = {'Authorization': f'Bearer {token}'}
    print("   ✅ 로그인 성공")
    
except Exception as e:
    print(f"   ❌ 로그인 에러: {e}")
    sys.exit(1)

dcm4chee_ok = False
qido_series_count = 0
qido_series_uids = []

# Dcm4chee QIDO 직접 호출 테스트
try:
    print("   📡 Dcm4chee QIDO /series 호출 중...")
    # project_id 없이 호출하려면 전역 접근 권한이 필요하므로
    # 대신 /api/dicom/studies로 테스트
    test_resp = requests.get(
        f'{BASE_URL}/api/dicom/studies?limit=1',
        headers=headers,
        timeout=30
    )
    
    if test_resp.status_code == 502:
        print(f"   ❌ Dcm4chee 연결 실패: 502 Bad Gateway")
        print("   → Dcm4chee 서버가 응답하지 않습니다")
    elif test_resp.status_code == 400:
        print(f"   ⚠️  project_id 필요 (정상)")
        # project_id와 함께 시도
        test_resp2 = requests.get(
            f'{BASE_URL}/api/dicom/studies?project_id=2&limit=1',
            headers=headers,
            timeout=30
        )
        if test_resp2.status_code == 502:
            print(f"   ❌ Dcm4chee 연결 실패: 502 Bad Gateway")
        elif test_resp2.status_code == 200:
            print(f"   ✅ Dcm4chee 연결 성공")
            dcm4chee_ok = True
        else:
            print(f"   ⚠️  응답 코드: {test_resp2.status_code}")
    elif test_resp.status_code == 200:
        print(f"   ✅ Dcm4chee 연결 성공")
        dcm4chee_ok = True
    else:
        print(f"   ⚠️  응답 코드: {test_resp.status_code}")
        print(f"      {test_resp.text[:200]}")
    
    # /api/me/dicom/series로 실제 QIDO 응답 확인
    print("   📡 /api/me/dicom/series?project_id=2 호출 중...")
    series_resp = requests.get(
        f'{BASE_URL}/api/me/dicom/series?project_id=2&limit=10',
        headers=headers,
        timeout=30
    )
    
    if series_resp.status_code == 200:
        series_data = series_resp.json()
        if isinstance(series_data, list):
            qido_series_count = len(series_data)
            print(f"   ✅ QIDO 응답: {qido_series_count}개 Series")
            
            # Series UID 추출
            for series in series_data:
                series_uid = series.get('0020000E', {}).get('Value', [None])[0]
                if series_uid:
                    qido_series_uids.append(series_uid)
            
            if qido_series_count > 0:
                print(f"   ✅ 추출된 Series UID: {len(qido_series_uids)}개")
                dcm4chee_ok = True
            else:
                print(f"   ⚠️  QIDO 응답은 있지만 Series가 없음")
        else:
            print(f"   ⚠️  응답 형식이 예상과 다름: {type(series_data)}")
    elif series_resp.status_code == 502:
        print(f"   ❌ Dcm4chee 연결 실패: 502 Bad Gateway")
    else:
        print(f"   ⚠️  응답 코드: {series_resp.status_code}")
        print(f"      {series_resp.text[:200]}")
        
except requests.exceptions.Timeout:
    print(f"   ❌ Dcm4chee 연결 타임아웃")
except requests.exceptions.ConnectionError:
    print(f"   ❌ 서버 연결 실패")
except Exception as e:
    print(f"   ❌ 에러: {e}")

# ============================================================================
# 3. 종합 분석
# ============================================================================
print("\n3️⃣ 종합 분석")
print("-" * 60)

if db_ok and dcm4chee_ok:
    print("✅ DB 데이터와 Dcm4chee 통신 모두 정상")
    print(f"   - DB 허용 Series UID: {len(allowed_series_uids)}개")
    print(f"   - QIDO 반환 Series: {qido_series_count}개")
    
    if len(allowed_series_uids) > 0 and qido_series_count == 0:
        print("\n   ⚠️  문제: 필터링 로직 문제 가능성")
        print("   → QIDO는 작동하지만 필터링 후 결과가 없음")
        print("   → Series UID 형식 불일치 가능성")
        
        # UID 형식 비교
        if len(qido_series_uids) > 0 and len(allowed_series_uids) > 0:
            print(f"\n   DB UID 샘플: {allowed_series_uids[0]}")
            print(f"   QIDO UID 샘플: {qido_series_uids[0]}")
            if allowed_series_uids[0] != qido_series_uids[0]:
                print("   ⚠️  UID 형식이 다를 수 있음")
    
    elif len(allowed_series_uids) == 0:
        print("\n   ❌ 문제: DB 데이터 문제")
        print("   → 허용된 Series UID가 없음")
        print("   → 데이터 할당이 제대로 되지 않았을 수 있음")
    
    elif qido_series_count == 0:
        print("\n   ❌ 문제: Dcm4chee 통신 문제")
        print("   → QIDO 응답이 없음")
        print("   → Dcm4chee 연결 또는 인증 문제 가능성")

elif not db_ok:
    print("❌ 문제: DB 데이터 문제")
    print("   → project_data에 데이터가 없거나")
    print("   → 조인 실패로 허용된 Series UID를 찾을 수 없음")
    print("\n   해결 방법:")
    print("   1. 데이터 할당 재실행")
    print("   2. DB 쿼리 직접 확인 (test_get_allowed_series_uids.sql)")

elif not dcm4chee_ok:
    print("❌ 문제: Dcm4chee 통신 문제")
    print("   → Dcm4chee 서버 연결 실패")
    print("   → 인증 문제 또는 네트워크 문제 가능성")
    print("\n   해결 방법:")
    print("   1. Dcm4chee 서버 상태 확인")
    print("   2. 네트워크 연결 확인")
    print("   3. 인증 설정 확인")

else:
    print("❌ 문제: DB 데이터와 Dcm4chee 통신 모두 문제")
    print("   → 두 가지 모두 확인 필요")

print("\n" + "=" * 60)
print("✅ 검사 완료")
print("=" * 60)

