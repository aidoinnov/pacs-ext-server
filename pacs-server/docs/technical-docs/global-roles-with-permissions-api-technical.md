# Global Roles with Permissions API 기술 문서

## 📋 개요

이 문서는 PACS 서버의 Global Roles with Permissions API 구현에 대한 상세한 기술 문서입니다. Clean Architecture 패턴을 따라 구현된 이 API는 글로벌 역할 목록을 권한 정보와 함께 페이지네이션으로 조회하는 기능을 제공합니다.

## 🏗️ 아키텍처 설계

### Clean Architecture 계층 구조

```
┌─────────────────────────────────────────┐
│           Presentation Layer            │
│        (Controllers, Routes)           │
├─────────────────────────────────────────┤
│           Application Layer             │
│        (Use Cases, DTOs)               │
├─────────────────────────────────────────┤
│             Domain Layer                │
│      (Entities, Services, Repos)        │
├─────────────────────────────────────────┤
│         Infrastructure Layer            │
│    (Database, External Services)       │
└─────────────────────────────────────────┘
```

### 의존성 방향
- Presentation → Application → Domain
- Infrastructure → Domain
- **핵심 원칙**: Domain 계층은 다른 계층에 의존하지 않음

## 🔧 핵심 컴포넌트

### 1. Domain Layer

#### Entities
```rust
// src/domain/entities/permission.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i32,
    pub resource_type: String,
    pub action: String,
}
```

#### Services
```rust
// src/domain/services/permission_service.rs
#[async_trait]
pub trait PermissionService: Send + Sync {
    async fn get_global_roles(&self) -> Result<Vec<Role>, ServiceError>;
    async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>, ServiceError>;
}
```

### 2. Application Layer

#### DTOs
```rust
// src/application/dto/permission_dto.rs
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RoleWithPermissionsResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub permissions: Vec<PermissionResponse>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RolesWithPermissionsListResponse {
    pub roles: Vec<RoleWithPermissionsResponse>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}
```

#### Use Cases
```rust
// src/application/use_cases/permission_use_case.rs
impl<P: PermissionService> PermissionUseCase<P> {
    pub async fn get_global_roles_with_permissions(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<RolesWithPermissionsListResponse, ServiceError> {
        // 페이지네이션 파라미터 검증
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        // 전체 Global 역할 조회
        let all_roles = self.permission_service.get_global_roles().await?;
        let total_count = all_roles.len() as i64;

        // 페이지네이션 적용
        let paginated_roles: Vec<_> = all_roles
            .into_iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .collect();

        // 각 역할의 권한 조회
        let mut roles_with_permissions = Vec::new();
        for role in paginated_roles {
            let permissions = self.permission_service
                .get_role_permissions(role.id)
                .await?;

            roles_with_permissions.push(RoleWithPermissionsResponse {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
                permissions: permissions
                    .into_iter()
                    .map(|p| PermissionResponse {
                        id: p.id,
                        resource_type: p.resource_type,
                        action: p.action,
                    })
                    .collect(),
            });
        }

        let total_pages = ((total_count + page_size as i64 - 1) / page_size as i64) as i32;

        Ok(RolesWithPermissionsListResponse {
            roles: roles_with_permissions,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }
}
```

### 3. Presentation Layer

#### Controllers
```rust
// src/presentation/controllers/permission_controller.rs
#[utoipa::path(
    get,
    path = "/api/roles/global/with-permissions",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20, max: 100)")
    ),
    responses(
        (status = 200, description = "Global roles with permissions retrieved successfully", body = RolesWithPermissionsListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "roles"
)]
pub async fn get_global_roles_with_permissions(
    permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    match permission_use_case
        .get_global_roles_with_permissions(query.page, query.page_size)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get global roles with permissions: {}", e)
        })),
    }
}
```

#### Routing
```rust
pub fn configure_routes<P: PermissionService + 'static>(
    cfg: &mut web::ServiceConfig,
    permission_use_case: Arc<PermissionUseCase<P>>,
) {
    cfg.app_data(web::Data::new(permission_use_case))
        .service(
            web::scope("/roles")
                .route("", web::post().to(PermissionController::<P>::create_role))
                .route("/global", web::get().to(PermissionController::<P>::get_global_roles))
                .route("/global/with-permissions", web::get().to(PermissionController::<P>::get_global_roles_with_permissions))
                .route("/project", web::get().to(PermissionController::<P>::get_project_roles))
                .route("/{role_id}", web::get().to(PermissionController::<P>::get_role)),
        );
}
```

## 🗄️ 데이터베이스 설계

### 테이블 구조
```sql
-- 역할 테이블
CREATE TABLE security_role (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    scope TEXT NOT NULL CHECK (scope IN ('GLOBAL', 'PROJECT')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 권한 테이블
CREATE TABLE security_permission (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    resource_type TEXT NOT NULL,
    action TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(resource_type, action)
);

-- 역할-권한 관계 테이블
CREATE TABLE security_role_permission (
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permission(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
```

### 인덱스 최적화
```sql
-- 역할 조회 최적화
CREATE INDEX idx_role_scope ON security_role(scope);

-- 권한 조회 최적화
CREATE INDEX idx_permission_resource_type ON security_permission(resource_type);
CREATE INDEX idx_permission_action ON security_permission(action);

-- 관계 테이블 최적화
CREATE INDEX idx_role_permission_role_id ON security_role_permission(role_id);
CREATE INDEX idx_role_permission_permission_id ON security_role_permission(permission_id);
```

## 🧪 테스트 전략

### 1. 단위 테스트

#### DTO 테스트
```rust
// tests/permission_dto_test.rs
#[test]
fn test_role_with_permissions_response_serialization() {
    let permission = PermissionResponse {
        id: 1,
        resource_type: "user".to_string(),
        action: "create".to_string(),
    };
    
    let role = RoleWithPermissionsResponse {
        id: 101,
        name: "Admin".to_string(),
        description: Some("System Administrator".to_string()),
        scope: "GLOBAL".to_string(),
        permissions: vec![permission],
    };
    
    let serialized = serde_json::to_string(&role).unwrap();
    assert!(serialized.contains("\"id\":101"));
    assert!(serialized.contains("\"name\":\"Admin\""));
}
```

#### Use Case 테스트
```rust
// tests/permission_use_case_test.rs
#[tokio::test]
async fn test_get_global_roles_with_permissions_empty() {
    let mut mock_service = MockPermissionService::new();
    mock_service.expect_get_global_roles()
        .once()
        .return_once(|_| Ok(vec![]));
    
    let use_case = PermissionUseCase::new(Arc::new(mock_service));
    let result = use_case.get_global_roles_with_permissions(None, None).await.unwrap();
    
    assert_eq!(result.roles.len(), 0);
    assert_eq!(result.total_count, 0);
    assert_eq!(result.page, 1);
    assert_eq!(result.page_size, 20);
    assert_eq!(result.total_pages, 0);
}
```

### 2. 통합 테스트

#### HTTP API 테스트
```bash
# scripts/test_integration.sh
test_basic_pagination() {
    local response=$(make_request "GET" "$API_BASE/roles/global/with-permissions" "" "200")
    
    if [ $? -eq 0 ]; then
        validate_json "$response" "roles"
        validate_json "$response" "total_count"
        validate_json "$response" "page"
        validate_json "$response" "page_size"
        validate_json "$response" "total_pages"
        
        local page=$(echo "$response" | jq -r '.page')
        local page_size=$(echo "$response" | jq -r '.page_size')
        
        if [ "$page" = "1" ] && [ "$page_size" = "20" ]; then
            log_success "기본 페이지네이션 값 확인"
        fi
    fi
}
```

#### Mock 서버 테스트
```python
# test_server.py
class TestServerHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if path == "/api/roles/global/with-permissions":
            # Mock 데이터 생성
            mock_roles = [
                {
                    "id": 1,
                    "name": "시스템 관리자",
                    "description": "전체 시스템 관리 권한",
                    "scope": "GLOBAL",
                    "permissions": [
                        {"id": 1, "resource_type": "user", "action": "create"},
                        # ... 더 많은 권한
                    ]
                }
                # ... 더 많은 역할
            ]
            
            # 페이지네이션 적용
            total_count = len(mock_roles)
            start_idx = (page - 1) * page_size
            end_idx = start_idx + page_size
            paginated_roles = mock_roles[start_idx:end_idx]
            
            response = {
                "roles": paginated_roles,
                "total_count": total_count,
                "page": page,
                "page_size": page_size,
                "total_pages": (total_count + page_size - 1) // page_size
            }
```

## 📊 성능 최적화

### 1. 데이터베이스 최적화

#### 쿼리 최적화
```sql
-- 역할과 권한을 함께 조회하는 최적화된 쿼리
SELECT 
    r.id, r.name, r.description, r.scope,
    p.id as permission_id, p.resource_type, p.action
FROM security_role r
LEFT JOIN security_role_permission rp ON r.id = rp.role_id
LEFT JOIN security_permission p ON rp.permission_id = p.id
WHERE r.scope = 'GLOBAL'
ORDER BY r.id, p.id;
```

#### 인덱스 전략
- 복합 인덱스 활용
- 쿼리 패턴 분석
- 실행 계획 최적화

### 2. 메모리 최적화

#### 데이터 구조 최적화
```rust
// 효율적인 메모리 사용을 위한 구조체 설계
#[derive(Debug, Clone)]
pub struct RoleWithPermissions {
    pub role: Role,
    pub permissions: Vec<Permission>,
}

// 참조 기반 처리로 메모리 사용량 최적화
impl RoleWithPermissions {
    pub fn from_role_and_permissions(role: Role, permissions: Vec<Permission>) -> Self {
        Self { role, permissions }
    }
}
```

### 3. 캐싱 전략

#### Redis 캐싱 (향후 구현)
```rust
// 캐시 키 전략
const CACHE_KEY_PREFIX: &str = "roles:global:";
const CACHE_TTL: u64 = 300; // 5분

impl PermissionUseCase {
    async fn get_cached_roles(&self, page: i32, page_size: i32) -> Option<RolesWithPermissionsListResponse> {
        let cache_key = format!("{}:{}:{}", CACHE_KEY_PREFIX, page, page_size);
        // Redis에서 조회
    }
    
    async fn set_cache(&self, page: i32, page_size: i32, data: &RolesWithPermissionsListResponse) {
        let cache_key = format!("{}:{}:{}", CACHE_KEY_PREFIX, page, page_size);
        // Redis에 저장
    }
}
```

## 🔒 보안 고려사항

### 1. 인증 및 권한

#### JWT 토큰 검증
```rust
// 미들웨어에서 토큰 검증
pub async fn authenticate_request(
    req: &ServiceRequest,
    jwt_service: &JwtService,
) -> Result<User, AuthError> {
    let auth_header = req.headers().get("Authorization");
    match auth_header {
        Some(header) => {
            let token = extract_bearer_token(header.to_str().unwrap())?;
            jwt_service.verify_token(&token).await
        }
        None => Err(AuthError::MissingToken),
    }
}
```

#### 역할 기반 접근 제어
```rust
// 권한 확인 로직
pub async fn check_permission(
    user_id: i32,
    resource_type: &str,
    action: &str,
    permission_service: &PermissionService,
) -> Result<bool, ServiceError> {
    let user_roles = permission_service.get_user_roles(user_id).await?;
    for role in user_roles {
        let permissions = permission_service.get_role_permissions(role.id).await?;
        for permission in permissions {
            if permission.resource_type == resource_type && permission.action == action {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
```

### 2. 입력 검증

#### DTO 검증
```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1, max = 1000))]
    pub page: Option<i32>,
    
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<i32>,
}
```

#### SQL 인젝션 방지
```rust
// SQLx를 사용한 안전한 쿼리
async fn get_roles_with_permissions(
    pool: &PgPool,
    scope: &str,
) -> Result<Vec<Role>, sqlx::Error> {
    sqlx::query_as!(
        Role,
        "SELECT id, name, description, scope FROM security_role WHERE scope = $1",
        scope
    )
    .fetch_all(pool)
    .await
}
```

## 📈 모니터링 및 로깅

### 1. 구조화된 로깅

#### 로그 레벨별 분류
```rust
use tracing::{info, warn, error, debug};

impl PermissionUseCase {
    pub async fn get_global_roles_with_permissions(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<RolesWithPermissionsListResponse, ServiceError> {
        info!("Getting global roles with permissions: page={:?}, page_size={:?}", page, page_size);
        
        let start_time = std::time::Instant::now();
        
        // 비즈니스 로직 실행
        let result = self.execute_business_logic(page, page_size).await;
        
        let duration = start_time.elapsed();
        info!("Global roles query completed in {:?}", duration);
        
        result
    }
}
```

### 2. 메트릭 수집

#### 성능 메트릭
```rust
// 메트릭 수집 구조체
#[derive(Debug, Clone)]
pub struct ApiMetrics {
    pub request_count: u64,
    pub response_time_ms: u64,
    pub error_count: u64,
    pub cache_hit_rate: f64,
}

impl ApiMetrics {
    pub fn record_request(&mut self, response_time_ms: u64, is_error: bool) {
        self.request_count += 1;
        self.response_time_ms = response_time_ms;
        if is_error {
            self.error_count += 1;
        }
    }
}
```

## 🚀 배포 및 운영

### 1. Docker 컨테이너화

#### Dockerfile 최적화
```dockerfile
# Multi-stage build for production
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pacs_server /usr/local/bin/
EXPOSE 8080
CMD ["pacs_server"]
```

### 2. 환경 설정

#### 설정 관리
```rust
// config/settings.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}
```

## 🔄 확장성 고려사항

### 1. 수평적 확장

#### 로드 밸런싱
```yaml
# docker-compose.yml
version: '3.8'
services:
  pacs-server:
    image: pacs-server:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/pacs
    depends_on:
      - db
      - redis

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - pacs-server
```

### 2. 데이터베이스 확장

#### 읽기 전용 복제본
```rust
// 읽기/쓰기 분리
pub struct DatabasePool {
    pub write_pool: PgPool,
    pub read_pool: PgPool,
}

impl DatabasePool {
    pub async fn get_roles(&self) -> Result<Vec<Role>, sqlx::Error> {
        // 읽기 전용 복제본 사용
        self.read_pool.acquire().await?.query_as("SELECT * FROM security_role")
    }
}
```

## 📚 API 문서

### OpenAPI 스키마
```yaml
# openapi.yaml
paths:
  /api/roles/global/with-permissions:
    get:
      summary: Get global roles with permissions
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            minimum: 1
            default: 1
        - name: page_size
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/RolesWithPermissionsListResponse'
```

## 🐛 문제 해결 가이드

### 1. 일반적인 문제

#### 메모리 부족
```bash
# 메모리 사용량 모니터링
docker stats pacs-server

# 메모리 제한 설정
docker run --memory=512m pacs-server
```

#### 데이터베이스 연결 문제
```rust
// 연결 풀 설정
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
}
```

### 2. 성능 문제

#### 느린 쿼리 분석
```sql
-- 쿼리 실행 계획 분석
EXPLAIN ANALYZE 
SELECT r.*, p.* 
FROM security_role r
LEFT JOIN security_role_permission rp ON r.id = rp.role_id
LEFT JOIN security_permission p ON rp.permission_id = p.id
WHERE r.scope = 'GLOBAL';
```

#### 인덱스 최적화
```sql
-- 복합 인덱스 생성
CREATE INDEX idx_role_scope_name ON security_role(scope, name);

-- 부분 인덱스 생성
CREATE INDEX idx_active_roles ON security_role(id) WHERE scope = 'GLOBAL';
```

---

**문서 버전**: 1.0  
**최종 업데이트**: 2025-01-24  
**다음 리뷰**: 2025-02-01
