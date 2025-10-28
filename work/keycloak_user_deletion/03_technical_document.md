# Keycloak 사용자 삭제 기술 문서

## 아키텍처

### 시스템 구성

```
Client Application
       ↓
   HTTP Request (DELETE /api/users/{user_id})
       ↓
Auth Controller
       ↓
UserRegistrationUseCase
       ↓
UserRegistrationServiceImpl
       ↓
    ┌─────────────┬──────────────┐
    ↓             ↓              ↓
Keycloak      PACS DB      Audit Log
(Delete)      (Delete)     (Record)
```

### 데이터 흐름

1. **요청 수신**: HTTP DELETE 요청 수신
2. **사용자 조회**: PACS DB에서 사용자 정보 조회
3. **Keycloak 삭제**: Keycloak에서 사용자 삭제
4. **DB 삭제**: PACS DB에서 사용자 삭제
5. **감사 로그**: 삭제 작업 기록
6. **응답 반환**: 성공/실패 응답 반환

## 구현 세부사항

### 1. Service Account 인증

#### Keycloak 설정

1. **Client 설정**
   - Client ID: `pacs-extension-server`
   - Client Secret: 서비스 계정 시크릿
   - Service Accounts Enabled: `true`

2. **필요한 권한**
   - `realm-management` 역할
   - `manage-users`: 사용자 관리
   - `query-users`: 사용자 조회

#### 토큰 획득

```rust
async fn get_admin_token(&self) -> Result<String, ServiceError> {
    let url = format!("{}/realms/{}/protocol/openid-connect/token", 
                     self.base_url, self.realm);
    
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &self.client_id),
        ("client_secret", &self.client_secret),
    ];
    
    let response = self.http_client
        .post(&url)
        .form(&params)
        .send()
        .await?;
    
    // 토큰 추출
    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
}
```

### 2. 사용자 삭제 프로세스

#### 트랜잭션 흐름

```rust
async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<(), ServiceError> {
    let mut tx = self.pool.begin().await?;
    
    // 1. 사용자 조회
    let user = sqlx::query!(
        "SELECT keycloak_id, username, email FROM security_user WHERE id = $1",
        user_id
    )
    .fetch_optional(&mut *tx)
    .await?;
    
    let user = user.ok_or_else(|| ServiceError::NotFound("User not found".into()))?;
    
    // 2. Keycloak에서 사용자 삭제
    let keycloak_result = self.keycloak_client.delete_user(&user.keycloak_id.to_string()).await;
    
    // 3. 실패 시 롤백
    if let Err(e) = &keycloak_result {
        let _ = tx.rollback().await;
        // 감사 로그 기록
        return Err(e.clone());
    }
    
    // 4. DB에서 사용자 삭제
    sqlx::query!("DELETE FROM security_user WHERE id = $1", user_id)
        .execute(&mut *tx)
        .await?;
    
    // 5. 감사 로그 기록
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
    
    // 6. 트랜잭션 커밋
    tx.commit().await?;
    
    Ok(())
}
```

### 3. 에러 처리

#### 에러 타입

| 에러 상황 | HTTP Status | Error Message |
|-----------|-------------|---------------|
| 사용자 미존재 | 404 | "User not found" |
| Keycloak 삭제 실패 | 500 | "Keycloak delete user failed" |
| DB 삭제 실패 | 500 | "Database delete user failed" |

#### 에러 처리 로직

```rust
// Keycloak 삭제 실패 시
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
```

## 보안 고려사항

### 1. 인증

- Service Account 사용으로 관리자 계정 불필요
- Client secret 안전하게 관리 필요

### 2. 권한

- Keycloak에서 적절한 역할 부여 필요
- `manage-users`, `query-users` 권한 필수

### 3. 감사 로그

- 모든 삭제 작업 기록
- 사용자 정보 보관 (삭제 후에도)
- 에러 메시지 기록

## 성능 고려사항

### 1. 트랜잭션

- Keycloak과 DB 간 원자적 작업
- 롤백으로 일관성 보장

### 2. 네트워크

- Keycloak API 호출 최소화
- 타임아웃 설정 필요

## 테스트

### 단위 테스트

```rust
#[tokio::test]
async fn test_delete_account_success() {
    // Given: 사용자가 존재함
    let user_id = create_test_user();
    
    // When: 삭제 요청
    let result = service.delete_account(user_id, Some(1)).await;
    
    // Then: 성공
    assert!(result.is_ok());
    
    // Keycloak에서 사용자 삭제 확인
    // DB에서 사용자 삭제 확인
}
```

### 통합 테스트

```rust
#[tokio::test]
async fn test_delete_account_integration() {
    // 1. 사용자 생성
    let user_id = create_user();
    
    // 2. 삭제
    let response = client.delete_user(user_id).await;
    assert_eq!(response.status(), 200);
    
    // 3. 삭제 확인
    let get_response = client.get_user(user_id).await;
    assert_eq!(get_response.status(), 404);
}
```

## 롤백 전략

### Keycloak 삭제 실패 시

1. Keycloak에서 사용자 삭제 실패
2. 트랜잭션 롤백
3. 감사 로그 기록 (FAILED)
4. 에러 반환

### DB 삭제 실패 시

1. Keycloak에서 사용자 삭제 성공
2. DB에서 삭제 실패
3. 트랜잭션 롤백
4. Keycloak 사용자 복원 필요 (수동)

## 향후 개선사항

1. [ ] Keycloak 사용자 복원 로직
2. [ ] Soft delete 옵션 추가
3. [ ] 삭제 전 의존성 확인
4. [ ] 감사 로그 조회 API

