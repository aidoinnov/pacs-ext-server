\timing on
\set ON_ERROR_STOP on

-- capture sample keys
SELECT project_id, study_uid FROM project_data_study WHERE study_uid IS NOT NULL ORDER BY id DESC LIMIT 1; \gset
SELECT id AS study_id FROM project_data_study WHERE project_id = :project_id ORDER BY id DESC LIMIT 1; \gset
SELECT series_uid, study_id FROM project_data_series WHERE series_uid IS NOT NULL ORDER BY id DESC LIMIT 1; \gset
SELECT instance_uid, series_id FROM project_data_instance WHERE instance_uid IS NOT NULL ORDER BY id DESC LIMIT 1; \gset
SELECT patient_id FROM project_data_study WHERE patient_id IS NOT NULL ORDER BY id DESC LIMIT 1; \gset
SELECT study_date FROM project_data_study WHERE study_date IS NOT NULL ORDER BY id DESC LIMIT 1; \gset
SELECT modality FROM project_data_series WHERE modality IS NOT NULL ORDER BY id DESC LIMIT 1; \gset
SELECT user_id, project_id FROM security_user_project ORDER BY 1 DESC LIMIT 1; \gset
SELECT COALESCE((SELECT id FROM project_data_access ORDER BY id DESC LIMIT 1), 0) AS any_pda; \gset
SELECT COALESCE((SELECT project_id FROM security_project_dicom_condition ORDER BY priority DESC NULLS LAST LIMIT 1), 0) AS any_proj; \gset
SELECT COALESCE((SELECT role_id FROM security_role_dicom_condition ORDER BY priority DESC NULLS LAST LIMIT 1), 0) AS any_role; \gset

\echo '--- UID mapping'
EXPLAIN ANALYZE SELECT id FROM project_data_study WHERE project_id = :project_id AND study_uid = :'study_uid';
EXPLAIN ANALYZE SELECT id FROM project_data_series WHERE study_id = :study_id AND series_uid = :'series_uid';
EXPLAIN ANALYZE SELECT id FROM project_data_instance WHERE series_id = :series_id AND instance_uid = :'instance_uid';

\echo '--- Study filters'
EXPLAIN ANALYZE SELECT id FROM project_data_study WHERE project_id = :project_id AND study_date = :study_date LIMIT 50;
EXPLAIN ANALYZE SELECT id FROM project_data_study WHERE project_id = :project_id AND patient_id = :'patient_id' LIMIT 50;

\echo '--- Series filters'
EXPLAIN ANALYZE SELECT id FROM project_data_series WHERE study_id = :study_id AND modality = :'modality' LIMIT 50;

\echo '--- Membership'
EXPLAIN ANALYZE SELECT 1 FROM security_user_project WHERE user_id = :user_id AND project_id = :project_id;

\echo '--- Explicit access'
EXPLAIN ANALYZE SELECT 1 FROM project_data_access WHERE user_id = :user_id AND project_id = :project_id AND resource_level IN ('STUDY','SERIES') LIMIT 50;

\echo '--- Rule links'
EXPLAIN ANALYZE SELECT access_condition_id FROM security_project_dicom_condition WHERE project_id = :any_proj ORDER BY priority DESC LIMIT 50;
EXPLAIN ANALYZE SELECT access_condition_id FROM security_role_dicom_condition WHERE role_id = :any_role ORDER BY priority DESC LIMIT 50;
