# Series API 페이지네이션 문제 분석

## 문제 상황

`/api/me/dicom/series?project_id=2&page=1&page_size=100` 호출 시:
- Page 1, 2, 3 모두 **동일한 11개 Series** 반환
- 페이지네이션이 작동하지 않음

## 원인 분석

### 현재 로직 흐름

1. **QIDO 호출**: `page`/`page_size` 파라미터를 QIDO에 전달하여 데이터 조회
2. **필터링**: `get_allowed_series_uids`로 프로젝트에 할당된 Series만 필터링
3. **응답 반환**: 필터링된 결과를 그대로 반환

### 문제점

- QIDO는 `page=1, page_size=100`으로 요청하면 100개를 반환
- 필터링 후 11개만 남음
- `page=2`로 요청해도 QIDO는 여전히 처음 100개를 반환하고, 필터링 후에도 같은 11개가 나옴
- **필터링 후 메모리에서 페이지네이션을 적용하지 않음**

## 해결 방법

### 옵션 1: 필터링 후 메모리 페이지네이션 (권장)

```rust
// 필터링 후
let filtered: Vec<serde_json::Value> = series_list
    .iter()
    .filter(|series| { /* ... */ })
    .cloned()
    .collect();

// 메모리에서 페이지네이션 적용
let page = query.page.unwrap_or(1);
let page_size = query.page_size.unwrap_or(50);
let offset = (page - 1) * page_size;
let paginated: Vec<serde_json::Value> = filtered
    .into_iter()
    .skip(offset)
    .take(page_size)
    .collect();

serde_json::json!(paginated)
```

### 옵션 2: QIDO에 더 큰 limit 전달 후 필터링 + 페이지네이션

QIDO에 충분히 큰 limit을 전달하고, 필터링 후 메모리에서 페이지네이션 적용

### 옵션 3: DB에서 먼저 허용된 Series UID 목록을 가져온 후 QIDO에 전달

하지만 QIDO는 Series UID 목록을 직접 필터링하는 파라미터를 지원하지 않을 수 있음

## 현재 상태

- **총 Series 개수**: 11개 (고유, 중복 없음)
- **Study 개수**: 2개
- **페이지네이션**: 작동하지 않음 (모든 페이지에서 동일한 11개 반환)

## 권장 수정 사항

`dicom_gateway_controller.rs`의 `get_series` 함수에서 필터링 후 메모리 페이지네이션을 적용해야 합니다.

