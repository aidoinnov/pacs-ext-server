# 토큰 갱신 API 기술 문서

## 1. 아키텍처 개요

### 1.1 전체 구조
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client        │    │   PACS Server   │    │   Keycloak      │
│                 │    │                 │    │                 │
│ POST /refresh   │───▶│ AuthController  │───▶│ /token endpoint │
│                 │    │                 │    │                 │
│ RefreshToken    │◀───│ AuthUseCase     │◀───│ TokenResponse   │
│ Response        │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 1.2 Clean Architecture 계층
```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              AuthController                         │   │
│  │  - refresh_token() handler                         │   │
│  │  - HTTP request/response handling                  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              AuthUseCase                            │   │
│  │  - refresh_token() business logic                  │   │
│  │  - DTO transformation                              │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              AuthService                            │   │
│  │  - refresh_token_with_keycloak() interface         │   │
│  │  - Business rules and validation                   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              KeycloakClient                         │   │
│  │  - refresh_access_token() implementation           │   │
│  │  - HTTP communication with Keycloak                │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 2. 구현 세부사항

### 2.1 KeycloakClient 구현

#### 2.1.1 구조체 정의
```rust
#[derive(Clone)]
pub struct KeycloakClient {
    base_url: String,
    realm: String,
    client_id: String,
    admin_username: String,
    admin_password: String,
    http_client: Client,
}
```

#### 2.1.2 refresh_access_token 메서드
```rust
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
```

### 2.2 DTO 정의

#### 2.2.1 KeycloakTokenResponse
```rust
#[derive(Deserialize, Debug, Clone)]
pub struct KeycloakTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub token_type: String,
}
```

#### 2.2.2 RefreshTokenRequest
```rust
#[derive(Serialize)]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}
```

### 2.3 AuthService 구현

#### 2.3.1 인터페이스 정의
```rust
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn refresh_token_with_keycloak(&self, refresh_token: &str) -> Result<RefreshTokenResponse, ServiceError>;
}
```

#### 2.3.2 구현체
```rust
pub struct AuthServiceImpl<U: UserRepository> {
    user_repository: U,
    jwt_service: JwtService,
    keycloak_client: Arc<KeycloakClient>,
}

#[async_trait]
impl<U: UserRepository> AuthService for AuthServiceImpl<U> {
    async fn refresh_token_with_keycloak(&self, refresh_token: &str) -> Result<RefreshTokenResponse, ServiceError> {
        let keycloak_response = self.keycloak_client.refresh_access_token(refresh_token).await?;
        
        Ok(RefreshTokenResponse {
            token: keycloak_response.access_token,
            token_type: keycloak_response.token_type,
            expires_in: keycloak_response.expires_in,
        })
    }
}
```

### 2.4 AuthUseCase 구현

```rust
impl<A: AuthService> AuthUseCase<A> {
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<RefreshTokenResponse, ServiceError> {
        self.auth_service.refresh_token_with_keycloak(&request.refresh_token).await
    }
}
```

### 2.5 AuthController 구현

```rust
impl<A: AuthService> AuthController<A> {
    pub async fn refresh_token(
        auth_use_case: web::Data<Arc<AuthUseCase<A>>>,
        req: web::Json<RefreshTokenRequest>,
    ) -> impl Responder {
        match auth_use_case.refresh_token(req.into_inner()).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::Unauthorized().json(json!({
                "error": format!("Token refresh failed: {}", e)
            })),
        }
    }
}
```

## 3. API 명세

### 3.1 엔드포인트
- **URL**: `POST /api/auth/refresh`
- **Content-Type**: `application/json`
- **Authentication**: None (refresh token을 body에 포함)

### 3.2 요청 스키마
```json
{
  "refresh_token": "string"
}
```

### 3.3 응답 스키마

#### 성공 응답 (200 OK)
```json
{
  "token": "string",
  "token_type": "string",
  "expires_in": "number"
}
```

#### 에러 응답 (401 Unauthorized)
```json
{
  "error": "string"
}
```

### 3.4 에러 코드
- `400 Bad Request`: 잘못된 요청 형식
- `401 Unauthorized`: 유효하지 않은 refresh token
- `500 Internal Server Error`: 서버 내부 오류

## 4. 테스트 전략

### 4.1 단위 테스트
- **KeycloakClient**: HTTP 요청/응답 모킹
- **AuthService**: KeycloakClient 모킹
- **AuthUseCase**: AuthService 모킹
- **AuthController**: 전체 플로우 모킹

### 4.2 통합 테스트
- Mockito를 사용한 Keycloak 서버 모킹
- 실제 HTTP 요청/응답 테스트
- 에러 시나리오 테스트

### 4.3 성능 테스트
- 응답 시간 측정
- 동시 요청 처리 테스트
- 메모리 사용량 모니터링

## 5. 보안 고려사항

### 5.1 토큰 보안
- Refresh token은 HTTPS를 통해서만 전송
- 토큰 만료 시간은 Keycloak에서 관리
- Refresh token rotation 지원

### 5.2 에러 처리
- 민감한 정보는 로그에 기록하지 않음
- 일반적인 에러 메시지만 클라이언트에 전달
- 상세한 에러는 서버 로그에만 기록

### 5.3 Rate Limiting
- Keycloak의 기본 rate limiting 활용
- 추가적인 rate limiting은 필요시 구현

## 6. 모니터링 및 로깅

### 6.1 로깅 포인트
- 토큰 갱신 요청 시작
- Keycloak 응답 수신
- 에러 발생 시 상세 정보
- 성능 메트릭

### 6.2 메트릭
- 토큰 갱신 성공/실패율
- 평균 응답 시간
- Keycloak 서버 응답 시간
- 에러 유형별 통계

## 7. 배포 및 운영

### 7.1 환경 변수
```bash
KEYCLOAK_URL=https://keycloak.example.com
KEYCLOAK_REALM=dcm4che
KEYCLOAK_CLIENT_ID=pacs-server
KEYCLOAK_CLIENT_SECRET=your-secret
KEYCLOAK_ADMIN_USERNAME=admin
KEYCLOAK_ADMIN_PASSWORD=adminPassword123!
```

### 7.2 헬스 체크
- Keycloak 서버 연결 상태 확인
- 토큰 갱신 엔드포인트 가용성 확인

### 7.3 장애 대응
- Keycloak 서버 장애 시 적절한 에러 응답
- 재시도 로직 구현 (필요시)
- 대체 인증 방식 준비

## 8. 확장성 고려사항

### 8.1 캐싱
- 현재는 캐싱 없이 구현
- 필요시 Redis를 통한 토큰 캐싱 고려

### 8.2 로드 밸런싱
- Keycloak 클러스터 지원
- 여러 Keycloak 인스턴스 간 로드 밸런싱

### 8.3 모니터링 확장
- Prometheus 메트릭 수집
- Grafana 대시보드 구성
- 알림 설정
