-- 사용자 1이 프로젝트 2의 멤버인지 확인
SELECT 
    up.user_id,
    up.project_id,
    up.role_id,
    r.name as role_name,
    up.created_at
FROM security_user_project up
LEFT JOIN security_role r ON up.role_id = r.id
WHERE up.user_id = 1 AND up.project_id = 2;

-- 프로젝트 2에 할당된 Series가 있는지 확인
SELECT 
    pd.project_id,
    COUNT(DISTINCT pds.id) as study_count,
    COUNT(DISTINCT pdser.id) as series_count,
    COUNT(DISTINCT pdser.series_uid) as series_uid_count
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL
GROUP BY pd.project_id;

-- 프로젝트 2의 allowed_series_uids (실제 쿼리)
SELECT DISTINCT pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
  AND pdser.series_uid IS NOT NULL
LIMIT 10;

-- 프로젝트 2의 전체 데이터 구조 확인
SELECT 
    pd.id as project_data_id,
    pd.project_id,
    pds.id as study_id,
    pds.study_uid,
    pdser.id as series_id,
    pdser.series_uid
FROM project_data pd
INNER JOIN project_data_study pds ON pd.study_id = pds.id
INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
WHERE pd.project_id = 2
LIMIT 20;

