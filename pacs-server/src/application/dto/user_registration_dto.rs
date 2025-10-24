use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 회원가입 요청 DTO
/// 
/// 사용자가 회원가입 시 제공하는 정보를 담는 구조체입니다.
/// 모든 필수 정보와 선택적 프로필 정보를 포함합니다.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignupRequest {
    /// 사용자명 (3자 이상)
    #[schema(example = "john_doe")]
    pub username: String,
    
    /// 이메일 주소
    #[schema(example = "john@example.com")]
    pub email: String,
    
    /// 비밀번호 (8자 이상)
    #[schema(example = "SecurePassword123!")]
    pub password: String,
    
    /// 실명 (선택사항)
    #[schema(example = "John Doe")]
    pub full_name: Option<String>,
    
    /// 소속 기관 (선택사항)
    #[schema(example = "Seoul National University Hospital")]
    pub organization: Option<String>,
    
    /// 소속 부서 (선택사항)
    #[schema(example = "Radiology Department")]
    pub department: Option<String>,
    
    /// 연락처 (선택사항)
    #[schema(example = "010-1234-5678")]
    pub phone: Option<String>,
}

/// 회원가입 응답 DTO
/// 
/// 회원가입 성공 시 반환되는 정보를 담는 구조체입니다.
#[derive(Debug, Serialize, ToSchema)]
pub struct SignupResponse {
    /// 생성된 사용자 ID
    #[schema(example = 123)]
    pub user_id: i32,
    
    /// 사용자명
    #[schema(example = "john_doe")]
    pub username: String,
    
    /// 이메일 주소
    #[schema(example = "john@example.com")]
    pub email: String,
    
    /// 계정 상태
    #[schema(example = "PENDING_EMAIL")]
    pub account_status: String,
    
    /// 응답 메시지
    #[schema(example = "회원가입이 완료되었습니다. 이메일 인증을 완료해주세요.")]
    pub message: String,
}

/// 이메일 인증 요청 DTO
/// 
/// 사용자가 이메일 인증을 완료했을 때 전송하는 요청입니다.
#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    /// 사용자 ID
    #[schema(example = 123)]
    pub user_id: i32,
}

/// 이메일 인증 응답 DTO
/// 
/// 이메일 인증 완료 시 반환되는 응답입니다.
#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    /// 응답 메시지
    #[schema(example = "이메일 인증이 완료되었습니다. 관리자 승인을 기다려주세요.")]
    pub message: String,
}

/// 사용자 승인 요청 DTO
/// 
/// 관리자가 사용자를 승인할 때 사용하는 요청입니다.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApproveUserRequest {
    /// 승인할 사용자 ID
    #[schema(example = 123)]
    pub user_id: i32,
}

/// 사용자 승인 응답 DTO
/// 
/// 사용자 승인 완료 시 반환되는 응답입니다.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApproveUserResponse {
    /// 응답 메시지
    #[schema(example = "사용자가 승인되었습니다.")]
    pub message: String,
}

/// 계정 삭제 요청 DTO
/// 
/// 사용자 계정을 삭제할 때 사용하는 요청입니다.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteAccountRequest {
    /// 삭제할 사용자 ID
    #[schema(example = 123)]
    pub user_id: i32,
}

/// 계정 삭제 응답 DTO
/// 
/// 계정 삭제 완료 시 반환되는 응답입니다.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteAccountResponse {
    /// 응답 메시지
    #[schema(example = "계정이 삭제되었습니다.")]
    pub message: String,
}

/// 사용자 상태 조회 응답 DTO
/// 
/// 사용자의 현재 상태를 조회할 때 반환되는 정보입니다.
#[derive(Debug, Serialize, ToSchema)]
pub struct UserStatusResponse {
    /// 사용자 ID
    #[schema(example = 123)]
    pub user_id: i32,
    
    /// 사용자명
    #[schema(example = "john_doe")]
    pub username: String,
    
    /// 이메일 주소
    #[schema(example = "john@example.com")]
    pub email: String,
    
    /// 계정 상태
    #[schema(example = "PENDING_EMAIL")]
    pub account_status: String,
    
    /// 이메일 인증 완료 여부
    #[schema(example = false)]
    pub email_verified: bool,
    
    /// 승인 여부
    #[schema(example = false)]
    pub is_approved: bool,
    
    /// 승인자 ID (승인된 경우)
    #[schema(example = 1)]
    pub approved_by: Option<i32>,
    
    /// 승인 시간 (승인된 경우)
    #[schema(example = "2025-01-27T10:00:00Z")]
    pub approved_at: Option<String>,
}
