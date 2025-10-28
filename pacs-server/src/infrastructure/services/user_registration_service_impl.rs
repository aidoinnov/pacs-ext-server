use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entities::{User, NewUserAuditLog};
use crate::domain::services::UserRegistrationService;
use crate::domain::ServiceError;
use crate::infrastructure::external::KeycloakClient;

/// 사용자 회원가입 및 계정 관리 서비스 구현체
/// 
/// Keycloak과 PostgreSQL 데이터베이스 간의 원자적 트랜잭션을 보장하며,
/// 모든 작업을 감사 로그에 기록합니다.
pub struct UserRegistrationServiceImpl {
    pool: PgPool,
    keycloak_client: KeycloakClient,
}

impl UserRegistrationServiceImpl {
    /// 새로운 UserRegistrationServiceImpl 인스턴스를 생성합니다.
    /// 
    /// # Arguments
    /// * `pool` - PostgreSQL 연결 풀
    /// * `keycloak_client` - Keycloak 클라이언트
    /// 
    /// # Returns
    /// * `Self` - 새로운 서비스 인스턴스
    pub fn new(pool: PgPool, keycloak_client: KeycloakClient) -> Self {
        Self { pool, keycloak_client }
    }
}

#[async_trait]
impl UserRegistrationService for UserRegistrationServiceImpl {
    async fn signup(
        &self,
        username: String,
        email: String,
        password: String,
        full_name: Option<String>,
        organization: Option<String>,
        department: Option<String>,
        phone: Option<String>,
    ) -> Result<User, ServiceError> {
        // 1. 중복 체크
        let existing = sqlx::query!(
            "SELECT id FROM security_user WHERE username = $1 OR email = $2",
            &username,
            &email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if existing.is_some() {
            return Err(ServiceError::AlreadyExists("Username or email already exists".into()));
        }
        
        // 2. Keycloak에 사용자 생성 시도
        let keycloak_result = self.keycloak_client
            .create_user(&username, &email, &password)
            .await;
        
        let keycloak_user_id = match keycloak_result {
            Ok(id) => id,
            Err(e) => {
                // Keycloak 실패 시 감사 로그만 기록하고 에러 반환
                let _ = self.log_audit(NewUserAuditLog {
                    user_id: None,
                    action: "SIGNUP_REQUESTED".to_string(),
                    actor_id: None,
                    keycloak_sync_status: Some("FAILED".to_string()),
                    keycloak_user_id: None,
                    error_message: Some(e.to_string()),
                    metadata: Some(serde_json::json!({
                        "username": username,
                        "email": email
                    })),
                }).await;
                return Err(e);
            }
        };
        
        // 3. DB에 사용자 생성 (트랜잭션 시작)
        let mut tx = self.pool.begin().await?;
        
        // 관리자 승인 대기 상태로 생성: PENDING_APPROVAL
        let user_result = sqlx::query_as::<_, User>(
            "INSERT INTO security_user 
             (keycloak_id, username, email, full_name, organization, department, phone, 
              account_status, email_verified)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'PENDING_APPROVAL', true)
             RETURNING id, keycloak_id, username, email, full_name, organization, department, phone, 
                       created_at, updated_at, account_status, email_verified, 
                       email_verification_token, email_verification_expires_at,
                       approved_by, approved_at, suspended_at, suspended_reason, deleted_at"
        )
        .bind(Uuid::parse_str(&keycloak_user_id).map_err(|e| ServiceError::ValidationError(e.to_string()))?)
        .bind(&username)
        .bind(&email)
        .bind(&full_name)
        .bind(&organization)
        .bind(&department)
        .bind(&phone)
        .fetch_one(&mut *tx)
        .await;
        
        // 4. DB 실패 시 Keycloak 롤백
        let user = match user_result {
            Ok(u) => u,
            Err(e) => {
                // 트랜잭션 롤백
                let _ = tx.rollback().await;
                
                // Keycloak 사용자 삭제 (롤백)
                let delete_result = self.keycloak_client.delete_user(&keycloak_user_id).await;
                
                // 감사 로그 기록
                let _ = self.log_audit(NewUserAuditLog {
                    user_id: None,
                    action: "SIGNUP_REQUESTED".to_string(),
                    actor_id: None,
                    keycloak_sync_status: Some(if delete_result.is_ok() { "ROLLED_BACK" } else { "ROLLBACK_FAILED" }.to_string()),
                    keycloak_user_id: Some(keycloak_user_id.clone()),
                    error_message: Some(format!("DB insert failed: {}", e)),
                    metadata: Some(serde_json::json!({
                        "username": username,
                        "email": email,
                        "rollback_status": if delete_result.is_ok() { "success" } else { "failed" }
                    })),
                }).await;
                
                return Err(ServiceError::DatabaseError(e.to_string()));
            }
        };
        
        // 5. 감사 로그 기록 (성공)
        sqlx::query(
            "INSERT INTO security_user_audit_log 
             (user_id, action, keycloak_sync_status, keycloak_user_id, metadata)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(user.id)
        .bind("SIGNUP_REQUESTED")
        .bind("SUCCESS")
        .bind(&keycloak_user_id)
        .bind(serde_json::json!({
            "username": username,
            "email": email,
            "full_name": full_name,
            "organization": organization
        }))
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        Ok(user)
    }
    
    async fn verify_email(&self, user_id: i32) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;
        
        // 상태 업데이트: PENDING_EMAIL → PENDING_APPROVAL
        sqlx::query!(
            "UPDATE security_user 
             SET account_status = 'PENDING_APPROVAL', email_verified = true 
             WHERE id = $1 AND account_status = 'PENDING_EMAIL'",
            user_id
        )
        .execute(&mut *tx)
        .await?;
        
        // 감사 로그
        sqlx::query(
            "INSERT INTO security_user_audit_log (user_id, action, keycloak_sync_status)
             VALUES ($1, $2, $3)"
        )
        .bind(user_id)
        .bind("EMAIL_VERIFIED")
        .bind("SUCCESS")
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        Ok(())
    }
    
    async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;
        
        // 사용자 조회 (keycloak_id 필요)
        let user = sqlx::query!(
            "SELECT keycloak_id FROM security_user WHERE id = $1",
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // Keycloak에서 사용자 활성화
        let keycloak_result = self.keycloak_client
            .update_user_enabled(&user.keycloak_id.to_string(), true)
            .await;
        
        if let Err(e) = keycloak_result {
            let _ = tx.rollback().await;
            
            let _ = self.log_audit(NewUserAuditLog {
                user_id: Some(user_id),
                action: "APPROVAL_REQUESTED".to_string(),
                actor_id: Some(admin_id),
                keycloak_sync_status: Some("FAILED".to_string()),
                keycloak_user_id: Some(user.keycloak_id.to_string()),
                error_message: Some(e.to_string()),
                metadata: None,
            }).await;
            
            return Err(e);
        }
        
        // 상태 업데이트: PENDING_APPROVAL → ACTIVE
        sqlx::query!(
            "UPDATE security_user 
             SET account_status = 'ACTIVE', approved_by = $1, approved_at = CURRENT_TIMESTAMP
             WHERE id = $2",
            admin_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;
        
        // 감사 로그
        sqlx::query(
            "INSERT INTO security_user_audit_log (user_id, action, actor_id, keycloak_sync_status)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(user_id)
        .bind("APPROVED")
        .bind(admin_id)
        .bind("SUCCESS")
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        Ok(())
    }
    
    async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;
        
        // 사용자 조회 (존재 확인)
        let user = sqlx::query!(
            "SELECT keycloak_id, username, email FROM security_user WHERE id = $1",
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        
        // 사용자가 존재하지 않으면 에러 반환
        let user = user.ok_or_else(|| ServiceError::NotFound("User not found".into()))?;
        
        // Keycloak에서 사용자 삭제
        // keycloak_id는 UUID이므로 문자열로 변환
        let keycloak_user_id = user.keycloak_id.to_string();
        eprintln!("DEBUG: Attempting to delete user from Keycloak: {}", keycloak_user_id);
        
        let keycloak_result = self.keycloak_client
            .delete_user(&keycloak_user_id)
            .await;
        
        if let Err(e) = &keycloak_result {
            let _ = tx.rollback().await;
            
            let _ = self.log_audit(NewUserAuditLog {
                user_id: Some(user_id),
                action: "DELETE_REQUESTED".to_string(),
                actor_id,
                keycloak_sync_status: Some("FAILED".to_string()),
                keycloak_user_id: Some(user.keycloak_id.to_string()),
                error_message: Some(e.to_string()),
                metadata: Some(serde_json::json!({
                    "username": user.username,
                    "email": user.email
                })),
            }).await;
            
            return Err(e.clone());
        }
        
        // DB에서 사용자 삭제 (감사 로그는 별도 보관됨)
        sqlx::query!("DELETE FROM security_user WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;
        
        // 감사 로그 (user_id는 NULL이 아닌 삭제된 ID 유지)
        sqlx::query(
            "INSERT INTO security_user_audit_log 
             (user_id, action, actor_id, keycloak_sync_status, keycloak_user_id, metadata)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(user_id)
        .bind("DELETED")
        .bind(actor_id)
        .bind("SUCCESS")
        .bind(user.keycloak_id.to_string())
        .bind(serde_json::json!({
            "username": user.username,
            "email": user.email
        }))
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        Ok(())
    }
    
    async fn log_audit(&self, log: NewUserAuditLog) -> Result<(), ServiceError> {
        sqlx::query(
            "INSERT INTO security_user_audit_log 
             (user_id, action, actor_id, keycloak_sync_status, keycloak_user_id, error_message, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(log.user_id)
        .bind(&log.action)
        .bind(log.actor_id)
        .bind(&log.keycloak_sync_status)
        .bind(&log.keycloak_user_id)
        .bind(&log.error_message)
        .bind(&log.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
