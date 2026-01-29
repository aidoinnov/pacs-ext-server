# Study 중복 문제 조사 로그

## 🔍 문제 발견

### 사용자 보고
```
이거 한번확인해볼래? 이거 지금 스터디가 11개인데..엄청많이 나오거든...?
http://localhost:8080/api/dicom/studies?check_assignment_for_project=2&page=1&page_size=50
```

### 초기 조사

**1단계: API 응답 확인**
```bash
curl "http://localhost:8080/api/dicom/studies?check_assignment_for_project=2&page_size=100"
```

결과:
- 반환된 항목: 38개
- 예상: 11개 (PACS 아카이브 전체 Study 개수)

---

## 📊 데이터 분석

### PACS 아카이브 확인
```sql
-- Dcm4chee DB (port 5432)
SELECT COUNT(*) FROM study;
-- 결과: 11개
```

### RBAC DB 확인
```sql
-- RBAC DB (port 5456)
SELECT COUNT(*) FROM project_data_study;
-- 결과: 11개

SELECT COUNT(DISTINCT pd.study_id)
FROM project_data pd
WHERE pd.project_id = 2 AND pd.resource_level = 'STUDY';
-- 결과: 8개 (프로젝트 2에 할당됨)
```

### Series 레벨 할당 확인
```sql
SELECT COUNT(*) 
FROM project_data pd
WHERE pd.project_id = 2 AND pd.resource_level = 'SERIES';
-- 결과: 0개
```

**결론:** Series 레벨 할당은 없음. 중복 원인이 아님.

---

## 🔬 중복 패턴 분석

### Study UID별 중복 현황
```
1.2.410.200022.500.202205101053010.12252192374    : 6번 중복
1.2.410.200022.500.12252244130                    : 5번 중복
1.2.410.200022.500.12252244131                    : 4번 중복
1.2.410.200022.500.202205101527084.111024842193   : 4번 중복
1.2.410.200022.500.202206131353042.111024842193   : 4번 중복
1.2.410.200022.500.12252244129                    : 4번 중복
1.2.410.200022.500.202205101052995.12252192373    : 4번 중복
1.2.410.200022.500.202205101053010.12252192375    : 4번 중복
1.2.826.0.1.3680043.8.498.570382422794263150883757: 1번 (중복 없음)
1.2.826.0.1.3680043.8.498.741912587998356403150816: 1번 (중복 없음)
1.2.410.200022.500.12352526057                    : 1번 (중복 없음)
```

**총계:**
- 고유 Study UID: 11개
- 총 반환 개수: 38개
- 중복 발생: 27개

---

## 🔍 코드 분석

### 비교: project_id vs check_assignment_for_project

**`project_id` 파라미터 (정상 작동):**
```rust
// 라인 523-561
if let Some(pid) = project_id_opt {
    if let Some(array) = qido_response.as_array() {
        let mut allowed_items = Vec::new();
        let mut study_uids_seen = std::collections::HashSet::new(); // ✅ 중복 제거
        for item in array.iter() {
            if let Some(study_uid) = extract_study_uid(item) {
                if study_uids_seen.contains(&study_uid) {
                    continue; // ✅ 중복 건너뛰기
                }
                study_uids_seen.insert(study_uid.clone());
                // ...
            }
        }
    }
}
```

**`check_assignment_for_project` 파라미터 (문제 발생):**
```rust
// 라인 570-600 (수정 전)
if let Some(check_pid) = check_assignment_project_id {
    if let Some(array) = filtered.as_array() {
        let mut enriched_items = Vec::new();
        // ❌ 중복 제거 로직 없음!
        for item in array.iter() {
            if let Some(study_uid) = extract_study_uid(item) {
                let is_assigned = check_study_assignment(...).await;
                enriched_items.push(enriched_item); // ❌ 중복도 그대로 추가
            }
        }
    }
}
```

---

## 💡 해결 방안

### 선택지 1: 클라이언트에서 중복 제거
```javascript
const response = await fetch('/api/dicom/studies?check_assignment_for_project=2');
const studies = await response.json();
const uniqueStudies = Array.from(
    new Map(studies.map(s => [s['0020000D'].Value[0], s])).values()
);
```

**단점:**
- 네트워크 대역폭 낭비
- 클라이언트 부담 증가
- 근본적인 해결이 아님

### 선택지 2: 서버에서 중복 제거 ✅ (채택)
```rust
let mut study_uids_seen = std::collections::HashSet::new();
for item in array.iter() {
    if let Some(study_uid) = extract_study_uid(item) {
        if study_uids_seen.contains(&study_uid) {
            continue;
        }
        study_uids_seen.insert(study_uid.clone());
        // ...
    }
}
```

**장점:**
- 근본적인 해결
- 네트워크 효율성 향상
- 클라이언트 부담 감소
- `project_id` 파라미터와 일관성 유지

---

## 🧪 테스트 시나리오

### 테스트 1: 중복 제거 확인
```python
response = requests.get(
    f"{BASE_URL}/api/dicom/studies",
    params={"check_assignment_for_project": 2, "page_size": 100},
    headers=headers
)
studies = response.json()
study_uids = [s["0020000D"]["Value"][0] for s in studies]

assert len(studies) == len(set(study_uids)), "중복 발견!"
```

### 테스트 2: is_assigned 필드 확인
```python
assigned_count = sum(1 for s in studies if s.get("is_assigned") == True)
assert assigned_count == 8, "할당된 Study 개수 불일치"
```

### 테스트 3: 전체 개수 확인
```python
assert len(studies) == 11, "전체 Study 개수 불일치"
```

---

## 📈 성능 영향

### 수정 전
- 반환 데이터 크기: ~38개 Study (약 3.5배 증가)
- 네트워크 대역폭: 불필요하게 증가
- 클라이언트 처리 시간: 증가

### 수정 후
- 반환 데이터 크기: 11개 Study (정확)
- 네트워크 대역폭: 최적화
- 클라이언트 처리 시간: 감소
- 서버 처리 시간: HashSet 연산 추가 (무시할 수준)

---

## ✅ 검증 완료

### 수정 후 테스트 결과
```
📊 반환된 Study 개수: 11개
📊 고유 Study UID: 11개
📊 중복 발생: 0개

📊 is_assigned 통계:
  - assigned=true:  8개
  - assigned=false: 3개
  - 합계: 11개
```

**결론:** ✅ 문제 완전히 해결됨

---

**조사 일자:** 2026-01-29  
**조사자:** Augment Agent

