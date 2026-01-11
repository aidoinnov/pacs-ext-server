# `/api/me/dicom/series?project_id=2` 로직 분석

## 현재 흐름

1. **Dcm4chee QIDO 호출** (`qido_series_all_with_bearer`)
   - 모든 Series를 Dcm4chee에서 가져옴
   - 실패 시 502 Bad Gateway 반환

2. **허용된 Series UID 조회** (`get_allowed_series_uids`)
   ```sql
   SELECT DISTINCT pdser.series_uid
   FROM project_data pd
   INNER JOIN project_data_study pds ON pd.study_id = pds.id
   INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
   WHERE pd.project_id = 2
     AND pdser.series_uid IS NOT NULL
   ```

3. **필터링**
   - QIDO 결과에서 허용된 Series만 필터링

## 문제점

### 1. Dcm4chee QIDO 실패
- 현재 502 에러 발생
- 하지만 실제로는 200 OK를 반환하고 있음
- 이는 QIDO가 성공했지만 빈 결과를 반환했거나, 필터링 후 결과가 없는 것

### 2. `get_allowed_series_uids` 쿼리
- `project_data`에서 `project_id=2`인 행을 찾음
- `project_data_study`와 조인 (`pd.study_id = pds.id`)
- `project_data_series`와 조인 (`pds.id = pdser.study_id`)

**조인 조건:**
- `pd.study_id = pds.id` ✅ (할당 시 저장됨)
- `pds.id = pdser.study_id` ✅ (Series는 Study에 속함)

### 3. 할당 로직 확인
`assign_series_to_project`는:
```rust
INSERT INTO project_data (project_id, resource_level, study_id, series_id)
VALUES ($1, 'SERIES', $2, $3)
```
- `study_id`와 `series_id`를 모두 저장 ✅
- 따라서 쿼리는 작동해야 함

## 가능한 원인

1. **데이터가 실제로 저장되지 않았을 수 있음**
   - 할당 API 호출은 성공했지만 DB에 저장되지 않았을 수 있음
   - 트랜잭션 롤백 가능성

2. **쿼리 조건 문제**
   - `pdser.series_uid IS NOT NULL` 조건이 문제일 수 있음
   - `series_uid`가 NULL인 경우 제외됨

3. **조인 실패**
   - `pds.id = pdser.study_id` 조인이 실패할 수 있음
   - Series의 `study_id`가 Study의 `id`와 일치하지 않을 수 있음

## 확인 방법

1. **DB 직접 확인**
   ```sql
   -- project_data 확인
   SELECT * FROM project_data WHERE project_id = 2 LIMIT 10;
   
   -- 조인 테스트
   SELECT pdser.series_uid
   FROM project_data pd
   INNER JOIN project_data_study pds ON pd.study_id = pds.id
   INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
   WHERE pd.project_id = 2
     AND pdser.series_uid IS NOT NULL;
   ```

2. **서버 로그 확인**
   - `get_allowed_series_uids` 결과 로그 확인
   - `tracing::debug!("Gateway /series: Found {} allowed series UIDs for project {}", allowed_series_uids.len(), project_id);`

3. **단계별 테스트**
   - QIDO 호출만 테스트
   - `get_allowed_series_uids` 쿼리만 테스트
   - 필터링 로직만 테스트

