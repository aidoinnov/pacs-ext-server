# 사용자 회원가입 및 계정 삭제 API 기술 문서

## 📋 개요
이 문서는 PACS Extension Server에 구현된 사용자 회원가입 및 계정 삭제 API의 기술적 세부사항을 설명합니다.

## 🏗️ 시스템 아키텍처

### Clean Architecture 4계층 구조
```
┌─────────────────────────────────────────┐
│           Presentation Layer            │
│        (auth_controller.rs)             │
├─────────────────────────────────────────┤
│           Application Layer             │
│    (UserRegistrationUseCase, DTOs)      │
├─────────────────────────────────────────┤
│             Domain Layer                │
│  (User, UserRegistrationService)        │
├─────────────────────────────────────────┤
│          Infrastructure Layer           │
│ (UserRegistrationServiceImpl, Keycloak) │
└─────────────────────────────────────────┘
```

### 의존성 방향
- Presentation → Application → Domain ← Infrastructure
- 모든 계층이 Domain 계층을 향해 의존
- Infrastructure는 Domain의 인터페이스를 구현

## 🔧 핵심 컴포넌트

### 1. Domain Layer

#### User 엔티티 확장
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub keycloak_id: Option<Uuid>,
    pub full_name: Option<String>,
    pub organization: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub account_status: UserAccountStatus,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    pub approved_by: Option<i32>,
    pub approved_at: Option<DateTime<Utc>>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_reason: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

#### UserAccountStatus 열거형
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum UserAccountStatus {
    PendingEmail,      // 이메일 인증 대기
    PendingApproval,   // 관리자 승인 대기
    Active,            // 활성 상태
    Suspended,         // 정지 상태
    Deleted,           // 삭제 상태
}
```

#### UserRegistrationService 트레이트
```rust
#[async_trait]
pub trait UserRegistrationService: Send + Sync {
    async fn signup(&self, request: SignupRequest) -> Result<SignupResponse, ServiceError>;
    async fn verify_email(&self, user_id: i32) -> Result<VerifyEmailResponse, ServiceError>;
    async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<ApproveUserResponse, ServiceError>;
    async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<DeleteAccountResponse, ServiceError>;
    async fn log_audit(&self, user_id: i32, action: String, details: Option<serde_json::Value>) -> Result<(), ServiceError>;
}
```

### 2. Application Layer

#### UserRegistrationUseCase
```rust
pub struct UserRegistrationUseCase<S: UserRegistrationService> {
    service: Arc<S>,
}

impl<S: UserRegistrationService> UserRegistrationUseCase<S> {
    pub async fn signup(&self, request: SignupRequest) -> Result<SignupResponse, ServiceError> {
        // 비즈니스 로직 검증
        validate_signup_request(&request)?;
        
        // 서비스 호출
        self.service.signup(request).await
    }
    
    // ... 다른 메서드들
}
```

#### DTOs (Data Transfer Objects)
```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
    pub organization: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignupResponse {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub account_status: String,
    pub message: String,
}
```

### 3. Infrastructure Layer

#### UserRegistrationServiceImpl
```rust
pub struct UserRegistrationServiceImpl {
    db_pool: PgPool,
    keycloak_client: Arc<KeycloakClient>,
}

#[async_trait]
impl UserRegistrationService for UserRegistrationServiceImpl {
    async fn signup(&self, request: SignupRequest) -> Result<SignupResponse, ServiceError> {
        let mut tx = self.db_pool.begin().await?;
        
        // Keycloak에서 사용자 생성
        let keycloak_user = self.keycloak_client.create_user(&request).await?;
        
        // 데이터베이스에 사용자 저장
        let user = self.create_user_in_db(&mut tx, &request, keycloak_user.id).await?;
        
        // 감사 로그 기록
        self.log_audit(user.id, "SIGNUP".to_string(), None).await?;
        
        tx.commit().await?;
        
        Ok(SignupResponse {
            user_id: user.id,
            username: user.username,
            email: user.email,
            account_status: user.account_status.to_string(),
            message: "회원가입이 완료되었습니다. 이메일 인증을 완료해주세요.".to_string(),
        })
    }
}
```

#### KeycloakClient
```rust
pub struct KeycloakClient {
    base_url: String,
    realm: String,
    admin_username: String,
    admin_password: String,
    client: reqwest::Client,
}

impl KeycloakClient {
    pub async fn get_admin_token(&self) -> Result<String, ServiceError> {
        // 관리자 토큰 획득
    }
    
    pub async fn create_user(&self, request: &SignupRequest) -> Result<KeycloakUser, ServiceError> {
        // Keycloak에서 사용자 생성
    }
    
    pub async fn delete_user(&self, keycloak_id: Uuid) -> Result<(), ServiceError> {
        // Keycloak에서 사용자 삭제
    }
}
```

#### S3ObjectStorageService
```rust
pub struct S3ObjectStorageService {
    client: S3Client,
    bucket_name: String,
    region: String,
}

#[async_trait]
impl ObjectStorageService for S3ObjectStorageService {
    async fn generate_upload_url(&self, file_path: &str, options: SignedUrlOptions) -> Result<String, ObjectStorageError> {
        // S3 업로드 URL 생성
    }
    
    async fn generate_download_url(&self, file_path: &str, ttl_seconds: u64) -> Result<String, ObjectStorageError> {
        // S3 다운로드 URL 생성
    }
    
    // ... 다른 메서드들
}
```

### 4. Presentation Layer

#### auth_controller
```rust
pub async fn signup(
    user_registration_use_case: web::Data<Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>>,
    req: web::Json<SignupRequest>,
) -> impl Responder {
    match user_registration_use_case.signup(req.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "error": format!("Signup failed: {}", e)
        })),
    }
}
```

## 🗄️ 데이터베이스 스키마

### 1. 사용자 테이블 확장
```sql
-- 계정 상태 열거형
CREATE TYPE user_account_status_enum AS ENUM (
    'PENDING_EMAIL',
    'PENDING_APPROVAL', 
    'ACTIVE',
    'SUSPENDED',
    'DELETED'
);

-- security_user 테이블 확장
ALTER TABLE security_user ADD COLUMN account_status user_account_status_enum;
ALTER TABLE security_user ADD COLUMN email_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE security_user ADD COLUMN email_verification_token VARCHAR(255);
ALTER TABLE security_user ADD COLUMN email_verification_expires_at TIMESTAMP;
ALTER TABLE security_user ADD COLUMN approved_by INTEGER;
ALTER TABLE security_user ADD COLUMN approved_at TIMESTAMP;
ALTER TABLE security_user ADD COLUMN suspended_at TIMESTAMP;
ALTER TABLE security_user ADD COLUMN suspended_reason TEXT;
ALTER TABLE security_user ADD COLUMN deleted_at TIMESTAMP;

-- 인덱스 생성
CREATE INDEX IF NOT EXISTS idx_security_user_account_status ON security_user(account_status);
CREATE INDEX IF NOT EXISTS idx_security_user_email_verified ON security_user(email_verified);
CREATE INDEX IF NOT EXISTS idx_security_user_keycloak_id ON security_user(keycloak_id);
```

### 2. 감사 로그 테이블
```sql
CREATE TABLE security_user_audit_log (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    action VARCHAR(50) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES security_user(id) ON DELETE CASCADE
);

-- 인덱스 생성
CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON security_user_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON security_user_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON security_user_audit_log(created_at);
```

## 🔌 API 명세

### 1. 회원가입 API
```http
POST /api/auth/signup
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "Password123!",
  "full_name": "Test User",
  "organization": "Test Org",
  "department": "IT",
  "phone": "010-1234-5678"
}
```

**응답:**
```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "user_id": 123,
  "username": "testuser",
  "email": "test@example.com",
  "account_status": "PENDING_EMAIL",
  "message": "회원가입이 완료되었습니다. 이메일 인증을 완료해주세요."
}
```

### 2. 이메일 인증 API
```http
POST /api/auth/verify-email
Content-Type: application/json

{
  "user_id": 123
}
```

**응답:**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "message": "이메일 인증이 완료되었습니다. 관리자 승인을 기다려주세요."
}
```

### 3. 사용자 승인 API
```http
POST /api/auth/admin/users/approve
Content-Type: application/json

{
  "user_id": 123
}
```

**응답:**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "message": "사용자가 승인되었습니다."
}
```

### 4. 계정 삭제 API
```http
DELETE /api/auth/users/123
```

**응답:**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "message": "계정이 삭제되었습니다."
}
```

## 🔐 보안 구현

### 1. 비밀번호 정책
```rust
fn is_strong_password(password: &str) -> bool {
    password.len() >= 8 &&
    password.chars().any(|c| c.is_uppercase()) &&
    password.chars().any(|c| c.is_lowercase()) &&
    password.chars().any(|c| c.is_numeric()) &&
    password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
}
```

### 2. 이메일 검증
```rust
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && 
    email.contains('.') && 
    !email.starts_with('@') && 
    !email.ends_with('@') &&
    !email.starts_with('.') && 
    !email.ends_with('.')
}
```

### 3. 감사 로깅
```rust
async fn log_audit(&self, user_id: i32, action: String, details: Option<serde_json::Value>) -> Result<(), ServiceError> {
    let audit_log = NewUserAuditLog {
        user_id,
        action,
        details,
        ip_address: None, // TODO: 요청에서 추출
        user_agent: None, // TODO: 요청에서 추출
    };
    
    sqlx::query_as!(
        UserAuditLog,
        "INSERT INTO security_user_audit_log (user_id, action, details, ip_address, user_agent) 
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
        audit_log.user_id,
        audit_log.action,
        audit_log.details,
        audit_log.ip_address,
        audit_log.user_agent
    )
    .fetch_one(&self.db_pool)
    .await?;
    
    Ok(())
}
```

## 🧪 테스트 전략

### 1. 단위 테스트
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        UserRegistrationService {}
        
        #[async_trait]
        impl UserRegistrationService for UserRegistrationService {
            async fn signup(&self, request: SignupRequest) -> Result<SignupResponse, ServiceError>;
            async fn verify_email(&self, user_id: i32) -> Result<VerifyEmailResponse, ServiceError>;
            async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<ApproveUserResponse, ServiceError>;
            async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<DeleteAccountResponse, ServiceError>;
            async fn log_audit(&self, user_id: i32, action: String, details: Option<serde_json::Value>) -> Result<(), ServiceError>;
        }
    }
    
    #[tokio::test]
    async fn test_signup_success() {
        let mut mock_service = MockUserRegistrationService::new();
        mock_service.expect_signup()
            .times(1)
            .returning(|_| Ok(SignupResponse {
                user_id: 1,
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                account_status: "PENDING_EMAIL".to_string(),
                message: "Success".to_string(),
            }));
        
        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));
        let request = SignupRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
            full_name: Some("Test User".to_string()),
            organization: Some("Test Org".to_string()),
            department: None,
            phone: None,
        };
        
        let result = use_case.signup(request).await;
        assert!(result.is_ok());
    }
}
```

### 2. 통합 테스트
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_signup_integration() {
        // 실제 데이터베이스와 Keycloak을 사용한 통합 테스트
        // (테스트 환경에서만 실행)
    }
}
```

## 🚀 배포 및 운영

### 1. 환경 변수 설정
```bash
# .env 파일
APP_ENV=production
APP_DATABASE_URL=postgresql://user:password@localhost/pacs_db
APP_KEYCLOAK__BASE_URL=http://localhost:8080
APP_KEYCLOAK__REALM=dcm4che
APP_KEYCLOAK__ADMIN_USERNAME=admin
APP_KEYCLOAK__ADMIN_PASSWORD=adminPassword123!
APP_OBJECT_STORAGE__PROVIDER=s3
APP_OBJECT_STORAGE__BUCKET_NAME=pacs-masks
APP_OBJECT_STORAGE__REGION=ap-northeast-2
APP_OBJECT_STORAGE__ACCESS_KEY_ID=your_access_key
APP_OBJECT_STORAGE__SECRET_ACCESS_KEY=your_secret_key
```

### 2. 데이터베이스 마이그레이션
```bash
# 마이그레이션 실행
sqlx migrate run
```

### 3. 서버 실행
```bash
# 개발 환경
RUST_LOG=debug cargo run

# 프로덕션 환경
cargo build --release
./target/release/pacs_server
```

## 📊 모니터링 및 로깅

### 1. 로그 레벨
- **ERROR**: 시스템 오류, 예외 상황
- **WARN**: 경고 상황, 비정상적인 사용 패턴
- **INFO**: 일반적인 사용자 액션, API 호출
- **DEBUG**: 상세한 디버깅 정보

### 2. 감사 로그 모니터링
```sql
-- 최근 사용자 액션 조회
SELECT u.username, al.action, al.created_at, al.details
FROM security_user_audit_log al
JOIN security_user u ON al.user_id = u.id
ORDER BY al.created_at DESC
LIMIT 100;

-- 사용자별 액션 통계
SELECT u.username, al.action, COUNT(*) as count
FROM security_user_audit_log al
JOIN security_user u ON al.user_id = u.id
GROUP BY u.username, al.action
ORDER BY count DESC;
```

## 🔄 확장성 고려사항

### 1. 수평적 확장
- 로드 밸런서를 통한 다중 서버 배포
- 데이터베이스 읽기 전용 복제본 활용
- Redis를 통한 세션 관리

### 2. 성능 최적화
- 데이터베이스 쿼리 최적화
- 캐싱 전략 수립
- 비동기 처리 활용

### 3. 보안 강화
- Rate Limiting 구현
- IP 화이트리스트 관리
- 2FA 인증 추가

## 📝 결론

이 기술 문서는 사용자 회원가입 및 계정 삭제 API의 구현 세부사항을 포괄적으로 다루고 있습니다. Clean Architecture 패턴을 준수하여 유지보수성과 확장성을 확보했으며, 엔터프라이즈급 보안 요구사항을 충족하는 시스템을 구축했습니다.