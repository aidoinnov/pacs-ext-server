# 관리자 사용자 승인 API

## 개요

관리자가 회원가입한 사용자를 승인하는 API입니다. 사용자를 승인하면 Keycloak에서 해당 사용자를 활성화하고, PACS 시스템의 계정 상태를 ACTIVE로 변경합니다.

---

## API 엔드포인트

**Endpoint**: `POST /api/auth/admin/users/approve`

**Authentication**: Required (Admin JWT Token)

**Content-Type**: `application/json`

---

## 요청

### Request Body

```json
{
  "user_id": 114
}
```

### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `user_id` | integer | Yes | 승인할 사용자 ID | `114` |

---

## 응답

### Success Response (200 OK)

```json
{
  "message": "사용자가 승인되었습니다."
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
| 400 | Invalid request | `{"error": "..."}` |
| 404 | User not found | `{"error": "User not found: Not found: User not found"}` |
| 500 | Internal server error | `{"error": "..."}` |

---

## 동작 흐름

### 1. 사용자 상태 확인
- PACS DB에서 `keycloak_id` 조회
- 사용자가 존재하는지 확인

### 2. Keycloak 사용자 활성화
- Keycloak Admin API를 호출하여 사용자 활성화
- `enabled: true`로 변경
- 실패 시 에러 반환

### 3. PACS DB 상태 업데이트
- `account_status`를 `ACTIVE`로 변경
- `approved_by`, `approved_at` 필드 업데이트

### 4. 감사 로그 기록
- 승인 액션을 감사 로그에 기록
- `APPROVED` 액션으로 저장

---

## 사용 예시

### cURL

```bash
curl -X POST http://localhost:8080/api/auth/admin/users/approve \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "user_id": 114
  }'
```

### JavaScript (Fetch)

```javascript
const response = await fetch('http://localhost:8080/api/auth/admin/users/approve', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer YOUR_JWT_TOKEN'
  },
  body: JSON.stringify({
    user_id: 114
  })
});

const data = await response.json();
console.log(data);
```

### Python (requests)

```python
import requests

url = 'http://localhost:8080/api/auth/admin/users/approve'
headers = {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer YOUR_JWT_TOKEN'
}
data = {
    'user_id': 114
}

response = requests.post(url, json=data, headers=headers)
print(response.json())
```

---

## 승인 전후 상태

### 승인 전

**PACS DB:**
```json
{
  "id": 114,
  "username": "testuser_enabled_false",
  "account_status": "PENDING_APPROVAL",
  "email_verified": true,
  "approved_by": null,
  "approved_at": null
}
```

**Keycloak:**
```json
{
  "id": "73f96103-e289-4a13-8908-6db36f009d2f",
  "username": "testuser_enabled_false",
  "enabled": false,
  "emailVerified": true
}
```

### 승인 후

**PACS DB:**
```json
{
  "id": 114,
  "username": "testuser_enabled_false",
  "account_status": "ACTIVE",
  "email_verified": true,
  "approved_by": 1,
  "approved_at": "2025-10-28T01:42:48Z"
}
```

**Keycloak:**
```json
{
  "id": "73f96103-e289-4a13-8908-6db36f009d2f",
  "username": "testuser_enabled_false",
  "enabled": true,
  "emailVerified": true
}
```

---

## 트랜잭션 처리

이 API는 원자적 트랜잭션을 보장합니다:

1. **사용자 조회**: 트랜잭션 시작
2. **Keycloak 활성화**: Keycloak에 사용자 활성화 요청
3. **DB 업데이트**: Keycloak 활성화 성공 시에만 DB 상태 업데이트
4. **감사 로그**: 모든 작업을 감사 로그에 기록
5. **트랜잭션 커밋**: 모든 작업이 성공하면 커밋, 실패하면 롤백

만약 Keycloak 활성화가 실패하면:
- DB 상태는 변경되지 않음 (롤백)
- 감사 로그에는 실패 상태로 기록
- 에러 메시지 반환

---

## 관련 API

- **회원가입**: `POST /api/auth/signup` - 사용자 회원가입
- **계정 삭제**: `DELETE /api/users/{user_id}` - 계정 삭제
- **사용자 조회**: `GET /api/users/{user_id}` - 사용자 정보 조회
- **사용자 목록**: `GET /api/users` - 사용자 목록 조회

---

## 보안 고려사항

1. **인증 필요**: 관리자 권한이 있는 JWT 토큰이 필요합니다
2. **권한 검증**: 실제 관리자인지 확인하는 미들웨어 필요 (현재 TODO 상태)
3. **감사 로그**: 모든 승인 작업이 감사 로그에 기록됨
4. **트랜잭션**: 데이터 일관성을 보장하기 위한 원자적 트랜잭션 사용

---

## 구현 세부사항

### 서비스 레이어

```rust
async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<(), ServiceError> {
    // 1. 트랜잭션 시작
    let mut tx = self.pool.begin().await?;
    
    // 2. 사용자 조회 (keycloak_id 필요)
    let user = sqlx::query!(
        "SELECT keycloak_id FROM security_user WHERE id = $1",
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // 3. Keycloak에서 사용자 활성화
    let keycloak_result = self.keycloak_client
        .update_user_enabled(&user.keycloak_id.to_string(), true)
        .await;
    
    // 4. 실패 시 롤백
    if let Err(e) = keycloak_result {
        let _ = tx.rollback().await;
        // 감사 로그 기록
        return Err(e);
    }
    
    // 5. 상태 업데이트: PENDING_APPROVAL → ACTIVE
    sqlx::query!(
        "UPDATE security_user 
         SET account_status = 'ACTIVE', approved_by = $1, approved_at = CURRENT_TIMESTAMP
         WHERE id = $2",
        admin_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    
    // 6. 감사 로그
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
    
    // 7. 커밋
    tx.commit().await?;
    
    Ok(())
}
```

---

## 테스트

### 단위 테스트

```rust
#[tokio::test]
async fn test_approve_user_success() {
    let user_id = 114;
    let admin_id = 1;
    
    // 승인 전 상태 확인
    // ... 사용자 조회 및 상태 확인
    
    // 승인 API 호출
    let response = client
        .post("/api/auth/admin/users/approve")
        .json(&json!({"user_id": user_id}))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // 승인 후 상태 확인
    // Keycloak: enabled = true
    // PACS DB: account_status = ACTIVE
}
```

### 통합 테스트

실제 Keycloak과 데이터베이스를 사용하여 테스트:

```rust
#[tokio::test]
async fn test_approve_user_integration() {
    // 1. 테스트 사용자 생성
    let signup_response = create_test_user().await;
    let user_id = signup_response.user_id;
    
    // 2. Keycloak 상태 확인 (enabled = false)
    // 3. PACS DB 상태 확인 (account_status = PENDING_APPROVAL)
    
    // 4. 승인 API 호출
    approve_user(user_id).await;
    
    // 5. 승인 후 상태 확인
    // - Keycloak: enabled = true
    // - PACS DB: account_status = ACTIVE
    
    // 6. 정리
    delete_user(user_id).await;
}
```

---

## TODO

- [ ] 실제 관리자 ID를 JWT 토큰에서 추출하도록 미들웨어 구현
- [ ] 권한 검증 미들웨어 추가
- [ ] 이메일 알림 기능 추가 (승인 완료 알림)
- [ ] 승인 이력 조회 API 추가

---

## 변경 이력

| Date | Version | Description |
|------|---------|-------------|
| 2025-10-28 | 1.0 | 초기 문서 작성 |
| 2025-10-28 | 1.1 | 사용자 생성 시 enabled=false로 변경 반영 |

