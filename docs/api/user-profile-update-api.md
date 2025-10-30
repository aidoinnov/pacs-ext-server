
# 사용자 프로필 업데이트 API

## 개요

사용자의 프로필 정보(이메일, 실명, 소속 기관, 부서, 연락처)를 업데이트하는 API입니다.

**Base URL**: `/api/users`

---

## API 엔드포인트

### 사용자 프로필 업데이트

사용자의 프로필 정보를 업데이트합니다.

**Endpoint**: `PUT /api/users/{user_id}`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 업데이트할 사용자 ID |

#### Request Body

부분 업데이트를 지원합니다. 업데이트하지 않는 필드는 `null` 또는 포함하지 않으면 됩니다.

**Request DTO**: `UpdateUserRequest`

```json
{
  "email": "newemail@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

**Request Schema**:

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `email` | string | No | 이메일 주소 | `"hong@example.com"` |
| `full_name` | string | No | 사용자의 실명 | `"홍길동"` |
| `organization` | string | No | 소속 기관 | `"서울대학교병원"` |
| `department` | string | No | 소속 부서/그룹 | `"영상의학과"` |
| `phone` | string | No | 연락처 | `"010-1234-5678"` |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 123,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "honggildong",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-02T00:00:00Z"
}
```

**Response Schema**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | 사용자 ID |
| `keycloak_id` | string (UUID) | Keycloak 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `full_name` | string | 사용자의 실명 |
| `organization` | string | 소속 기관 |
| `department` | string | 소속 부서/그룹 |
| `phone` | string | 연락처 |
| `created_at` | string (ISO 8601) | 생성 시간 |
| `updated_at` | string (ISO 8601) | 업데이트 시간 |

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "Failed to update user: ..."}` |
| 404 | User not found | `{"error": "Failed to update user: User not found"}` |
| 409 | Email already taken | `{"error": "Failed to update user: Email already taken"}` |
| 500 | Internal server error | `{"error": "Failed to update user: ..."}` |

---

## 사용 예시

### cURL 요청 예시

#### 전체 프로필 업데이트

```bash
curl -X PUT "http://localhost:8080/api/users/123" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newemail@example.com",
    "full_name": "홍길동",
    "organization": "서울대학교병원",
    "department": "영상의학과",
    "phone": "010-1234-5678"
  }'
```

#### 부분 업데이트 (이메일만 변경)

```bash
curl -X PUT "http://localhost:8080/api/users/123" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newemail@example.com"
  }'
```

#### 부분 업데이트 (소속 정보만 변경)

```bash
curl -X PUT "http://localhost:8080/api/users/123" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "organization": "세종대학교병원",
    "department": "방사선종양학과"
  }'
```

### JavaScript (fetch) 예시

```javascript
async function updateUserProfile(userId, profileData) {
  try {
    const response = await fetch(`http://localhost:8080/api/users/${userId}`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${yourJwtToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        email: profileData.email,
        full_name: profileData.fullName,
        organization: profileData.organization,
        department: profileData.department,
        phone: profileData.phone
      })
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const updatedUser = await response.json();
    console.log('Profile updated:', updatedUser);
    return updatedUser;
  } catch (error) {
    console.error('Error updating profile:', error);
    throw error;
  }
}

// 사용 예시
updateUserProfile(123, {
  email: 'newemail@example.com',
  fullName: '홍길동',
  organization: '서울대학교병원',
  department: '영상의학과',
  phone: '010-1234-5678'
});
```

### TypeScript 예시

```typescript
interface UpdateUserRequest {
  email?: string;
  full_name?: string;
  organization?: string;
  department?: string;
  phone?: string;
}

interface UserResponse {
  id: number;
  keycloak_id: string;
  username: string;
  email: string;
  full_name: string | null;
  organization: string | null;
  department: string | null;
  phone: string | null;
  created_at: string;
  updated_at: string | null;
}

async function updateUserProfile(
  userId: number,
  updateData: UpdateUserRequest
): Promise<UserResponse> {
  const response = await fetch(`http://localhost:8080/api/users/${userId}`, {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${getJwtToken()}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(updateData)
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to update user profile');
  }

  return response.json();
}
```

---

## 업데이트 동작 방식

### COALESCE를 사용한 부분 업데이트

데이터베이스 쿼리에서 `COALESCE`를 사용하여 부분 업데이트를 구현합니다:

```sql
UPDATE security_user 
SET email = COALESCE($2, email),
    full_name = COALESCE($3, full_name),
    organization = COALESCE($4, organization),
    department = COALESCE($5, department),
    phone = COALESCE($6, phone)
WHERE id = $1
RETURNING ...
```

**동작 방식**:
- 요청에 포함된 필드는 새 값으로 업데이트
- `null`로 전달된 필드는 기존 값 유지
- 요청에 포함되지 않은 필드는 기존 값 유지

### 이메일 중복 검사

이메일을 변경할 때 다른 사용자가 이미 사용 중인 이메일인지 확인합니다:

```rust
if let Some(ref email) = update_user.email {
    if let Some(existing_user) = self.user_repository.find_by_email(email).await? {
        if existing_user.id != update_user.id {
            return Err(ServiceError::AlreadyExists("Email already taken".into()));
        }
    }
}
```

**에러 상황**:
- 다른 사용자가 이미 사용 중인 이메일로 변경하려고 하면 `409 Conflict` 응답

### 사용자 존재 확인

업데이트 전에 사용자가 존재하는지 확인합니다:

```rust
self.user_repository
    .find_by_id(update_user.id)
    .await?
    .ok_or(ServiceError::NotFound("User not found".into()))?;
```

**에러 상황**:
- 존재하지 않는 사용자 ID로 요청하면 `404 Not Found` 응답

---

## 필드 설명

### 필수 수정 불가능한 필드

다음 필드들은 업데이트할 수 없습니다:

- `id`: 사용자 ID (변경 불가)
- `keycloak_id`: Keycloak 연동 ID (변경 불가)
- `username`: 사용자명 (변경 불가)

### 수정 가능한 필드

- **email**: 이메일 주소 (다른 사용자가 사용 중인지 확인)
- **full_name**: 사용자의 실명
- **organization**: 소속 기관
- **department**: 소속 부서/그룹
- **phone**: 연락처

---

## 에러 처리

### 공통 에러 응답 형식

```json
{
  "error": "Failed to update user: [에러 메시지]"
}
```

### 에러 코드별 설명

| HTTP Status | 에러 내용 | 설명 |
|-------------|---------|------|
| 400 | Validation error | 요청 데이터가 유효하지 않음 |
| 404 | User not found | 사용자가 존재하지 않음 |
| 409 | Email already taken | 이메일이 이미 사용 중임 |
| 500 | Internal server error | 서버 내부 오류 |

---

## 구현 세부사항

### 레이어별 구현

#### 1. Controller (`user_controller.rs`)

```rust
pub async fn update_user<U: UserService + 'static>(
    user_use_case: web::Data<Arc<UserUseCase<U>>>,
    path: web::Path<i32>,
    req: web::Json<UpdateUserRequest>,
) -> impl Responder {
    let user_id = path.into_inner();
    
    match user_use_case.update_user(user_id, req.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            let status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                ServiceError::AlreadyExists(_) => HttpResponse::Conflict(),
                ServiceError::ValidationError(_) => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("Failed to update user: {}", e)}))
        }
    }
}
```

#### 2. Use Case (`user_use_case.rs`)

```rust
pub async fn update_user(&self, user_id: i32, request: UpdateUserRequest) -> Result<UserResponse, ServiceError> {
    let update_user = UpdateUser {
        id: user_id,
        email: request.email,
        full_name: request.full_name,
        organization: request.organization,
        department: request.department,
        phone: request.phone,
    };

    let user = self.user_service.update_user(update_user).await?;
    Ok(user.into())
}
```

#### 3. Service (`user_service.rs`)

```rust
async fn update_user(&self, update_user: UpdateUser) -> Result<User, ServiceError> {
    // 사용자 존재 여부 확인
    self.user_repository
        .find_by_id(update_user.id)
        .await?
        .ok_or(ServiceError::NotFound("User not found".into()))?;

    // 이메일 중복 검사
    if let Some(ref email) = update_user.email {
        if let Some(existing_user) = self.user_repository.find_by_email(email).await? {
            if existing_user.id != update_user.id {
                return Err(ServiceError::AlreadyExists("Email already taken".into()));
            }
        }
    }

    // 사용자 정보 업데이트
    Ok(self.user_repository.update(&update_user).await?)
}
```

#### 4. Repository (`user_repository_impl.rs`)

```rust
async fn update(&self, update_user: &UpdateUser) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE security_user 
         SET email = COALESCE($2, email),
             full_name = COALESCE($3, full_name),
             organization = COALESCE($4, organization),
             department = COALESCE($5, department),
             phone = COALESCE($6, phone)
         WHERE id = $1
         RETURNING id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at"
    )
    .bind(update_user.id)
    .bind(&update_user.email)
    .bind(&update_user.full_name)
    .bind(&update_user.organization)
    .bind(&update_user.department)
    .bind(&update_user.phone)
    .fetch_one(&self.pool)
    .await
}
```

---

## 테스트 예시

### Swagger UI에서 테스트

1. `http://localhost:8080/swagger-ui/` 접속
2. `/api/users/{user_id}` PUT 엔드포인트 선택
3. "Authorize" 버튼 클릭하여 JWT 토큰 입력
4. Path Parameter에 `user_id` 입력
5. Request Body에 업데이트할 필드 입력
6. "Execute" 버튼 클릭

### 로컬에서 테스트

```bash
# 사용자 프로필 업데이트
curl -X PUT "http://localhost:8080/api/users/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "hong@example.com",
    "full_name": "홍길동",
    "organization": "서울대학교병원",
    "department": "영상의학과",
    "phone": "010-1234-5678"
  }'
```

---

## 참고사항

### 제한사항

1. **username 변경 불가**: 사용자명은 변경할 수 없습니다. Keycloak 설정에서 변경이 가능합니다.
2. **이메일 중복 검사**: 다른 사용자가 이미 사용 중인 이메일로 변경할 수 없습니다.
3. **인증 필요**: 모든 요청은 유효한 JWT 토큰이 필요합니다.

### 보안 고려사항

1. **인증**: 현재 사용자만 자신의 프로필을 업데이트할 수 있도록 권한 검사가 필요합니다 (향후 구현 예정).
2. **입력 검증**: 이메일 형식, 전화번호 형식 등의 검증이 필요할 수 있습니다 (향후 구현 예정).
3. **SQL Injection 방지**: Prepared statement를 사용하여 SQL Injection을 방지합니다.

### 성능 고려사항

1. **부분 업데이트**: COALESCE를 사용하여 업데이트할 필드만 수정합니다.
2. **이메일 중복 검사**: 이메일 변경 시에만 중복 검사를 수행합니다.
3. **인덱스 활용**: `id`, `email` 컬럼에 인덱스가 있어 빠른 조회가 가능합니다.

---

## 관련 문서

- [사용자 관리 API](./user-management-api.md) (향후 작성 예정)
- [Keycloak 연동 문서](./keycloak-integration.md) (향후 작성 예정)
- [인증 및 권한 관리](./authentication-authorization.md) (향후 작성 예정)

---

**최종 업데이트**: 2025-01-27



