# PACS Server Repository 작성 가이드

## 📋 목차
1. [Repository 개요](#repository-개요)
2. [아키텍처 패턴](#아키텍처-패턴)
3. [Domain Repository 패턴](#domain-repository-패턴)
4. [Infrastructure Repository 패턴](#infrastructure-repository-패턴)
5. [공통 구조와 문법](#공통-구조와-문법)
6. [SQL 쿼리 패턴](#sql-쿼리-패턴)
7. [에러 처리 패턴](#에러-처리-패턴)
8. [트랜잭션 처리](#트랜잭션-처리)
9. [실제 예제 분석](#실제-예제-분석)
10. [연습 문제](#연습-문제)
11. [체크리스트](#체크리스트)

---

## Repository 개요

### Repository 패턴이란?
**Repository Pattern** - 데이터 접근 로직을 추상화하여 도메인 로직과 데이터 저장소를 분리하는 디자인 패턴입니다.

### 프로젝트에서의 역할
- **Domain Layer**: 데이터 접근 인터페이스 정의 (Trait)
- **Infrastructure Layer**: 실제 데이터베이스 구현체 (PostgreSQL)
- **의존성 역전**: 도메인이 인프라에 의존하지 않도록 함

---

## 아키텍처 패턴

### 1. 계층 구조
```
Domain Layer (Trait)
    ↓ (의존)
Infrastructure Layer (구현체)
    ↓ (사용)
Database (PostgreSQL)
```

### 2. 파일 구조
```
src/
├── domain/repositories/           # 인터페이스 정의
│   ├── user_repository.rs
│   ├── project_repository.rs
│   └── mod.rs
└── infrastructure/repositories/   # 구현체
    ├── user_repository_impl.rs
    ├── project_repository_impl.rs
    └── mod.rs
```

---

## Domain Repository 패턴

### 1. 기본 구조
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Entity, NewEntity};

#[async_trait]
pub trait EntityRepository: Send + Sync {
    // CRUD 기본 메서드
    async fn find_by_id(&self, id: i32) -> Result<Option<Entity>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Entity>, sqlx::Error>;
    async fn create(&self, new_entity: NewEntity) -> Result<Entity, sqlx::Error>;
    async fn update(&self, id: i32, new_entity: NewEntity) -> Result<Option<Entity>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    
    // 공통 메서드
    fn pool(&self) -> &PgPool;
}
```

### 2. 필수 Import 패턴
```rust
use async_trait::async_trait;  // 비동기 트레이트
use sqlx::PgPool;              // PostgreSQL 연결 풀
use crate::domain::entities::{Entity, NewEntity};  // 도메인 엔티티
```

### 3. 트레이트 제약조건
```rust
pub trait EntityRepository: Send + Sync {
    // Send: 스레드 간 이동 가능
    // Sync: 여러 스레드에서 동시 접근 가능
}
```

---

## Infrastructure Repository 패턴

### 1. 기본 구조
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Entity, NewEntity};
use crate::domain::repositories::EntityRepository;

#[derive(Clone)]
pub struct EntityRepositoryImpl {
    pool: PgPool,
}

impl EntityRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EntityRepository for EntityRepositoryImpl {
    // 구현...
}
```

### 2. 구조체 패턴
- `#[derive(Clone)]`: 연결 풀 공유를 위한 클론 가능
- `pool: PgPool`: PostgreSQL 연결 풀
- `new()`: 생성자 함수

---

## 공통 구조와 문법

### 1. 메서드 네이밍 컨벤션

#### 조회 메서드
```rust
async fn find_by_id(&self, id: i32) -> Result<Option<Entity>, sqlx::Error>;
async fn find_by_name(&self, name: &str) -> Result<Option<Entity>, sqlx::Error>;
async fn find_by_email(&self, email: &str) -> Result<Option<Entity>, sqlx::Error>;
async fn find_all(&self) -> Result<Vec<Entity>, sqlx::Error>;
async fn find_active(&self) -> Result<Vec<Entity>, sqlx::Error>;
```

#### 생성/수정/삭제 메서드
```rust
async fn create(&self, new_entity: NewEntity) -> Result<Entity, sqlx::Error>;
async fn update(&self, id: i32, new_entity: NewEntity) -> Result<Option<Entity>, sqlx::Error>;
async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
```

#### 특수 메서드
```rust
async fn set_active(&self, id: i32, is_active: bool) -> Result<bool, sqlx::Error>;
async fn count(&self, filters: FilterOptions) -> Result<i64, sqlx::Error>;
```

### 2. 반환 타입 패턴

#### 단일 엔티티
```rust
Result<Option<Entity>, sqlx::Error>  // 조회 (없을 수 있음)
Result<Entity, sqlx::Error>          // 생성 (반드시 있음)
```

#### 복수 엔티티
```rust
Result<Vec<Entity>, sqlx::Error>     // 목록 조회
```

#### 불린 값
```rust
Result<bool, sqlx::Error>            // 삭제/업데이트 성공 여부
```

---

## SQL 쿼리 패턴

### 1. 기본 조회 패턴
```rust
async fn find_by_id(&self, id: i32) -> Result<Option<Entity>, sqlx::Error> {
    sqlx::query_as::<_, Entity>(
        "SELECT id, name, email, created_at
         FROM table_name
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}
```

### 2. 목록 조회 패턴
```rust
async fn find_all(&self) -> Result<Vec<Entity>, sqlx::Error> {
    sqlx::query_as::<_, Entity>(
        "SELECT id, name, email, created_at
         FROM table_name
         ORDER BY created_at DESC"
    )
    .fetch_all(&self.pool)
    .await
}
```

### 3. 생성 패턴
```rust
async fn create(&self, new_entity: NewEntity) -> Result<Entity, sqlx::Error> {
    sqlx::query_as::<_, Entity>(
        "INSERT INTO table_name (name, email)
         VALUES ($1, $2)
         RETURNING id, name, email, created_at"
    )
    .bind(new_entity.name)
    .bind(new_entity.email)
    .fetch_one(&self.pool)
    .await
}
```

### 4. 업데이트 패턴
```rust
async fn update(&self, id: i32, new_entity: NewEntity) -> Result<Option<Entity>, sqlx::Error> {
    sqlx::query_as::<_, Entity>(
        "UPDATE table_name
         SET name = $2, email = $3
         WHERE id = $1
         RETURNING id, name, email, created_at"
    )
    .bind(id)
    .bind(new_entity.name)
    .bind(new_entity.email)
    .fetch_optional(&self.pool)
    .await
}
```

### 5. 삭제 패턴
```rust
async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM table_name WHERE id = $1")
        .bind(id)
        .execute(&self.pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
```

---

## 에러 처리 패턴

### 1. 기본 에러 처리
```rust
// SQLx 에러를 그대로 반환
async fn find_by_id(&self, id: i32) -> Result<Option<Entity>, sqlx::Error> {
    // ...
}
```

### 2. 커스텀 에러 처리
```rust
use crate::domain::ServiceError;

async fn create(&self, new_entity: NewEntity) -> Result<Entity, ServiceError> {
    sqlx::query_as::<_, Entity>(/* ... */)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create entity: {}", e)))
}
```

### 3. 에러 변환 패턴
```rust
.map_err(|e| ServiceError::DatabaseError(format!("Failed to create entity: {}", e)))
.map_err(|e| ServiceError::NotFound(format!("Entity not found: {}", e)))
.map_err(|e| ServiceError::ValidationError(format!("Invalid data: {}", e)))
```

---

## 트랜잭션 처리

### 1. 기본 트랜잭션 패턴
```rust
async fn create_with_history(&self, new_entity: NewEntity) -> Result<Entity, sqlx::Error> {
    let mut tx = self.pool.begin().await?;

    // 엔티티 생성
    let entity = sqlx::query_as::<_, Entity>(/* ... */)
        .fetch_one(&mut *tx)
        .await?;

    // 히스토리 생성
    let _ = sqlx::query_as::<_, History>(/* ... */)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(entity)
}
```

### 2. 트랜잭션 롤백 패턴
```rust
async fn complex_operation(&self) -> Result<(), sqlx::Error> {
    let mut tx = self.pool.begin().await?;

    match self.perform_operation(&mut *tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}
```

---

## 실제 예제 분석

### 1. 사용자 레포지토리 (user_repository.rs)

#### Domain Interface
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
```

**패턴 분석:**
- ✅ 표준 CRUD 메서드
- ✅ 다양한 조회 조건 (id, keycloak_id, username, email)
- ✅ 공통 pool() 메서드

#### Infrastructure Implementation
```rust
#[derive(Clone)]
pub struct UserRepositoryImpl {
    pool: PgPool,
}

impl UserRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, created_at
             FROM security_user
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
    // ... 다른 메서드들
}
```

**패턴 분석:**
- ✅ 표준 구조체 패턴
- ✅ SQLx 쿼리 사용
- ✅ 적절한 바인딩과 fetch 메서드

### 2. 어노테이션 레포지토리 (annotation_repository.rs)

#### 복잡한 조회 메서드
```rust
async fn find_by_project_and_study(&self, project_id: i32, study_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
async fn find_shared_annotations(&self, project_id: i32) -> Result<Vec<Annotation>, sqlx::Error>;
```

#### 트랜잭션 처리
```rust
async fn create(&self, new_annotation: NewAnnotation) -> Result<Annotation, sqlx::Error> {
    let mut tx = self.pool.begin().await?;

    // annotation 생성
    let annotation = sqlx::query_as::<_, Annotation>(/* ... */)
        .fetch_one(&mut *tx)
        .await?;

    // history 생성
    let _ = sqlx::query_as::<_, AnnotationHistory>(/* ... */)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(annotation)
}
```

**패턴 분석:**
- ✅ 복잡한 비즈니스 로직
- ✅ 트랜잭션을 통한 데이터 일관성
- ✅ 히스토리 추적

### 3. 마스크 레포지토리 (mask_repository.rs)

#### 고급 쿼리 패턴
```rust
async fn list(
    &self,
    mask_group_id: Option<i32>,
    sop_instance_uid: Option<String>,
    label_name: Option<String>,
    mime_type: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Mask>, ServiceError> {
    let results = sqlx::query!(
        r#"
        SELECT id, mask_group_id, slice_index, sop_instance_uid, label_name,
               file_path, mime_type, file_size, checksum, width, height, created_at, updated_at
        FROM annotation_mask
        WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
          AND ($2::TEXT IS NULL OR sop_instance_uid = $2)
          AND ($3::TEXT IS NULL OR label_name = $3)
          AND ($4::TEXT IS NULL OR mime_type = $4)
        ORDER BY slice_index ASC, created_at ASC
        OFFSET COALESCE($5, 0)
        LIMIT COALESCE($6, 50)
        "#,
        mask_group_id,
        sop_instance_uid,
        label_name,
        mime_type,
        offset.unwrap_or(0) as i32,
        limit.unwrap_or(50) as i32
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| ServiceError::DatabaseError(format!("Failed to list masks: {}", e)))?;
    // ...
}
```

**패턴 분석:**
- ✅ 동적 필터링 (NULL 체크)
- ✅ 페이지네이션 (OFFSET, LIMIT)
- ✅ 정렬 (ORDER BY)
- ✅ 커스텀 에러 처리

---

## 연습 문제

### 문제 1: 기본 레포지토리 작성
다음 요구사항에 맞는 레포지토리를 작성하세요:

**요구사항:**
- 병원 정보를 관리하는 레포지토리
- 필드: id, name, address, phone, created_at
- 메서드: find_by_id, find_by_name, find_all, create, update, delete

<details>
<summary>정답 보기</summary>

**Domain Repository (hospital_repository.rs)**
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Hospital, NewHospital};

#[async_trait]
pub trait HospitalRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Hospital>, sqlx::Error>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Hospital>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Hospital>, sqlx::Error>;
    async fn create(&self, new_hospital: NewHospital) -> Result<Hospital, sqlx::Error>;
    async fn update(&self, id: i32, new_hospital: NewHospital) -> Result<Option<Hospital>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
```

**Infrastructure Repository (hospital_repository_impl.rs)**
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Hospital, NewHospital};
use crate::domain::repositories::HospitalRepository;

#[derive(Clone)]
pub struct HospitalRepositoryImpl {
    pool: PgPool,
}

impl HospitalRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HospitalRepository for HospitalRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Hospital>, sqlx::Error> {
        sqlx::query_as::<_, Hospital>(
            "SELECT id, name, address, phone, created_at
             FROM hospitals
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Hospital>, sqlx::Error> {
        sqlx::query_as::<_, Hospital>(
            "SELECT id, name, address, phone, created_at
             FROM hospitals
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Hospital>, sqlx::Error> {
        sqlx::query_as::<_, Hospital>(
            "SELECT id, name, address, phone, created_at
             FROM hospitals
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_hospital: NewHospital) -> Result<Hospital, sqlx::Error> {
        sqlx::query_as::<_, Hospital>(
            "INSERT INTO hospitals (name, address, phone)
             VALUES ($1, $2, $3)
             RETURNING id, name, address, phone, created_at"
        )
        .bind(new_hospital.name)
        .bind(new_hospital.address)
        .bind(new_hospital.phone)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, id: i32, new_hospital: NewHospital) -> Result<Option<Hospital>, sqlx::Error> {
        sqlx::query_as::<_, Hospital>(
            "UPDATE hospitals
             SET name = $2, address = $3, phone = $4
             WHERE id = $1
             RETURNING id, name, address, phone, created_at"
        )
        .bind(id)
        .bind(new_hospital.name)
        .bind(new_hospital.address)
        .bind(new_hospital.phone)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM hospitals WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

</details>

### 문제 2: 복잡한 조회 메서드 작성
다음 요구사항에 맞는 조회 메서드를 작성하세요:

**요구사항:**
- 의료진 검색 레포지토리
- 필터: department, specialty, min_experience_years
- 정렬: name, experience_years
- 페이지네이션: offset, limit

<details>
<summary>정답 보기</summary>

```rust
// Domain Repository에 추가할 메서드
async fn search_doctors(
    &self,
    department: Option<String>,
    specialty: Option<String>,
    min_experience_years: Option<i32>,
    sort_by: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Doctor>, sqlx::Error>;

async fn count_doctors(
    &self,
    department: Option<String>,
    specialty: Option<String>,
    min_experience_years: Option<i32>,
) -> Result<i64, sqlx::Error>;

// Infrastructure Repository 구현
async fn search_doctors(
    &self,
    department: Option<String>,
    specialty: Option<String>,
    min_experience_years: Option<i32>,
    sort_by: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Doctor>, sqlx::Error> {
    let order_clause = match sort_by.as_deref() {
        Some("experience_years") => "ORDER BY experience_years DESC, name ASC",
        Some("name") => "ORDER BY name ASC, experience_years DESC",
        _ => "ORDER BY created_at DESC",
    };

    sqlx::query_as::<_, Doctor>(
        &format!(
            r#"
            SELECT id, name, department, specialty, experience_years, license_number, created_at
            FROM doctors
            WHERE ($1::TEXT IS NULL OR department = $1)
              AND ($2::TEXT IS NULL OR specialty = $2)
              AND ($3::INTEGER IS NULL OR experience_years >= $3)
            {}
            OFFSET COALESCE($4, 0)
            LIMIT COALESCE($5, 50)
            "#,
            order_clause
        )
    )
    .bind(department)
    .bind(specialty)
    .bind(min_experience_years)
    .bind(offset.unwrap_or(0) as i32)
    .bind(limit.unwrap_or(50) as i32)
    .fetch_all(&self.pool)
    .await
}

async fn count_doctors(
    &self,
    department: Option<String>,
    specialty: Option<String>,
    min_experience_years: Option<i32>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM doctors
        WHERE ($1::TEXT IS NULL OR department = $1)
          AND ($2::TEXT IS NULL OR specialty = $2)
          AND ($3::INTEGER IS NULL OR experience_years >= $3)
        "#,
        department,
        specialty,
        min_experience_years
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(result.unwrap_or(0))
}
```

</details>

### 문제 3: 트랜잭션 처리 작성
다음 요구사항에 맞는 트랜잭션 처리를 작성하세요:

**요구사항:**
- 환자 등록 시 자동으로 기본 진료 기록 생성
- 두 작업이 모두 성공해야 함 (원자성)

<details>
<summary>정답 보기</summary>

```rust
// Domain Repository에 추가할 메서드
async fn create_patient_with_record(
    &self,
    new_patient: NewPatient,
    initial_record: NewMedicalRecord,
) -> Result<(Patient, MedicalRecord), sqlx::Error>;

// Infrastructure Repository 구현
async fn create_patient_with_record(
    &self,
    new_patient: NewPatient,
    initial_record: NewMedicalRecord,
) -> Result<(Patient, MedicalRecord), sqlx::Error> {
    let mut tx = self.pool.begin().await?;

    // 환자 생성
    let patient = sqlx::query_as::<_, Patient>(
        "INSERT INTO patients (name, birth_date, gender, phone)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, birth_date, gender, phone, created_at"
    )
    .bind(new_patient.name)
    .bind(new_patient.birth_date)
    .bind(new_patient.gender)
    .bind(new_patient.phone)
    .fetch_one(&mut *tx)
    .await?;

    // 진료 기록 생성 (환자 ID 사용)
    let medical_record = sqlx::query_as::<_, MedicalRecord>(
        "INSERT INTO medical_records (patient_id, diagnosis, treatment, notes)
         VALUES ($1, $2, $3, $4)
         RETURNING id, patient_id, diagnosis, treatment, notes, created_at"
    )
    .bind(patient.id)
    .bind(initial_record.diagnosis)
    .bind(initial_record.treatment)
    .bind(initial_record.notes)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((patient, medical_record))
}
```

</details>

### 문제 4: 통계 쿼리 작성
다음 요구사항에 맞는 통계 쿼리를 작성하세요:

**요구사항:**
- 병원별 환자 수 통계
- 진료과별 환자 분포
- 월별 환자 등록 추이

<details>
<summary>정답 보기</summary>

```rust
// Domain Repository에 추가할 메서드
async fn get_patient_stats(&self) -> Result<PatientStats, sqlx::Error>;

// 통계 구조체
#[derive(Debug, Serialize)]
pub struct PatientStats {
    pub total_patients: i64,
    pub patients_by_hospital: HashMap<String, i64>,
    pub patients_by_department: HashMap<String, i64>,
    pub monthly_registrations: Vec<MonthlyStats>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyStats {
    pub year: i32,
    pub month: i32,
    pub count: i64,
}

// Infrastructure Repository 구현
async fn get_patient_stats(&self) -> Result<PatientStats, sqlx::Error> {
    // 전체 환자 수
    let total_patients = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM patients"
    )
    .fetch_one(&self.pool)
    .await?
    .unwrap_or(0);

    // 병원별 환자 수
    let hospital_stats = sqlx::query!(
        "SELECT h.name, COUNT(p.id) as count
         FROM hospitals h
         LEFT JOIN patients p ON h.id = p.hospital_id
         GROUP BY h.id, h.name
         ORDER BY count DESC"
    )
    .fetch_all(&self.pool)
    .await?;

    let mut patients_by_hospital = HashMap::new();
    for row in hospital_stats {
        patients_by_hospital.insert(row.name, row.count.unwrap_or(0));
    }

    // 진료과별 환자 수
    let department_stats = sqlx::query!(
        "SELECT department, COUNT(*) as count
         FROM patients
         WHERE department IS NOT NULL
         GROUP BY department
         ORDER BY count DESC"
    )
    .fetch_all(&self.pool)
    .await?;

    let mut patients_by_department = HashMap::new();
    for row in department_stats {
        patients_by_department.insert(row.department, row.count.unwrap_or(0));
    }

    // 월별 등록 추이 (최근 12개월)
    let monthly_stats = sqlx::query_as::<_, MonthlyStats>(
        "SELECT 
            EXTRACT(YEAR FROM created_at) as year,
            EXTRACT(MONTH FROM created_at) as month,
            COUNT(*) as count
         FROM patients
         WHERE created_at >= NOW() - INTERVAL '12 months'
         GROUP BY EXTRACT(YEAR FROM created_at), EXTRACT(MONTH FROM created_at)
         ORDER BY year, month"
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(PatientStats {
        total_patients,
        patients_by_hospital,
        patients_by_department,
        monthly_registrations: monthly_stats,
    })
}
```

</details>

---

## 체크리스트

### Repository 작성 전 체크리스트
- [ ] 도메인 엔티티가 정의되었는가?
- [ ] 데이터베이스 스키마가 확인되었는가?
- [ ] 필요한 쿼리가 명확한가?

### Domain Repository 작성 체크리스트
- [ ] async_trait 사용
- [ ] Send + Sync 제약조건
- [ ] 적절한 메서드명 사용
- [ ] 표준 CRUD 메서드 포함
- [ ] pool() 메서드 포함

### Infrastructure Repository 작성 체크리스트
- [ ] Clone derive 사용
- [ ] new() 생성자 구현
- [ ] 모든 도메인 메서드 구현
- [ ] 적절한 SQL 쿼리 작성
- [ ] 에러 처리 구현

### SQL 쿼리 작성 체크리스트
- [ ] 파라미터 바인딩 사용 ($1, $2, ...)
- [ ] 적절한 fetch 메서드 사용
- [ ] ORDER BY 절 포함 (목록 조회 시)
- [ ] 인덱스 활용 고려

### 트랜잭션 처리 체크리스트
- [ ] begin()으로 트랜잭션 시작
- [ ] commit()으로 커밋
- [ ] 에러 시 rollback() 고려
- [ ] 원자성 보장

---

## 추가 학습 자료

### 관련 문서
- [SQLx 공식 문서](https://docs.rs/sqlx/latest/sqlx/)
- [Async-trait 공식 문서](https://docs.rs/async-trait/latest/async_trait/)
- [PostgreSQL 공식 문서](https://www.postgresql.org/docs/)

### 프로젝트 내 관련 파일
- `src/domain/repositories/` - 도메인 인터페이스
- `src/infrastructure/repositories/` - 구현체
- `src/domain/entities/` - 도메인 엔티티

---

## 마무리

이 가이드를 통해 PACS Server 프로젝트의 Repository 작성 패턴을 익혔습니다.
실제 개발 시에는 이 패턴을 참고하여 일관성 있는 Repository를 작성하고,
복잡한 비즈니스 로직에서는 트랜잭션을 적절히 활용하여 데이터 일관성을 보장하세요.

**핵심 포인트:**
1. **분리**: Domain과 Infrastructure 계층 분리
2. **일관성**: 표준 네이밍과 구조 사용
3. **안전성**: 트랜잭션과 에러 처리
4. **성능**: 적절한 쿼리와 인덱스 활용
