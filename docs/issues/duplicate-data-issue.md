# Duplicate Data Issue in Project Data

**날짜**: 2025-12-18
**해결 날짜**: 2026-01-24
**상태**: ✅ 해결됨
**우선순위**: Medium

## 📋 문제 요약

`project_data` 테이블에 동일한 Study가 중복으로 등록되어 있는 문제

## 🔍 발견된 중복 데이터

### Project ID 2 통계
- **project_data 레코드 수**: 171개
- **실제 고유 Study 수**: 10개
- **중복 비율**: 약 17배 중복

### 중복 예시
```sql
-- Study UID: 1.2.410.2000010.82.2291.3279974230427007
-- "Chest CT (contrast) + 3D (Chest with other CT)"
-- 이 Study가 project_data에 여러 번 등록됨
```

## 📊 데이터 분석

### 전체 Project별 데이터 분포
```
 project_id | study_count 
------------+-------------
          2 |         171  ← 중복 많음
         22 |           2
        111 |           3
        113 |           2
        ... (총 33개 프로젝트)
```

### Project 2의 실제 Study 목록 (중복 제거)
```
1. 2025-09-11: Histopathology
2. 2023-05-16: Chest CT (contrast) + 3D
3. 2022-02-24: MR BRAIN W AND WO IV CONTRAST TUMOR
4. 2021-08-05: MRI Brain w/ + w/o Contrast
5. 2021-04-16: Pelvis
6. 2021-01-05: Thorax^ZJ_Thorax_Abd_CE_30S (Adult)
7. 2020-12-07: Thorax^ZJ_Thorax_Abd_CE_30S (Adult)
8. 2020-04-02: Abdomen^A08_Gb_PancreasDynamic_CT (Adult)
9. 2017-05-31: CT-3D (Abdomen-Pelvis +CE)
10. 2006-12-20: CT, Gastrography
```

## 🔎 원인 분석

### 테이블 구조
```sql
-- project_data 테이블
CREATE TABLE project_data (
    id             SERIAL PRIMARY KEY,
    project_id     INTEGER NOT NULL,
    resource_level resource_level_enum NOT NULL DEFAULT 'STUDY',
    study_id       INTEGER,
    series_id      INTEGER,
    instance_id    INTEGER,
    ...
    UNIQUE (project_id, study_id, series_id, instance_id)
);
```

### UNIQUE 제약 조건
- `(project_id, study_id, series_id, instance_id)` 조합에 대한 UNIQUE 제약이 있음
- 하지만 **실제로는 중복 데이터가 존재**

### 의심되는 원인
1. **Sync 로직 문제**: 동기화 시 중복 체크가 제대로 되지 않음
2. **UPSERT 로직 오류**: INSERT ... ON CONFLICT 처리가 잘못됨
3. **NULL 값 처리**: `series_id`, `instance_id`가 NULL일 때 UNIQUE 제약이 제대로 작동하지 않을 수 있음
4. **동시성 문제**: 여러 동기화 작업이 동시에 실행되어 중복 삽입

## 🧪 중복 확인 쿼리

### 중복 데이터 조회
```sql
-- Project 2의 중복된 Study 찾기
SELECT 
    study_id,
    COUNT(*) as duplicate_count
FROM project_data 
WHERE project_id = 2 
  AND resource_level = 'STUDY'
GROUP BY study_id
HAVING COUNT(*) > 1
ORDER BY duplicate_count DESC;
```

### 중복 레코드 상세 조회
```sql
-- 특정 Study의 중복 레코드 확인
SELECT 
    pd.id,
    pd.project_id,
    pd.study_id,
    pd.series_id,
    pd.instance_id,
    pd.created_at,
    pds.study_uid
FROM project_data pd
JOIN project_data_study pds ON pd.study_id = pds.id
WHERE pd.project_id = 2
  AND pds.study_uid = '1.2.410.2000010.82.2291.3279974230427007'
ORDER BY pd.created_at;
```

## 📝 관련 코드

### Sync Worker
- **파일**: `pacs-server/src/infrastructure/services/sync_worker.rs`
- **메서드**: `sync_studies()`, `upsert_study()`

### 예상되는 문제 코드
```rust
// sync_worker.rs의 upsert 로직
async fn upsert_study(&self, study: &Study) -> Result<i32> {
    // INSERT ... ON CONFLICT 처리
    // 여기서 중복 체크가 제대로 안 될 수 있음
}
```

## 🛠️ 해결 방안

### 1. 즉시 조치 (데이터 정리)
```sql
-- 중복 데이터 삭제 (최신 것만 남기고 삭제)
WITH duplicates AS (
    SELECT 
        id,
        ROW_NUMBER() OVER (
            PARTITION BY project_id, study_id 
            ORDER BY created_at DESC
        ) as rn
    FROM project_data
    WHERE project_id = 2 
      AND resource_level = 'STUDY'
)
DELETE FROM project_data
WHERE id IN (
    SELECT id FROM duplicates WHERE rn > 1
);
```

### 2. 근본 원인 수정

#### A. UNIQUE 제약 조건 강화
```sql
-- NULL 값을 고려한 UNIQUE 제약 추가
CREATE UNIQUE INDEX idx_project_data_unique_study
ON project_data (project_id, study_id)
WHERE resource_level = 'STUDY' 
  AND series_id IS NULL 
  AND instance_id IS NULL;
```

#### B. Sync 로직 수정
```rust
// sync_worker.rs
async fn upsert_study(&self, study: &Study, project_id: i32) -> Result<i32> {
    // 1. 먼저 존재 여부 확인
    let existing = sqlx::query!(
        "SELECT id FROM project_data 
         WHERE project_id = $1 AND study_id = $2 
           AND resource_level = 'STUDY'",
        project_id, study_id
    )
    .fetch_optional(&self.pool)
    .await?;
    
    if existing.is_some() {
        // 이미 존재하면 스킵
        return Ok(existing.unwrap().id);
    }
    
    // 2. 존재하지 않으면 INSERT
    // ...
}
```

#### C. Transaction 사용
```rust
// 동기화 작업을 트랜잭션으로 묶기
async fn sync_studies(&self) -> Result<usize> {
    let mut tx = self.pool.begin().await?;
    
    // 동기화 작업
    for study in studies {
        self.upsert_study_tx(&mut tx, &study).await?;
    }
    
    tx.commit().await?;
    Ok(studies.len())
}
```

### 3. 모니터링 추가
```rust
// 중복 삽입 시도 시 로그 출력
if existing.is_some() {
    eprintln!(
        "⚠️ [Sync] Duplicate study detected: {} for project {}",
        study_uid, project_id
    );
}
```

## 🎯 다음 단계

1. **중복 데이터 분석**: 정확히 어떤 패턴으로 중복되는지 확인
2. **Sync 로직 검토**: `sync_worker.rs`의 UPSERT 로직 확인
3. **데이터 정리**: 중복 데이터 삭제 스크립트 실행
4. **제약 조건 강화**: UNIQUE 인덱스 추가
5. **테스트**: 수정 후 동기화 재실행하여 중복 발생 여부 확인

## 📌 참고 사항

- 현재 데이터는 사용 가능하지만 중복으로 인한 성능 저하 가능성 있음
- 중복 데이터 삭제 전 백업 필수
- 다른 프로젝트에도 동일한 문제가 있을 수 있음

---

## ✅ 해결 완료 (2026-01-24)

### 🔍 문제 재확인
2026-01-24 기준으로 데이터베이스를 재확인한 결과:
- **현재 중복 데이터 없음** (9개 레코드, 9개 고유 조합)
- UNIQUE 제약조건이 정상 작동 중
- 과거 중복 데이터는 이미 정리된 상태

### 🛠️ 구현된 해결책

#### 1. Repository 레벨 수정
**파일**: `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs`

**변경 내용**:
```rust
// BEFORE: DO NOTHING은 충돌 시 행을 반환하지 않아 fetch_one() 실패
ON CONFLICT (project_id, study_id, series_id, instance_id) DO NOTHING

// AFTER: DO UPDATE는 충돌 시에도 행을 반환하여 idempotency 보장
ON CONFLICT (project_id, study_id, series_id, instance_id)
DO UPDATE SET updated_at = CURRENT_TIMESTAMP
RETURNING id, project_id, created_at
```

**효과**:
- ✅ **Idempotency 보장**: 동일한 요청을 여러 번 보내도 안전
- ✅ **항상 행 반환**: 새로 생성되거나 기존 레코드 모두 반환
- ✅ **중복 방지**: UNIQUE 제약조건과 함께 작동

#### 2. API 레벨 중복 체크
**API**: `POST /api/projects/{project_id}/studies/assign`

**동작**:
- 첫 번째 할당: `200 OK` 또는 `201 Created` + `study_id` 반환
- 중복 할당 시도: `409 Conflict` + "Study already assigned to this project" 메시지

**장점**:
- ✅ 명확한 에러 메시지
- ✅ HTTP 표준 상태 코드 사용
- ✅ 클라이언트가 중복 여부를 쉽게 판단 가능

#### 3. E2E 테스트 추가
**파일**: `pacs-server/e2e/test_project_data_duplicate_prevention.py`

**테스트 시나리오**:
1. **Test 1: Duplicate Study Assignment Prevention**
   - 동일한 Study를 같은 프로젝트에 두 번 할당
   - 두 번째 시도 시 409 Conflict 반환 확인
   - 데이터베이스에 하나의 레코드만 존재 확인

2. **Test 2: Concurrent Study Assignment**
   - 5개의 동시 요청으로 동일한 Study 할당
   - 일부는 성공(200), 일부는 충돌(409)
   - 모든 성공 요청이 동일한 `study_id` 반환 확인
   - 데이터베이스에 하나의 레코드만 존재 확인

3. **Test 3: Same Study in Different Projects**
   - 동일한 Study를 서로 다른 프로젝트에 할당
   - 각 프로젝트에 별도 레코드 생성 확인
   - 프로젝트 간 독립성 확인

**테스트 결과**: 🎉 **ALL TESTS PASSED**

### 📊 최종 검증

#### 데이터베이스 상태
```sql
-- 중복 확인 쿼리 결과
SELECT COUNT(*) as total_records,
       COUNT(DISTINCT (project_id, study_id)) as unique_combinations
FROM project_data
WHERE resource_level = 'STUDY';

-- 결과: 9 total_records, 9 unique_combinations (중복 없음)
```

#### UNIQUE 제약조건
```sql
\d project_data

-- Indexes:
--   "project_data_project_id_study_id_series_id_instance_id_key"
--   UNIQUE CONSTRAINT, btree (project_id, study_id, series_id, instance_id)
```

### 🎯 결론

1. ✅ **중복 데이터 문제 해결**: 현재 데이터베이스에 중복 없음
2. ✅ **Repository 로직 개선**: Idempotency 보장
3. ✅ **API 동작 개선**: 409 Conflict로 명확한 에러 처리
4. ✅ **테스트 커버리지**: 3가지 시나리오 E2E 테스트 추가
5. ✅ **동시성 안전**: 동시 요청에도 중복 생성 방지

### 📝 관련 파일
- `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs` (수정)
- `pacs-server/e2e/test_project_data_duplicate_prevention.py` (신규)
- `docs/issues/duplicate-data-issue.md` (업데이트)

