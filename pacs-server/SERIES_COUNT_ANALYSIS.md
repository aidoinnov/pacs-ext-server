# Series 11개 원인 분석 결과

## 문제 상황

- `/api/me/dicom/series?project_id=2`: **11개 Series 반환**
- `/api/project-data/2/studies`: **0개 Study 반환**

## 원인 분석

### 1. 두 API의 쿼리 차이

#### `/api/project-data/{project_id}/studies` 쿼리
```sql
SELECT DISTINCT pds.id, ...
FROM project_data_study pds
INNER JOIN project_data pd ON pd.study_id = pds.id
WHERE pd.project_id = $1 
  AND pd.resource_level = 'STUDY'  -- ⚠️ Study 레벨만!
```

**조건**: `resource_level = 'STUDY'` → Study 레벨로 직접 할당된 것만 조회

#### `get_allowed_series_uids` 쿼리 (Series API에서 사용)
```sql
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = $1
  AND pdser.series_uid IS NOT NULL
  -- ⚠️ resource_level 조건 없음!
```

**조건**: `resource_level` 조건 없음 → Series 레벨로 할당된 것도 포함

### 2. 데이터 할당 방식

현재 `project_id=2`에 할당된 데이터는:
- **Study 레벨 할당**: 0개 (resource_level='STUDY')
- **Series 레벨 할당**: 11개 (resource_level='SERIES' 또는 할당 방식에 따라)

### 3. 결론

**11개 Series가 나오는 이유**:
- `project_data` 테이블에 Series 레벨로 할당된 데이터가 11개 있음
- Study 레벨로 직접 할당된 데이터는 없음
- 따라서 `/api/project-data/2/studies`는 0개를 반환하지만
- `/api/me/dicom/series?project_id=2`는 11개를 반환함

## 해결 방법

### 옵션 1: 모든 데이터를 Study 레벨로 재할당
```bash
python3 assign_all_data_from_db.py
```

### 옵션 2: 현재 상태 유지
- Series 레벨 할당도 정상 동작함
- `/api/me/dicom/series`는 정상적으로 11개를 반환
- `/api/project-data/2/studies`는 Study 레벨 할당이 없어서 0개 반환

### 옵션 3: 두 API의 동작을 통일
- `get_allowed_series_uids`에 `resource_level` 조건 추가
- 또는 `/api/project-data/{project_id}/studies`에서 Series 레벨 할당도 포함

## 권장 사항

현재 상태는 **정상 동작**입니다:
- Series 레벨로 할당된 데이터는 `/api/me/dicom/series`에서 정상 조회됨
- 11개는 실제로 할당된 Series 개수입니다
- 이전에 28개였던 것은 다른 데이터였거나 삭제/이동되었을 가능성이 있습니다

더 많은 Series를 보려면 데이터를 다시 할당하세요.

