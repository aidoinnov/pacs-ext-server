-- 기본 시나리오 시드 템플릿 (예시)
-- 실제 테스트에서는 동적 값 바인딩을 사용하세요.

-- 기관
-- INSERT INTO security_institution (institution_code, institution_name, is_active) VALUES ('INSTXXXX', 'Test Institution', true);
-- INSERT INTO project_data_institution (institution_code, institution_name, is_active) VALUES ('DATAXXXX', 'Data Institution', true);

-- 사용자/프로젝트
-- INSERT INTO security_user (keycloak_id, username, email, account_status, institution_id)
-- VALUES ('00000000-0000-0000-0000-000000000000', 'user_xxxx', 'user_xxxx@test.com', 'ACTIVE', 1);
-- INSERT INTO security_project (name, description, status) VALUES ('ProjectA_xxxx', 'Project A', 'ACTIVE');
-- INSERT INTO security_user_project (user_id, project_id) VALUES (1, 1);

-- 프로젝트 데이터 (Study/Series)
-- INSERT INTO project_data (project_id, study_uid) VALUES (1, '1.2.3.4.5.A1.xxxx');
-- INSERT INTO project_data_study (project_id, study_uid, data_institution_id, modality)
-- VALUES (1, '1.2.3.4.5.A1.xxxx', 1, 'CT');
-- INSERT INTO project_data_series (study_id, series_uid) VALUES (1, '1.2.3.4.5.A1.S1.xxxx');

-- 명시 권한 (Study 레벨)
-- INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, project_data_id)
-- VALUES (1, 1, 'STUDY', 1, 'APPROVED', 1);

-- 규칙 (ALLOW/DENY/LIMIT)
-- ALLOW Modality=CT
-- INSERT INTO security_access_condition (resource_level, resource_type, dicom_tag, operator, value, condition_type)
-- VALUES ('STUDY', 'study', '00080060', 'EQ', 'CT', 'ALLOW');
-- DENY StudyDate in 2023
-- INSERT INTO security_access_condition (resource_level, resource_type, dicom_tag, operator, value, condition_type)
-- VALUES ('STUDY', 'study', '00080020', 'RANGE', '20230101-20231231', 'DENY');
-- 프로젝트 연결
-- INSERT INTO security_project_dicom_condition (project_id, access_condition_id, priority) VALUES (1, 1, 10);
-- INSERT INTO security_project_dicom_condition (project_id, access_condition_id, priority) VALUES (1, 2, 20);
