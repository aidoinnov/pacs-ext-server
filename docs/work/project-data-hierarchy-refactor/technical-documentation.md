# Project Data Hierarchy Refactor - 기술 문서

## 📋 개요

이 문서는 `project_data_study` 테이블에서 `project_id`를 제거하고 계층적 리소스 매핑 구조를 구현한 작업의 기술적 세부사항을 설명합니다.

## 🎯 아키텍처 변경

### 이전 구조 (Before)

```
┌─────────────────────────┐
│  project_data_study     │
├─────────────────────────┤
│ id                      │
│ project_id  ←───────────┼─── ❌ 프로젝트에 직접 종속
│ study_uid               │
│ study_description       │
│ patient_id              │
│ ...                     │
└─────────────────────────┘
```

**문제점**:
- Study가 프로젝트에 직접 종속됨
- 여러 프로젝트에서 같은 Study 공유 불가
- 데이터 중복 발생 가능
- Series/Instance 레벨 접근 제어 불가

### 새로운 구조 (After)

```
┌─────────────────────────┐
│  project_data_study     │  ← 전역 엔티티
├─────────────────────────┤
│ id                      │
│ study_uid (UNIQUE)      │
│ study_description       │
│ patient_id              │
│ ...                     │
└─────────────────────────┘
            ↑
            │
            │ study_id
            │
┌─────────────────────────┐
│  project_data           │  ← 매핑 테이블
├─────────────────────────┤
│ id                      │
│ project_id              │
│ resource_level          │  ← STUDY/SERIES/INSTANCE
│ study_id                │
│ series_id               │
│ instance_id             │
└─────────────────────────┘
            ↑
            │ project_id
            │
┌─────────────────────────┐
│  security_project       │
├─────────────────────────┤
│ id                      │
│ name                    │
│ ...                     │
└─────────────────────────┘
```

**장점**:
- Study는 전역 엔티티 (여러 프로젝트에서 공유 가능)
- 데이터 중복 없음
- 계층적 리소스 매핑 (STUDY/SERIES/INSTANCE)
- 세밀한 접근 제어 가능

## 🗄️ 데이터베이스 스키마

### 1. `project_data_study` 테이블

```sql
CREATE TABLE project_data_study (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- project_id 제거됨 ✅
    study_uid TEXT NOT NULL,
    study_description TEXT,
    patient_id TEXT,
    patient_name TEXT,
    patient_birth_date DATE,
    study_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    data_institution_id INTEGER REFERENCES data_institution(id),
    institution_code VARCHAR(255),
    accession_no VARCHAR(255),
    modality VARCHAR(16),
    patient_sex CHAR(1),
    study_time VARCHAR(16),
    referring_physician VARCHAR(255),
    performing_physician VARCHAR(255),
    series_count INTEGER,
    instance_count INTEGER,
    is_active BOOLEAN DEFAULT true,
    sync_status VARCHAR(50),
    
    UNIQUE (study_uid)  -- 전역 고유
);
```

### 2. `project_data` 테이블

```sql
-- resource_level ENUM 타입
CREATE TYPE resource_level_enum AS ENUM ('STUDY', 'SERIES', 'INSTANCE');

CREATE TABLE project_data (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES security_project(id) ON DELETE CASCADE,
    
    -- 리소스 레벨
    resource_level resource_level_enum NOT NULL DEFAULT 'STUDY',
    
    -- 계층적 참조
    study_id INTEGER REFERENCES project_data_study(id) ON DELETE CASCADE,
    series_id INTEGER REFERENCES project_data_series(id) ON DELETE CASCADE,
    instance_id INTEGER REFERENCES project_data_instance(id) ON DELETE CASCADE,
    
    -- 메타데이터
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 제약 조건
    CONSTRAINT chk_project_data_study_required CHECK (study_id IS NOT NULL),
    CONSTRAINT chk_project_data_study_level CHECK (
        (resource_level = 'STUDY' AND series_id IS NULL AND instance_id IS NULL) OR
        (resource_level = 'SERIES' AND series_id IS NOT NULL AND instance_id IS NULL) OR
        (resource_level = 'INSTANCE' AND series_id IS NOT NULL AND instance_id IS NOT NULL)
    ),
    
    -- 유니크 제약
    UNIQUE (project_id, study_id, series_id, instance_id)
);
```

### 3. `project_data_instance` 테이블 (신규)

```sql
CREATE TABLE project_data_instance (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES project_data_series(id) ON DELETE CASCADE,
    instance_uid TEXT NOT NULL,
    sop_class_uid TEXT,
    instance_number INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE (series_id, instance_uid)
);
```

### 4. 인덱스

```sql
-- project_data 테이블 인덱스
CREATE INDEX idx_project_data_project ON project_data(project_id);
CREATE INDEX idx_project_data_study ON project_data(study_id) WHERE study_id IS NOT NULL;
CREATE INDEX idx_project_data_series ON project_data(series_id) WHERE series_id IS NOT NULL;
CREATE INDEX idx_project_data_instance ON project_data(instance_id) WHERE instance_id IS NOT NULL;
CREATE INDEX idx_project_data_level ON project_data(resource_level);
CREATE INDEX idx_project_data_project_level ON project_data(project_id, resource_level);

-- project_data_study 테이블 인덱스
CREATE INDEX idx_study_uid ON project_data_study(study_uid);
CREATE INDEX idx_study_patient ON project_data_study(patient_id);
CREATE INDEX idx_study_date ON project_data_study(study_date);

-- project_data_instance 테이블 인덱스
CREATE INDEX idx_project_data_instance_series ON project_data_instance(series_id);
CREATE INDEX idx_project_data_instance_uid ON project_data_instance(instance_uid);
```

## 🔄 데이터 마이그레이션

### 마이그레이션 전략

1. **백업 생성**
   ```sql
   CREATE TABLE _backup_project_data_study_with_project AS
   SELECT * FROM project_data_study;
   ```

2. **제약 조건 제거**
   ```sql
   ALTER TABLE project_data_study DROP CONSTRAINT IF EXISTS project_data_study_project_id_study_uid_key;
   DROP INDEX IF EXISTS idx_project_data_study_project;
   ```

3. **컬럼 제거**
   ```sql
   ALTER TABLE project_data_study DROP COLUMN IF EXISTS project_id;
   ```

4. **새 테이블 생성**
   ```sql
   CREATE TABLE project_data (...);
   ```

5. **데이터 마이그레이션**
   ```sql
   INSERT INTO project_data (project_id, resource_level, study_id, created_at)
   SELECT backup.project_id, 'STUDY'::resource_level_enum, pds.id, backup.created_at
   FROM _backup_project_data_study_with_project backup
   INNER JOIN project_data_study pds ON pds.study_uid = backup.study_uid;
   ```

## 💻 코드 변경

### 1. 엔티티 (Domain Layer)

#### `pacs-server/src/domain/entities/project_data.rs`

```rust
/// 전역 엔티티 (프로젝트 독립적)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectDataStudy {
    pub id: i32,
    // pub project_id: i32,  // ❌ 제거됨
    pub study_uid: String,
    pub study_description: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub patient_birth_date: Option<NaiveDate>,
    pub study_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // ... 기타 필드
}
```

### 2. Repository (Infrastructure Layer)

#### `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs`

**변경 전**:
```rust
async fn find_study_by_uid(&self, project_id: i32, study_uid: &str) -> Result<Option<ProjectDataStudy>, RepositoryError> {
    let study = sqlx::query_as::<_, ProjectDataStudy>(
        "SELECT * FROM project_data_study 
         WHERE project_id = $1 AND study_uid = $2"
    )
    .bind(project_id)
    .bind(study_uid)
    .fetch_optional(&self.pool)
    .await?;
    Ok(study)
}
```

**변경 후**:
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

### 3. RBAC Evaluator (Infrastructure Layer)

#### `pacs-server/src/infrastructure/services/dicom_rbac_evaluator_impl.rs`

**새로운 RBAC 로직**:

```rust
async fn evaluate_study_access(&self, user_id: i32, project_id: i32, study_id: i32) -> RbacEvaluationResult {
    // 1. 프로젝트 멤버십 확인
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM security_user_project 
         WHERE user_id = $1 AND project_id = $2)"
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_one(&self.pool)
    .await
    .unwrap_or(false);

    if !is_member {
        return RbacEvaluationResult {
            allowed: false,
            reason: Some("not_project_member".to_string()),
        };
    }

    // 2. 명시적 DENIED 체크 (최우선)
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

    // 3. 명시적 APPROVED 체크
    let is_approved: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM project_data_access 
         WHERE user_id = $1 AND project_id = $2 
         AND status = 'APPROVED' AND resource_level = 'STUDY' AND study_id = $3)"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(study_id)
    .fetch_one(&self.pool)
    .await
    .unwrap_or(false);

    if is_approved {
        return RbacEvaluationResult {
            allowed: true,
            reason: Some("explicit_study_approved".to_string()),
        };
    }

    // 4. 기관 기반 접근 체크
    // ... (생략)

    // 5. 룰 기반 조건 평가
    // ... (생략)

    // 6. 기본값: 프로젝트 멤버면 허용
    RbacEvaluationResult {
        allowed: true,
        reason: Some("project_member_default_access".to_string()),
    }
}
```

## 🎯 사용 예시

### 1. Study 전체를 프로젝트에 포함

```sql
-- Study 생성 (전역)
INSERT INTO project_data_study (study_uid, study_description, patient_id)
VALUES ('1.2.3.100', 'Test Study', 'P001');

-- 프로젝트에 Study 매핑
INSERT INTO project_data (project_id, resource_level, study_id)
SELECT 1, 'STUDY', id FROM project_data_study WHERE study_uid = '1.2.3.100';
```

### 2. 특정 Series만 프로젝트에 포함

```sql
-- Series 생성
INSERT INTO project_data_series (study_id, series_uid, series_description)
SELECT id, '1.2.3.100.1', 'CT Series' FROM project_data_study WHERE study_uid = '1.2.3.100';

-- 프로젝트에 Series 매핑
INSERT INTO project_data (project_id, resource_level, study_id, series_id)
SELECT 1, 'SERIES', s.study_id, s.id 
FROM project_data_series s 
WHERE s.series_uid = '1.2.3.100.1';
```

### 3. 사용자별 접근 제어

```sql
-- User 2에게 Study 100 거부
INSERT INTO project_data_access (project_id, user_id, resource_level, study_id, status, project_data_id)
SELECT p.id, u.id, 'STUDY', s.id, 'DENIED', pd.id
FROM security_project p
CROSS JOIN security_user u
CROSS JOIN project_data_study s
INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
WHERE p.id = 1 AND u.username = 'user2' AND s.study_uid = '1.2.3.100';
```

## 🔍 쿼리 패턴

### 프로젝트에 포함된 모든 Study 조회

```sql
SELECT pds.*
FROM project_data_study pds
INNER JOIN project_data pd ON pd.study_id = pds.id
WHERE pd.project_id = $1 AND pd.resource_level = 'STUDY';
```

### 사용자가 접근 가능한 Study 조회

```rust
let studies = sqlx::query_as::<_, ProjectDataStudy>(
    "SELECT DISTINCT pds.*
     FROM project_data_study pds
     INNER JOIN project_data pd ON pd.study_id = pds.id
     INNER JOIN security_user_project sup ON sup.project_id = pd.project_id
     LEFT JOIN project_data_access pda ON pda.project_id = pd.project_id 
         AND pda.user_id = sup.user_id 
         AND pda.study_id = pds.id 
         AND pda.resource_level = 'STUDY'
     WHERE sup.user_id = $1 
       AND pd.project_id = $2
       AND (pda.status IS NULL OR pda.status != 'DENIED')"
)
.bind(user_id)
.bind(project_id)
.fetch_all(&pool)
.await?;
```

## 📊 성능 고려사항

### 인덱스 전략
- `project_data(project_id)`: 프로젝트별 데이터 조회
- `project_data(study_id)`: Study별 매핑 조회
- `project_data(project_id, resource_level)`: 프로젝트별 리소스 레벨 필터링
- `project_data_study(study_uid)`: Study UID 조회

### 쿼리 최적화
- JOIN 최소화: 필요한 경우에만 `project_data` 테이블과 JOIN
- 인덱스 활용: WHERE 절에 인덱스 컬럼 사용
- 부분 인덱스: `WHERE study_id IS NOT NULL` 등

## ⚠️ 주의사항

### 1. 외래 키 제약 조건
- `project_data_access.project_data_id`는 `project_data.id`를 참조
- 마이그레이션 후 외래 키 업데이트 필요

### 2. 데이터 일관성
- `resource_level`에 따라 `study_id`, `series_id`, `instance_id` 설정 필수
- CHECK 제약 조건으로 일관성 보장

### 3. 롤백 계획
- 백업 테이블 (`_backup_project_data_study_with_project`) 유지
- 롤백 시 백업에서 복원 가능

## 🚀 향후 개선 방향

1. **성능 최적화**
   - 대량 데이터 조회 시 페이지네이션
   - 캐싱 전략 (Redis)

2. **기능 확장**
   - PATIENT 레벨 리소스 매핑
   - MODALITY 레벨 리소스 매핑

3. **모니터링**
   - 쿼리 성능 모니터링
   - RBAC 평가 시간 측정

## 📚 참고 자료

- [DICOM Standard](https://www.dicomstandard.org/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Actix Web Documentation](https://actix.rs/)

