//! # 사용자 엔티티 모듈
//! 
//! 이 모듈은 시스템의 사용자 정보를 나타내는 엔티티들을 정의합니다.
//! 사용자는 PACS 시스템에 접근할 수 있는 개인 또는 시스템 계정을 의미합니다.

// 날짜/시간 처리를 위한 chrono 라이브러리
use chrono::NaiveDateTime;
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};
// SQLx를 통한 데이터베이스 행 매핑을 위한 트레이트
use sqlx::FromRow;
// UUID 생성 및 처리를 위한 라이브러리
use uuid::Uuid;

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
/// - `created_at`: 사용자 계정이 생성된 시각
/// 
/// # 예시
/// ```rust
/// let user = User {
///     id: 1,
///     keycloak_id: Uuid::new_v4(),
///     username: "john_doe".to_string(),
///     email: "john@example.com".to_string(),
///     created_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
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
    /// 사용자 계정이 생성된 시각
    pub created_at: NaiveDateTime,
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
/// 
/// # 예시
/// ```rust
/// let new_user = NewUser {
///     keycloak_id: Uuid::new_v4(),
///     username: "jane_doe".to_string(),
///     email: "jane@example.com".to_string(),
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
}
