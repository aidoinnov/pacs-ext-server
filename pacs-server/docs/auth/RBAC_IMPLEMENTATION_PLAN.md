# RBAC (Role-Based Access Control) 구현 계획

## 개요

PACS Extension Server에 Role 기반 접근 제어 시스템을 구현하는 계획서입니다. 이는 AuthGuard JWT 인증 구현 완료 후 Phase 2로 진행되는 작업입니다.

## 목표

- 사용자 Role 기반 리소스 접근 제어
- 프로젝트별 권한 관리
- 리소스 소유자 확인
- 세밀한 권한 제어 (읽기, 쓰기, 삭제 등)

## 아키텍처

### 1. 권한 체계
```
User → Roles → Permissions → Resources
```

### 2. 권한 검증 흐름
```
Controller → Permission Guard → Use Case → Service
                ↓
            Role 검증
            Permission 확인
            리소스 소유자 확인
```

## 데이터 모델

### 1. Claims 확장
```rust
pub struct Claims {
    pub sub: String,           // User ID
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,    // 추가: 사용자 Role 목록
    pub iat: i64,
    pub exp: i64,
}
```

### 2. Role 구조
```rust
pub enum RoleScope {
    Global,    // 시스템 전체 권한
    Project,   // 프로젝트별 권한
}

pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: RoleScope,
    pub created_at: NaiveDateTime,
}
```

### 3. Permission 구조
```rust
pub struct Permission {
    pub id: i32,
    pub resource_type: String,  // "annotation", "mask_group", "project"
    pub action: String,         // "read", "write", "delete", "admin"
    pub description: Option<String>,
}
```

## 구현 세부사항

### 1. Permission Guard Middleware

#### 구조체
```rust
pub struct PermissionGuard {
    required_permission: String,
    resource_type: String,
}

impl PermissionGuard {
    pub fn new(required_permission: &str, resource_type: &str) -> Self {
        Self {
            required_permission: required_permission.to_string(),
            resource_type: resource_type.to_string(),
        }
    }
}
```

#### 권한 검증 로직
1. Claims에서 사용자 Role 추출
2. Role에 할당된 Permission 확인
3. 요청된 리소스에 대한 권한 검증
4. 리소스 소유자 확인 (필요시)

### 2. Role 기반 접근 제어

#### 기본 Role 정의
```rust
pub const ROLES: &[&str] = &[
    "SUPER_ADMIN",    // 시스템 전체 관리
    "ADMIN",          // 프로젝트 관리
    "DOCTOR",         // 의료진 (읽기/쓰기)
    "TECHNICIAN",     // 기술자 (읽기/쓰기)
    "VIEWER",         // 조회 전용
];
```

#### Permission 정의
```rust
pub const PERMISSIONS: &[&str] = &[
    "annotation:read",
    "annotation:write",
    "annotation:delete",
    "mask_group:read",
    "mask_group:write",
    "mask_group:delete",
    "project:read",
    "project:write",
    "project:admin",
    "user:read",
    "user:write",
    "user:admin",
];
```

### 3. 리소스 소유자 확인

#### Annotation 소유자 확인
```rust
pub async fn can_access_annotation(
    user_id: i32,
    annotation_id: i32,
    required_permission: &str,
) -> Result<bool, ServiceError> {
    // 1. Annotation 소유자 확인
    // 2. 프로젝트 멤버십 확인
    // 3. Role 기반 권한 확인
}
```

#### Project 멤버십 확인
```rust
pub async fn is_project_member(
    user_id: i32,
    project_id: i32,
) -> Result<bool, ServiceError> {
    // 프로젝트 멤버십 테이블 확인
}
```

### 4. 컨트롤러 수정

#### 권한 검증 매크로
```rust
#[require_permission("annotation:write")]
pub async fn create_annotation(
    user: AuthenticatedUser,
    req: web::Json<CreateAnnotationRequest>,
    use_case: web::Data<...>,
) -> impl Responder {
    // 권한 검증은 매크로가 자동 처리
    // ...
}
```

#### 또는 수동 권한 검증
```rust
pub async fn create_annotation(
    user: AuthenticatedUser,
    req: web::Json<CreateAnnotationRequest>,
    use_case: web::Data<...>,
) -> impl Responder {
    // 수동 권한 검증
    if !can_access_resource(&user.0, "annotation", "write").await? {
        return HttpResponse::Forbidden().json(json!({
            "error": "Forbidden",
            "message": "Insufficient permissions"
        }));
    }
    // ...
}
```

## 데이터베이스 스키마

### 1. Role 테이블 (이미 존재)
```sql
CREATE TABLE security_role (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    scope TEXT NOT NULL CHECK (scope IN ('GLOBAL', 'PROJECT')),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 2. Permission 테이블 (이미 존재)
```sql
CREATE TABLE security_permission (
    id SERIAL PRIMARY KEY,
    resource_type VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(resource_type, action)
);
```

### 3. Role-Permission 매핑 테이블
```sql
CREATE TABLE security_role_permission (
    id SERIAL PRIMARY KEY,
    role_id INTEGER REFERENCES security_role(id) ON DELETE CASCADE,
    permission_id INTEGER REFERENCES security_permission(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(role_id, permission_id)
);
```

### 4. User-Role 매핑 테이블
```sql
CREATE TABLE security_user_role (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES security_user(id) ON DELETE CASCADE,
    role_id INTEGER REFERENCES security_role(id) ON DELETE CASCADE,
    project_id INTEGER REFERENCES security_project(id) ON DELETE CASCADE, -- Project scope용
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, role_id, project_id)
);
```

## 파일 구조

### 새로 생성할 파일
```
src/infrastructure/middleware/permission_guard.rs
src/application/services/permission_service.rs
src/domain/services/permission_service.rs
src/presentation/extractors/permission_extractor.rs
tests/rbac_integration_test.rs
```

### 수정할 파일
```
src/infrastructure/auth/claims.rs
src/domain/entities/role.rs
src/domain/entities/permission.rs
src/presentation/controllers/*.rs (모든 컨트롤러)
src/application/use_cases/*.rs (권한 검증 로직 추가)
```

## 권한 매트릭스

| Role | annotation:read | annotation:write | annotation:delete | project:admin |
|------|----------------|------------------|-------------------|---------------|
| SUPER_ADMIN | ✅ | ✅ | ✅ | ✅ |
| ADMIN | ✅ | ✅ | ✅ | ✅ (자신의 프로젝트) |
| DOCTOR | ✅ | ✅ | ❌ | ❌ |
| TECHNICIAN | ✅ | ✅ | ❌ | ❌ |
| VIEWER | ✅ | ❌ | ❌ | ❌ |

## 에러 처리

### 403 Forbidden 응답
```json
{
    "error": "Forbidden",
    "message": "Insufficient permissions",
    "required_permission": "annotation:write",
    "user_roles": ["VIEWER"]
}
```

### 404 Not Found (리소스 접근 권한 없음)
```json
{
    "error": "Not Found",
    "message": "Resource not found or access denied"
}
```

## 테스트 계획

### 단위 테스트
- Permission Guard 로직
- Role-Permission 매핑
- 리소스 소유자 확인
- Claims에서 Role 추출

### 통합 테스트
- 각 Role별 권한 테스트
- 프로젝트별 권한 격리 테스트
- 리소스 소유자만 접근 가능 테스트
- 권한 없는 사용자 접근 거부 테스트

## 구현 순서

1. **Claims 확장** (roles 필드 추가)
2. **Permission Service 구현** (권한 검증 로직)
3. **Permission Guard 구현** (미들웨어)
4. **데이터베이스 마이그레이션** (Role-Permission 매핑 테이블)
5. **기본 Role/Permission 데이터 시딩**
6. **컨트롤러 권한 검증 추가**
7. **Use Case 레이어 권한 검증**
8. **테스트 작성 및 검증**

## 보안 고려사항

1. **최소 권한 원칙**: 사용자에게 필요한 최소한의 권한만 부여
2. **권한 격리**: 프로젝트별 권한 격리 보장
3. **감사 로그**: 모든 권한 변경 및 접근 시도 로깅
4. **토큰 갱신**: Role 변경시 토큰 무효화
5. **정기 검토**: 권한 설정 정기 검토 및 정리

## 성능 고려사항

1. **권한 캐싱**: 자주 사용되는 권한 정보 캐싱
2. **배치 권한 검증**: 여러 리소스에 대한 권한을 한 번에 검증
3. **인덱스 최적화**: 권한 관련 쿼리 성능 최적화
4. **연결 풀**: 데이터베이스 연결 풀 최적화

## 마이그레이션 전략

1. **단계적 적용**: 컨트롤러별로 순차 적용
2. **기본 권한**: 기존 사용자에게 기본 권한 부여
3. **롤백 계획**: 문제 발생시 이전 상태로 복구
4. **모니터링**: 권한 시스템 동작 모니터링

## 참고사항

- 기존 AuthGuard와 연동
- JWT 토큰에 Role 정보 포함
- 프로젝트별 권한 격리 중요
- 의료 데이터 보안 요구사항 준수
