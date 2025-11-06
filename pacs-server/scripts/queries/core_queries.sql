-- Core EXPLAIN targets (copy/paste with scripts/explain.sh)
-- Usage example:
--   APP_DATABASE_URL=postgres://... ./scripts/explain.sh "<paste one of the queries>"

-- 1) Study UID → internal id 매핑
SELECT id FROM project_data_study WHERE study_uid = '1.2.3.4.5' AND project_id = 1;

-- 2) Series UID → internal id 매핑 (study_id 선행 필요)
SELECT id FROM project_data_series WHERE series_uid = '1.2.3.4.5.6' AND study_id = 123;

-- 3) Instance UID → internal id 매핑 (series_id 선행 필요)
SELECT id FROM project_data_instance WHERE instance_uid = '1.2.3.4.5.6.7' AND series_id = 456;

-- 4) 프로젝트 멤버십 확인
SELECT 1 FROM security_user_project WHERE user_id = 10 AND project_id = 1;

-- 5) 명시 권한 존재 여부 (Study 레벨)
SELECT 1
FROM project_data_access
WHERE user_id = 10
  AND project_id = 1
  AND resource_level = 'STUDY'
  AND study_id = 123
  AND status = 'APPROVED'
LIMIT 1;

-- 6) 규칙 병합(프로젝트 규칙)
SELECT ac.*
FROM security_access_condition ac
JOIN security_project_dicom_condition pc ON pc.access_condition_id = ac.id
WHERE pc.project_id = 1
ORDER BY pc.priority DESC, ac.id ASC;

-- 7) 규칙 병합(역할 규칙)
SELECT ac.*
FROM security_access_condition ac
JOIN security_role_dicom_condition rc ON rc.access_condition_id = ac.id
WHERE rc.role_id = 20
ORDER BY rc.priority DESC, ac.id ASC;

-- 8) Study 리스트(프로젝트 범위 + 대표 필터)
SELECT study_uid, modality, study_date, patient_id
FROM project_data_study
WHERE project_id = 1
  AND study_date BETWEEN '2024-01-01' AND '2024-12-31'
  AND modality = 'CT';

-- 9) Series 리스트(특정 Study 하위)
SELECT series_uid
FROM project_data_series
WHERE study_id = 123;

-- 10) Instance 리스트(특정 Series 하위)
SELECT instance_uid
FROM project_data_instance
WHERE series_id = 456;
