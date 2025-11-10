# 인증 및 토큰 관리 API

## 개요

PACS Extension Server의 인증 시스템은 Keycloak과 JWT를 결합하여 사용합니다.
- **Keycloak**: 사용자 인증 및 관리 (백엔드에서 중계)
- **JWT**: API 요청 인증 토큰
- **클라이언트**: 이 서버만 통신하면 됨 (Keycloak 직접 통신 불필요)

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **인증 방식**: Bearer Token (JWT)
- **액세스 토큰 유효기간**: 5분 (300초)
- **Refresh Token 유효기간**: 30분 (1800초)
- **Refresh Token**: Keycloak에서 발급 (서버가 중계)

---

## API 엔드포인트

### 1. 로그인 (Login)

username/password로 Keycloak 인증 후 JWT 토큰과 refresh token을 발급받습니다.

**Endpoint**: `POST /api/auth/login`

**Request Body**:
```json
{
  "username": "john_doe",
  "password": "SecurePassword123!"
}
```

**Request Fields**:
| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| username | String | ✅ | 사용자명 |
| password | String | ✅ | 비밀번호 |

**Response (200 OK)**:
```json
{
  "user_id": 123,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 300,
  "refresh_expires_in": 1800
}
```

**Response Fields**:
| 필드 | 타입 | 설명 |
|------|------|------|
| user_id | Integer | 시스템 내부 사용자 ID |
| keycloak_id | UUID | Keycloak 사용자 ID |
| username | String | 사용자명 |
| email | String | 이메일 주소 |
| token | String | JWT 액세스 토큰 (API 요청에 사용) |
| refresh_token | String | Keycloak refresh token (토큰 갱신에 사용) |
| token_type | String | 토큰 타입 (항상 "Bearer") |
| expires_in | Integer | 액세스 토큰 만료 시간 (초 단위, 기본 300초 = 5분) |
| refresh_expires_in | Integer | Refresh token 만료 시간 (초 단위, 기본 1800초 = 30분) |

**Error Response (401 Unauthorized)**:
```json
{
  "error": "Login failed: Invalid username or password"
}
```

**사용 예시**:
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "password": "SecurePassword123!"
  }'
```

---

### 2. 토큰 검증 (Verify Token)

JWT 토큰의 유효성을 검증하고 사용자 정보를 반환합니다.

**Endpoint**: `GET /api/auth/verify/{token}`

**Path Parameters**:
| 파라미터 | 타입 | 필수 | 설명 |
|----------|------|------|------|
| token | String | ✅ | 검증할 JWT 토큰 |

**Response (200 OK)**:
```json
{
  "user_id": 123,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "is_valid": true
}
```

**Response Fields**:
| 필드 | 타입 | 설명 |
|------|------|------|
| user_id | Integer | 시스템 내부 사용자 ID |
| keycloak_id | UUID | Keycloak 사용자 ID |
| username | String | 사용자명 |
| email | String | 이메일 주소 |
| is_valid | Boolean | 토큰 유효성 여부 |

**Error Response (401 Unauthorized)**:
```json
{
  "valid": false,
  "error": "Token has expired"
}
```

**사용 예시**:
```bash
curl -X GET http://localhost:8080/api/auth/verify/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

### 3. 토큰 갱신 (Refresh Token)

Keycloak의 refresh token을 사용하여 새로운 access token과 refresh token을 발급받습니다.

**Endpoint**: `POST /api/auth/refresh`

**Request Body**:
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Request Fields**:
| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| refresh_token | String | ✅ | 로그인 시 받은 refresh token |

**Response (200 OK)**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 300,
  "refresh_expires_in": 1800
}
```

**Response Fields**:
| 필드 | 타입 | 설명 |
|------|------|------|
| token | String | 새로 발급된 JWT 액세스 토큰 |
| refresh_token | String | 새로 발급된 refresh token |
| token_type | String | 토큰 타입 (항상 "Bearer") |
| expires_in | Integer | 액세스 토큰 만료 시간 (초 단위) |
| refresh_expires_in | Integer | Refresh token 만료 시간 (초 단위) |

**Error Response (401 Unauthorized)**:
```json
{
  "error": "Token refresh failed: Invalid refresh token"
}
```

**사용 예시**:
```bash
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'
```

---

## 인증 플로우

### 1. 초기 로그인 플로우

```mermaid
sequenceDiagram
    participant Client
    participant API Server
    participant Keycloak
    participant Database

    Client->>API Server: 1. POST /api/auth/login (username, password)
    API Server->>Keycloak: 2. 사용자 인증 요청
    Keycloak-->>API Server: 3. access_token, refresh_token
    API Server->>Database: 4. 사용자 조회
    Database-->>API Server: 5. User 정보
    API Server->>API Server: 6. JWT 토큰 생성
    API Server-->>Client: 7. JWT token + refresh_token + user info
    Client->>Client: 8. 토큰 저장 (localStorage/sessionStorage)
```

### 2. API 요청 플로우

```mermaid
sequenceDiagram
    participant Client
    participant API Server
    participant JWT Service

    Client->>API Server: 1. API 요청 (Authorization: Bearer {token})
    API Server->>JWT Service: 2. 토큰 검증
    JWT Service-->>API Server: 3. Claims (user_id, keycloak_id, etc.)
    API Server->>API Server: 4. 비즈니스 로직 처리
    API Server-->>Client: 5. 응답 데이터
```

### 3. 토큰 갱신 플로우

```mermaid
sequenceDiagram
    participant Client
    participant API Server
    participant Keycloak

    Client->>Client: 1. JWT 토큰 만료 감지
    Client->>API Server: 2. POST /api/auth/refresh (refresh_token)
    API Server->>Keycloak: 3. Refresh token 검증 및 갱신 요청
    Keycloak-->>API Server: 4. 새로운 access_token
    API Server-->>Client: 5. 새로운 JWT token
    Client->>Client: 6. 토큰 업데이트
```

---

## 토큰 사용 방법

### HTTP 헤더에 토큰 포함

모든 인증이 필요한 API 요청에는 다음과 같이 Authorization 헤더를 포함해야 합니다:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**예시**:
```bash
curl -X GET http://localhost:8080/api/projects \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

## JWT 토큰 구조

### Claims 정보

JWT 토큰에는 다음 정보가 포함됩니다:

```json
{
  "sub": "123",
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "exp": 1735689600,
  "iat": 1735603200
}
```

| 필드 | 설명 |
|------|------|
| sub | Subject - 사용자 ID (문자열) |
| keycloak_id | Keycloak 사용자 UUID |
| username | 사용자명 |
| email | 이메일 주소 |
| exp | Expiration Time - 만료 시간 (Unix timestamp) |
| iat | Issued At - 발급 시간 (Unix timestamp) |

### 토큰 디코딩 예시 (JavaScript)

```javascript
function parseJwt(token) {
  const base64Url = token.split('.')[1];
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
  const jsonPayload = decodeURIComponent(
    atob(base64)
      .split('')
      .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
      .join('')
  );
  return JSON.parse(jsonPayload);
}

const token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
const claims = parseJwt(token);
console.log(claims.username); // "john_doe"
```

---

## 에러 코드

| HTTP 상태 코드 | 설명 | 해결 방법 |
|---------------|------|----------|
| 200 OK | 요청 성공 | - |
| 401 Unauthorized | 인증 실패 또는 토큰 만료 | 로그인 다시 시도 또는 토큰 갱신 |
| 400 Bad Request | 잘못된 요청 형식 | 요청 데이터 확인 |
| 500 Internal Server Error | 서버 내부 오류 | 서버 로그 확인 |

---

## 보안 고려사항

### 1. 토큰 저장
- ✅ **권장**: `httpOnly` 쿠키 또는 메모리에 저장
- ⚠️ **주의**: `localStorage`는 XSS 공격에 취약
- ❌ **비권장**: URL 파라미터에 토큰 포함

### 2. HTTPS 사용
- 프로덕션 환경에서는 반드시 HTTPS를 사용하여 토큰 탈취 방지

### 3. 토큰 만료 처리
- 클라이언트에서 토큰 만료 시간을 추적하고 자동으로 갱신
- 만료된 토큰으로 요청 시 401 에러 발생

### 4. Refresh Token 관리
- Refresh token은 안전하게 저장 (httpOnly 쿠키 권장)
- Refresh token도 만료되면 재로그인 필요

---

## 클라이언트 구현 예시

### React + Axios 예시

```javascript
import axios from 'axios';

const API_BASE_URL = 'http://localhost:8080/api';

// Axios 인스턴스 생성
const apiClient = axios.create({
  baseURL: API_BASE_URL,
});

// 요청 인터셉터: 모든 요청에 토큰 추가
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('access_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// 응답 인터셉터: 401 에러 시 토큰 갱신
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;

    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      try {
        const refreshToken = localStorage.getItem('refresh_token');
        const response = await axios.post(`${API_BASE_URL}/auth/refresh`, {
          refresh_token: refreshToken,
        });

        const { token } = response.data;
        localStorage.setItem('access_token', token);

        originalRequest.headers.Authorization = `Bearer ${token}`;
        return apiClient(originalRequest);
      } catch (refreshError) {
        // Refresh 실패 시 로그인 페이지로 리다이렉트
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        window.location.href = '/login';
        return Promise.reject(refreshError);
      }
    }

    return Promise.reject(error);
  }
);

// 로그인 함수
export async function login(keycloakId, username, email) {
  const response = await apiClient.post('/auth/login', {
    keycloak_id: keycloakId,
    username,
    email,
  });

  const { token } = response.data;
  localStorage.setItem('access_token', token);
  
  return response.data;
}

// 토큰 검증 함수
export async function verifyToken(token) {
  const response = await apiClient.get(`/auth/verify/${token}`);
  return response.data;
}

export default apiClient;
```

---

## 테스트

### 로그인 테스트
```bash
# 1. 로그인
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com"
  }'

# 응답에서 token 값을 복사
```

### 토큰으로 API 호출 테스트
```bash
# 2. 토큰을 사용하여 프로젝트 목록 조회
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

curl -X GET http://localhost:8080/api/projects \
  -H "Authorization: Bearer $TOKEN"
```

### 토큰 검증 테스트
```bash
# 3. 토큰 검증
curl -X GET http://localhost:8080/api/auth/verify/$TOKEN
```

---

## 관련 문서

- [사용자 관리 시스템](../technical/USER_MANAGEMENT_SYSTEM.md)
- [Keycloak 설정 가이드](../setup/KEYCLOAK_SETUP.md)
- [API 전체 문서](./API_REFERENCE.md)

