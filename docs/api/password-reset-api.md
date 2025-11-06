# 비밀번호 재설정 API

## 개요

사용자가 비밀번호를 잊었을 때 사용자명과 이메일을 입력하여 새로운 비밀번호로 재설정하는 API입니다.

---

## API 엔드포인트

**Endpoint**: `POST /api/auth/reset-password`

**Authentication**: Not Required (Public)

**Content-Type**: `application/json`

---

## 요청

### Request Body

```json
{
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "new_password": "NewSecurePassword123!"
}
```

### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `username` | string | Yes | 사용자명 | "heeya8876" |
| `email` | string | Yes | 이메일 주소 | "heeya8876@naver.com" |
| `new_password` | string | Yes | 새로운 비밀번호 (8자 이상, 대소문자+숫자 포함) | "NewSecurePassword123!" |

### 비밀번호 요구사항

- 최소 8자 이상
- 대문자 최소 1개
- 소문자 최소 1개
- 숫자 최소 1개
- 특수문자 권장

**예시:**
- ✅ `Qlalfqjsgh1!` (유효)
- ❌ `qlalfqjsgh1!` (대문자 없음)
- ❌ `Qlalfqjsgh` (숫자 없음)

---

## 응답

### Success Response (200 OK)

```json
{
  "message": "비밀번호가 성공적으로 재설정되었습니다."
}
```

### Response Schema

| Field | Type | Description |
|-------|------|-------------|
| `message` | string | 응답 메시지 |

---

## 에러 응답

### Error Responses

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "비밀번호 재설정 실패: Validation error: ..."}` |
| 404 | User not found | `{"error": "비밀번호 재설정 실패: Not found: 사용자를 찾을 수 없습니다."}` |
| 500 | Internal server error | `{"error": "비밀번호 재설정 실패: ..."}` |

---

## 동작 흐름

### 1. 입력 검증

- 비밀번호 강도 검증 (최소 8자, 대소문자+숫자)
- username과 email 일치 확인

### 2. 사용자 확인

- PACS DB에서 username으로 사용자 조회
- email 정보 일치 확인

### 3. Keycloak 비밀번호 재설정

- Keycloak Admin API를 호출하여 비밀번호 재설정
- 비밀번호 영구 적용 (temporary=false)

### 4. 성공 응답

- 비밀번호 재설정 완료 메시지 반환

---

## 사용 예시

### cURL

```bash
curl -X POST http://localhost:8080/api/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "username": "heeya8876",
    "email": "heeya8876@naver.com",
    "new_password": "NewSecurePassword123!"
  }'
```

### JavaScript (Fetch)

```javascript
const resetPassword = async (username, email, newPassword) => {
  const response = await fetch('http://localhost:8080/api/auth/reset-password', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      username,
      email,
      new_password: newPassword
    })
  });
  
  return await response.json();
};

// 사용 예시
try {
  const result = await resetPassword('heeya8876', 'heeya8876@naver.com', 'NewSecurePassword123!');
  console.log(result.message);
} catch (error) {
  console.error('비밀번호 재설정 실패:', error);
}
```

### Python (requests)

```python
import requests

def reset_password(username, email, new_password):
    url = 'http://localhost:8080/api/auth/reset-password'
    
    response = requests.post(url, json={
        'username': username,
        'email': email,
        'new_password': new_password
    })
    
    return response.json()

# 사용 예시
result = reset_password('heeya8876', 'heeya8876@naver.com', 'NewSecurePassword123!')
print(result['message'])
```

---

## 에러 처리

### 일반적인 에러

#### 비밀번호 너무 짧음

```json
{
  "error": "비밀번호 재설정 실패: Validation error: 비밀번호는 최소 8자 이상이어야 합니다."
}
```

#### 사용자를 찾을 수 없음

```json
{
  "error": "비밀번호 재설정 실패: Not found: 사용자를 찾을 수 없습니다."
}
```

#### 이메일 정보 불일치

```json
{
  "error": "비밀번호 재설정 실패: Validation error: 이메일 정보가 일치하지 않습니다."
}
```

#### 비밀번호 강도 부족

```json
{
  "error": "비밀번호 재설정 실패: Validation error: Password must contain at least one uppercase letter, one lowercase letter, and one number"
}
```

---

## 보안 고려사항

### 1. 검증

- **비밀번호 강도 검증**: 최소 8자, 대소문자+숫자 포함
- **이중 확인**: username과 email 일치 확인
- **사용자 존재 확인**: 사용자가 존재하는지 확인

### 2. Keycloak 연동

- Admin 권한으로 비밀번호 재설정
- 비밀번호는 영구 적용 (temporary=false)
- Keycloak 동기화 상태 확인

### 3. 로깅

- 비밀번호 재설정 시도 기록
- 실패 원인 기록 (감사 로그)

---

## 구현 세부사항

### 1. AuthService 구현

```rust
async fn reset_password_by_credentials(
    &self,
    username: &str,
    email: &str,
    new_password: &str,
) -> Result<(), ServiceError> {
    // 1. 비밀번호 강도 검증
    if new_password.len() < 8 {
        return Err(ServiceError::ValidationError(
            "비밀번호는 최소 8자 이상이어야 합니다.".into()
        ));
    }
    
    // 2. 사용자 존재 확인 (username + email 일치 확인)
    let user = self.user_repository
        .find_by_username(username)
        .await?
        .ok_or(ServiceError::NotFound("사용자를 찾을 수 없습니다.".into()))?;
    
    if user.email != email {
        return Err(ServiceError::ValidationError(
            "이메일 정보가 일치하지 않습니다.".into()
        ));
    }
    
    // 3. Keycloak 비밀번호 재설정
    self.keycloak_client
        .reset_user_password(&user.keycloak_id.to_string(), new_password)
        .await?;
    
    Ok(())
}
```

### 2. Keycloak Client 구현

```rust
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
```

---

## 관련 API

- **회원가입**: `POST /api/auth/signup` - 새로운 사용자 등록
- **로그인**: `POST /api/auth/login` - 사용자 로그인
- **아이디 찾기**: `POST /api/auth/find-username` - 이메일로 사용자명 찾기
- **계정 삭제**: `DELETE /api/users/{user_id}` - 계정 삭제

---

## 테스트

### 단위 테스트

```rust
#[tokio::test]
async fn test_reset_password_success() {
    let username = "john.doe";
    let email = "john@example.com";
    let new_password = "NewSecurePassword123!";
    
    // Given: 사용자가 존재함
    create_test_user(username, email);
    
    // When: 비밀번호 재설정
    let result = auth_service
        .reset_password_by_credentials(username, email, new_password)
        .await;
    
    // Then: 성공
    assert!(result.is_ok());
}
```

### 통합 테스트

```rust
#[tokio::test]
async fn test_reset_password_integration() {
    // 1. 테스트 사용자 생성
    let user = create_user();
    
    // 2. 비밀번호 재설정
    let response = client.post("/api/auth/reset-password")
        .json(&json!({
            "username": user.username,
            "email": user.email,
            "new_password": "NewSecurePassword123!"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    
    // 3. 새로운 비밀번호로 로그인 가능한지 확인
    // ...
}
```

---

## FAQ

### Q: 비밀번호 재설정 후 기존 세션은 어떻게 되나요?
A: 기존 세션은 유지됩니다. 사용자가 다음 로그인 시 새로운 비밀번호를 사용해야 합니다.

### Q: 이메일 인증이 필요한가요?
A: 아니요. username과 email을 정확히 입력하면 비밀번호를 재설정할 수 있습니다.

### Q: 이전 비밀번호를 알 필요가 있나요?
A: 아니요. username과 email만 확인하면 됩니다.

### Q: 비밀번호 재설정 횟수 제한이 있나요?
A: 현재는 제한이 없으나, 향후 Rate Limiting을 추가할 예정입니다.

---

## 변경 이력

| Date | Version | Description |
|------|---------|-------------|
| 2025-10-28 | 1.0 | 초기 문서 작성 |

