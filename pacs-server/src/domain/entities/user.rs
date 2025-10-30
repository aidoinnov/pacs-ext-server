//! # 사용자 엔티티 모듈
//! 
//! 이 모듈은 시스템의 사용자 정보를 나타내는 엔티티들을 정의합니다.
//! 사용자는 PACS 시스템에 접근할 수 있는 개인 또는 시스템 계정을 의미합니다.

// 날짜/시간 처리를 위한 chrono 라이브러리
// use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};
// SQLx를 통한 데이터베이스 행 매핑을 위한 트레이트
use sqlx::FromRow;
// UUID 생성 및 처리를 위한 라이브러리
use uuid::Uuid;

/// 사용자 계정 상태를 나타내는 열거형
/// 
/// 사용자의 계정이 현재 어떤 상태인지를 나타냅니다.
/// 회원가입부터 활성화까지의 전체 프로세스를 추적합니다.
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

/// 사용자 계정 감사 로그 엔티티
/// 
/// 사용자의 모든 계정 관련 활동을 추적하는 로그입니다.
/// 사용자가 삭제된 후에도 로그는 영구 보관됩니다.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserAuditLog {
    /// 로그 고유 ID
    pub id: i32,
    /// 사용자 ID (삭제 후에도 NULL이 아닌 ID 유지)
    pub user_id: Option<i32>,
    /// 수행된 작업 (SIGNUP_REQUESTED, EMAIL_VERIFIED, APPROVED, DELETED 등)
    pub action: String,
    /// 작업을 수행한 사용자 ID (시스템 작업의 경우 NULL)
    pub actor_id: Option<i32>,
    /// Keycloak 동기화 상태 (SUCCESS, FAILED, PENDING, ROLLED_BACK)
    pub keycloak_sync_status: Option<String>,
    /// Keycloak에서의 사용자 ID
    pub keycloak_user_id: Option<String>,
    /// 오류 발생 시 오류 메시지
    pub error_message: Option<String>,
    /// 추가 메타데이터 (IP, User-Agent, 요청 데이터 등)
    pub metadata: Option<serde_json::Value>,
    /// 로그 생성 시간
    pub created_at: DateTime<Utc>,
}

/// 새로운 사용자 감사 로그 생성을 위한 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserAuditLog {
    /// 사용자 ID (삭제 후에도 NULL이 아닌 ID 유지)
    pub user_id: Option<i32>,
    /// 수행된 작업
    pub action: String,
    /// 작업을 수행한 사용자 ID
    pub actor_id: Option<i32>,
    /// Keycloak 동기화 상태
    pub keycloak_sync_status: Option<String>,
    /// Keycloak에서의 사용자 ID
    pub keycloak_user_id: Option<String>,
    /// 오류 발생 시 오류 메시지
    pub error_message: Option<String>,
    /// 추가 메타데이터
    pub metadata: Option<serde_json::Value>,
}

/// 시스템 사용자를 나타내는 엔티티
/// 
/// 이 구조체는 데이터베이스의 `security_user` 테이블과 매핑되며,
/// 시스템에 등록된 사용자의 기본 정보를 저장합니다.
/// 
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `keycloak_id`: Keycloak 인증 시스템에서 사용하는 사용자 식별자
/// - `username`: 사용자 로그인에 사용되는 고유한 사용자명
/// - `email`: 사용자의 이메일 주소 (로그인 및 알림에 사용)
/// - `full_name`: 사용자의 실명 (한글명/영문명)
/// - `organization`: 소속 기관
/// - `department`: 소속 부서/그룹
/// - `phone`: 연락처
/// - `created_at`: 사용자 계정이 생성된 시각
/// - `updated_at`: 마지막 업데이트 시각
/// 
/// # 예시
/// ```ignore
/// let user = User {
///     id: 1,
///     keycloak_id: Uuid::new_v4(),
///     username: "john_doe".to_string(),
///     email: "john@example.com".to_string(),
///     full_name: Some("홍길동".to_string()),
///     organization: Some("서울대학교병원".to_string()),
///     department: Some("영상의학과".to_string()),
///     phone: Some("010-1234-5678".to_string()),
///     created_at: DateTime::from_timestamp(1640995200, 0).unwrap(),
///     updated_at: Some(DateTime::from_timestamp(1640995200, 0).unwrap()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// Keycloak 인증 시스템에서 사용하는 사용자 식별자
    pub keycloak_id: Uuid,
    /// 사용자 로그인에 사용되는 고유한 사용자명
    pub username: String,
    /// 사용자의 이메일 주소 (로그인 및 알림에 사용)
    pub email: String,
    /// 사용자의 실명 (한글명/영문명)
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

/// 새로운 사용자 생성을 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 사용자 생성 요청 시 전달되는 데이터를 나타냅니다.
/// 데이터베이스에 저장되기 전의 사용자 정보를 담고 있습니다.
/// 
/// # 필드
/// - `keycloak_id`: Keycloak에서 발급받은 사용자 식별자
/// - `username`: 생성할 사용자명 (중복되지 않아야 함)
/// - `email`: 사용자의 이메일 주소 (유효한 이메일 형식이어야 함)
/// - `full_name`: 사용자의 실명 (선택사항)
/// - `organization`: 소속 기관 (선택사항)
/// - `department`: 소속 부서/그룹 (선택사항)
/// - `phone`: 연락처 (선택사항)
/// 
/// # 예시
/// ```ignore
/// let new_user = NewUser {
///     keycloak_id: Uuid::new_v4(),
///     username: "jane_doe".to_string(),
///     email: "jane@example.com".to_string(),
///     full_name: Some("김영희".to_string()),
///     organization: Some("서울대학교병원".to_string()),
///     department: Some("영상의학과".to_string()),
///     phone: Some("010-9876-5432".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    /// Keycloak에서 발급받은 사용자 식별자
    pub keycloak_id: Uuid,
    /// 생성할 사용자명 (중복되지 않아야 함)
    pub username: String,
    /// 사용자의 이메일 주소 (유효한 이메일 형식이어야 함)
    pub email: String,
    /// 사용자의 실명 (선택사항)
    pub full_name: Option<String>,
    /// 소속 기관 (선택사항)
    pub organization: Option<String>,
    /// 소속 부서/그룹 (선택사항)
    pub department: Option<String>,
    /// 연락처 (선택사항)
    pub phone: Option<String>,
}

/// 사용자 정보 업데이트를 위한 엔티티 (Builder 패턴)
/// 
/// 이 구조체는 사용자 정보 업데이트 시 사용되며, 제공된 필드만 업데이트합니다.
/// Builder 패턴을 사용하여 유연한 업데이트를 지원합니다.
/// 
/// # 필드
/// - `id`: 업데이트할 사용자 ID
/// - `email`: 이메일 주소 (선택사항)
/// - `full_name`: 실명 (선택사항)
/// - `organization`: 소속 기관 (선택사항)
/// - `department`: 소속 부서/그룹 (선택사항)
/// - `phone`: 연락처 (선택사항)
/// 
/// # 예시
/// ```ignore
/// let update_user = UpdateUser::new(1)
///     .with_email("new_email@example.com".to_string())
///     .with_full_name("새로운 이름".to_string())
///     .with_organization("새로운 기관".to_string());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    /// 업데이트할 사용자 ID
    pub id: i32,
    /// 이메일 주소 (선택사항)
    pub email: Option<String>,
    /// 실명 (선택사항)
    pub full_name: Option<String>,
    /// 소속 기관 (선택사항)
    pub organization: Option<String>,
    /// 소속 부서/그룹 (선택사항)
    pub department: Option<String>,
    /// 연락처 (선택사항)
    pub phone: Option<String>,
}

impl UpdateUser {
    /// 새로운 UpdateUser 인스턴스를 생성합니다.
    /// 
    /// # Arguments
    /// * `id` - 업데이트할 사용자 ID
    /// 
    /// # Returns
    /// * `UpdateUser` - 빈 업데이트 객체
    pub fn new(id: i32) -> Self {
        Self {
            id,
            email: None,
            full_name: None,
            organization: None,
            department: None,
            phone: None,
        }
    }

    /// 이메일 주소를 설정합니다.
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// 실명을 설정합니다.
    pub fn with_full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    /// 소속 기관을 설정합니다.
    pub fn with_organization(mut self, organization: String) -> Self {
        self.organization = Some(organization);
        self
    }

    /// 소속 부서/그룹을 설정합니다.
    pub fn with_department(mut self, department: String) -> Self {
        self.department = Some(department);
        self
    }

    /// 연락처를 설정합니다.
    pub fn with_phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }
}
