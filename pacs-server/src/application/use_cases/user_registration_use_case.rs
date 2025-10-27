use crate::application::dto::user_registration_dto::*;
use crate::domain::services::UserRegistrationService;
use crate::domain::ServiceError;

/// 사용자 회원가입 및 계정 관리 유스케이스
/// 
/// 사용자의 회원가입, 이메일 인증, 관리자 승인, 계정 삭제 등의
/// 비즈니스 로직을 처리하는 유스케이스입니다.
pub struct UserRegistrationUseCase<S: UserRegistrationService> {
    service: S,
}

impl<S: UserRegistrationService> UserRegistrationUseCase<S> {
    /// 새로운 UserRegistrationUseCase 인스턴스를 생성합니다.
    /// 
    /// # Arguments
    /// * `service` - 사용자 등록 서비스
    /// 
    /// # Returns
    /// * `Self` - 새로운 유스케이스 인스턴스
    pub fn new(service: S) -> Self {
        Self { service }
    }
    
    /// 회원가입 처리
    /// 
    /// 사용자 입력을 검증하고 회원가입을 처리합니다.
    /// 
    /// # Arguments
    /// * `request` - 회원가입 요청
    /// 
    /// # Returns
    /// * `Ok(SignupResponse)` - 회원가입 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    pub async fn signup(&self, request: SignupRequest) -> Result<SignupResponse, ServiceError> {
        // 입력 검증
        if request.username.is_empty() || request.username.len() < 3 {
            return Err(ServiceError::ValidationError("Username must be at least 3 characters".into()));
        }
        
        if !request.email.contains('@') {
            return Err(ServiceError::ValidationError("Invalid email format".into()));
        }
        
        if request.password.len() < 8 {
            return Err(ServiceError::ValidationError("Password must be at least 8 characters".into()));
        }
        
        // 이메일 형식 더 정확한 검증
        if !is_valid_email(&request.email) {
            return Err(ServiceError::ValidationError("Invalid email format".into()));
        }
        
        // 비밀번호 강도 검증
        if !is_strong_password(&request.password) {
            return Err(ServiceError::ValidationError("Password must contain at least one uppercase letter, one lowercase letter, and one number".into()));
        }
        
        // 서비스 호출
        let user = self.service.signup(
            request.username,
            request.email,
            request.password,
            request.full_name,
            request.organization,
            request.department,
            request.phone,
        ).await?;
        
        Ok(SignupResponse {
            user_id: user.id,
            username: user.username,
            email: user.email,
            account_status: format!("{:?}", user.account_status),
            message: "회원가입이 완료되었습니다. 관리자 승인을 기다려주세요.".to_string(),
        })
    }
    
    /// 이메일 인증 처리
    /// 
    /// 사용자가 이메일 인증을 완료했을 때 호출됩니다.
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// 
    /// # Returns
    /// * `Ok(VerifyEmailResponse)` - 이메일 인증 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    pub async fn verify_email(&self, user_id: i32) -> Result<VerifyEmailResponse, ServiceError> {
        self.service.verify_email(user_id).await?;
        
        Ok(VerifyEmailResponse {
            message: "이메일 인증이 완료되었습니다. 관리자 승인을 기다려주세요.".to_string(),
        })
    }
    
    /// 사용자 승인 처리
    /// 
    /// 관리자가 사용자를 승인할 때 호출됩니다.
    /// 
    /// # Arguments
    /// * `user_id` - 승인할 사용자 ID
    /// * `admin_id` - 승인하는 관리자 ID
    /// 
    /// # Returns
    /// * `Ok(ApproveUserResponse)` - 승인 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    pub async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<ApproveUserResponse, ServiceError> {
        self.service.approve_user(user_id, admin_id).await?;
        
        Ok(ApproveUserResponse {
            message: "사용자가 승인되었습니다.".to_string(),
        })
    }
    
    /// 계정 삭제 처리
    /// 
    /// 사용자 계정을 삭제할 때 호출됩니다.
    /// 
    /// # Arguments
    /// * `user_id` - 삭제할 사용자 ID
    /// * `actor_id` - 삭제를 수행하는 사용자 ID (시스템 작업의 경우 None)
    /// 
    /// # Returns
    /// * `Ok(DeleteAccountResponse)` - 삭제 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    pub async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<DeleteAccountResponse, ServiceError> {
        self.service.delete_account(user_id, actor_id).await?;
        
        Ok(DeleteAccountResponse {
            message: "계정이 삭제되었습니다.".to_string(),
        })
    }
}

/// 이메일 형식 검증 함수
/// 
/// # Arguments
/// * `email` - 검증할 이메일 주소
/// 
/// # Returns
/// * `bool` - 유효한 이메일 형식이면 true
fn is_valid_email(email: &str) -> bool {
    // 간단한 이메일 형식 검증
    email.contains('@') && 
    email.contains('.') && 
    !email.starts_with('@') && 
    !email.ends_with('@') &&
    !email.starts_with('.') && 
    !email.ends_with('.')
}

/// 비밀번호 강도 검증 함수
/// 
/// # Arguments
/// * `password` - 검증할 비밀번호
/// 
/// # Returns
/// * `bool` - 강한 비밀번호이면 true
fn is_strong_password(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    
    has_uppercase && has_lowercase && has_digit
}
