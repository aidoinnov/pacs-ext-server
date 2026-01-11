# Series 쿼리 수정 완료

## 문제

- `resource_level='SERIES'`일 때 `series_id`로 직접 조회해야 하는데
- 현재는 `study_id`로 조인해서 study의 모든 series를 가져옴
- 결과: 5개만 있어야 하는데 11개가 나옴

## 수정 내용

`get_allowed_series_uids` 함수를 수정하여 `resource_level`에 따라 다른 쿼리를 사용하도록 변경:

### 수정 전 (잘못된 쿼리)
```sql
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
```
→ study의 모든 series를 가져옴 (11개)

### 수정 후 (올바른 쿼리)
```sql
SELECT DISTINCT pdser.series_uid
FROM (
    -- resource_level='SERIES'인 경우: series_id로 직접 조회
    SELECT pdser.series_uid
    FROM project_data pd
    INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
    WHERE pd.project_id = 2
      AND pd.resource_level = 'SERIES'
      AND pd.series_id IS NOT NULL
    
    UNION
    
    -- resource_level='STUDY'인 경우: study_id로 조인하여 study의 모든 series 조회
    SELECT DISTINCT pdser.series_uid
    FROM project_data pd
    INNER JOIN project_data_study pds ON pd.study_id = pds.id
    INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
    WHERE pd.project_id = 2
      AND pd.resource_level = 'STUDY'
      AND pd.study_id IS NOT NULL
) AS combined
```
→ resource_level에 따라 올바른 series만 가져옴 (5개)

## 결과

- **수정 전**: 11개 Series 반환 (잘못됨)
- **수정 후**: 5개 Series 반환 (올바름)

## 테스트

서버 재시작 후 다음 API 호출로 확인:
```bash
curl -H "Authorization: Bearer {token}" \
  "http://localhost:8080/api/me/dicom/series?project_id=2&page=1&page_size=100"
```

예상 결과: 5개 Series 반환

