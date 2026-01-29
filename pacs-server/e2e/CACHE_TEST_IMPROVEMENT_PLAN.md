# ETag 캐시 E2E 테스트 개선 계획

## 📊 현재 상태

### ✅ 구현된 기본 테스트
- Subject API: `test_subject_cache.py`
- Project Data Access API: `test_project_data_cache.py`
- Study List View API: `test_study_list_view_cache.py`

### 🎯 각 테스트의 현재 커버리지
1. ✅ ETag 생성 확인
2. ✅ 304 Not Modified 응답 확인
3. ✅ Cache-Control 헤더 검증
4. ✅ no-cache 헤더 처리

---

## ❌ 부족한 테스트 시나리오

### 1. 캐시 무효화 (Cache Invalidation)
**중요도:** 🔴 높음

**테스트 시나리오:**
```python
def test_cache_invalidation_after_data_change(token, project_id=634):
    """데이터 변경 후 ETag가 변경되는지 확인"""
    # 1. 첫 요청 - ETag1 획득
    # 2. Subject/Study/View 추가 또는 수정
    # 3. 두 번째 요청 - ETag2 획득
    # 4. ETag1 != ETag2 확인
    # 5. If-None-Match: ETag1로 요청 → 200 OK (새 데이터)
```

**구현 필요성:**
- ETag가 실제로 데이터 변경을 반영하는지 검증
- 캐시가 stale 데이터를 반환하지 않는지 확인

---

### 2. 잘못된 ETag 처리
**중요도:** 🟡 중간

**테스트 시나리오:**
```python
def test_invalid_etag_format(token):
    """잘못된 ETag 형식 처리 확인"""
    # If-None-Match: "invalid-format" → 200 OK
    # If-None-Match: "" → 200 OK
    # If-None-Match: "12345" (존재하지 않는 타임스탬프) → 200 OK
```

**구현 필요성:**
- 서버가 잘못된 ETag를 안전하게 처리하는지 확인
- 보안 취약점 방지

---

### 3. 빈 데이터 처리
**중요도:** 🟡 중간

**테스트 시나리오:**
```python
def test_empty_list_etag(token):
    """빈 목록에 대한 ETag 생성 확인"""
    # Subject가 없는 프로젝트 조회
    # ETag 생성 확인 (기본값: W/"0" 또는 W/"-62135596800")
    # 두 번째 요청 시 304 응답 확인
```

**구현 필요성:**
- 빈 목록도 캐싱이 정상 작동하는지 확인
- 기본값 타임스탬프 검증

---

### 4. 필터 파라미터별 캐시 구분
**중요도:** 🟢 낮음 (Study List View만 해당)

**테스트 시나리오:**
```python
def test_etag_varies_by_filter(token):
    """필터 파라미터별로 다른 ETag 생성 확인"""
    # GET /api/study-list-views?scopeType=project&scopeId=1 → ETag1
    # GET /api/study-list-views?scopeType=user&scopeId=2 → ETag2
    # ETag1 != ETag2 확인 (다른 필터 = 다른 데이터 = 다른 ETag)
```

**구현 필요성:**
- 필터별로 캐시가 올바르게 구분되는지 확인
- 잘못된 캐시 재사용 방지

---

### 5. 성능 측정
**중요도:** 🟡 중간

**테스트 시나리오:**
```python
def test_cache_performance(token):
    """304 응답이 200 응답보다 빠른지 확인"""
    # 1차 요청 (200 OK) - 시간 측정
    # 2차 요청 (304 Not Modified) - 시간 측정
    # 3차 요청 (304 Not Modified) - 시간 측정
    # 평균 304 응답 시간 < 200 응답 시간 확인
```

**구현 필요성:**
- 캐시가 실제로 성능 개선을 제공하는지 검증
- 네트워크 대역폭 절감 확인

---

### 6. 동시 요청 처리
**중요도:** 🟢 낮음

**테스트 시나리오:**
```python
def test_concurrent_requests(token):
    """동시에 여러 요청 시 ETag 일관성 확인"""
    # 10개 동시 요청
    # 모든 요청이 동일한 ETag 반환 확인
    # 모든 요청이 성공 (200 또는 304) 확인
```

**구현 필요성:**
- Race condition 방지
- 캐시 일관성 검증

---

## 🎯 우선순위별 구현 계획

### Phase 1: 필수 테스트 (🔴 높음)
1. **캐시 무효화 테스트** - 가장 중요!
   - Subject: Subject 추가 후 ETag 변경 확인
   - Project Data: Study 추가 후 ETag 변경 확인
   - Study List View: View 수정 후 ETag 변경 확인

### Phase 2: 권장 테스트 (🟡 중간)
2. **잘못된 ETag 처리**
3. **빈 데이터 처리**
4. **성능 측정**

### Phase 3: 선택 테스트 (🟢 낮음)
5. **필터 파라미터별 캐시 구분** (Study List View만)
6. **동시 요청 처리**

---

## 📋 구현 예시

### Subject API - 캐시 무효화 테스트
```python
def test_subject_cache_invalidation_after_create(token, project_id=634):
    """Subject 추가 후 ETag 변경 확인"""
    headers = {"Authorization": f"Bearer {token}"}
    url = f"{BASE_URL}/api/projects/{project_id}/subjects"
    
    # 1. 첫 요청 - ETag1 획득
    response1 = requests.get(url, headers=headers)
    etag1 = response1.headers["ETag"]
    count1 = len(response1.json())
    
    # 2. Subject 추가
    create_response = requests.post(
        url,
        headers=headers,
        json={"subject_no": f"TEST-{int(time.time())}"}
    )
    assert create_response.status_code == 201
    
    # 3. 두 번째 요청 - ETag2 획득
    response2 = requests.get(url, headers=headers)
    etag2 = response2.headers["ETag"]
    count2 = len(response2.json())
    
    # 4. 검증
    assert etag1 != etag2, "ETag should change after data modification"
    assert count2 == count1 + 1, "Subject count should increase"
    
    # 5. 이전 ETag로 요청 시 200 OK (새 데이터)
    response3 = requests.get(
        url,
        headers={**headers, "If-None-Match": etag1}
    )
    assert response3.status_code == 200, "Old ETag should return new data"
    
    print("✅ Cache invalidation works correctly")
```

---

## 🔧 기존 테스트 개선 방법

### 현재 코드 구조
```python
# test_subject_cache.py
def main():
    token = login()
    results = []
    results.append(test_subjects_etag_cache(token))
    results.append(test_cache_invalidation_on_no_cache(token))
    # ...
```

### 개선된 코드 구조
```python
# test_subject_cache.py
def main():
    token = login()
    
    test_suite = [
        ("ETag 생성 및 304 응답", test_subjects_etag_cache),
        ("no-cache 헤더 처리", test_cache_invalidation_on_no_cache),
        ("데이터 변경 후 캐시 무효화", test_cache_invalidation_after_create),  # 추가
        ("잘못된 ETag 처리", test_invalid_etag_handling),  # 추가
        ("빈 목록 ETag", test_empty_list_etag),  # 추가
        ("성능 측정", test_cache_performance),  # 추가
    ]
    
    results = []
    for name, test_func in test_suite:
        print(f"\n🧪 {name}")
        try:
            result = test_func(token)
            results.append((name, result))
        except Exception as e:
            print(f"❌ 예외 발생: {e}")
            results.append((name, False))
    
    # 상세 결과 출력
    print("\n" + "="*60)
    print("📊 테스트 결과 상세")
    print("="*60)
    for name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{status} - {name}")
```

---

## 📌 결론

### 현재 테스트 평가: ⚠️ **기본적이지만 불충분**

**장점:**
- ✅ 기본적인 ETag 캐싱 동작 검증
- ✅ 304 Not Modified 응답 확인
- ✅ 간단하고 이해하기 쉬움

**단점:**
- ❌ 캐시 무효화 시나리오 부재 (가장 중요!)
- ❌ 엣지 케이스 처리 부족
- ❌ 성능 측정 부재
- ❌ 실제 데이터 변경 시나리오 부족

### 권장사항:
1. **Phase 1 (필수)**: 캐시 무효화 테스트 추가
2. **Phase 2 (권장)**: 잘못된 ETag, 빈 데이터, 성능 측정 추가
3. **Phase 3 (선택)**: 필터별 캐시, 동시 요청 테스트 추가

**최소한 Phase 1은 반드시 구현해야 합니다!** 🔴

