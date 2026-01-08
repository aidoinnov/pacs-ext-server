-- ============================================================================
-- 035_add_report_review_ext_fields.sql
-- report_status와 review extension 필드 추가
-- ============================================================================

-- report_status와 review 확장 필드 추가
INSERT INTO ext_field_def (field_key, label, level, value_type, source_system, source_config, sortable, filterable, default_visible, default_order) 
VALUES
    -- Report Status (series_user_report 테이블에서 조회)
    ('report_status', 'Report Status', 'study', 'enum', 'internal',
     '{"type": "db", "table": "series_user_report", "column": "status", "join_key": "study_instance_uid", "enum_values": ["unread", "approval", "unapproval"]}',
     TRUE, TRUE, TRUE, 60),
    
    -- Review/Annotation (annotation 테이블 기반)
    ('review', 'Review', 'study', 'object', 'annotation',
     '{"type": "computed", "includes": ["reviewStage", "availableStages", "annotationSummary"]}',
     FALSE, FALSE, TRUE, 61)

ON CONFLICT (field_key) DO UPDATE SET
    label = EXCLUDED.label,
    source_config = EXCLUDED.source_config,
    default_order = EXCLUDED.default_order;

-- default view에 report_status와 review 필드 추가
INSERT INTO study_list_view_field (view_id, field_source, field_key, display_order, visible, pinned) VALUES
    ('default', 'extension', 'report_status', 10, TRUE, FALSE),
    ('default', 'extension', 'review', 11, TRUE, FALSE)
ON CONFLICT (view_id, field_source, field_key) DO NOTHING;

