use async_trait::async_trait;
use crate::domain::entities::{User, NewUserAuditLog};
use crate::domain::ServiceError;

/// 사용자 회원가입 및 계정 관리 서비스 트레이트
/// 
/// 이 서비스는 사용자의 회원가입, 이메일 인증, 관리자 승인, 계정 삭제 등의
/// 기능을 제공하며, Keycloak과 데이터베이스 간의 원자적 트랜잭션을 보장합니다.
#[async_trait]
pub trait UserRegistrationService: Send + Sync {
    /// 회원가입 (원자적 트랜잭션)
    /// 
    /// Keycloak과 데이터베이스에 사용자를 생성합니다.
    /// 어느 한쪽이라도 실패하면 전체 작업을 롤백합니다.
    /// 
    /// # Arguments
    /// * `username` - 사용자명
    /// * `email` - 이메일 주소
    /// * `password` - 비밀번호
    /// * `full_name` - 실명 (선택사항)
    /// * `organization` - 소속 기관 (선택사항)
    /// * `department` - 소속 부서 (선택사항)
    /// * `phone` - 연락처 (선택사항)
    /// 
    /// # Returns
    /// * `Ok(User)` - 생성된 사용자 정보
    /// * `Err(ServiceError)` - 실패 시 에러
    async fn signup(
        &self,
        username: String,
        email: String,
        password: String,
        full_name: Option<String>,
        organization: Option<String>,
        department: Option<String>,
        phone: Option<String>,
    ) -> Result<User, ServiceError>;
    
    /// 이메일 인증 완료 처리
    /// 
    /// 사용자가 이메일 인증을 완료했을 때 호출됩니다.
    /// 계정 상태를 PENDING_EMAIL에서 PENDING_APPROVAL로 변경합니다.
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// 
    /// # Returns
    /// * `Ok(())` - 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    async fn verify_email(&self, user_id: i32) -> Result<(), ServiceError>;
    
    /// 관리자 승인
    /// 
    /// 관리자가 사용자를 승인했을 때 호출됩니다.
    /// Keycloak에서 사용자를 활성화하고 계정 상태를 ACTIVE로 변경합니다.
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `admin_id` - 승인한 관리자 ID
    /// 
    /// # Returns
    /// * `Ok(())` - 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<(), ServiceError>;
    
    /// 계정 삭제 (원자적)
    /// 
    /// Keycloak과 데이터베이스에서 사용자를 삭제합니다.
    /// 감사 로그는 별도 보관됩니다.
    /// 
    /// # Arguments
    /// * `user_id` - 삭제할 사용자 ID
    /// * `actor_id` - 삭제를 수행한 사용자 ID (시스템 작업의 경우 None)
    /// 
    /// # Returns
    /// * `Ok(())` - 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<(), ServiceError>;
    
    /// 감사 로그 기록
    /// 
    /// 사용자의 모든 활동을 감사 로그에 기록합니다.
    /// 
    /// # Arguments
    /// * `log` - 기록할 감사 로그 정보
    /// 
    /// # Returns
    /// * `Ok(())` - 성공
    /// * `Err(ServiceError)` - 실패 시 에러
    async fn log_audit(&self, log: NewUserAuditLog) -> Result<(), ServiceError>;
}
