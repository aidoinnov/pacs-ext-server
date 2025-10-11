# PACS Server Service 작성 가이드

## 📋 목차
1. [Service 개요](#service-개요)
2. [아키텍처 패턴](#아키텍처-패턴)
3. [Domain Service 패턴](#domain-service-패턴)
4. [Application Service 패턴](#application-service-패턴)
5. [공통 구조와 문법](#공통-구조와-문법)
6. [비즈니스 로직 패턴](#비즈니스-로직-패턴)
7. [에러 처리 패턴](#에러-처리-패턴)
8. [의존성 주입 패턴](#의존성-주입-패턴)
9. [실제 예제 분석](#실제-예제-분석)
10. [연습 문제](#연습-문제)
11. [체크리스트](#체크리스트)

---

## Service 개요

### Service 패턴이란?
**Service Pattern** - 비즈니스 로직을 캡슐화하고 도메인 규칙을 구현하는 계층입니다.

### 프로젝트에서의 역할
- **Domain Services**: 핵심 비즈니스 로직과 도메인 규칙 구현
- **Application Services**: 외부 시스템과의 통합 및 애플리케이션 흐름 제어
- **의존성 주입**: Repository와 외부 서비스를 조합하여 복잡한 비즈니스 로직 구현

---

## 아키텍처 패턴

### 1. 계층 구조
```
Presentation Layer (Controllers)
    ↓ (의존)
Application Layer (Services)
    ↓ (의존)
Domain Layer (Services + Repositories)
    ↓ (의존)
Infrastructure Layer (Repository Implementations)
```

### 2. 파일 구조
```
src/
├── domain/services/           # 도메인 서비스
│   ├── user_service.rs
│   ├── project_service.rs
│   └── mod.rs
└── application/services/      # 애플리케이션 서비스
    ├── object_storage_service.rs
    └── mod.rs
```

---

## Domain Service 패턴

### 1. 기본 구조
```rust
use async_trait::async_trait;
use crate::domain::entities::{Entity, NewEntity};
use crate::domain::repositories::{EntityRepository, RelatedRepository};
use crate::domain::ServiceError;

/// [기능] 도메인 서비스
#[async_trait]
pub trait EntityService: Send + Sync {
    /// [기능] 메서드
    async fn create_entity(&self, data: NewEntity) -> Result<Entity, ServiceError>;
    
    /// [기능] 메서드
    async fn get_entity(&self, id: i32) -> Result<Entity, ServiceError>;
    
    /// [기능] 메서드
    async fn update_entity(&self, id: i32, data: NewEntity) -> Result<Entity, ServiceError>;
    
    /// [기능] 메서드
    async fn delete_entity(&self, id: i32) -> Result<(), ServiceError>;
}

pub struct EntityServiceImpl<R1, R2>
where
    R1: EntityRepository,
    R2: RelatedRepository,
{
    entity_repository: R1,
    related_repository: R2,
}

impl<R1, R2> EntityServiceImpl<R1, R2>
where
    R1: EntityRepository,
    R2: RelatedRepository,
{
    pub fn new(entity_repository: R1, related_repository: R2) -> Self {
        Self {
            entity_repository,
            related_repository,
        }
    }
}

#[async_trait]
impl<R1, R2> EntityService for EntityServiceImpl<R1, R2>
where
    R1: EntityRepository,
    R2: RelatedRepository,
{
    // 구현...
}
```

### 2. 필수 Import 패턴
```rust
use async_trait::async_trait;  // 비동기 트레이트
use crate::domain::entities::{Entity, NewEntity};  // 도메인 엔티티
use crate::domain::repositories::{EntityRepository, RelatedRepository};  // 레포지토리
use crate::domain::ServiceError;  // 서비스 에러
```

---

## Application Service 패턴

### 1. 기본 구조
```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// [기능] 애플리케이션 서비스 에러
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// [기능] 애플리케이션 서비스
#[async_trait]
pub trait ApplicationService: Send + Sync {
    /// [기능] 메서드
    async fn perform_action(&self, data: ActionData) -> Result<ActionResult, ServiceError>;
}

/// [기능] 서비스 구현체
pub struct ApplicationServiceImpl {
    // 외부 서비스 의존성
}

impl ApplicationServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ApplicationService for ApplicationServiceImpl {
    // 구현...
}
```

### 2. 팩토리 패턴
```rust
/// 서비스 팩토리
pub struct ServiceFactory;

impl ServiceFactory {
    pub async fn create_service(
        provider: &str,
        config: ServiceConfig,
    ) -> Result<Box<dyn ApplicationService>, ServiceError> {
        match provider {
            "provider1" => Ok(Box::new(Provider1Service::new(config).await?)),
            "provider2" => Ok(Box::new(Provider2Service::new(config).await?)),
            _ => Err(ServiceError::ConfigError("Unsupported provider".into())),
        }
    }
}
```

---

## 공통 구조와 문법

### 1. 메서드 네이밍 컨벤션

#### CRUD 메서드
```rust
async fn create_[entity](&self, data: NewEntity) -> Result<Entity, ServiceError>;
async fn get_[entity](&self, id: i32) -> Result<Entity, ServiceError>;
async fn get_[entity]_by_[field](&self, field: FieldType) -> Result<Entity, ServiceError>;
async fn update_[entity](&self, id: i32, data: UpdateEntity) -> Result<Entity, ServiceError>;
async fn delete_[entity](&self, id: i32) -> Result<(), ServiceError>;
```

#### 비즈니스 메서드
```rust
async fn [action]_[entity](&self, id: i32) -> Result<Entity, ServiceError>;
async fn [check]_[condition](&self, params: Params) -> Result<bool, ServiceError>;
async fn [get]_[entity]_[relationship](&self, id: i32) -> Result<Vec<RelatedEntity>, ServiceError>;
```

### 2. 반환 타입 패턴

#### 단일 엔티티
```rust
Result<Entity, ServiceError>           // 조회/생성/수정
Result<Option<Entity>, ServiceError>   // 선택적 조회
```

#### 복수 엔티티
```rust
Result<Vec<Entity>, ServiceError>      // 목록 조회
```

#### 불린 값
```rust
Result<bool, ServiceError>             // 조건 확인
Result<(), ServiceError>               // 작업 완료
```

### 3. 제네릭 타입 제약조건
```rust
pub struct ServiceImpl<R1, R2>
where
    R1: Repository + Send + Sync,
    R2: RelatedRepository + Send + Sync,
{
    repository: R1,
    related_repository: R2,
}
```

---

## 비즈니스 로직 패턴

### 1. 검증 패턴
```rust
async fn create_entity(&self, data: NewEntity) -> Result<Entity, ServiceError> {
    // 중복 체크
    if let Some(_) = self.repository.find_by_field(&data.field).await? {
        return Err(ServiceError::AlreadyExists("Entity already exists".into()));
    }
    
    // 비즈니스 규칙 검증
    if !self.validate_business_rule(&data) {
        return Err(ServiceError::ValidationError("Business rule violation".into()));
    }
    
    // 생성
    Ok(self.repository.create(data).await?)
}
```

### 2. 권한 확인 패턴
```rust
async fn access_entity(&self, user_id: i32, entity_id: i32) -> Result<Entity, ServiceError> {
    // 엔티티 존재 확인
    let entity = self.get_entity(entity_id).await?;
    
    // 권한 확인
    if !self.has_permission(user_id, &entity).await? {
        return Err(ServiceError::Unauthorized("Access denied".into()));
    }
    
    Ok(entity)
}
```

### 3. 트랜잭션 패턴
```rust
async fn complex_operation(&self, data: ComplexData) -> Result<ComplexResult, ServiceError> {
    let mut tx = self.repository.pool().begin().await?;
    
    // 첫 번째 작업
    let result1 = self.perform_first_operation(&mut *tx, &data).await?;
    
    // 두 번째 작업
    let result2 = self.perform_second_operation(&mut *tx, &result1).await?;
    
    tx.commit().await?;
    Ok(ComplexResult { result1, result2 })
}
```

### 4. UPSERT 패턴
```rust
async fn upsert_entity(&self, data: UpsertData) -> Result<Entity, ServiceError> {
    let entity = sqlx::query_as::<_, Entity>(
        "INSERT INTO table (field1, field2)
         VALUES ($1, $2)
         ON CONFLICT (field1) DO UPDATE
         SET field2 = EXCLUDED.field2
         RETURNING id, field1, field2, created_at"
    )
    .bind(data.field1)
    .bind(data.field2)
    .fetch_one(self.repository.pool())
    .await?;
    
    Ok(entity)
}
```

---

## 에러 처리 패턴

### 1. ServiceError 정의
```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}
```

### 2. 에러 변환 패턴
```rust
// Repository 에러를 Service 에러로 변환
.map_err(|e| ServiceError::DatabaseError(e.to_string()))

// Option을 ServiceError로 변환
.ok_or(ServiceError::NotFound("Entity not found".into()))

// 조건부 에러
if condition {
    return Err(ServiceError::ValidationError("Invalid data".into()));
}
```

### 3. 에러 체이닝
```rust
async fn complex_operation(&self, data: Data) -> Result<Result, ServiceError> {
    let entity = self.repository
        .find_by_id(data.id)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to find entity: {}", e)))?
        .ok_or(ServiceError::NotFound("Entity not found".into()))?;
    
    // 추가 처리...
    Ok(result)
}
```

---

## 의존성 주입 패턴

### 1. 생성자 주입
```rust
pub struct ServiceImpl<R1, R2> {
    repository: R1,
    related_repository: R2,
}

impl<R1, R2> ServiceImpl<R1, R2>
where
    R1: Repository + Send + Sync,
    R2: RelatedRepository + Send + Sync,
{
    pub fn new(repository: R1, related_repository: R2) -> Self {
        Self {
            repository,
            related_repository,
        }
    }
}
```

### 2. 팩토리 패턴
```rust
pub struct ServiceFactory;

impl ServiceFactory {
    pub async fn create_user_service(
        user_repo: impl UserRepository + Send + Sync,
        project_repo: impl ProjectRepository + Send + Sync,
    ) -> impl UserService {
        UserServiceImpl::new(user_repo, project_repo)
    }
}
```

### 3. 빌더 패턴
```rust
pub struct ServiceBuilder<R1, R2> {
    repository: Option<R1>,
    related_repository: Option<R2>,
}

impl<R1, R2> ServiceBuilder<R1, R2> {
    pub fn new() -> Self {
        Self {
            repository: None,
            related_repository: None,
        }
    }
    
    pub fn repository(mut self, repository: R1) -> Self {
        self.repository = Some(repository);
        self
    }
    
    pub fn related_repository(mut self, related_repository: R2) -> Self {
        self.related_repository = Some(related_repository);
        self
    }
    
    pub fn build(self) -> Result<ServiceImpl<R1, R2>, ServiceError> {
        Ok(ServiceImpl {
            repository: self.repository.ok_or(ServiceError::ConfigError("Repository required".into()))?,
            related_repository: self.related_repository.ok_or(ServiceError::ConfigError("Related repository required".into()))?,
        })
    }
}
```

---

## 실제 예제 분석

### 1. 사용자 서비스 (user_service.rs)

#### Domain Service Interface
```rust
#[async_trait]
pub trait UserService: Send + Sync {
    async fn create_user(&self, username: String, email: String, keycloak_id: Uuid) -> Result<User, ServiceError>;
    async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError>;
    async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<User, ServiceError>;
    async fn delete_user(&self, id: i32) -> Result<(), ServiceError>;
    async fn user_exists(&self, keycloak_id: Uuid) -> Result<bool, ServiceError>;
    
    // 프로젝트 멤버십 관리
    async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;
    async fn remove_user_from_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;
    async fn get_user_projects(&self, user_id: i32) -> Result<Vec<Project>, ServiceError>;
    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError>;
}
```

**패턴 분석:**
- ✅ 표준 CRUD 메서드
- ✅ 비즈니스 메서드 (프로젝트 멤버십)
- ✅ 명확한 네이밍 컨벤션

#### Implementation with Validation
```rust
async fn create_user(&self, username: String, email: String, keycloak_id: Uuid) -> Result<User, ServiceError> {
    // 중복 체크
    if let Some(_) = self.user_repository.find_by_keycloak_id(keycloak_id).await? {
        return Err(ServiceError::AlreadyExists("User with this keycloak_id already exists".into()));
    }

    if let Some(_) = self.user_repository.find_by_username(&username).await? {
        return Err(ServiceError::AlreadyExists("Username already taken".into()));
    }

    // 이메일 형식 검증
    if !email.contains('@') {
        return Err(ServiceError::ValidationError("Invalid email format".into()));
    }

    let new_user = NewUser {
        keycloak_id,
        username,
        email,
    };

    Ok(self.user_repository.create(new_user).await?)
}
```

**패턴 분석:**
- ✅ 비즈니스 규칙 검증
- ✅ 중복 체크
- ✅ 적절한 에러 처리

### 2. 인증 서비스 (auth_service.rs)

#### JWT 기반 인증
```rust
async fn login(&self, keycloak_id: Uuid, username: String, email: String) -> Result<AuthResponse, ServiceError> {
    // UPSERT 패턴으로 동시 로그인 Race condition 방지
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO security_user (keycloak_id, username, email)
         VALUES ($1, $2, $3)
         ON CONFLICT (keycloak_id) DO UPDATE
         SET username = EXCLUDED.username,
             email = EXCLUDED.email
         RETURNING id, keycloak_id, username, email, created_at"
    )
    .bind(keycloak_id)
    .bind(&username)
    .bind(&email)
    .fetch_one(self.user_repository.pool())
    .await?;

    // JWT 토큰 생성
    let claims = Claims::new(
        user.id,
        user.keycloak_id,
        user.username.clone(),
        user.email.clone(),
        24, // 24시간 유효
    );

    let token = self.jwt_service
        .create_token(&claims)
        .map_err(|e| ServiceError::Unauthorized(format!("Failed to create token: {}", e)))?;

    Ok(AuthResponse { user, token })
}
```

**패턴 분석:**
- ✅ UPSERT 패턴으로 Race condition 방지
- ✅ 외부 서비스 통합 (JWT)
- ✅ 에러 변환

### 3. Object Storage 서비스 (object_storage_service.rs)

#### Application Service with Factory Pattern
```rust
#[async_trait]
pub trait ObjectStorageService: Send + Sync {
    async fn generate_upload_url(&self, file_path: &str, options: SignedUrlOptions) -> Result<String, ObjectStorageError>;
    async fn generate_download_url(&self, file_path: &str, ttl_seconds: u64) -> Result<String, ObjectStorageError>;
    async fn delete_file(&self, file_path: &str) -> Result<(), ObjectStorageError>;
    async fn get_file_metadata(&self, file_path: &str) -> Result<UploadedFile, ObjectStorageError>;
}

pub struct ObjectStorageServiceFactory;

impl ObjectStorageServiceFactory {
    pub async fn create(
        provider: &str,
        bucket_name: &str,
        region: &str,
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Box<dyn ObjectStorageService>, ObjectStorageError> {
        match provider.to_lowercase().as_str() {
            "s3" => {
                let s3_service = S3ObjectStorageService::new(
                    bucket_name, region, access_key, secret_key,
                ).await?;
                Ok(Box::new(s3_service))
            }
            _ => Err(ObjectStorageError::ConfigError(
                format!("Unsupported provider: {}", provider)
            ))
        }
    }
}
```

**패턴 분석:**
- ✅ 팩토리 패턴으로 구현체 선택
- ✅ 외부 서비스 추상화
- ✅ 설정 기반 서비스 생성

---

## 연습 문제

### 문제 1: 기본 도메인 서비스 작성
다음 요구사항에 맞는 도메인 서비스를 작성하세요:

**요구사항:**
- 병원 관리 서비스
- 필드: name, address, phone, created_at
- 비즈니스 규칙: 이름 중복 불가, 주소 필수, 전화번호 형식 검증

<details>
<summary>정답 보기</summary>

**Domain Service Interface**
```rust
use async_trait::async_trait;
use crate::domain::entities::{Hospital, NewHospital};
use crate::domain::repositories::HospitalRepository;
use crate::domain::ServiceError;

#[async_trait]
pub trait HospitalService: Send + Sync {
    async fn create_hospital(&self, name: String, address: String, phone: String) -> Result<Hospital, ServiceError>;
    async fn get_hospital(&self, id: i32) -> Result<Hospital, ServiceError>;
    async fn get_hospital_by_name(&self, name: &str) -> Result<Hospital, ServiceError>;
    async fn get_all_hospitals(&self) -> Result<Vec<Hospital>, ServiceError>;
    async fn update_hospital(&self, id: i32, name: String, address: String, phone: String) -> Result<Hospital, ServiceError>;
    async fn delete_hospital(&self, id: i32) -> Result<(), ServiceError>;
}

pub struct HospitalServiceImpl<R>
where
    R: HospitalRepository + Send + Sync,
{
    hospital_repository: R,
}

impl<R> HospitalServiceImpl<R>
where
    R: HospitalRepository + Send + Sync,
{
    pub fn new(hospital_repository: R) -> Self {
        Self { hospital_repository }
    }
}

#[async_trait]
impl<R> HospitalService for HospitalServiceImpl<R>
where
    R: HospitalRepository + Send + Sync,
{
    async fn create_hospital(&self, name: String, address: String, phone: String) -> Result<Hospital, ServiceError> {
        // 이름 중복 체크
        if let Some(_) = self.hospital_repository.find_by_name(&name).await? {
            return Err(ServiceError::AlreadyExists("Hospital name already exists".into()));
        }

        // 비즈니스 규칙 검증
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Hospital name cannot be empty".into()));
        }

        if address.trim().is_empty() {
            return Err(ServiceError::ValidationError("Hospital address cannot be empty".into()));
        }

        // 전화번호 형식 검증 (간단한 예시)
        if !phone.chars().any(|c| c.is_ascii_digit()) {
            return Err(ServiceError::ValidationError("Invalid phone number format".into()));
        }

        let new_hospital = NewHospital {
            name,
            address,
            phone,
        };

        Ok(self.hospital_repository.create(new_hospital).await?)
    }

    async fn get_hospital(&self, id: i32) -> Result<Hospital, ServiceError> {
        self.hospital_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Hospital not found".into()))
    }

    async fn get_hospital_by_name(&self, name: &str) -> Result<Hospital, ServiceError> {
        self.hospital_repository
            .find_by_name(name)
            .await?
            .ok_or(ServiceError::NotFound("Hospital not found".into()))
    }

    async fn get_all_hospitals(&self) -> Result<Vec<Hospital>, ServiceError> {
        Ok(self.hospital_repository.find_all().await?)
    }

    async fn update_hospital(&self, id: i32, name: String, address: String, phone: String) -> Result<Hospital, ServiceError> {
        // 기존 병원 조회
        let existing_hospital = self.get_hospital(id).await?;

        // 이름 중복 체크 (자신 제외)
        if let Some(hospital) = self.hospital_repository.find_by_name(&name).await? {
            if hospital.id != id {
                return Err(ServiceError::AlreadyExists("Hospital name already exists".into()));
            }
        }

        // 비즈니스 규칙 검증
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Hospital name cannot be empty".into()));
        }

        if address.trim().is_empty() {
            return Err(ServiceError::ValidationError("Hospital address cannot be empty".into()));
        }

        if !phone.chars().any(|c| c.is_ascii_digit()) {
            return Err(ServiceError::ValidationError("Invalid phone number format".into()));
        }

        let update_hospital = NewHospital {
            name,
            address,
            phone,
        };

        self.hospital_repository
            .update(id, update_hospital)
            .await?
            .ok_or(ServiceError::NotFound("Hospital not found".into()))
    }

    async fn delete_hospital(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self.hospital_repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Hospital not found".into()))
        }
    }
}
```

</details>

### 문제 2: 복잡한 비즈니스 로직 서비스 작성
다음 요구사항에 맞는 서비스를 작성하세요:

**요구사항:**
- 의료진 관리 서비스
- 의료진은 병원에 소속되어야 함
- 전문과목별로 분류
- 경력 연수에 따른 등급 시스템

<details>
<summary>정답 보기</summary>

```rust
use async_trait::async_trait;
use crate::domain::entities::{Doctor, NewDoctor, Hospital, Specialty};
use crate::domain::repositories::{DoctorRepository, HospitalRepository, SpecialtyRepository};
use crate::domain::ServiceError;

#[derive(Debug, Clone)]
pub enum DoctorGrade {
    Intern,      // 0-1년
    Resident,    // 1-3년
    Fellow,      // 3-5년
    Attending,   // 5년 이상
}

impl DoctorGrade {
    pub fn from_experience_years(years: i32) -> Self {
        match years {
            0..=1 => DoctorGrade::Intern,
            2..=3 => DoctorGrade::Resident,
            4..=5 => DoctorGrade::Fellow,
            _ => DoctorGrade::Attending,
        }
    }
}

#[async_trait]
pub trait DoctorService: Send + Sync {
    async fn create_doctor(
        &self,
        name: String,
        hospital_id: i32,
        specialty_id: i32,
        experience_years: i32,
        license_number: String,
    ) -> Result<Doctor, ServiceError>;

    async fn get_doctor(&self, id: i32) -> Result<Doctor, ServiceError>;
    async fn get_doctors_by_hospital(&self, hospital_id: i32) -> Result<Vec<Doctor>, ServiceError>;
    async fn get_doctors_by_specialty(&self, specialty_id: i32) -> Result<Vec<Doctor>, ServiceError>;
    async fn get_doctors_by_grade(&self, grade: DoctorGrade) -> Result<Vec<Doctor>, ServiceError>;
    async fn promote_doctor(&self, id: i32, new_experience_years: i32) -> Result<Doctor, ServiceError>;
    async fn transfer_doctor(&self, id: i32, new_hospital_id: i32) -> Result<Doctor, ServiceError>;
}

pub struct DoctorServiceImpl<D, H, S>
where
    D: DoctorRepository + Send + Sync,
    H: HospitalRepository + Send + Sync,
    S: SpecialtyRepository + Send + Sync,
{
    doctor_repository: D,
    hospital_repository: H,
    specialty_repository: S,
}

impl<D, H, S> DoctorServiceImpl<D, H, S>
where
    D: DoctorRepository + Send + Sync,
    H: HospitalRepository + Send + Sync,
    S: SpecialtyRepository + Send + Sync,
{
    pub fn new(doctor_repository: D, hospital_repository: H, specialty_repository: S) -> Self {
        Self {
            doctor_repository,
            hospital_repository,
            specialty_repository,
        }
    }
}

#[async_trait]
impl<D, H, S> DoctorService for DoctorServiceImpl<D, H, S>
where
    D: DoctorRepository + Send + Sync,
    H: HospitalRepository + Send + Sync,
    S: SpecialtyRepository + Send + Sync,
{
    async fn create_doctor(
        &self,
        name: String,
        hospital_id: i32,
        specialty_id: i32,
        experience_years: i32,
        license_number: String,
    ) -> Result<Doctor, ServiceError> {
        // 병원 존재 확인
        if self.hospital_repository.find_by_id(hospital_id).await?.is_none() {
            return Err(ServiceError::NotFound("Hospital not found".into()));
        }

        // 전문과목 존재 확인
        if self.specialty_repository.find_by_id(specialty_id).await?.is_none() {
            return Err(ServiceError::NotFound("Specialty not found".into()));
        }

        // 면허번호 중복 체크
        if let Some(_) = self.doctor_repository.find_by_license_number(&license_number).await? {
            return Err(ServiceError::AlreadyExists("License number already exists".into()));
        }

        // 비즈니스 규칙 검증
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Doctor name cannot be empty".into()));
        }

        if experience_years < 0 {
            return Err(ServiceError::ValidationError("Experience years cannot be negative".into()));
        }

        if license_number.trim().is_empty() {
            return Err(ServiceError::ValidationError("License number cannot be empty".into()));
        }

        let new_doctor = NewDoctor {
            name,
            hospital_id,
            specialty_id,
            experience_years,
            license_number,
        };

        Ok(self.doctor_repository.create(new_doctor).await?)
    }

    async fn get_doctor(&self, id: i32) -> Result<Doctor, ServiceError> {
        self.doctor_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Doctor not found".into()))
    }

    async fn get_doctors_by_hospital(&self, hospital_id: i32) -> Result<Vec<Doctor>, ServiceError> {
        // 병원 존재 확인
        if self.hospital_repository.find_by_id(hospital_id).await?.is_none() {
            return Err(ServiceError::NotFound("Hospital not found".into()));
        }

        Ok(self.doctor_repository.find_by_hospital_id(hospital_id).await?)
    }

    async fn get_doctors_by_specialty(&self, specialty_id: i32) -> Result<Vec<Doctor>, ServiceError> {
        // 전문과목 존재 확인
        if self.specialty_repository.find_by_id(specialty_id).await?.is_none() {
            return Err(ServiceError::NotFound("Specialty not found".into()));
        }

        Ok(self.doctor_repository.find_by_specialty_id(specialty_id).await?)
    }

    async fn get_doctors_by_grade(&self, grade: DoctorGrade) -> Result<Vec<Doctor>, ServiceError> {
        let min_years = match grade {
            DoctorGrade::Intern => 0,
            DoctorGrade::Resident => 2,
            DoctorGrade::Fellow => 4,
            DoctorGrade::Attending => 6,
        };

        let max_years = match grade {
            DoctorGrade::Intern => 1,
            DoctorGrade::Resident => 3,
            DoctorGrade::Fellow => 5,
            DoctorGrade::Attending => i32::MAX,
        };

        Ok(self.doctor_repository
            .find_by_experience_range(min_years, max_years)
            .await?)
    }

    async fn promote_doctor(&self, id: i32, new_experience_years: i32) -> Result<Doctor, ServiceError> {
        let doctor = self.get_doctor(id).await?;

        if new_experience_years <= doctor.experience_years {
            return Err(ServiceError::ValidationError("New experience years must be greater than current".into()));
        }

        let update_doctor = NewDoctor {
            name: doctor.name,
            hospital_id: doctor.hospital_id,
            specialty_id: doctor.specialty_id,
            experience_years: new_experience_years,
            license_number: doctor.license_number,
        };

        self.doctor_repository
            .update(id, update_doctor)
            .await?
            .ok_or(ServiceError::NotFound("Doctor not found".into()))
    }

    async fn transfer_doctor(&self, id: i32, new_hospital_id: i32) -> Result<Doctor, ServiceError> {
        let doctor = self.get_doctor(id).await?;

        // 새 병원 존재 확인
        if self.hospital_repository.find_by_id(new_hospital_id).await?.is_none() {
            return Err(ServiceError::NotFound("Target hospital not found".into()));
        }

        let update_doctor = NewDoctor {
            name: doctor.name,
            hospital_id: new_hospital_id,
            specialty_id: doctor.specialty_id,
            experience_years: doctor.experience_years,
            license_number: doctor.license_number,
        };

        self.doctor_repository
            .update(id, update_doctor)
            .await?
            .ok_or(ServiceError::NotFound("Doctor not found".into()))
    }
}
```

</details>

### 문제 3: 애플리케이션 서비스 작성
다음 요구사항에 맞는 애플리케이션 서비스를 작성하세요:

**요구사항:**
- 이메일 발송 서비스
- 여러 이메일 제공업체 지원 (SendGrid, AWS SES)
- 템플릿 기반 이메일 발송
- 발송 결과 추적

<details>
<summary>정답 보기</summary>

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum EmailServiceError {
    #[error("SendGrid error: {0}")]
    SendGridError(String),
    
    #[error("AWS SES error: {0}")]
    AwsSesError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub subject: String,
    pub html_content: String,
    pub text_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRequest {
    pub to: String,
    pub template_id: String,
    pub variables: HashMap<String, String>,
    pub attachments: Option<Vec<EmailAttachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResult {
    pub message_id: String,
    pub status: EmailStatus,
    pub sent_at: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailStatus {
    Sent,
    Delivered,
    Bounced,
    Failed,
}

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(&self, request: EmailRequest) -> Result<EmailResult, EmailServiceError>;
    async fn get_email_status(&self, message_id: &str) -> Result<EmailStatus, EmailServiceError>;
    async fn create_template(&self, template: EmailTemplate) -> Result<(), EmailServiceError>;
    async fn get_template(&self, template_id: &str) -> Result<EmailTemplate, EmailServiceError>;
}

pub struct EmailServiceImpl {
    provider: String,
    api_key: String,
    from_email: String,
    from_name: String,
}

impl EmailServiceImpl {
    pub fn new(provider: String, api_key: String, from_email: String, from_name: String) -> Self {
        Self {
            provider,
            api_key,
            from_email,
            from_name,
        }
    }
}

#[async_trait]
impl EmailService for EmailServiceImpl {
    async fn send_email(&self, request: EmailRequest) -> Result<EmailResult, EmailServiceError> {
        // 이메일 주소 검증
        if !request.to.contains('@') {
            return Err(EmailServiceError::ValidationError("Invalid email address".into()));
        }

        // 템플릿 조회
        let template = self.get_template(&request.template_id).await?;

        // 변수 치환
        let subject = self.replace_variables(&template.subject, &request.variables)?;
        let html_content = self.replace_variables(&template.html_content, &request.variables)?;

        // 제공업체별 발송
        match self.provider.as_str() {
            "sendgrid" => self.send_via_sendgrid(&request.to, &subject, &html_content, request.attachments).await,
            "ses" => self.send_via_ses(&request.to, &subject, &html_content, request.attachments).await,
            _ => Err(EmailServiceError::ConfigError("Unsupported email provider".into())),
        }
    }

    async fn get_email_status(&self, message_id: &str) -> Result<EmailStatus, EmailServiceError> {
        match self.provider.as_str() {
            "sendgrid" => self.get_sendgrid_status(message_id).await,
            "ses" => self.get_ses_status(message_id).await,
            _ => Err(EmailServiceError::ConfigError("Unsupported email provider".into())),
        }
    }

    async fn create_template(&self, template: EmailTemplate) -> Result<(), EmailServiceError> {
        match self.provider.as_str() {
            "sendgrid" => self.create_sendgrid_template(template).await,
            "ses" => self.create_ses_template(template).await,
            _ => Err(EmailServiceError::ConfigError("Unsupported email provider".into())),
        }
    }

    async fn get_template(&self, template_id: &str) -> Result<EmailTemplate, EmailServiceError> {
        match self.provider.as_str() {
            "sendgrid" => self.get_sendgrid_template(template_id).await,
            "ses" => self.get_ses_template(template_id).await,
            _ => Err(EmailServiceError::ConfigError("Unsupported email provider".into())),
        }
    }
}

impl EmailServiceImpl {
    fn replace_variables(&self, content: &str, variables: &HashMap<String, String>) -> Result<String, EmailServiceError> {
        let mut result = content.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }

    async fn send_via_sendgrid(&self, to: &str, subject: &str, html_content: &str, attachments: Option<Vec<EmailAttachment>>) -> Result<EmailResult, EmailServiceError> {
        // SendGrid API 호출 구현
        // 실제 구현에서는 reqwest 등을 사용하여 HTTP 요청
        Ok(EmailResult {
            message_id: "sg_123456789".to_string(),
            status: EmailStatus::Sent,
            sent_at: chrono::Utc::now().to_rfc3339(),
            error_message: None,
        })
    }

    async fn send_via_ses(&self, to: &str, subject: &str, html_content: &str, attachments: Option<Vec<EmailAttachment>>) -> Result<EmailResult, EmailServiceError> {
        // AWS SES API 호출 구현
        Ok(EmailResult {
            message_id: "ses_123456789".to_string(),
            status: EmailStatus::Sent,
            sent_at: chrono::Utc::now().to_rfc3339(),
            error_message: None,
        })
    }

    async fn get_sendgrid_status(&self, message_id: &str) -> Result<EmailStatus, EmailServiceError> {
        // SendGrid 상태 조회 구현
        Ok(EmailStatus::Delivered)
    }

    async fn get_ses_status(&self, message_id: &str) -> Result<EmailStatus, EmailServiceError> {
        // SES 상태 조회 구현
        Ok(EmailStatus::Delivered)
    }

    async fn create_sendgrid_template(&self, template: EmailTemplate) -> Result<(), EmailServiceError> {
        // SendGrid 템플릿 생성 구현
        Ok(())
    }

    async fn create_ses_template(&self, template: EmailTemplate) -> Result<(), EmailServiceError> {
        // SES 템플릿 생성 구현
        Ok(())
    }

    async fn get_sendgrid_template(&self, template_id: &str) -> Result<EmailTemplate, EmailServiceError> {
        // SendGrid 템플릿 조회 구현
        Ok(EmailTemplate {
            id: template_id.to_string(),
            subject: "Welcome!".to_string(),
            html_content: "<h1>Welcome to our service!</h1>".to_string(),
            text_content: Some("Welcome to our service!".to_string()),
        })
    }

    async fn get_ses_template(&self, template_id: &str) -> Result<EmailTemplate, EmailServiceError> {
        // SES 템플릿 조회 구현
        Ok(EmailTemplate {
            id: template_id.to_string(),
            subject: "Welcome!".to_string(),
            html_content: "<h1>Welcome to our service!</h1>".to_string(),
            text_content: Some("Welcome to our service!".to_string()),
        })
    }
}

// 팩토리 패턴
pub struct EmailServiceFactory;

impl EmailServiceFactory {
    pub async fn create(
        provider: &str,
        api_key: String,
        from_email: String,
        from_name: String,
    ) -> Result<Box<dyn EmailService>, EmailServiceError> {
        match provider {
            "sendgrid" | "ses" => {
                let service = EmailServiceImpl::new(
                    provider.to_string(),
                    api_key,
                    from_email,
                    from_name,
                );
                Ok(Box::new(service))
            }
            _ => Err(EmailServiceError::ConfigError(
                format!("Unsupported email provider: {}", provider)
            ))
        }
    }
}
```

</details>

### 문제 4: 트랜잭션 처리 서비스 작성
다음 요구사항에 맞는 트랜잭션 서비스를 작성하세요:

**요구사항:**
- 환자 등록 시 자동으로 기본 진료 기록 생성
- 의료진 배정
- 알림 발송
- 모든 작업이 성공해야 함 (원자성)

<details>
<summary>정답 보기</summary>

```rust
use async_trait::async_trait;
use crate::domain::entities::{Patient, NewPatient, MedicalRecord, NewMedicalRecord, Doctor, Notification};
use crate::domain::repositories::{PatientRepository, MedicalRecordRepository, DoctorRepository, NotificationRepository};
use crate::domain::ServiceError;

#[async_trait]
pub trait PatientRegistrationService: Send + Sync {
    async fn register_patient_with_initial_record(
        &self,
        patient_data: NewPatient,
        initial_diagnosis: String,
        assigned_doctor_id: i32,
    ) -> Result<PatientRegistrationResult, ServiceError>;
}

#[derive(Debug, Clone)]
pub struct PatientRegistrationResult {
    pub patient: Patient,
    pub medical_record: MedicalRecord,
    pub assigned_doctor: Doctor,
    pub notification_sent: bool,
}

pub struct PatientRegistrationServiceImpl<P, M, D, N>
where
    P: PatientRepository + Send + Sync,
    M: MedicalRecordRepository + Send + Sync,
    D: DoctorRepository + Send + Sync,
    N: NotificationRepository + Send + Sync,
{
    patient_repository: P,
    medical_record_repository: M,
    doctor_repository: D,
    notification_repository: N,
}

impl<P, M, D, N> PatientRegistrationServiceImpl<P, M, D, N>
where
    P: PatientRepository + Send + Sync,
    M: MedicalRecordRepository + Send + Sync,
    D: DoctorRepository + Send + Sync,
    N: NotificationRepository + Send + Sync,
{
    pub fn new(
        patient_repository: P,
        medical_record_repository: M,
        doctor_repository: D,
        notification_repository: N,
    ) -> Self {
        Self {
            patient_repository,
            medical_record_repository,
            doctor_repository,
            notification_repository,
        }
    }
}

#[async_trait]
impl<P, M, D, N> PatientRegistrationService for PatientRegistrationServiceImpl<P, M, D, N>
where
    P: PatientRepository + Send + Sync,
    M: MedicalRecordRepository + Send + Sync,
    D: DoctorRepository + Send + Sync,
    N: NotificationRepository + Send + Sync,
{
    async fn register_patient_with_initial_record(
        &self,
        patient_data: NewPatient,
        initial_diagnosis: String,
        assigned_doctor_id: i32,
    ) -> Result<PatientRegistrationResult, ServiceError> {
        // 트랜잭션 시작
        let mut tx = self.patient_repository.pool().begin().await?;

        // 1. 의료진 존재 확인
        let doctor = self.doctor_repository
            .find_by_id(assigned_doctor_id)
            .await?
            .ok_or(ServiceError::NotFound("Doctor not found".into()))?;

        // 2. 환자 생성
        let patient = sqlx::query_as::<_, Patient>(
            "INSERT INTO patients (name, birth_date, gender, phone, hospital_id)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, birth_date, gender, phone, hospital_id, created_at"
        )
        .bind(&patient_data.name)
        .bind(&patient_data.birth_date)
        .bind(&patient_data.gender)
        .bind(&patient_data.phone)
        .bind(&patient_data.hospital_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create patient: {}", e)))?;

        // 3. 기본 진료 기록 생성
        let medical_record = sqlx::query_as::<_, MedicalRecord>(
            "INSERT INTO medical_records (patient_id, doctor_id, diagnosis, treatment, notes)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, patient_id, doctor_id, diagnosis, treatment, notes, created_at"
        )
        .bind(patient.id)
        .bind(assigned_doctor_id)
        .bind(&initial_diagnosis)
        .bind("Initial consultation")
        .bind("Patient registered and initial assessment completed")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create medical record: {}", e)))?;

        // 4. 의료진-환자 관계 생성
        sqlx::query(
            "INSERT INTO doctor_patient_assignments (doctor_id, patient_id, assigned_at)
             VALUES ($1, $2, CURRENT_TIMESTAMP)"
        )
        .bind(assigned_doctor_id)
        .bind(patient.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to assign doctor to patient: {}", e)))?;

        // 5. 알림 생성 (트랜잭션 내에서)
        let notification = sqlx::query_as::<_, Notification>(
            "INSERT INTO notifications (doctor_id, patient_id, type, message, created_at)
             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
             RETURNING id, doctor_id, patient_id, type, message, is_read, created_at"
        )
        .bind(assigned_doctor_id)
        .bind(patient.id)
        .bind("new_patient_assignment")
        .bind(&format!("New patient {} has been assigned to you", patient.name))
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create notification: {}", e)))?;

        // 6. 트랜잭션 커밋
        tx.commit().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        // 7. 외부 알림 발송 (트랜잭션 외부에서)
        let notification_sent = self.send_external_notification(&doctor, &patient).await.unwrap_or(false);

        Ok(PatientRegistrationResult {
            patient,
            medical_record,
            assigned_doctor: doctor,
            notification_sent,
        })
    }
}

impl<P, M, D, N> PatientRegistrationServiceImpl<P, M, D, N>
where
    P: PatientRepository + Send + Sync,
    M: MedicalRecordRepository + Send + Sync,
    D: DoctorRepository + Send + Sync,
    N: NotificationRepository + Send + Sync,
{
    async fn send_external_notification(&self, doctor: &Doctor, patient: &Patient) -> Result<bool, ServiceError> {
        // 외부 알림 서비스 호출 (이메일, SMS 등)
        // 실제 구현에서는 이메일 서비스나 SMS 서비스를 호출
        println!("Sending notification to doctor {} about new patient {}", doctor.name, patient.name);
        
        // 성공/실패 시뮬레이션
        Ok(true)
    }
}
```

</details>

---

## 체크리스트

### Service 작성 전 체크리스트
- [ ] 비즈니스 요구사항이 명확한가?
- [ ] 필요한 Repository가 정의되었는가?
- [ ] 도메인 엔티티가 정의되었는가?

### Domain Service 작성 체크리스트
- [ ] async_trait 사용
- [ ] Send + Sync 제약조건
- [ ] 적절한 메서드명 사용
- [ ] 비즈니스 규칙 검증 포함
- [ ] 에러 처리 구현

### Application Service 작성 체크리스트
- [ ] 외부 서비스 추상화
- [ ] 팩토리 패턴 사용 (필요시)
- [ ] 설정 기반 서비스 생성
- [ ] 에러 타입 정의

### 비즈니스 로직 구현 체크리스트
- [ ] 검증 로직 포함
- [ ] 권한 확인 구현
- [ ] 트랜잭션 처리 (필요시)
- [ ] Race condition 방지

### 에러 처리 체크리스트
- [ ] ServiceError 정의
- [ ] 적절한 에러 변환
- [ ] 에러 메시지 명확성
- [ ] 로깅 고려

---

## 추가 학습 자료

### 관련 문서
- [Async-trait 공식 문서](https://docs.rs/async-trait/latest/async_trait/)
- [Thiserror 공식 문서](https://docs.rs/thiserror/latest/thiserror/)
- [SQLx 공식 문서](https://docs.rs/sqlx/latest/sqlx/)

### 프로젝트 내 관련 파일
- `src/domain/services/` - 도메인 서비스
- `src/application/services/` - 애플리케이션 서비스
- `src/domain/entities/` - 도메인 엔티티
- `src/domain/repositories/` - 레포지토리 인터페이스

---

## 마무리

이 가이드를 통해 PACS Server 프로젝트의 Service 작성 패턴을 익혔습니다.
실제 개발 시에는 이 패턴을 참고하여 일관성 있는 Service를 작성하고,
복잡한 비즈니스 로직에서는 트랜잭션을 적절히 활용하여 데이터 일관성을 보장하세요.

**핵심 포인트:**
1. **분리**: Domain과 Application 서비스 분리
2. **검증**: 비즈니스 규칙과 데이터 검증
3. **에러**: 명확한 에러 처리와 메시지
4. **트랜잭션**: 원자성 보장을 위한 트랜잭션 활용
5. **의존성**: 적절한 의존성 주입과 인터페이스 활용
