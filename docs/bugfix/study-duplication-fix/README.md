# Study 중복 반환 문제 수정

## 📋 문제 요약

### 발견된 문제
`/api/dicom/studies?check_assignment_for_project=2` 엔드포인트가 중복된 Study를 반환하는 문제 발견

**증상:**
- PACS 아카이브: 11개 Study
- API 반환: **38개** (27개 중복!)
- 같은 Study UID가 최대 6번까지 중복 반환

### 원인 분석

QIDO-RS 응답에 중복된 Study가 포함되어 있을 때:
- `project_id` 파라미터 사용 시: 중복 제거 로직이 작동 ✅
- `check_assignment_for_project` 파라미터 사용 시: 중복 제거 로직 없음 ❌

**코드 위치:** `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`

---

## 🔧 수정 내용

### 수정 파일
- `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs` (라인 570-608)

### 변경 사항

**수정 전:**
```rust
let final_response = if let Some(check_pid) = check_assignment_project_id {
    if let Some(array) = filtered.as_array() {
        let mut enriched_items = Vec::new();
        for item in array.iter() {
            if let Some(study_uid) = extract_study_uid(item) {
                // 중복 제거 없음!
                let is_assigned = check_study_assignment(...).await;
                enriched_items.push(enriched_item);
            }
        }
```

**수정 후:**
```rust
let final_response = if let Some(check_pid) = check_assignment_project_id {
    if let Some(array) = filtered.as_array() {
        let mut enriched_items = Vec::new();
        let mut study_uids_seen = std::collections::HashSet::new(); // ← 추가
        for item in array.iter() {
            if let Some(study_uid) = extract_study_uid(item) {
                // 중복 제거 로직 추가
                if study_uids_seen.contains(&study_uid) {
                    continue;
                }
                study_uids_seen.insert(study_uid.clone());
                
                let is_assigned = check_study_assignment(...).await;
                enriched_items.push(enriched_item);
            }
        }
```

---

## ✅ 테스트 결과

### 수정 전
```
📊 반환된 Study 개수: 38개
📊 고유 Study UID: 11개
📊 중복 발생: 27개

Study UID별 중복 현황:
  - 1.2.410.200022.500.202205101053010.12252192374: 6번 중복
  - 1.2.410.200022.500.12252244130: 5번 중복
  - 1.2.410.200022.500.12252244131: 4번 중복
  - ...
```

### 수정 후
```
📊 반환된 Study 개수: 11개
📊 고유 Study UID: 11개
📊 중복 발생: 0개

📊 is_assigned 통계:
  - assigned=true:  8개 (프로젝트 2에 할당됨)
  - assigned=false: 3개 (할당되지 않음)
  - 합계: 11개
```

---

## 📝 테스트 스크립트

테스트 파일: `pacs-server/e2e/test_study_count_issue.py`

**실행 방법:**
```bash
cd pacs-server/e2e
python3 test_study_count_issue.py
```

**테스트 내용:**
1. `check_assignment_for_project=2` 파라미터로 Study 조회
2. 반환된 Study 개수 확인
3. Study UID별 중복 확인
4. `is_assigned` 필드 통계 확인
5. `project_id=2` 파라미터와 비교

---

## 🎯 영향 범위

### 수정된 기능
- `GET /api/dicom/studies?check_assignment_for_project={project_id}`

### 영향받는 컴포넌트
- DICOM Gateway Controller
- Study 조회 API

### 하위 호환성
- ✅ 기존 API 동작 유지
- ✅ 응답 형식 변경 없음
- ✅ 중복만 제거되어 더 정확한 결과 반환

---

## 📚 관련 문서

- [DICOM Gateway API 문서](../../api/)
- [E2E 테스트 커버리지](../../E2E_TEST_COVERAGE.md)

---

## 🔍 추가 정보

### 왜 중복이 발생했나?

QIDO-RS 서버(Dcm4chee)가 Study를 조회할 때, 내부적으로 Series와 JOIN하여 결과를 반환하는 경우가 있습니다. 
한 Study에 여러 Series가 있으면 같은 Study가 여러 번 반환될 수 있습니다.

### 왜 project_id에서는 문제가 없었나?

`project_id` 파라미터 사용 시에는 이미 중복 제거 로직이 구현되어 있었습니다 (라인 527-533):
```rust
let mut study_uids_seen = std::collections::HashSet::new();
if study_uids_seen.contains(&study_uid) {
    continue;
}
study_uids_seen.insert(study_uid.clone());
```

`check_assignment_for_project` 파라미터에도 동일한 로직을 추가하여 문제를 해결했습니다.

---

**수정 일자:** 2026-01-29  
**수정자:** Augment Agent  
**관련 이슈:** Study 중복 반환 문제

