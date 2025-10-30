-- Core indexes and constraints for RBAC/DICOM gateway performance

-- project_data_study
ALTER TABLE project_data_study
  ADD CONSTRAINT IF NOT EXISTS uq_pds_project_studyuid UNIQUE (project_id, study_uid);
CREATE INDEX IF NOT EXISTS idx_pds_project_studydate ON project_data_study (project_id, study_date);
CREATE INDEX IF NOT EXISTS idx_pds_project_patient ON project_data_study (project_id, patient_id);
CREATE INDEX IF NOT EXISTS idx_pds_study_uid ON project_data_study (study_uid);

-- project_data_series
ALTER TABLE project_data_series
  ADD CONSTRAINT IF NOT EXISTS uq_pdsr_study_seriesuid UNIQUE (study_id, series_uid);
CREATE INDEX IF NOT EXISTS idx_pdsr_study ON project_data_series (study_id);
CREATE INDEX IF NOT EXISTS idx_pdsr_study_modality ON project_data_series (study_id, modality);
CREATE INDEX IF NOT EXISTS idx_pdsr_series_uid ON project_data_series (series_uid);

-- project_data_instance
ALTER TABLE project_data_instance
  ADD CONSTRAINT IF NOT EXISTS uq_pdi_series_instance UNIQUE (series_id, instance_uid);
CREATE INDEX IF NOT EXISTS idx_pdi_series ON project_data_instance (series_id);
CREATE INDEX IF NOT EXISTS idx_pdi_content_date ON project_data_instance (content_date);
CREATE INDEX IF NOT EXISTS idx_pdi_instance_uid ON project_data_instance (instance_uid);

-- security_user_project
ALTER TABLE security_user_project
  ADD CONSTRAINT IF NOT EXISTS uq_sup_user_project UNIQUE (user_id, project_id);
CREATE INDEX IF NOT EXISTS idx_sup_project ON security_user_project (project_id);

-- project_data_access
CREATE INDEX IF NOT EXISTS idx_pda_user_project_level ON project_data_access (user_id, project_id, resource_level);
CREATE INDEX IF NOT EXISTS idx_pda_study_when_study ON project_data_access (study_id) WHERE resource_level = 'STUDY';
CREATE INDEX IF NOT EXISTS idx_pda_series_when_series ON project_data_access (series_id) WHERE resource_level = 'SERIES';

-- 규칙 조건 링크
CREATE INDEX IF NOT EXISTS idx_proj_cond_project_priority ON security_project_dicom_condition (project_id, priority DESC);
CREATE INDEX IF NOT EXISTS idx_role_cond_role_priority ON security_role_dicom_condition (role_id, priority DESC);
CREATE INDEX IF NOT EXISTS idx_access_condition_level ON security_access_condition (resource_level);
CREATE INDEX IF NOT EXISTS idx_access_condition_tag ON security_access_condition (dicom_tag);
