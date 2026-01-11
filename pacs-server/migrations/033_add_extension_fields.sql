-- ============================================================================
-- 033_add_extension_fields.sql
-- Extension 필드 정의 추가
-- ============================================================================

-- 기존 확장 필드가 없으면 추가
INSERT INTO ext_field_def (field_key, label, level, value_type, source_system, source_config, sortable, filterable, default_visible, default_order) 
VALUES
    -- Project 관련
    ('project', 'Project', 'study', 'string', 'internal', 
     '{"type": "db", "table": "project_data", "column": "project_name", "join_key": "study_instance_uid"}', 
     TRUE, TRUE, TRUE, 50),
    
    -- ScanDate (실제 스캔 날짜, StudyDate와 다를 수 있음)
    ('scanDate', 'Scan Date', 'study', 'date', 'internal',
     '{"type": "db", "table": "project_data", "column": "scan_date", "join_key": "study_instance_uid"}',
     TRUE, TRUE, FALSE, 53),
    
    -- TimePoint (Baseline, Follow-up 등)
    ('timePoint', 'Time Point', 'study', 'string', 'internal',
     '{"type": "db", "table": "project_data", "column": "time_point", "join_key": "study_instance_uid"}',
     TRUE, TRUE, TRUE, 54),
    
    -- Visit Type (Screening, Treatment 등)
    ('visitType', 'Visit Type', 'study', 'string', 'internal',
     '{"type": "db", "table": "project_data", "column": "visit_type", "join_key": "study_instance_uid"}',
     TRUE, TRUE, TRUE, 55),
    
    -- Visit Number
    ('visitNumber', 'Visit Number', 'study', 'number', 'internal',
     '{"type": "db", "table": "project_data", "column": "visit_number", "join_key": "study_instance_uid"}',
     TRUE, TRUE, FALSE, 56),
    
    -- Annotation Count (어노테이션 DB에서 조회)
    ('annotationCount', 'Annotation', 'study', 'number', 'annotation',
     '{"type": "db", "table": "annotations", "aggregate": "count", "join_key": "study_instance_uid"}',
     TRUE, FALSE, TRUE, 57),
    
    -- Status (워크플로우 상태)
    ('status', 'Status', 'study', 'enum', 'workflow',
     '{"type": "db", "table": "study_workflow", "column": "status", "join_key": "study_instance_uid", "enum_values": ["New", "InProgress", "Reviewed", "Completed"]}',
     TRUE, TRUE, TRUE, 58)

ON CONFLICT (field_key) DO UPDATE SET
    label = EXCLUDED.label,
    source_config = EXCLUDED.source_config,
    default_order = EXCLUDED.default_order;

-- Age, Gender 필드도 추가 (DICOM에서 계산됨)
INSERT INTO dicom_field_def (field_key, tag, vr, label, level, value_type, sortable, filterable, default_visible, default_order)
VALUES
    ('PatientAge', '00101010', 'AS', 'Age', 'study', 'string', TRUE, TRUE, TRUE, 9)
ON CONFLICT (field_key) DO NOTHING;

-- 코멘트
COMMENT ON TABLE ext_field_def IS 'Extension 필드 정의 - DICOM 외 확장 메타데이터 필드';

