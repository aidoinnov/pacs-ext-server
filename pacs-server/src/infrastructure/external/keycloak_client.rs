use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::domain::ServiceError;
use crate::infrastructure::config::KeycloakConfig;

#[derive(Clone)]
pub struct KeycloakClient {
    base_url: String,
    realm: String,
    client_id: String,
    admin_username: String,
    admin_password: String,
    http_client: Client,
}

#[derive(Serialize)]
struct TokenRequest {
    grant_type: String,
    username: String,
    password: String,
    client_id: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Serialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    enabled: bool,
    #[serde(rename = "emailVerified")]
    email_verified: bool,
    credentials: Vec<Credential>,
    #[serde(rename = "requiredActions")]
    required_actions: Vec<String>,
}

#[derive(Serialize)]
struct Credential {
    #[serde(rename = "type")]
    credential_type: String,
    value: String,
    temporary: bool,
}

#[derive(Serialize)]
struct UpdateUserRequest {
    enabled: bool,
    #[serde(rename = "emailVerified")]
    email_verified: bool,
}

#[derive(Serialize)]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KeycloakTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub token_type: String,
}

impl KeycloakClient {
    pub fn new(config: KeycloakConfig) -> Self {
        Self {
            base_url: config.url,
            realm: config.realm,
            client_id: config.client_id,
            admin_username: config.admin_username,
            admin_password: config.admin_password,
            http_client: Client::new(),
        }
    }

    /// 1. Admin 토큰 획득
    async fn get_admin_token(&self) -> Result<String, ServiceError> {
        let url = format!("{}/realms/master/protocol/openid-connect/token", self.base_url);
        
        let params = [
            ("grant_type", "password"),
            ("username", &self.admin_username),
            ("password", &self.admin_password),
            ("client_id", "admin-cli"),
        ];
        
        let response = self.http_client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Keycloak token request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceError::ExternalServiceError(
                format!("Keycloak token failed ({}): {}", status, body)
            ));
        }
        
        let token_response: TokenResponse = response.json().await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Failed to parse token: {}", e)))?;
        
        Ok(token_response.access_token)
    }

    /// 2. 사용자 생성 (이메일 인증 필수, user 역할 자동 할당)
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<String, ServiceError> {
        let token = self.get_admin_token().await?;
        let url = format!("{}/admin/realms/{}/users", self.base_url, self.realm);
        
        let create_request = CreateUserRequest {
            username: username.to_string(),
            email: email.to_string(),
            enabled: true,
            email_verified: false,
            credentials: vec![Credential {
                credential_type: "password".to_string(),
                value: password.to_string(),
                temporary: false,
            }],
            required_actions: vec!["VERIFY_EMAIL".to_string()],
        };
        
        let response = self.http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&create_request)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Keycloak create user failed: {}", e)))?;
        
        if response.status() == StatusCode::CONFLICT {
            return Err(ServiceError::AlreadyExists("User already exists in Keycloak".into()));
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceError::ExternalServiceError(
                format!("Keycloak create user failed ({}): {}", status, body)
            ));
        }
        
        // Location 헤더에서 사용자 ID 추출
        let location = response.headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ServiceError::ExternalServiceError("No Location header".into()))?;
        
        let user_id = location.split('/').last()
            .ok_or_else(|| ServiceError::ExternalServiceError("Invalid Location header".into()))?
            .to_string();
        
        // user 역할 할당
        let _ = self.assign_realm_role(&token, &user_id, "user").await;
        
        // 이메일 인증 메일 발송
        let _ = self.send_verification_email(&token, &user_id).await;
        
        Ok(user_id)
    }

    /// 3. 사용자 삭제
    pub async fn delete_user(&self, keycloak_user_id: &str) -> Result<(), ServiceError> {
        let token = self.get_admin_token().await?;
        let url = format!("{}/admin/realms/{}/users/{}", self.base_url, self.realm, keycloak_user_id);
        
        let response = self.http_client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Keycloak delete user failed: {}", e)))?;
        
        if response.status() == StatusCode::NOT_FOUND {
            // 이미 삭제됨 - 성공으로 간주
            return Ok(());
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceError::ExternalServiceError(
                format!("Keycloak delete user failed ({}): {}", status, body)
            ));
        }
        
        Ok(())
    }

    /// 4. 이메일 인증 메일 발송
    async fn send_verification_email(&self, token: &str, keycloak_user_id: &str) -> Result<(), ServiceError> {
        let url = format!(
            "{}/admin/realms/{}/users/{}/send-verify-email",
            self.base_url, self.realm, keycloak_user_id
        );
        
        let response = self.http_client
            .put(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Send verification email failed: {}", e)))?;
        
        if !response.status().is_success() {
            tracing::warn!("Failed to send verification email: {}", response.status());
        }
        
        Ok(())
    }

    /// 5. Realm 역할 할당
    async fn assign_realm_role(&self, token: &str, keycloak_user_id: &str, role_name: &str) -> Result<(), ServiceError> {
        // 1. 역할 조회
        let role_url = format!("{}/admin/realms/{}/roles/{}", self.base_url, self.realm, role_name);
        let role_response = self.http_client
            .get(&role_url)
            .bearer_auth(token)
            .send()
            .await?;
        
        if !role_response.status().is_success() {
            return Err(ServiceError::ExternalServiceError(format!("Role '{}' not found", role_name)));
        }
        
        let role: serde_json::Value = role_response.json().await?;
        
        // 2. 역할 할당
        let assign_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.base_url, self.realm, keycloak_user_id
        );
        
        let response = self.http_client
            .post(&assign_url)
            .bearer_auth(token)
            .json(&vec![role])
            .send()
            .await?;
        
        if !response.status().is_success() {
            tracing::warn!("Failed to assign role '{}': {}", role_name, response.status());
        }
        
        Ok(())
    }

    /// 6. 사용자 활성화/비활성화
    pub async fn update_user_enabled(&self, keycloak_user_id: &str, enabled: bool) -> Result<(), ServiceError> {
        let token = self.get_admin_token().await?;
        let url = format!("{}/admin/realms/{}/users/{}", self.base_url, self.realm, keycloak_user_id);
        
        let update_request = UpdateUserRequest {
            enabled,
            email_verified: enabled, // 활성화 시 이메일 인증됨으로 표시
        };
        
        let response = self.http_client
            .put(&url)
            .bearer_auth(&token)
            .json(&update_request)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Update user failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceError::ExternalServiceError(
                format!("Update user failed ({}): {}", status, body)
            ));
        }
        
        Ok(())
    }

    /// 사용자 비밀번호 재설정 (관리자 권한)
    pub async fn reset_user_password(
        &self,
        keycloak_user_id: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        let token = self.get_admin_token().await?;
        
        let url = format!(
            "{}/admin/realms/{}/users/{}/reset-password",
            self.base_url, self.realm, keycloak_user_id
        );
        
        let credential = json!({
            "type": "password",
            "value": new_password,
            "temporary": false
        });
        
        let response = self.http_client
            .put(&url)
            .bearer_auth(&token)
            .json(&credential)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceError::ExternalServiceError(
                format!("비밀번호 재설정 실패 ({}): {}", status, body)
            ));
        }
        
        Ok(())
    }

    /// Refresh access token using Keycloak's refresh token endpoint
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<KeycloakTokenResponse, ServiceError> {
        let url = format!("{}/realms/{}/protocol/openid-connect/token", self.base_url, self.realm);
        
        let request = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: self.client_id.clone(),
        };
        
        let response = self.http_client
            .post(&url)
            .form(&request)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Refresh token request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceError::ExternalServiceError(
                format!("Refresh token failed ({}): {}", status, body)
            ));
        }
        
        let token_response: KeycloakTokenResponse = response.json().await
            .map_err(|e| ServiceError::ExternalServiceError(format!("Failed to parse refresh token response: {}", e)))?;
        
        Ok(token_response)
    }
}
