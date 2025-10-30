-- Drop core indexes/constraints (safe if missing)

-- project_data_study
ALTER TABLE project_data_study DROP CONSTRAINT IF EXISTS uq_pds_project_studyuid;
DROP INDEX IF EXISTS idx_pds_project_studydate;
DROP INDEX IF EXISTS idx_pds_project_patient;
DROP INDEX IF EXISTS idx_pds_study_uid;

-- project_data_series
ALTER TABLE project_data_series DROP CONSTRAINT IF EXISTS uq_pdsr_study_seriesuid;
DROP INDEX IF EXISTS idx_pdsr_study;
DROP INDEX IF EXISTS idx_pdsr_study_modality;
DROP INDEX IF EXISTS idx_pdsr_series_uid;

-- project_data_instance
ALTER TABLE project_data_instance DROP CONSTRAINT IF EXISTS uq_pdi_series_instance;
DROP INDEX IF EXISTS idx_pdi_series;
DROP INDEX IF EXISTS idx_pdi_content_date;
DROP INDEX IF EXISTS idx_pdi_instance_uid;

-- security_user_project
ALTER TABLE security_user_project DROP CONSTRAINT IF EXISTS uq_sup_user_project;
DROP INDEX IF EXISTS idx_sup_project;

-- project_data_access
DROP INDEX IF EXISTS idx_pda_user_project_level;
DROP INDEX IF EXISTS idx_pda_study_when_study;
DROP INDEX IF EXISTS idx_pda_series_when_series;

-- 규칙 조건 링크
DROP INDEX IF EXISTS idx_proj_cond_project_priority;
DROP INDEX IF EXISTS idx_role_cond_role_priority;
DROP INDEX IF EXISTS idx_access_condition_level;
DROP INDEX IF EXISTS idx_access_condition_tag;
