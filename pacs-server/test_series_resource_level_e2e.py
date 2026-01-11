#!/usr/bin/env python3
"""
Series API resource_level 필터링 및 페이지네이션 E2E 테스트
"""
import requests
import json
import sys
from typing import Optional, List, Dict

BASE_URL = "http://localhost:8080"

def get_token() -> Optional[str]:
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

def get_series_list(token: str, project_id: int, page: int = 1, page_size: int = 100) -> List[Dict]:
    """Series 목록 조회"""
    headers = {'Authorization': f'Bearer {token}'}
    url = f'{BASE_URL}/api/me/dicom/series?project_id={project_id}&page={page}&page_size={page_size}'
    
    try:
        resp = requests.get(url, headers=headers, timeout=60)
        if resp.status_code == 200:
            data = resp.json()
            if isinstance(data, list):
                return data
            elif isinstance(data, dict):
                return data.get('series', [])
        else:
            print(f"❌ API 에러: {resp.status_code}")
            print(f"   {resp.text[:200]}")
    except Exception as e:
        print(f"❌ 요청 에러: {e}")
    
    return []

def extract_series_uid(series: Dict) -> Optional[str]:
    """Series UID 추출"""
    series_uid_tag = series.get('0020000E', {})
    if isinstance(series_uid_tag, dict):
        value = series_uid_tag.get('Value', [])
        if isinstance(value, list) and len(value) > 0:
            return str(value[0])
    return None

def test_resource_level_series_filtering(token: str, project_id: int = 2):
    """resource_level='SERIES' 필터링 테스트"""
    print("=" * 60)
    print("🧪 Test 1: resource_level='SERIES' 필터링")
    print("=" * 60)
    
    series_list = get_series_list(token, project_id, page=1, page_size=1000)
    series_uids = [extract_series_uid(s) for s in series_list if extract_series_uid(s)]
    
    print(f"\n📊 결과:")
    print(f"   총 Series 개수: {len(series_uids)}개")
    print(f"   고유 Series UID: {len(set(series_uids))}개")
    
    if len(series_uids) != len(set(series_uids)):
        print(f"   ⚠️  중복된 Series UID 발견!")
        from collections import Counter
        duplicates = {uid: count for uid, count in Counter(series_uids).items() if count > 1}
        for uid, count in duplicates.items():
            print(f"      - {uid}: {count}회")
        return False
    
    print(f"\n✅ 검증:")
    print(f"   - 중복 없음: ✅")
    print(f"   - 예상 개수: 5개 (project_data에 5개 레코드, 모두 SERIES 레벨)")
    
    if len(set(series_uids)) == 5:
        print(f"   - 실제 개수: {len(set(series_uids))}개 ✅")
        print(f"\n   Series UID 목록:")
        for i, uid in enumerate(sorted(set(series_uids)), 1):
            print(f"     {i}. {uid}")
        return True
    else:
        print(f"   - 실제 개수: {len(set(series_uids))}개 ❌ (예상: 5개)")
        return False

def test_pagination(token: str, project_id: int = 2):
    """페이지네이션 테스트"""
    print("\n" + "=" * 60)
    print("🧪 Test 2: 페이지네이션")
    print("=" * 60)
    
    # 전체 Series 조회
    all_series = get_series_list(token, project_id, page=1, page_size=1000)
    all_series_uids = sorted([extract_series_uid(s) for s in all_series if extract_series_uid(s)])
    total_count = len(all_series_uids)
    
    print(f"\n📊 전체 Series: {total_count}개")
    
    if total_count == 0:
        print("   ⚠️  Series가 없어 페이지네이션 테스트를 건너뜁니다")
        return True
    
    # 페이지별 조회
    page_size = 2  # 작은 페이지 크기로 테스트
    total_pages = (total_count + page_size - 1) // page_size
    
    print(f"\n📄 페이지네이션 테스트 (page_size={page_size}):")
    print("-" * 60)
    
    all_paginated_uids = []
    for page in range(1, total_pages + 2):  # 마지막 페이지 + 1 (빈 페이지 테스트)
        page_series = get_series_list(token, project_id, page=page, page_size=page_size)
        page_uids = sorted([extract_series_uid(s) for s in page_series if extract_series_uid(s)])
        
        print(f"   Page {page}: {len(page_uids)}개 Series")
        
        if page <= total_pages:
            if len(page_uids) > page_size:
                print(f"      ❌ 페이지 크기 초과! (예상: 최대 {page_size}개, 실제: {len(page_uids)}개)")
                return False
            
            all_paginated_uids.extend(page_uids)
        else:
            # 빈 페이지
            if len(page_uids) > 0:
                print(f"      ⚠️  빈 페이지여야 하는데 {len(page_uids)}개 반환됨")
            else:
                print(f"      ✅ 빈 페이지 (정상)")
    
    # 중복 확인
    if len(all_paginated_uids) != len(set(all_paginated_uids)):
        print(f"\n   ❌ 페이지네이션 결과에 중복이 있습니다!")
        from collections import Counter
        duplicates = {uid: count for uid, count in Counter(all_paginated_uids).items() if count > 1}
        for uid, count in duplicates.items():
            print(f"      - {uid}: {count}회")
        return False
    
    # 전체 Series와 비교
    all_paginated_uids_sorted = sorted(set(all_paginated_uids))
    if all_paginated_uids_sorted != all_series_uids:
        print(f"\n   ❌ 페이지네이션 결과가 전체 결과와 일치하지 않습니다!")
        print(f"      전체: {len(all_series_uids)}개")
        print(f"      페이지네이션: {len(all_paginated_uids_sorted)}개")
        
        missing = set(all_series_uids) - set(all_paginated_uids_sorted)
        extra = set(all_paginated_uids_sorted) - set(all_series_uids)
        
        if missing:
            print(f"      누락된 Series: {len(missing)}개")
            for uid in list(missing)[:5]:
                print(f"        - {uid}")
        
        if extra:
            print(f"      추가된 Series: {len(extra)}개")
            for uid in list(extra)[:5]:
                print(f"        - {uid}")
        
        return False
    
    print(f"\n✅ 페이지네이션 검증:")
    print(f"   - 총 페이지: {total_pages}개")
    print(f"   - 전체 Series: {total_count}개")
    print(f"   - 페이지네이션으로 수집한 Series: {len(all_paginated_uids_sorted)}개")
    print(f"   - 중복 없음: ✅")
    print(f"   - 전체 결과와 일치: ✅")
    
    return True

def test_different_page_sizes(token: str, project_id: int = 2):
    """다양한 페이지 크기 테스트"""
    print("\n" + "=" * 60)
    print("🧪 Test 3: 다양한 페이지 크기")
    print("=" * 60)
    
    all_series = get_series_list(token, project_id, page=1, page_size=1000)
    all_series_uids = sorted([extract_series_uid(s) for s in all_series if extract_series_uid(s)])
    total_count = len(all_series_uids)
    
    print(f"\n📊 전체 Series: {total_count}개")
    
    if total_count == 0:
        print("   ⚠️  Series가 없어 테스트를 건너뜁니다")
        return True
    
    page_sizes = [1, 2, 3, 5, 10, 20]
    all_passed = True
    
    for page_size in page_sizes:
        if page_size > total_count:
            continue
        
        print(f"\n   페이지 크기: {page_size}")
        print("-" * 60)
        
        page_series = get_series_list(token, project_id, page=1, page_size=page_size)
        page_uids = sorted([extract_series_uid(s) for s in page_series if extract_series_uid(s)])
        
        if len(page_uids) > page_size:
            print(f"      ❌ 페이지 크기 초과! (예상: 최대 {page_size}개, 실제: {len(page_uids)}개)")
            all_passed = False
        elif len(page_uids) == min(page_size, total_count):
            print(f"      ✅ 올바른 개수: {len(page_uids)}개")
        else:
            print(f"      ⚠️  예상과 다른 개수: {len(page_uids)}개 (예상: {min(page_size, total_count)}개)")
    
    return all_passed

def test_edge_cases(token: str, project_id: int = 2):
    """엣지 케이스 테스트"""
    print("\n" + "=" * 60)
    print("🧪 Test 4: 엣지 케이스")
    print("=" * 60)
    
    all_passed = True
    
    # 1. page=0 테스트
    print("\n   1. page=0 테스트")
    print("-" * 60)
    series = get_series_list(token, project_id, page=0, page_size=10)
    if len(series) == 0:
        print("      ✅ page=0은 빈 결과 반환 (또는 page=1로 처리)")
    else:
        print(f"      ⚠️  page=0에서 {len(series)}개 반환됨")
    
    # 2. page_size=0 테스트
    print("\n   2. page_size=0 테스트")
    print("-" * 60)
    series = get_series_list(token, project_id, page=1, page_size=0)
    if len(series) == 0:
        print("      ✅ page_size=0은 빈 결과 반환")
    else:
        print(f"      ⚠️  page_size=0에서 {len(series)}개 반환됨")
        all_passed = False
    
    # 3. 매우 큰 page_size 테스트
    print("\n   3. 매우 큰 page_size 테스트")
    print("-" * 60)
    series = get_series_list(token, project_id, page=1, page_size=10000)
    all_series = get_series_list(token, project_id, page=1, page_size=1000)
    if len(series) == len(all_series):
        print(f"      ✅ 큰 page_size도 정상 처리: {len(series)}개")
    else:
        print(f"      ⚠️  큰 page_size 처리 결과: {len(series)}개 (전체: {len(all_series)}개)")
    
    # 4. 음수 page 테스트
    print("\n   4. 음수 page 테스트")
    print("-" * 60)
    series = get_series_list(token, project_id, page=-1, page_size=10)
    if len(series) == 0:
        print("      ✅ 음수 page는 빈 결과 반환 (또는 에러 처리)")
    else:
        print(f"      ⚠️  음수 page에서 {len(series)}개 반환됨")
    
    return all_passed

def main():
    print("=" * 60)
    print("🧪 Series API resource_level 필터링 및 페이지네이션 E2E 테스트")
    print("=" * 60)
    
    # 1. 로그인
    print("\n1️⃣ 로그인")
    print("-" * 60)
    token = get_token()
    if not token:
        print("❌ 토큰 획득 실패")
        sys.exit(1)
    print("✅ 토큰 획득 성공")
    
    project_id = 2
    
    # 2. resource_level 필터링 테스트
    test1_passed = test_resource_level_series_filtering(token, project_id)
    
    # 3. 페이지네이션 테스트
    test2_passed = test_pagination(token, project_id)
    
    # 4. 다양한 페이지 크기 테스트
    test3_passed = test_different_page_sizes(token, project_id)
    
    # 5. 엣지 케이스 테스트
    test4_passed = test_edge_cases(token, project_id)
    
    # 결과 요약
    print("\n" + "=" * 60)
    print("📊 테스트 결과 요약")
    print("=" * 60)
    print(f"Test 1 (resource_level 필터링): {'✅ 통과' if test1_passed else '❌ 실패'}")
    print(f"Test 2 (페이지네이션): {'✅ 통과' if test2_passed else '❌ 실패'}")
    print(f"Test 3 (다양한 페이지 크기): {'✅ 통과' if test3_passed else '❌ 실패'}")
    print(f"Test 4 (엣지 케이스): {'✅ 통과' if test4_passed else '❌ 실패'}")
    
    all_passed = test1_passed and test2_passed and test3_passed and test4_passed
    
    if all_passed:
        print("\n✅ 모든 테스트 통과!")
        sys.exit(0)
    else:
        print("\n❌ 일부 테스트 실패")
        sys.exit(1)

if __name__ == '__main__':
    main()

