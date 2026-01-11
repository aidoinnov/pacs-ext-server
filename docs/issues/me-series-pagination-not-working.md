# /api/me/dicom/series Pagination Not Working

**날짜**: 2026-01-02  
**상태**: 🔴 확인됨  
**우선순위**: High

## 📋 문제 요약

`GET /api/me/dicom/series` API의 페이지네이션이 작동하지 않음. `page` 파라미터를 변경해도 항상 동일한 데이터가 반환됨.

## 🔍 증상

### 테스트 결과
```python
# Test 1: page=1, page_size=5
GET /api/me/dicom/series?project_id=2&page=1&page_size=5
→ 5개 Series 반환 ✅

# Test 2: page=2, page_size=5  
GET /api/me/dicom/series?project_id=2&page=2&page_size=5
→ 5개 Series 반환 ❌ (0개여야 함)

# Test 3: 전체 조회
GET /api/me/dicom/series?project_id=2&page=1&page_size=100
→ 5개 Series 반환 (전체 데이터)
```

### 문제점
1. **Page 1과 Page 2가 완전히 동일한 데이터 반환**
2. 전체 데이터가 5개인데, Page 2에서도 5개 반환
3. 모든 Series UID가 중복됨

## 🔎 원인 분석

### 코드 위치
**파일**: `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`  
**함수**: `get_all_user_series` (Line 1792-2214)

### 페이지네이션 로직
```rust
// Line 1883-1893: 페이지네이션 파라미터 추출
let page_size = query.extra
    .get("page_size")
    .and_then(|v| v.as_i64())
    .unwrap_or(50)
    .clamp(1, 200) as i64;
let page = query.extra
    .get("page")
    .and_then(|v| v.as_i64())
    .unwrap_or(1)
    .max(1);
let offset = (page - 1) * page_size;

// Line 2180-2196: 페이지네이션 적용
let start = offset as usize;
let end = std::cmp::min(start + page_size as usize, total_count);
let paginated_series = if let Some(array) = final_series.as_array() {
    if start < total_count {
        serde_json::Value::Array(array[start..end].to_vec())
    } else {
        serde_json::Value::Array(vec![])
    }
} else {
    serde_json::Value::Array(vec![])
};
```

### 의심되는 원인

#### 1. **QIDO 호출 시 limit 최적화 문제**
Line 1905-1909에서 QIDO 호출 시 `limit`를 최적화하는 로직이 있음:
```rust
let qido_limit = if projects_count == 1 {
    (page_size * 2).min(500) // 최대 500개로 제한
} else {
    (page_size * projects_count as i64).min(500)
};
```

이 로직은 **필터링 여유분**을 고려하여 더 많은 데이터를 가져오지만, 
**offset을 고려하지 않음**. 따라서 항상 처음부터 데이터를 가져옴.

#### 2. **QIDO에 offset 전달 안 함**
QIDO 호출 시 `offset` 파라미터를 전달하지 않음. 
Line 1919-1952의 QIDO 호출 코드를 보면 `limit`만 설정하고 `offset`은 설정하지 않음.

#### 3. **메모리 페이지네이션의 한계**
현재 구조:
1. QIDO에서 모든 데이터 가져오기 (또는 limit만큼)
2. RBAC 필터링
3. 메모리에서 페이지네이션 적용

문제: QIDO에서 항상 처음부터 데이터를 가져오므로, Page 2를 요청해도 
QIDO는 여전히 처음 데이터를 반환하고, 메모리 페이지네이션도 제대로 작동하지 않음.

## 🛠️ 해결 방안

### Option 1: QIDO에 offset 전달 (권장)

```rust
// Line 1928-1938 수정
let mut qido_params = if let Ok(conditions) = access_condition_repo_clone.list_by_project(project_id).await {
    let rule_params = build_qido_params_from_conditions(&conditions);
    merge_qido_params(rule_params, user_params_clone)
} else {
    user_params_clone
};

// offset 파라미터 추가
qido_params.push(("offset".to_string(), offset_clone.to_string()));

// limit 파라미터 설정
let limit_index = qido_params.iter().position(|(k, _)| k == "limit" || k == "Limit");
if let Some(idx) = limit_index {
    qido_params[idx] = ("limit".to_string(), page_size_clone.to_string());
} else {
    qido_params.push(("limit".to_string(), page_size_clone.to_string()));
}
```

### Option 2: 메모리 페이지네이션 수정

현재 메모리 페이지네이션 로직을 확인하고 디버그 로그 추가:

```rust
tracing::info!(
    "🔍 Pagination: page={}, page_size={}, offset={}, start={}, end={}, total={}",
    page, page_size, offset, start, end, total_count
);
```

## 🧪 테스트 스크립트

```python
#!/usr/bin/env python3
import requests

BASE_URL = "http://localhost:8080"

# 로그인
resp = requests.post(f"{BASE_URL}/api/auth/login", json={
    "username": "iaid-pacs-admin",
    "password": "Qlalfqjsgh1!"
})
token = resp.json().get('token')
headers = {'Authorization': f'Bearer {token}'}

# Page 1
resp1 = requests.get(f'{BASE_URL}/api/me/dicom/series?project_id=2&page=1&page_size=5', headers=headers)
data1 = resp1.json()
print(f"Page 1: {len(data1)} series")

# Page 2
resp2 = requests.get(f'{BASE_URL}/api/me/dicom/series?project_id=2&page=2&page_size=5', headers=headers)
data2 = resp2.json()
print(f"Page 2: {len(data2)} series")

# 중복 체크
uid1 = data1[0].get('0020000E', {}).get('Value', [''])[0] if data1 else ''
uid2 = data2[0].get('0020000E', {}).get('Value', [''])[0] if data2 else ''
if uid1 == uid2:
    print("❌ FAIL: Page 1 and Page 2 return same data!")
else:
    print("✅ PASS: Pagination working correctly")
```

## 📝 관련 파일

- `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs` (Line 1792-2214)
- `test_pagination.py` (테스트 스크립트)

## 🎯 다음 단계

1. ✅ 문제 확인 완료
2. ⏳ QIDO 호출 시 offset 파라미터 추가
3. ⏳ 빌드 및 테스트
4. ⏳ 검증

## 📌 참고 사항

- 다른 엔드포인트 (`/api/dicom/series`)도 동일한 문제가 있을 수 있음
- RBAC 필터링으로 인해 실제 반환되는 데이터 수가 예상과 다를 수 있음

