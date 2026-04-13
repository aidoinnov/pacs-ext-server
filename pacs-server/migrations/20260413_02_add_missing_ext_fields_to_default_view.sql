-- ============================================================================
-- 20260413_02_add_missing_ext_fields_to_default_view.sql
-- default View 필드 구성을 프로젝트 표준 컬럼 순서로 재구성
-- ============================================================================
--
-- 032에서 default View를 생성했으나, 033에서 추가한 extension 필드들이
-- View에 연결되지 않았고 컬럼 순서도 프로젝트 표준과 불일치했음.
--
-- 표준 컬럼 순서:
--   Project → Subject No → Age → Gender → Study Description →
--   Modality → Study Date → Time Point → Visit Type →
--   Visit Number → Annotation → Status
-- ============================================================================

-- 기존 default view 필드 전부 삭제 후 재구성
DELETE FROM study_list_view_field WHERE view_id = 'default';

INSERT INTO study_list_view_field (view_id, field_source, field_key, display_order, visible, pinned) VALUES
    ('default', 'extension', 'project',            0, TRUE, FALSE),
    ('default', 'extension', 'subjectNo',           1, TRUE, FALSE),
    ('default', 'dicom',     'PatientAge',          2, TRUE, FALSE),
    ('default', 'dicom',     'PatientSex',          3, TRUE, FALSE),
    ('default', 'dicom',     'StudyDescription',    4, TRUE, FALSE),
    ('default', 'dicom',     'ModalitiesInStudy',   5, TRUE, FALSE),
    ('default', 'dicom',     'StudyDate',           6, TRUE, FALSE),
    ('default', 'extension', 'timePoint',           7, TRUE, FALSE),
    ('default', 'extension', 'visitType',           8, TRUE, FALSE),
    ('default', 'extension', 'visitNumber',         9, TRUE, FALSE),
    ('default', 'extension', 'annotationCount',    10, TRUE, FALSE),
    ('default', 'extension', 'status',             11, TRUE, FALSE);
