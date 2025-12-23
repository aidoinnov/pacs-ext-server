# Duplicate Data Issue in Project Data

**날짜**: 2025-12-18  
**상태**: 🟡 확인됨  
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

