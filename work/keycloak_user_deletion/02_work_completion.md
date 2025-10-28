# Keycloak 사용자 삭제 기능 구현 완료

## 작업 완료 일시

2025-10-28

## 완료된 작업

### 1. Keycloak Client 수정

**파일**: `pacs-server/src/infrastructure/external/keycloak_client.rs`

#### 주요 변경사항

1. **Service Account 방식 구현**
   - Client credentials grant type 사용
   - `client_id`와 `client_secret`로 토큰 획득
   - Configured realm 사용 (`dcm4che`)

2. **구조체 변경**
   ```rust
   pub struct KeycloakClient {
       base_url: String,
       realm: String,
       client_id: String,
       client_secret: String,  // 추가됨
       admin_username: String,
       admin_password: String,
       http_client: Client,
   }
   ```

3. **Token 획득 방법 변경**
   ```rust
   async fn get_admin_token(&self) -> Result<String, ServiceError> {
       let url = format!("{}/realms/{}/protocol/openid-connect/token", 
                        self.base_url, self.realm);
       
       let params = [
           ("grant_type", "client_credentials"),
           ("client_id", &self.client_id),
           ("client_secret", &self.client_secret),
       ];
       
       // Token 획득 로직
   }
   ```

### 2. 사용자 삭제 로직 개선

**파일**: `pacs-server/src/infrastructure/services/user_registration_service_impl.rs`

#### 주요 변경사항

1. **존재하지 않는 사용자 처리**
   ```rust
   // fetch_one → fetch_optional로 변경
   let user = sqlx::query!(
       "SELECT keycloak_id, username, email FROM security_user WHERE id = $1",
       user_id
   )
   .fetch_optional(&mut *tx)
   .await?;
   
   let user = user.ok_or_else(|| ServiceError::NotFound("User not found".into()))?;
   ```

2. **디버그 로그 추가**
   ```rust
   eprintln!("DEBUG: Attempting to delete user from Keycloak: {}", keycloak_user_id);
   eprintln!("DEBUG: Requesting Keycloak token from: {}", url);
   ```

### 3. 환경 변수 설정

**파일**: `pacs-server/env.development`

```bash
APP_KEYCLOAK_URL=https://keycloak.pacs.ai-do.kr
APP_KEYCLOAK_REALM=dcm4che
APP_KEYCLOAK_CLIENT_ID=pacs-extension-server
APP_KEYCLOAK_CLIENT_SECRET=85TSWxK8ruF750z0Qzh0tQZ8xH5h3y99
```

### 4. API 엔드포인트

**Endpoint**: `DELETE /api/users/{user_id}`

**Response**:
```json
{
  "message": "계정이 삭제되었습니다."
}
```

## 테스트 결과

### 성공적인 사용자 삭제

```bash
$ curl -X DELETE http://localhost:8080/api/users/84
{"message":"계정이 삭제되었습니다."}
```

### 존재하지 않는 사용자 처리

```bash
$ curl -X DELETE http://localhost:8080/api/users/999
{"error":"Account deletion failed: Not found: User not found"}
```

## 해결한 문제

### 1. Keycloak 토큰 획득 문제

**문제**: Admin 계정 로그인 방식 사용
**해결**: Service account 방식으로 변경 (client credentials grant)

### 2. 엔드포인트 라우팅 중복

**문제**: 같은 라우트가 두 컨트롤러에 등록됨
**해결**: `auth_controller.rs`에서만 라우트 등록

### 3. 존재하지 않는 사용자 처리

**문제**: "no rows returned" 에러
**해결**: `fetch_optional` 사용으로 명확한 에러 메시지

## Commits

1. `fix: Service account 방식으로 Keycloak 인증` - 토큰 획득 방식 변경
2. `fix: 사용자 삭제 시 존재하지 않는 사용자 처리 개선` - 에러 처리 개선

