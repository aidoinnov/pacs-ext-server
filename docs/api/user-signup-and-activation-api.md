# 사용자 회원가입 및 활성화 API

## 개요

이 문서는 사용자 회원가입과 관리자 승인(활성화) API를 설명합니다.

### 회원가입 플로우

1. **사용자 회원가입**: `POST /api/auth/signup`
   - Keycloak에 사용자 생성 (enabled=false, emailVerified=true)
   - PACS DB에 사용자 생성 (account_status=ACTIVE)
   - 즉시 활성화됨 (관리자 승인 필요 없음)

2. **관리자 승인** (선택): `POST /api/auth/admin/users/approve`
   - Keycloak에서 사용자 활성화 (enabled=true)
   - 이미 ACTIVE 상태이므로 상태 변경 없음

---

## 1. 회원가입 API

### 1.1. 회원가입 요청

사용자가 새로운 계정을 등록합니다.

**Endpoint**: `POST /api/auth/signup`

**Authentication**: Not Required (Public)

**Content-Type**: `application/json`

#### Request Body

```json
{
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "password": "Qlalfqjsgh1!",
  "full_name": "정희렬",
  "organization": "주식회사 아이두 이노베이션",
  "department": "개발팀",
  "phone": "010-4003-8107"
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `username` | string | Yes | 사용자명 (3자 이상) | "heeya8876" |
| `email` | string | Yes | 이메일 주소 | "heeya8876@naver.com" |
| `password` | string | Yes | 비밀번호 (8자 이상, 대소문자+숫자 포함) | "Qlalfqjsgh1!" |
| `full_name` | string | Optional | 실명 | "정희렬" |
| `organization` | string | Optional | 소속 기관 | "주식회사 아이두 이노베이션" |
| `department` | string | Optional | 소속 부서 | "개발팀" |
| `phone` | string | Optional | 연락처 | "010-4003-8107" |

#### Success Response (201 Created)

```json
{
  "user_id": 115,
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "account_status": "Active",
  "message": "회원가입이 완료되었습니다. 관리자 승인을 기다려주세요."
}
```

#### Error Responses

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "Signup failed: Validation error: ..."}` |
| 409 | Already exists | `{"error": "Signup failed: Already exists: Username or email already exists"}` |
| 500 | Internal server error | `{"error": "Signup failed: ..."}` |

#### 유효성 검증

**사용자명 (username)**
- 최소 3자 이상
- 영문, 숫자, 언더스코어(_), 하이픈(-) 허용

**이메일 (email)**
- 올바른 이메일 형식 (@ 포함)
- 중복 불가

**비밀번호 (password)**
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

## 2. 사용자 활성화 API (관리자)

### 2.1. 사용자 승인/활성화

관리자가 사용자를 승인하여 시스템을 사용할 수 있도록 합니다.

**Endpoint**: `POST /api/auth/admin/users/approve`

**Authentication**: Required (Admin JWT Token)

**Content-Type**: `application/json`

#### Request Body

```json
{
  "user_id": 114
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `user_id` | integer | Yes | 승인할 사용자 ID | 114 |

#### Success Response (200 OK)

```json
{
  "message": "사용자가 승인되었습니다."
}
```

#### Error Responses

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "User approval failed: ..."}` |
| 404 | User not found | `{"error": "User approval failed: Not found: User not found"}` |
| 500 | Internal server error | `{"error": "User approval failed: ..."}` |

#### 동작 과정

1. **사용자 조회**: PACS DB에서 `keycloak_id` 조회
2. **Keycloak 활성화**: Keycloak에서 사용자 `enabled=true` 설정
3. **DB 상태 업데이트**: `account_status=ACTIVE` (이미 ACTIVE 상태면 변경 없음)
4. **감사 로그**: 승인 액션 기록

---

## 사용 예시

### 1. 회원가입 (cURL)

```bash
curl -X POST http://localhost:8080/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "heeya8876",
    "email": "heeya8876@naver.com",
    "password": "Qlalfqjsgh1!",
    "full_name": "정희렬",
    "organization": "주식회사 아이두 이노베이션",
    "department": "개발팀",
    "phone": "010-4003-8107"
  }'
```

### 2. 사용자 승인 (cURL)

```bash
curl -X POST http://localhost:8080/api/auth/admin/users/approve \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "user_id": 115
  }'
```

### 3. JavaScript (Fetch)

#### 회원가입
```javascript
const signup = async (userData) => {
  const response = await fetch('http://localhost:8080/api/auth/signup', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      username: 'heeya8876',
      email: 'heeya8876@naver.com',
      password: 'Qlalfqjsgh1!',
      full_name: '정희렬',
      organization: '주식회사 아이두 이노베이션',
      department: '개발팀',
      phone: '010-4003-8107'
    })
  });
  
  return await response.json();
};
```

#### 관리자 승인
```javascript
const approveUser = async (userId, token) => {
  const response = await fetch('http://localhost:8080/api/auth/admin/users/approve', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      user_id: userId
    })
  });
  
  return await response.json();
};
```

### 4. Python (requests)

#### 회원가입
```python
import requests

def signup(user_data):
    url = 'http://localhost:8080/api/auth/signup'
    
    response = requests.post(url, json={
        'username': 'heeya8876',
        'email': 'heeya8876@naver.com',
        'password': 'Qlalfqjsgh1!',
        'full_name': '정희렬',
        'organization': '주식회사 아이두 이노베이션',
        'department': '개발팀',
        'phone': '010-4003-8107'
    })
    
    return response.json()
```

#### 관리자 승인
```python
def approve_user(user_id, token):
    url = 'http://localhost:8080/api/auth/admin/users/approve'
    
    response = requests.post(
        url,
        json={'user_id': user_id},
        headers={'Authorization': f'Bearer {token}'}
    )
    
    return response.json()
```

---

## 상태 변화

### 회원가입 전

**사용자**: 존재하지 않음

### 회원가입 후

**PACS DB:**
```json
{
  "id": 115,
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "account_status": "ACTIVE",
  "email_verified": true,
  "approved_by": null,
  "approved_at": null
}
```

**Keycloak:**
```json
{
  "id": "73f96103-e289-4a13-8908-6db36f009d2f",
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "enabled": false,
  "emailVerified": true
}
```

**비고**: Keycloak에서 사용자는 `enabled=false` 상태로 생성되지만, PACS DB에서는 `ACTIVE` 상태입니다.

### 관리자 승인 후

**PACS DB:**
```json
{
  "id": 115,
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "account_status": "ACTIVE",
  "email_verified": true,
  "approved_by": 1,
  "approved_at": "2025-10-28T01:50:00Z"
}
```

**Keycloak:**
```json
{
  "id": "73f96103-e289-4a13-8908-6db36f009d2f",
  "username": "heeya8876",
  "email": "heeya8876@naver.com",
  "enabled": true,
  "emailVerified": true
}
```

**비고**: 이제 Keycloak에서도 `enabled=true` 상태가 되어 로그인이 가능합니다.

---

## 트랜잭션 처리

### 회원가입 트랜잭션

1. **Keycloak에 사용자 생성** 시도
2. **성공 시**: PACS DB에 사용자 생성
3. **실패 시**: 에러 반환, 롤백 없음 (Keycloak 사용자만 생성 안 됨)

### 승인 트랜잭션

1. **PACS DB 조회**: 트랜잭션 시작
2. **Keycloak 활성화** 시도
3. **성공 시**: DB 상태 업데이트 및 감사 로그 기록
4. **실패 시**: 트랜잭션 롤백, 에러 반환

---

## 보안 고려사항

### 회원가입 보안

1. **비밀번호 강도 검증**: 최소 8자, 대소문자+숫자 포함
2. **중복 방지**: username과 email 중복 확인
3. **이메일 인증**: Keycloak에서 emailVerified=true로 설정
4. **초기 비활성화**: Keycloak에서 enabled=false로 생성

### 관리자 승인 보안

1. **인증 필요**: 관리자 권한이 있는 JWT 토큰 필요
2. **권한 검증**: 실제 관리자인지 확인 (현재 TODO 상태)
3. **감사 로그**: 모든 승인 작업이 감사 로그에 기록
4. **트랜잭션**: 데이터 일관성 보장

---

## 에러 처리

### 회원가입 에러

```json
{
  "error": "Signup failed: Validation error: Password must contain at least one uppercase letter, one lowercase letter, and one number"
}
```

### 승인 에러

```json
{
  "error": "User approval failed: Not found: User not found"
}
```

---

## 관련 API

- **로그인**: `POST /api/auth/login` - 사용자 로그인
- **계정 삭제**: `DELETE /api/users/{user_id}` - 계정 삭제
- **사용자 조회**: `GET /api/users/{user_id}` - 사용자 정보 조회
- **사용자 목록**: `GET /api/users` - 사용자 목록 조회

---

## 구현 세부사항

### 회원가입 서비스 로직

```rust
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
    // 2. Keycloak에 사용자 생성 (enabled=false, emailVerified=true)
    // 3. DB에 사용자 생성 (account_status=ACTIVE)
    // 4. 감사 로그 기록
}
```

### 승인 서비스 로직

```rust
async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<(), ServiceError> {
    // 1. 트랜잭션 시작
    // 2. 사용자 조회 (keycloak_id 필요)
    // 3. Keycloak에서 사용자 활성화 (enabled=true)
    // 4. DB 상태 업데이트 (account_status=ACTIVE)
    // 5. 감사 로그 기록
    // 6. 트랜잭션 커밋
}
```

---

## 변경 이력

| Date | Version | Description |
|------|---------|-------------|
| 2025-10-28 | 1.0 | 초기 문서 작성 |
| 2025-10-28 | 1.1 | 회원가입 시 즉시 활성화 방식으로 변경 |
| 2025-10-28 | 1.2 | Keycloak 사용자 생성 시 enabled=false로 변경 |

---

## FAQ

### Q: 회원가입 후 바로 로그인할 수 있나요?
A: 아니요. 관리자 승인이 필요합니다. 회원가입 후에는 Keycloak에서 `enabled=false` 상태이므로 로그인이 불가능합니다.

### Q: 관리자 승인 없이 사용할 수 있나요?
A: 아니요. 관리자 승인은 필수입니다. 관리자가 `POST /api/auth/admin/users/approve`를 호출해야 사용자가 로그인할 수 있습니다.

### Q: 비밀번호 조건은 무엇인가요?
A: 최소 8자 이상이며, 대문자, 소문자, 숫자를 각각 최소 1개씩 포함해야 합니다. 특수문자는 권장됩니다.

### Q: 이메일 인증은 필수인가요?
A: 아니요. 회원가입 시 Keycloak에서 `emailVerified=true`로 설정되므로 별도의 이메일 인증 절차는 없습니다.

### Q: 관리자 승인 시 어떤 권한이 필요한가요?
A: 관리자 권한이 있는 JWT 토큰이 필요합니다. 현재는 실제 권한 검증이 TODO 상태입니다.

