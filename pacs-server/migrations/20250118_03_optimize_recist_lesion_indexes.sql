-- RECIST Lesion 성능 최적화를 위한 인덱스 추가
-- 작성일: 2025-01-18
-- 목적: 쿼리 성능 최적화 및 N+1 문제 해결

-- ============================================================
-- 1. recist_lesion 테이블 인덱스 최적화
-- ============================================================

-- 1.1 subject_id + lesion_type 복합 인덱스 (가장 많이 사용되는 쿼리)
-- 용도: GET /api/recist-lesions/subjects/{subject_id}?lesion_type=TARGET
CREATE INDEX IF NOT EXISTS idx_recist_lesion_subject_type 
ON recist_lesion(subject_id, lesion_type);

-- 1.2 subject_id + lesion_number 복합 인덱스 (정렬 최적화)
-- 용도: ORDER BY lesion_number 쿼리 최적화
CREATE INDEX IF NOT EXISTS idx_recist_lesion_subject_number 
ON recist_lesion(subject_id, lesion_number);

-- 1.3 baseline_timepoint_id 인덱스 (TimePoint별 조회)
-- 용도: Baseline TimePoint에 속한 모든 Lesion 조회
CREATE INDEX IF NOT EXISTS idx_recist_lesion_baseline_tp 
ON recist_lesion(baseline_timepoint_id) 
WHERE baseline_timepoint_id IS NOT NULL;

-- 1.4 project_id 인덱스 (프로젝트별 통계)
-- 용도: 프로젝트 전체 Lesion 통계 조회
CREATE INDEX IF NOT EXISTS idx_recist_lesion_project 
ON recist_lesion(project_id);

-- ============================================================
-- 2. recist_lesion_annotation_map 테이블 인덱스 최적화
-- ============================================================

-- 2.1 lesion_id 인덱스 (이미 FK로 존재하지만 명시적 생성)
-- 용도: Lesion의 모든 Annotation 조회
CREATE INDEX IF NOT EXISTS idx_recist_annotation_map_lesion 
ON recist_lesion_annotation_map(lesion_id);

-- 2.2 annotation_id 인덱스 (역방향 조회)
-- 용도: Annotation이 어떤 Lesion에 속하는지 조회
CREATE INDEX IF NOT EXISTS idx_recist_annotation_map_annotation 
ON recist_lesion_annotation_map(annotation_id);

-- 2.3 timepoint_id 인덱스 (TimePoint별 측정값 조회)
-- 용도: 특정 TimePoint의 모든 측정값 조회
CREATE INDEX IF NOT EXISTS idx_recist_annotation_map_timepoint 
ON recist_lesion_annotation_map(timepoint_id);

-- 2.4 lesion_id + timepoint_id 복합 인덱스 (가장 많이 사용)
-- 용도: 특정 Lesion의 TimePoint별 측정값 조회
CREATE INDEX IF NOT EXISTS idx_recist_annotation_map_lesion_tp 
ON recist_lesion_annotation_map(lesion_id, timepoint_id);

-- ============================================================
-- 3. 통계 정보 업데이트
-- ============================================================

-- PostgreSQL의 쿼리 플래너가 최적의 실행 계획을 선택할 수 있도록 통계 정보 업데이트
ANALYZE recist_lesion;
ANALYZE recist_lesion_annotation_map;

-- ============================================================
-- 4. 인덱스 사용 확인 쿼리 (주석)
-- ============================================================

-- 인덱스가 제대로 사용되는지 확인하려면 아래 쿼리 실행:
-- EXPLAIN ANALYZE SELECT * FROM recist_lesion WHERE subject_id = 1 AND lesion_type = 'TARGET';
-- EXPLAIN ANALYZE SELECT * FROM recist_lesion_annotation_map WHERE lesion_id = 1 AND timepoint_id = 1;

-- ============================================================
-- 5. 성능 모니터링 뷰 (선택사항)
-- ============================================================

-- 인덱스 사용 통계 확인 뷰
CREATE OR REPLACE VIEW v_recist_lesion_index_usage AS
SELECT
    schemaname,
    relname as tablename,
    indexrelname as indexname,
    idx_scan as index_scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched
FROM pg_stat_user_indexes
WHERE relname IN ('recist_lesion', 'recist_lesion_annotation_map')
ORDER BY idx_scan DESC;

-- 테이블 크기 및 인덱스 크기 확인 뷰
CREATE OR REPLACE VIEW v_recist_lesion_table_sizes AS
SELECT
    t.tablename,
    pg_size_pretty(pg_total_relation_size(quote_ident(t.schemaname)||'.'||quote_ident(t.tablename))) as total_size,
    pg_size_pretty(pg_relation_size(quote_ident(t.schemaname)||'.'||quote_ident(t.tablename))) as table_size,
    pg_size_pretty(pg_total_relation_size(quote_ident(t.schemaname)||'.'||quote_ident(t.tablename)) -
                   pg_relation_size(quote_ident(t.schemaname)||'.'||quote_ident(t.tablename))) as indexes_size
FROM pg_tables t
WHERE t.tablename IN ('recist_lesion', 'recist_lesion_annotation_map')
ORDER BY pg_total_relation_size(quote_ident(t.schemaname)||'.'||quote_ident(t.tablename)) DESC;

-- ============================================================
-- 완료
-- ============================================================

