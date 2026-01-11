-- 디버깅용 SQL 쿼리
-- User ID: iaid-pacs-admin의 실제 ID 확인
SELECT id, username, email FROM security_user WHERE username = 'iaid-pacs-admin';

-- Project ID 2의 정보
SELECT * FROM security_project WHERE id = 2;

-- User가 Project 2의 멤버인지 확인
SELECT * FROM security_user_project WHERE project_id = 2;

-- Study UID 1.2.410.200017.0.1.2.7.2780199001.0 정보
SELECT * FROM project_data_study WHERE study_uid = '1.2.410.200017.0.1.2.7.2780199001.0';

-- Series UID 1.2.410.200017.0.1.3.7.2780199001.3 정보
SELECT * FROM project_data_series WHERE series_uid = '1.2.410.200017.0.1.3.7.2780199001.3';

-- Project 2에 할당된 Study 확인
SELECT pd.*, pds.study_uid 
FROM project_data pd
JOIN project_data_study pds ON pd.study_id = pds.id
WHERE pd.project_id = 2 AND pd.resource_level = 'STUDY';

-- project_data_access 테이블 확인 (User와 Project 2)
SELECT pda.*, pds.study_uid, u.username
FROM project_data_access pda
LEFT JOIN project_data_study pds ON pda.study_id = pds.id
LEFT JOIN security_user u ON pda.user_id = u.id
WHERE pda.project_id = 2;

-- 특정 Study에 대한 접근 권한 확인
SELECT pda.*, pds.study_uid, u.username
FROM project_data_access pda
INNER JOIN project_data_study pds ON pda.study_id = pds.id
INNER JOIN security_user u ON pda.user_id = u.id
WHERE pda.project_id = 2 
  AND pds.study_uid = '1.2.410.200017.0.1.2.7.2780199001.0'
  AND pda.status = 'APPROVED'
  AND (pda.expires_at IS NULL OR pda.expires_at > NOW());

