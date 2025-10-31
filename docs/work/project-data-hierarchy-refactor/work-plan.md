# Project Data Hierarchy Refactor - 작업 계획

## 📋 작업 개요

**작업 일자**: 2025-10-31  
**작업자**: AI Assistant + User  
**목표**: `project_data_study` 테이블에서 `project_id` 제거 및 계층적 리소스 매핑 구조 구현

## 🎯 작업 목표

### 문제점
- `project_data_study` 테이블이 `project_id`를 직접 참조하여 Study가 프로젝트에 종속됨
- Study는 전역 엔티티여야 하며, 여러 프로젝트에서 공유 가능해야 함
- Study/Series/Instance 레벨의 세밀한 접근 제어가 불가능

### 해결 방안
1. `project_data_study` 테이블에서 `project_id` 제거 (전역 엔티티화)
2. `project_data` 테이블을 계층적 리소스 매핑 테이블로 재구성
3. `resource_level` (STUDY/SERIES/INSTANCE) 지원
4. RBAC 로직 수정 (DENIED 체크 추가, 기본 허용 로직 추가)

## 📊 새로운 데이터 구조

### 1. `project_data_study` (전역 Study)
```sql
CREATE TABLE project_data_study (
    id INTEGER PRIMARY KEY,
    study_uid TEXT UNIQUE,  -- 전역 고유
    study_description TEXT,
    patient_id TEXT,
    patient_name TEXT,
    study_date DATE,
    -- ❌ project_id 제거됨
)
```

### 2. `project_data` (계층적 리소스 매핑)
```sql
CREATE TABLE project_data (
    id INTEGER PRIMARY KEY,
    project_id INTEGER,  -- 어느 프로젝트에 포함되는지
    
    resource_level ENUM('STUDY', 'SERIES', 'INSTANCE'),
    
    -- 계층적 참조
    study_id INTEGER,     -- 필수 (항상 설정)
    series_id INTEGER,    -- SERIES/INSTANCE 레벨일 때만
    instance_id INTEGER,  -- INSTANCE 레벨일 때만
    
    CONSTRAINT chk_study_required CHECK (study_id IS NOT NULL),
    CONSTRAINT chk_level_consistency CHECK (...)
)
```

## 🔄 작업 단계

### Phase 1: 데이터베이스 스키마 변경
- [x] 마이그레이션 파일 작성 (`020_refactor_project_data_hierarchy.sql`)
- [x] `project_data_study`에서 `project_id` 제거
- [x] `project_data` 테이블 재구성
- [x] `project_data_instance` 테이블 생성
- [x] 데이터 마이그레이션 로직 추가
- [x] 인덱스 생성

### Phase 2: 엔티티 및 Repository 수정
- [x] `ProjectDataStudy` 엔티티에서 `project_id` 필드 제거
- [x] Repository 쿼리 수정 (`project_data` 테이블과 JOIN)
- [x] `find_study_by_uid`: `project_data` 테이블과 JOIN
- [x] `find_studies_by_project_id`: `resource_level = 'STUDY'` 필터 추가
- [x] `count_studies_by_project_id`: `project_data` 테이블과 JOIN

### Phase 3: RBAC 로직 수정
- [x] `evaluate_study_access`: DENIED 체크 추가, 기본 허용 로직 추가
- [x] `evaluate_series_access`: 계층적 권한 상속 구현
- [x] `evaluate_instance_access`: 계층적 권한 상속 구현
- [x] `evaluate_study_uid`: `project_data` 테이블과 JOIN
- [x] `evaluate_series_uid`: `project_data` 테이블과 JOIN

### Phase 4: 테스트 및 검증
- [x] 마이그레이션 실행
- [x] 외래 키 수정
- [x] 테스트 데이터 준비
- [x] RBAC 테스트 실행
- [x] 결과 검증

## 📝 RBAC 로직 우선순위

```
1. ✅ 프로젝트 멤버십 확인
   └─ 멤버가 아니면 → 즉시 거부

2. ✅ 명시적 거부 확인 (DENIED) - 최우선!
   └─ project_data_access에 status='DENIED' 레코드가 있으면 → 즉시 거부
   
3. ✅ 명시적 승인 확인 (APPROVED)
   └─ project_data_access에 status='APPROVED' 레코드가 있으면 → 즉시 허용

4. ✅ 상위 리소스 권한 상속
   └─ Series → Study 권한 상속
   └─ Instance → Series 권한 상속 (→ Study 권한 상속)
   
5. ✅ 기관 기반 접근 (Study만)
   └─ 같은 기관 또는 기관 간 허용

6. ✅ 룰 기반 조건 평가 (Study만)
   └─ access_condition + role/project 조건

7. ✅ 기본값: 프로젝트 멤버면 허용 (Study만)
   └─ 명시적 DENIED가 없고, 다른 제약도 없으면 허용
```

## 🎯 사용 예시

### Study 전체를 프로젝트에 포함
```sql
INSERT INTO project_data (project_id, resource_level, study_id)
VALUES (1, 'STUDY', 100);
-- → 프로젝트 1에 Study 100 전체 포함
```

### 특정 Series만 프로젝트에 포함
```sql
INSERT INTO project_data (project_id, resource_level, study_id, series_id)
VALUES (1, 'SERIES', 100, 500);
-- → 프로젝트 1에 Study 100의 Series 500만 포함
```

### 특정 Instance만 프로젝트에 포함
```sql
INSERT INTO project_data (project_id, resource_level, study_id, series_id, instance_id)
VALUES (1, 'INSTANCE', 100, 500, 1000);
-- → 프로젝트 1에 Study 100 > Series 500 > Instance 1000만 포함
```

## ✅ 장점

1. **Study는 전역 엔티티** - 여러 프로젝트에서 공유 가능
2. **세밀한 접근 제어** - Study/Series/Instance 레벨별로 프로젝트 포함 가능
3. **데이터 중복 없음** - Study 메타데이터는 한 번만 저장
4. **확장성** - 나중에 다른 리소스 레벨 추가 가능
5. **RBAC 강화** - 명시적 거부/승인 + 기본 허용 로직

## 🔍 영향 받는 파일

### 마이그레이션
- `pacs-server/migrations/020_refactor_project_data_hierarchy.sql`

### 엔티티
- `pacs-server/src/domain/entities/project_data.rs`

### Repository
- `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs`

### RBAC Evaluator
- `pacs-server/src/infrastructure/services/dicom_rbac_evaluator_impl.rs`

### 테스트 프로그램
- `pacs-server/examples/check_schema.rs`
- `pacs-server/examples/run_migration_020.rs`
- `pacs-server/examples/fix_foreign_key.rs`
- `pacs-server/examples/setup_test_data.rs`
- `pacs-server/examples/test_rbac.rs`

## 📅 타임라인

- **2025-10-31 10:00**: 작업 시작
- **2025-10-31 10:30**: 마이그레이션 파일 작성 완료
- **2025-10-31 11:00**: 엔티티 및 Repository 수정 완료
- **2025-10-31 11:30**: RBAC 로직 수정 완료
- **2025-10-31 12:00**: 마이그레이션 실행 및 테스트 완료
- **2025-10-31 12:30**: 문서화 및 Git 커밋

## 🚀 다음 단계

1. API 엔드포인트 테스트 (`GET /api/dicom/studies`)
2. 프론트엔드 통합 테스트
3. 성능 테스트 (대량 데이터)
4. 프로덕션 배포 계획

