-- get_allowed_series_uids 쿼리 테스트 및 문제 진단

-- 1. project_data 기본 확인
SELECT COUNT(*) as project_data_count 
FROM project_data 
WHERE project_id = 2;

-- 2. get_allowed_series_uids 쿼리 직접 실행
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL;

-- 3. 조인 문제 확인 (LEFT JOIN으로 상세 확인)
SELECT 
    pd.id as pd_id,
    pd.project_id,
    pd.resource_level,
    pd.study_id as pd_study_id,
    pd.series_id as pd_series_id,
    pds.id as pds_id,
    pds.study_uid,
    pdser.id as pdser_id,
    pdser.series_uid,
    pdser.study_id as pdser_study_id,
    CASE 
        WHEN pds.id IS NULL THEN '❌ project_data_study 조인 실패'
        WHEN pdser.id IS NULL THEN '❌ project_data_series 조인 실패'
        WHEN pds.id != pdser.study_id THEN '❌ pds.id != pdser.study_id'
        ELSE '✅ 정상'
    END as status
FROM project_data pd
LEFT JOIN project_data_study pds ON pd.study_id = pds.id
LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
WHERE pd.project_id = 2
LIMIT 10;

-- 4. 조인 조건 확인 (pds.id = pdser.study_id)
SELECT 
    pd.id as pd_id,
    pd.study_id as pd_study_id,
    pds.id as pds_id,
    pdser.id as pdser_id,
    pdser.study_id as pdser_study_id,
    CASE 
        WHEN pds.id = pdser.study_id THEN '✅ 일치'
        ELSE '❌ 불일치'
    END as match_status
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
LEFT JOIN project_data_series pdser ON pd.series_id = pdser.id
WHERE pd.project_id = 2
LIMIT 10;

-- 5. 원인 분석 쿼리
-- 5-1. study_id가 NULL인 행 확인
SELECT COUNT(*) as null_study_id_count
FROM project_data 
WHERE project_id = 2 AND study_id IS NULL;

-- 5-2. project_data_study 조인 실패 확인
SELECT COUNT(*) as study_join_fail_count
FROM project_data pd
LEFT JOIN project_data_study pds ON pd.study_id = pds.id
WHERE pd.project_id = 2 AND pds.id IS NULL;

-- 5-3. project_data_series 조인 실패 확인
SELECT COUNT(*) as series_join_fail_count
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
LEFT JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2 AND pdser.id IS NULL;

-- 5-4. series_uid가 NULL인 Series 확인
SELECT COUNT(*) as null_series_uid_count
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2 AND pdser.series_uid IS NULL;
