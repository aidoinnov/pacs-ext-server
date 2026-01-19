#!/usr/bin/env python3
"""
Series가 11개만 나오는 원인 확인
- QIDO 응답 확인
- 필터링 전후 비교
"""
import requests
import json

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

def check_series_detailed(token, project_id=2):
    """Series 상세 확인"""
    print("=" * 60)
    print(f"🔍 Series 상세 분석 (project_id={project_id})")
    print("=" * 60)
    
    headers = {
        'Authorization': f'Bearer {token}'
    }
    
    # 1. /api/me/dicom/series 응답 확인
    print("\n1️⃣ /api/me/dicom/series 응답")
    print("-" * 60)
    url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page=1&page_size=1000'
    print(f"URL: {url}")
    
    try:
        resp = requests.get(url, headers=headers, timeout=60)
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                series_list = data
            else:
                series_list = data.get('series', [])
            
            print(f"✅ 응답 Series 개수: {len(series_list)}개")
            
            # Series UID 목록
            series_uids = []
            for series in series_list:
                series_uid_tag = series.get('0020000E', {})
                series_uid = series_uid_tag.get('Value', [None])[0] if isinstance(series_uid_tag, dict) else None
                if series_uid:
                    series_uids.append(series_uid)
            
            print(f"\n📋 Series UID 목록:")
            for i, uid in enumerate(series_uids, 1):
                print(f"   {i:2d}. {uid}")
            
            # Study별 그룹화
            study_groups = {}
            for series in series_list:
                study_uid_tag = series.get('0020000D', {})
                study_uid = study_uid_tag.get('Value', [None])[0] if isinstance(study_uid_tag, dict) else None
                series_uid_tag = series.get('0020000E', {})
                series_uid = series_uid_tag.get('Value', [None])[0] if isinstance(series_uid_tag, dict) else None
                
                if study_uid and series_uid:
                    if study_uid not in study_groups:
                        study_groups[study_uid] = []
                    study_groups[study_uid].append(series_uid)
            
            print(f"\n📚 Study별 Series:")
            for study_uid, series_list in study_groups.items():
                print(f"   Study {study_uid[:60]}...")
                print(f"      → {len(series_list)}개 Series")
                for series_uid in series_list[:5]:  # 최대 5개만 표시
                    print(f"         - {series_uid}")
                if len(series_list) > 5:
                    print(f"         ... 외 {len(series_list) - 5}개")
    except Exception as e:
        print(f"❌ 에러: {e}")
        import traceback
        traceback.print_exc()
    
    # 2. 이전에 28개였던 것과 비교
    print("\n2️⃣ 이전 데이터와 비교")
    print("-" * 60)
    print("이전에 28개 Series가 나왔었습니다.")
    print("현재는 11개만 나옵니다.")
    print("\n가능한 원인:")
    print("  1. DB에서 일부 Series가 삭제되었거나 다른 프로젝트로 이동")
    print("  2. QIDO에서 일부 Series를 가져오지 못함")
    print("  3. 필터링 로직에서 일부가 제외됨")
    print("  4. project_data에 실제로 11개만 할당되어 있음")
    
    # 3. 서버 로그 확인 안내
    print("\n3️⃣ 서버 로그 확인 필요")
    print("-" * 60)
    print("서버 로그에서 다음을 확인하세요:")
    print("  - 'get_allowed_series_uids' 쿼리 결과")
    print("  - 'QIDO returned X series' 메시지")
    print("  - 'Filtered X series from Y QIDO results' 메시지")
    print("\n예상되는 로그:")
    print("  🔍 Gateway /series: Found 11 allowed series UIDs for project 2")
    print("  🔍 Gateway /series: QIDO returned X series")
    print("  🔍 Gateway /series: Filtered 11 series from X QIDO results")

def main():
    print("=" * 60)
    print("🔍 Series 11개 원인 분석")
    print("=" * 60)
    
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        return
    
    check_series_detailed(token, project_id=2)
    
    print("\n" + "=" * 60)
    print("💡 다음 단계")
    print("=" * 60)
    print("1. 서버 로그 확인:")
    print("   - allowed_series_uids 개수")
    print("   - QIDO 응답 Series 개수")
    print("   - 필터링 후 Series 개수")
    print("\n2. DB 직접 확인:")
    print("   - project_data 테이블에서 project_id=2인 데이터")
    print("   - project_data_series에서 실제 할당된 Series 개수")
    print("\n3. 이전 할당 스크립트 확인:")
    print("   - assign_all_data_from_db.py 실행 결과")
    print("   - 데이터가 실제로 할당되었는지 확인")

if __name__ == '__main__':
    main()

