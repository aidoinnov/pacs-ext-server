-- report_guide_template, user_custom_report_template에서 name 필드 및 제약조건 제거

-- 1. report_guide_template: UNIQUE(name) 제거 후 name 컬럼 제거
ALTER TABLE report_guide_template DROP CONSTRAINT IF EXISTS report_guide_template_name_key;
ALTER TABLE report_guide_template DROP COLUMN IF EXISTS name;

-- 2. user_custom_report_template: UNIQUE(user_id, name) 제거 후 name 컬럼 제거
ALTER TABLE user_custom_report_template DROP CONSTRAINT IF EXISTS user_custom_report_template_user_id_name_key;
ALTER TABLE user_custom_report_template DROP COLUMN IF EXISTS name;
