# PACS 서버 사용자 관리 시스템 기술 문서

## 📋 개요

이 문서는 PACS 서버의 사용자 관리 시스템에 대한 기술적 구현과 아키텍처를 설명합니다. JWT(JSON Web Token) 기반의 상태 비저장(Stateless) 인증 시스템을 사용하며, Clean Architecture 패턴을 따라 구현되었습니다.

## 🏗️ 아키텍처 개요

### Clean Architecture 계층 구조

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
│  ┌─────────────────┐  ┌─────────────────┐                 │
│  │ AuthController   │  │ UserController   │                 │
│  │ UserRegController│  │                 │                 │
│  └─────────────────┘  └─────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐                 │
│  │ AuthUseCase      │  │ UserUseCase      │                 │
│  │ UserRegUseCase   │  │                 │                 │
│  └─────────────────┘  └─────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐                 │
│  │ AuthService      │  │ UserService      │                 │
│  │ User Entity      │  │ UserRepository   │                 │
│  └─────────────────┘  └─────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                       │
│  ┌─────────────────┐  ┌─────────────────┐                 │
│  │ JwtService       │  │ UserRepository  │                 │
│  │ KeycloakClient   │  │ PostgreSQL      │                 │
│  └─────────────────┘  └─────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
```

## 🔐 JWT 기반 인증 시스템

### JWT 토큰 구조

```rust
// JWT Claims 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (사용자 ID)
    pub sub: String,
    /// Keycloak UUID
    pub keycloak_id: Uuid,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Issued At (토큰 발급 시간)
    pub iat: i64,
    /// Expiration (토큰 만료 시간)
    pub exp: i64,
}
```

### JWT 서비스 구현

```rust
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    /// JWT 토큰 생성
    pub fn create_token(&self, claims: &Claims) -> Result<String, JwtError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    /// JWT 토큰 검증 및 Claims 추출
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                if e.to_string().contains("ExpiredSignature") {
                    JwtError::ExpiredToken
                } else {
                    JwtError::TokenValidation(e.to_string())
                }
            })?;

        let claims = token_data.claims;

        // 추가 만료 확인
        if claims.is_expired() {
            return Err(JwtError::ExpiredToken);
        }

        Ok(claims)
    }
}
```

## 👤 사용자 엔티티 및 데이터 모델

### 사용자 엔티티 구조

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// Keycloak 인증 시스템에서 사용하는 사용자 식별자
    pub keycloak_id: Uuid,
    /// 사용자 로그인에 사용되는 고유한 사용자명
    pub username: String,
    /// 사용자의 이메일 주소
    pub email: String,
    /// 사용자의 실명
    pub full_name: Option<String>,
    /// 소속 기관
    pub organization: Option<String>,
    /// 소속 부서/그룹
    pub department: Option<String>,
    /// 연락처
    pub phone: Option<String>,
    /// 사용자 계정이 생성된 시각
    pub created_at: DateTime<Utc>,
    /// 마지막 업데이트 시각
    pub updated_at: Option<DateTime<Utc>>,
    /// 사용자 계정 상태
    pub account_status: UserAccountStatus,
    /// 이메일 인증 완료 여부
    pub email_verified: bool,
    /// 이메일 인증 토큰
    pub email_verification_token: Option<String>,
    /// 이메일 인증 토큰 만료 시간
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    /// 승인한 관리자 ID
    pub approved_by: Option<i32>,
    /// 승인 시간
    pub approved_at: Option<DateTime<Utc>>,
    /// 정지 시간
    pub suspended_at: Option<DateTime<Utc>>,
    /// 정지 사유
    pub suspended_reason: Option<String>,
    /// 삭제 시간
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### 사용자 계정 상태

```rust
#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "user_account_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserAccountStatus {
    /// 이메일 인증 대기 상태
    PendingEmail,
    /// 관리자 승인 대기 상태
    PendingApproval,
    /// 활성 상태 (정상 사용 가능)
    Active,
    /// 정지 상태
    Suspended,
    /// 삭제된 상태
    Deleted,
}
```

## 🔄 사용자 관리 CRUD 작업

### 1. 사용자 생성 (Create)

#### API 엔드포인트
```
POST /api/users
```

#### 요청 DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub organization: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
}
```

#### 구현 흐름
```rust
impl<U, P> UserService for UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    async fn create_user(
        &self, 
        username: String, 
        email: String, 
        keycloak_id: Uuid,
        full_name: Option<String>,
        organization: Option<String>,
        department: Option<String>,
        phone: Option<String>,
    ) -> Result<User, ServiceError> {
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
            full_name,
            organization,
            department,
            phone,
        };

        Ok(self.user_repository.create(new_user).await?)
    }
}
```

#### 데이터베이스 쿼리
```sql
INSERT INTO security_user (keycloak_id, username, email, full_name, organization, department, phone)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at
```

### 2. 사용자 조회 (Read)

#### API 엔드포인트
```
GET /api/users/{user_id}
GET /api/users/username/{username}
```

#### 구현 흐름
```rust
async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError> {
    self.user_repository
        .find_by_id(id)
        .await?
        .ok_or(ServiceError::NotFound("User not found".into()))
}

async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError> {
    self.user_repository
        .find_by_username(username)
        .await?
        .ok_or(ServiceError::NotFound("User not found".into()))
}
```

#### 데이터베이스 쿼리
```sql
-- ID로 조회
SELECT id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at
FROM security_user 
WHERE id = $1

-- Username으로 조회
SELECT id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at
FROM security_user 
WHERE username = $1
```

### 3. 사용자 정보 업데이트 (Update)

#### API 엔드포인트
```
PUT /api/users/{user_id}
```

#### 요청 DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub organization: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
}
```

#### 구현 흐름
```rust
async fn update_user(&self, update_user: UpdateUser) -> Result<User, ServiceError> {
    // 모든 필드를 업데이트 (NULL인 경우 기존 값 유지)
    sqlx::query_as::<_, User>(
        "UPDATE security_user 
         SET email = COALESCE($2, email),
             full_name = COALESCE($3, full_name),
             organization = COALESCE($4, organization),
             department = COALESCE($5, department),
             phone = COALESCE($6, phone)
         WHERE id = $1
         RETURNING id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at"
    )
    .bind(update_user.id)
    .bind(&update_user.email)
    .bind(&update_user.full_name)
    .bind(&update_user.organization)
    .bind(&update_user.department)
    .bind(&update_user.phone)
    .fetch_one(&self.pool)
    .await
}
```

### 4. 사용자 삭제 (Delete)

#### API 엔드포인트
```
DELETE /api/users/{user_id}
```

#### 구현 흐름
```rust
async fn delete_user(&self, id: i32) -> Result<(), ServiceError> {
    let result = sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(id)
        .execute(&self.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::NotFound("User not found".into()));
    }

    Ok(())
}
```

## 🔑 로그인 처리 시스템

### 로그인 API 엔드포인트
```
POST /api/auth/login
```

### 로그인 요청 DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
}
```

### 로그인 응답 DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user_id: i32,
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}
```

### 로그인 처리 흐름

```rust
impl<U: UserRepository> AuthService for AuthServiceImpl<U> {
    async fn login(&self, keycloak_id: Uuid, username: String, email: String) -> Result<AuthResponse, ServiceError> {
        // UPSERT 패턴으로 동시 로그인 Race condition 방지
        let user = sqlx::query_as::<_, crate::domain::entities::User>(
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
}
```

### 토큰 검증 및 사용자 조회

```rust
async fn verify_and_get_user(&self, token: &str) -> Result<User, ServiceError> {
    // 토큰 검증
    let claims = self.jwt_service
        .validate_token(token)
        .map_err(|e| ServiceError::Unauthorized(format!("Invalid token: {}", e)))?;

    // Claims의 만료 여부 확인
    if claims.is_expired() {
        return Err(ServiceError::Unauthorized("Token has expired".into()));
    }

    // 사용자 ID로 사용자 조회
    let user_id = claims.user_id()
        .map_err(|e| ServiceError::ValidationError(format!("Invalid user ID in token: {}", e)))?;

    self.user_repository
        .find_by_id(user_id)
        .await?
        .ok_or(ServiceError::NotFound("User not found".into()))
}
```

## 🛡️ 보안 고려사항

### 1. JWT 토큰 보안
- **HS256 알고리즘** 사용
- **24시간 만료** 시간 설정
- **60초 여유 시간** (leeway) 설정
- **Bearer 토큰** 형식 사용

### 2. 데이터베이스 보안
- **UPSERT 패턴** 사용으로 Race Condition 방지
- **Prepared Statement** 사용으로 SQL Injection 방지
- **트랜잭션** 사용으로 데이터 일관성 보장

### 3. 입력 검증
- **이메일 형식** 검증
- **중복 사용자명** 체크
- **Keycloak ID 중복** 체크

### 4. 에러 처리
```rust
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(String),
    Unauthorized(String),
    InternalError(String),
}
```

## 📡 API 라우팅 구조

### 인증 관련 API
```rust
web::scope("/auth")
    .route("/login", web::post().to(AuthController::<A>::login))
    .route("/verify/{token}", web::get().to(AuthController::<A>::verify_token))
    .route("/refresh", web::post().to(AuthController::<A>::refresh_token))
    .route("/signup", web::post().to(AuthController::<A>::signup))
    .route("/admin/users/approve", web::post().to(AuthController::<A>::approve_user))
    // .route("/verify-email", ...) — 2026-02 이메일 인증 비활성화로 제거
    .route("/users/{user_id}", web::delete().to(AuthController::<A>::delete_account))
```

### 사용자 관리 API
```rust
web::scope("/users")
    .route("", web::post().to(UserController::<U>::create_user))
    .route("/{user_id}", web::get().to(UserController::<U>::get_user))
    .route("/{user_id}", web::put().to(update_user::<U>))
    .route("/username/{username}", web::get().to(UserController::<U>::get_user_by_username))
```

## 🗄️ 데이터베이스 스키마

### security_user 테이블
```sql
CREATE TABLE security_user (
    id SERIAL PRIMARY KEY,
    keycloak_id UUID UNIQUE NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    full_name VARCHAR(100),
    organization VARCHAR(100),
    department VARCHAR(100),
    phone VARCHAR(20),
    account_status user_account_status_enum DEFAULT 'PENDING_EMAIL',
    email_verified BOOLEAN DEFAULT FALSE,
    email_verification_token VARCHAR(255),
    email_verification_expires_at TIMESTAMP WITH TIME ZONE,
    approved_by INTEGER,
    approved_at TIMESTAMP WITH TIME ZONE,
    suspended_at TIMESTAMP WITH TIME ZONE,
    suspended_reason TEXT,
    deleted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### user_account_status_enum 타입
```sql
CREATE TYPE user_account_status_enum AS ENUM (
    'PENDING_EMAIL',
    'PENDING_APPROVAL',
    'ACTIVE',
    'SUSPENDED',
    'DELETED'
);
```

## 🔄 트랜잭션 처리

### UPSERT 패턴 최적화
```rust
/// 최적화된 UPSERT 패턴 (CTE 사용)
pub async fn login_optimized(&self, keycloak_id: Uuid, username: String, email: String) -> Result<AuthResponse, ServiceError> {
    let pool = self.user_repository.pool();

    // CTE를 사용한 원자적 UPSERT + 단일 쿼리
    let user = sqlx::query_as::<_, User>(
        r#"
        WITH upserted AS (
            INSERT INTO security_user (keycloak_id, username, email)
            VALUES ($1, $2, $3)
            ON CONFLICT (keycloak_id) DO UPDATE
            SET username = EXCLUDED.username,
                email = EXCLUDED.email
            RETURNING id, keycloak_id, username, email, created_at
        )
        SELECT * FROM upserted
        "#
    )
    .bind(keycloak_id)
    .bind(&username)
    .bind(&email)
    .fetch_one(pool)
    .await?;

    // JWT 토큰 생성
    let claims = Claims::new(
        user.id,
        user.keycloak_id,
        user.username.clone(),
        user.email.clone(),
        24,
    );

    let token = self.jwt_service
        .create_token(&claims)
        .map_err(|e| ServiceError::Unauthorized(format!("Failed to create token: {}", e)))?;

    Ok(AuthResponse { user, token })
}
```

## 🧪 테스트 전략

### 단위 테스트
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let config = get_test_config();
        let jwt_service = JwtService::new(&config);

        let claims = Claims::new(
            1,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        let token = jwt_service.create_token(&claims).unwrap();
        assert!(!token.is_empty());

        let validated_claims = jwt_service.validate_token(&token).unwrap();
        assert_eq!(validated_claims.sub, "1");
        assert_eq!(validated_claims.username, "testuser");
    }
}
```

## 📊 성능 최적화

### 1. 데이터베이스 최적화
- **인덱스** 설정: `keycloak_id`, `username`, `email`
- **UPSERT 패턴** 사용으로 Race Condition 방지
- **CTE(Common Table Expression)** 사용으로 쿼리 최적화

### 2. 메모리 최적화
- **Arc<T>** 사용으로 참조 카운팅
- **Clone** 최소화
- **String vs &str** 적절한 사용

### 3. 비동기 처리
- **async/await** 패턴 사용
- **tokio** 런타임 활용
- **병렬 처리** 가능한 작업 분리

## 🚀 배포 및 운영

### 환경 변수 설정
```bash
# JWT 설정
JWT_SECRET=your-secret-key-at-least-32-characters-long
JWT_EXPIRATION_HOURS=24

# 데이터베이스 설정
DATABASE_URL=postgresql://username:password@localhost:5432/pacs_db

# Keycloak 설정
KEYCLOAK_URL=http://localhost:8080
KEYCLOAK_REALM=pacs-realm
KEYCLOAK_CLIENT_ID=pacs-client
```

### Docker 설정
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pacs-server /usr/local/bin/pacs-server
EXPOSE 8080
CMD ["pacs-server"]
```

## 📝 결론

PACS 서버의 사용자 관리 시스템은 다음과 같은 특징을 가집니다:

1. **Clean Architecture** 패턴을 따른 계층화된 구조
2. **JWT 기반** 상태 비저장 인증 시스템
3. **UPSERT 패턴**을 통한 Race Condition 방지
4. **PostgreSQL** 기반의 안정적인 데이터 저장
5. **Keycloak** 연동을 통한 중앙화된 인증 관리
6. **Rust**의 타입 안전성과 성능 최적화

이 시스템은 의료 영상 관리 환경에서 요구되는 보안성과 안정성을 제공하며, 확장 가능한 아키텍처를 통해 향후 기능 추가가 용이합니다.

