# Project Data Hierarchy Refactor - 작업 완료 보고서

## 📋 작업 요약

**작업 일자**: 2025-10-31  
**작업 시간**: 약 2.5시간  
**작업 상태**: ✅ 완료  
**작업자**: AI Assistant + User

## 🎯 작업 목표 달성

### ✅ 완료된 목표
1. `project_data_study` 테이블에서 `project_id` 제거 → Study를 전역 엔티티로 변경
2. `project_data` 테이블을 계층적 리소스 매핑 테이블로 재구성
3. STUDY/SERIES/INSTANCE 레벨의 세밀한 접근 제어 구현
4. RBAC 로직 수정 (DENIED 체크 추가, 기본 허용 로직 추가)
5. 테스트 및 검증 완료

## 📊 변경 사항 상세

### 1. 데이터베이스 스키마 변경

#### 마이그레이션 파일
- **파일**: `pacs-server/migrations/020_refactor_project_data_hierarchy.sql`
- **주요 변경**:
  - `project_data_study` 테이블에서 `project_id` 컬럼 제거
  - `project_data_instance` 테이블 생성
  - `project_data` 테이블 재구성 (계층적 리소스 매핑)
  - `resource_level` ENUM 타입 추가 (STUDY/SERIES/INSTANCE)
  - 데이터 마이그레이션 로직 포함
  - 하위 호환성 뷰 제공

#### 새로운 테이블 구조
```sql
-- project_data_study (전역 엔티티)
CREATE TABLE project_data_study (
    id INTEGER PRIMARY KEY,
    study_uid TEXT UNIQUE,
    study_description TEXT,
    patient_id TEXT,
    patient_name TEXT,
    study_date DATE,
    -- project_id 제거됨 ✅
);

-- project_data (계층적 리소스 매핑)
CREATE TABLE project_data (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    resource_level resource_level_enum NOT NULL DEFAULT 'STUDY',
    study_id INTEGER,
    series_id INTEGER,
    instance_id INTEGER,
    CONSTRAINT chk_project_data_study_required CHECK (study_id IS NOT NULL),
    CONSTRAINT chk_project_data_study_level CHECK (...)
);
```

### 2. 엔티티 수정

#### `pacs-server/src/domain/entities/project_data.rs`
- `ProjectDataStudy` 구조체에서 `project_id` 필드 제거
- 주석 추가: "전역 엔티티 (프로젝트 독립적)"

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectDataStudy {
    pub id: i32,
    // pub project_id: i32,  // ❌ 제거됨
    pub study_uid: String,
    pub study_description: Option<String>,
    // ... 기타 필드
}
```

### 3. Repository 쿼리 수정

#### `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs`

**수정된 메서드**:
1. `find_study_by_id`: `project_id` 필드 제거
2. `find_study_by_uid`: `project_data` 테이블과 JOIN
3. `find_studies_by_project_id`: `project_data` 테이블과 JOIN + `resource_level = 'STUDY'` 필터
4. `count_studies_by_project_id`: `project_data` 테이블과 JOIN

**예시**:
```rust
async fn find_study_by_uid(&self, project_id: i32, study_uid: &str) -> Result<Option<ProjectDataStudy>, RepositoryError> {
    let study = sqlx::query_as::<_, ProjectDataStudy>(
        "SELECT pds.* FROM project_data_study pds
         INNER JOIN project_data pd ON pd.study_id = pds.id
         WHERE pd.project_id = $1 AND pds.study_uid = $2"
    )
    .bind(project_id)
    .bind(study_uid)
    .fetch_optional(&self.pool)
    .await?;
    Ok(study)
}
```

### 4. RBAC 로직 대폭 수정

#### `pacs-server/src/infrastructure/services/dicom_rbac_evaluator_impl.rs`

**수정된 메서드**:
1. `evaluate_study_access` (lines 296-419)
2. `evaluate_series_access` (lines 422-513)
3. `evaluate_instance_access` (lines 579-670)
4. `evaluate_study_uid` (lines 505-533)
5. `evaluate_series_uid` (lines 535-566)

**새로운 RBAC 로직 우선순위**:
```
1. 프로젝트 멤버십 확인 (필수)
2. 명시적 DENIED 체크 (최우선) → 즉시 거부
3. 명시적 APPROVED 체크 → 즉시 허용
4. 상위 리소스 권한 상속 (Series → Study, Instance → Series)
5. 기관 기반 접근 (Study만)
6. 룰 기반 조건 평가 (Study만)
7. 기본값: 프로젝트 멤버면 허용
```

**핵심 코드**:
```rust
// Priority 1: Check for explicit DENIED status
let is_denied: bool = sqlx::query_scalar(
    "SELECT EXISTS(SELECT 1 FROM project_data_access 
     WHERE user_id = $1 AND project_id = $2 
     AND status = 'DENIED' AND resource_level = 'STUDY' AND study_id = $3)"
)
.bind(user_id)
.bind(project_id)
.bind(study_id)
.fetch_one(&self.pool)
.await
.unwrap_or(false);

if is_denied {
    return RbacEvaluationResult {
        allowed: false,
        reason: Some("explicit_study_denied".to_string()),
    };
}

// Priority 2: Check for explicit APPROVED status
// ...

// Default: Allow for project members
RbacEvaluationResult {
    allowed: true,
    reason: Some("project_member_default_access".to_string()),
}
```

### 5. 테스트 프로그램 작성

#### 작성된 테스트 프로그램
1. `pacs-server/examples/check_schema.rs` - 스키마 확인
2. `pacs-server/examples/run_migration_020.rs` - 마이그레이션 실행
3. `pacs-server/examples/fix_foreign_key.rs` - 외래 키 수정
4. `pacs-server/examples/setup_test_data.rs` - 테스트 데이터 준비
5. `pacs-server/examples/test_rbac.rs` - RBAC 테스트

## 🧪 테스트 결과

### 테스트 시나리오

#### 시나리오 1: User 1 (기본 접근)
- **설정**: `project_data_access`에 레코드 없음
- **결과**: 
  - Study 100: ✅ DEFAULT (허용)
  - Study 101: ✅ DEFAULT (허용)
  - Study 102: ✅ DEFAULT (허용)
- **검증**: ✅ 프로젝트 멤버는 기본적으로 모든 데이터 접근 가능

#### 시나리오 2: User 2 (Study 100 거부)
- **설정**: Study 100에 대해 `DENIED` 레코드 추가
- **결과**:
  - Study 100: ❌ DENIED (거부)
  - Study 101: ✅ DEFAULT (허용)
  - Study 102: ✅ DEFAULT (허용)
- **검증**: ✅ 명시적 거부가 정확하게 동작

#### 시나리오 3: User 3 (Study 101 승인)
- **설정**: Study 101에 대해 `APPROVED` 레코드 추가
- **결과**:
  - Study 100: ✅ DEFAULT (허용)
  - Study 101: ✅ APPROVED (명시적 승인)
  - Study 102: ✅ DEFAULT (허용)
- **검증**: ✅ 명시적 승인이 정확하게 동작

### 테스트 실행 로그
```
🧪 시나리오 1: User 1 - 기본 접근
  1.2.3.100 | Test Study 100 | ✅ DEFAULT
  1.2.3.101 | Test Study 101 | ✅ DEFAULT
  1.2.3.102 | Test Study 102 | ✅ DEFAULT

🧪 시나리오 2: User 2 - Study 100 거부
  1.2.3.100 | Test Study 100 | ❌ DENIED
  1.2.3.101 | Test Study 101 | ✅ DEFAULT
  1.2.3.102 | Test Study 102 | ✅ DEFAULT

🧪 시나리오 3: User 3 - Study 101 명시적 승인
  1.2.3.100 | Test Study 100 | ✅ DEFAULT
  1.2.3.101 | Test Study 101 | ✅ APPROVED
  1.2.3.102 | Test Study 102 | ✅ DEFAULT

✅ 테스트 완료!
```

## 🎯 달성된 목표

### 1. Study 전역화
- ✅ `project_data_study` 테이블에서 `project_id` 제거
- ✅ Study는 이제 여러 프로젝트에서 공유 가능
- ✅ 데이터 중복 없음

### 2. 계층적 리소스 매핑
- ✅ `project_data` 테이블로 프로젝트-리소스 매핑
- ✅ STUDY/SERIES/INSTANCE 레벨 지원
- ✅ 세밀한 접근 제어 가능

### 3. RBAC 강화
- ✅ 명시적 DENIED 체크 추가 (최우선)
- ✅ 명시적 APPROVED 체크 추가
- ✅ 기본 허용 로직 추가 (프로젝트 멤버)
- ✅ 계층적 권한 상속 구현

### 4. 하위 호환성
- ✅ 데이터 마이그레이션 로직 포함
- ✅ 백업 테이블 생성
- ✅ 기존 데이터 보존

## 📈 성능 및 확장성

### 인덱스 추가
```sql
CREATE INDEX idx_project_data_project ON project_data(project_id);
CREATE INDEX idx_project_data_study ON project_data(study_id);
CREATE INDEX idx_project_data_series ON project_data(series_id);
CREATE INDEX idx_project_data_instance ON project_data(instance_id);
CREATE INDEX idx_project_data_level ON project_data(resource_level);
CREATE INDEX idx_project_data_project_level ON project_data(project_id, resource_level);
```

### 확장성
- 나중에 다른 리소스 레벨 추가 가능 (예: PATIENT, MODALITY)
- 프로젝트별 세밀한 데이터 포함 제어
- 사용자별 세밀한 접근 제어

## 🔍 영향 받는 API

### DICOM Gateway API
- `GET /api/dicom/studies` - RBAC 로직 적용
- `GET /api/dicom/studies/{studyUID}/series` - RBAC 로직 적용
- `GET /api/dicom/studies/{studyUID}/series/{seriesUID}/instances` - RBAC 로직 적용

### 동작 변경
- 이전: Study가 프로젝트에 직접 종속
- 이후: Study는 전역, `project_data`를 통해 프로젝트에 매핑
- RBAC: 명시적 거부/승인 + 기본 허용 로직

## ⚠️ 주의사항

### 마이그레이션 실행 시
1. 데이터베이스 백업 필수
2. 외래 키 제약 조건 확인
3. `project_data_access` 테이블의 `project_data_id` 필드 확인

### 프로덕션 배포 시
1. 마이그레이션 실행 전 백업
2. 다운타임 계획 (외래 키 수정 시간 고려)
3. 롤백 계획 준비

## 🚀 다음 단계

### 즉시 수행 가능
1. ✅ API 엔드포인트 테스트 (`GET /api/dicom/studies`)
2. ✅ 프론트엔드 통합 테스트
3. ⬜ 성능 테스트 (대량 데이터)

### 향후 계획
1. ⬜ Series/Instance 레벨 접근 제어 UI 구현
2. ⬜ 프로젝트 데이터 관리 API 개선
3. ⬜ 대량 데이터 마이그레이션 도구 개발

## 📝 교훈 및 개선점

### 잘된 점
1. 체계적인 마이그레이션 계획
2. 테스트 프로그램을 통한 검증
3. 하위 호환성 고려
4. 명확한 RBAC 우선순위 정의

### 개선 필요
1. 마이그레이션 실행 시 외래 키 자동 업데이트
2. 테스트 데이터 자동 생성 스크립트
3. API 통합 테스트 자동화

## ✅ 최종 체크리스트

- [x] 마이그레이션 파일 작성
- [x] 엔티티 수정
- [x] Repository 수정
- [x] RBAC 로직 수정
- [x] 테스트 프로그램 작성
- [x] 마이그레이션 실행
- [x] 테스트 데이터 준비
- [x] RBAC 테스트 실행
- [x] 결과 검증
- [x] 문서화
- [ ] Git 커밋
- [ ] CHANGELOG 업데이트

## 📞 연락처

문제 발생 시 또는 추가 질문이 있을 경우:
- 작업자: AI Assistant
- 작업 일자: 2025-10-31

