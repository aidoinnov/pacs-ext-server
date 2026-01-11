#!/usr/bin/env python3
"""me/series API 페이지네이션 테스트"""
import requests
import json

BASE_URL = "http://localhost:8080"

def login():
    """로그인"""
    resp = requests.post(f"{BASE_URL}/api/auth/login", json={
        "username": "iaid-pacs-admin",
        "password": "Qlalfqjsgh1!"
    })
    if resp.status_code != 200:
        print(f"❌ 로그인 실패: {resp.status_code}")
        print(f"   Response: {resp.text[:200]}")
        return None
    token = resp.json().get('token')
    if not token:
        print(f"❌ 토큰이 없습니다")
        print(f"   Response: {resp.json()}")
        return None
    print(f"✅ 로그인 성공 (token length: {len(token)})")
    return token

def get_series_uid(series_obj):
    """Series UID 추출"""
    return series_obj.get('0020000E', {}).get('Value', ['Unknown'])[0]

def test_pagination(token):
    """페이지네이션 테스트"""
    headers = {'Authorization': f'Bearer {token}'}
    
    print("\n" + "=" * 70)
    print("📊 /api/me/dicom/series 페이지네이션 테스트")
    print("=" * 70)
    
    # Test 1: page=1, page_size=5
    print("\n1️⃣ Test 1: page=1, page_size=5")
    print("-" * 70)
    resp1 = requests.get(
        f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=5',
        headers=headers,
        timeout=30
    )
    
    if resp1.status_code != 200:
        print(f"❌ Error: {resp1.status_code}")
        print(f"   Response: {resp1.text[:200]}")
        return
    
    data1 = resp1.json()
    print(f"✅ Status: {resp1.status_code}")
    print(f"📦 반환된 Series 수: {len(data1)}")
    
    if len(data1) > 0:
        print(f"\n처음 3개 Series UID:")
        for i, series in enumerate(data1[:3], 1):
            uid = get_series_uid(series)
            print(f"   {i}. {uid[:50]}...")
    
    # Test 2: page=2, page_size=5
    print("\n2️⃣ Test 2: page=2, page_size=5")
    print("-" * 70)
    resp2 = requests.get(
        f'{BASE_URL}/api/me/dicom/series?project_id=2&page=2&page_size=5',
        headers=headers,
        timeout=30
    )
    
    if resp2.status_code != 200:
        print(f"❌ Error: {resp2.status_code}")
        print(f"   Response: {resp2.text[:200]}")
        return
    
    data2 = resp2.json()
    print(f"✅ Status: {resp2.status_code}")
    print(f"📦 반환된 Series 수: {len(data2)}")
    
    if len(data2) > 0:
        print(f"\n처음 3개 Series UID:")
        for i, series in enumerate(data2[:3], 1):
            uid = get_series_uid(series)
            print(f"   {i}. {uid[:50]}...")
    
    # 중복 체크
    print("\n3️⃣ Test 3: 중복 체크 (Page 1 vs Page 2)")
    print("-" * 70)
    
    if len(data1) > 0 and len(data2) > 0:
        uid1_first = get_series_uid(data1[0])
        uid2_first = get_series_uid(data2[0])
        
        if uid1_first == uid2_first:
            print(f"❌ 실패: Page 1과 Page 2의 첫 번째 Series가 동일합니다!")
            print(f"   UID: {uid1_first[:50]}...")
        else:
            print(f"✅ 성공: Page 1과 Page 2의 데이터가 다릅니다")
            print(f"   Page 1 첫 번째: {uid1_first[:50]}...")
            print(f"   Page 2 첫 번째: {uid2_first[:50]}...")
        
        # 전체 중복 체크
        uids1 = set([get_series_uid(s) for s in data1])
        uids2 = set([get_series_uid(s) for s in data2])
        overlap = uids1 & uids2
        
        if overlap:
            print(f"⚠️  경고: {len(overlap)}개의 Series가 중복됩니다!")
            for uid in list(overlap)[:3]:
                print(f"   - {uid[:50]}...")
        else:
            print(f"✅ 중복 없음: Page 1과 Page 2에 겹치는 Series가 없습니다")
    
    # Test 4: page=1, page_size=100 (전체)
    print("\n4️⃣ Test 4: page=1, page_size=100 (전체 조회)")
    print("-" * 70)
    resp_all = requests.get(
        f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=100',
        headers=headers,
        timeout=30
    )
    
    if resp_all.status_code != 200:
        print(f"❌ Error: {resp_all.status_code}")
        return
    
    data_all = resp_all.json()
    print(f"✅ Status: {resp_all.status_code}")
    print(f"📦 전체 Series 수: {len(data_all)}")
    
    # 페이지네이션 검증
    print("\n5️⃣ Test 5: 페이지네이션 검증")
    print("-" * 70)
    
    expected_page1_count = min(5, len(data_all))
    expected_page2_count = min(5, max(0, len(data_all) - 5))
    
    print(f"전체 Series 수: {len(data_all)}")
    print(f"Page 1 예상 개수: {expected_page1_count}, 실제: {len(data1)}")
    print(f"Page 2 예상 개수: {expected_page2_count}, 실제: {len(data2)}")
    
    if len(data1) == expected_page1_count:
        print(f"✅ Page 1 개수 일치")
    else:
        print(f"❌ Page 1 개수 불일치!")
    
    if len(data2) == expected_page2_count:
        print(f"✅ Page 2 개수 일치")
    else:
        print(f"❌ Page 2 개수 불일치!")
    
    # UID 순서 검증
    if len(data_all) >= 10:
        all_uids = [get_series_uid(s) for s in data_all]
        page1_uids = [get_series_uid(s) for s in data1]
        page2_uids = [get_series_uid(s) for s in data2]
        
        expected_page1_uids = all_uids[:5]
        expected_page2_uids = all_uids[5:10]
        
        if page1_uids == expected_page1_uids:
            print(f"✅ Page 1 UID 순서 일치")
        else:
            print(f"❌ Page 1 UID 순서 불일치!")
            print(f"   예상: {expected_page1_uids[0][:30]}...")
            print(f"   실제: {page1_uids[0][:30]}...")
        
        if page2_uids == expected_page2_uids:
            print(f"✅ Page 2 UID 순서 일치")
        else:
            print(f"❌ Page 2 UID 순서 불일치!")
            if len(expected_page2_uids) > 0 and len(page2_uids) > 0:
                print(f"   예상: {expected_page2_uids[0][:30]}...")
                print(f"   실제: {page2_uids[0][:30]}...")

if __name__ == "__main__":
    token = login()
    if token:
        test_pagination(token)
        print("\n" + "=" * 70)
        print("테스트 완료!")
        print("=" * 70)

