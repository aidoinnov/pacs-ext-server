-- ==========================
-- MEASUREMENT VALUES MIGRATION
-- ==========================

-- annotation_annotation 테이블에 measurement_values 컬럼 추가
ALTER TABLE annotation_annotation 
ADD COLUMN measurement_values JSONB DEFAULT NULL;

-- JSONB 쿼리 성능을 위한 인덱스 추가
CREATE INDEX idx_annotation_measurement_values ON annotation_annotation USING GIN (measurement_values);

-- 문서화를 위한 주석 추가
COMMENT ON COLUMN annotation_annotation.measurement_values IS 
'구조화된 측정값을 JSON 형식으로 저장: [{"id": "m1", "type": "raw", "values": [42.3], "unit": "mm"}]';
