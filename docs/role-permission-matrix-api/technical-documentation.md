# Role-Permission Matrix API 기술 문서

## 📋 개요

Role-Permission Matrix API는 역할과 권한 간의 관계를 매트릭스 형태로 조회하고 관리할 수 있는 RESTful API입니다. 이 API를 통해 사용자는 역할별로 할당된 권한을 시각적으로 확인하고, 개별 권한을 ON/OFF할 수 있습니다.

## 🏗️ 아키텍처

### Clean Architecture 패턴

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  role_permission_matrix_controller.rs                  │ │
│  │  - HTTP 엔드포인트 처리                                 │ │
│  │  - 요청/응답 변환                                       │ │
│  │  - 에러 처리                                           │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  RolePermissionMatrixUseCase                           │ │
│  │  - 비즈니스 로직 오케스트레이션                        │ │
│  │  - DTO 변환                                             │ │
│  │  - 에러 처리                                           │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  role_permission_matrix_dto.rs                         │ │
│  │  - 요청/응답 DTO 정의                                   │ │
│  │  - 직렬화/역직렬화                                      │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  PermissionService (Trait)                              │ │
│  │  - 매트릭스 조회 인터페이스                            │ │
│  │  - 권한 할당/제거 인터페이스                           │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  PermissionServiceImpl                                  │ │
│  │  - 데이터베이스 쿼리 실행                              │ │
│  │  - SQL 최적화                                           │ │
│  │  - 트랜잭션 관리                                        │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 🗄️ 데이터베이스 설계

### 테이블 구조

#### security_role
```sql
CREATE TABLE security_role (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    scope TEXT NOT NULL CHECK (scope IN ('GLOBAL', 'PROJECT')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### security_permission
```sql
CREATE TABLE security_permission (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    resource_type TEXT NOT NULL,  -- 카테고리로 활용
    action TEXT NOT NULL,
    UNIQUE(resource_type, action)
);
```

#### security_role_permission
```sql
CREATE TABLE security_role_permission (
    role_id INTEGER NOT NULL REFERENCES security_role(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permission(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
```

### 쿼리 최적화

#### 글로벌 역할-권한 매트릭스 조회
```sql
SELECT r.id, r.name, r.description, r.scope, r.created_at
FROM security_role r
WHERE r.scope = 'GLOBAL'
ORDER BY r.name;
```

#### 프로젝트별 역할-권한 매트릭스 조회
```sql
SELECT r.id, r.name, r.description, r.scope, r.created_at
FROM security_role r
INNER JOIN security_project_role pr ON r.id = pr.role_id
WHERE pr.project_id = $1
ORDER BY r.name;
```

#### 역할-권한 할당 정보 조회
```sql
SELECT rp.role_id, rp.permission_id
FROM security_role_permission rp
WHERE rp.role_id IN (SELECT id FROM security_role WHERE scope = 'GLOBAL');
```

## 🔧 API 명세

### 1. 글로벌 역할-권한 매트릭스 조회

**엔드포인트**: `GET /api/roles/global/permissions/matrix`

**응답 예시**:
```json
{
  "roles": [
    {
      "id": 1,
      "name": "Admin",
      "description": "Administrator role",
      "scope": "GLOBAL"
    }
  ],
  "permissions_by_category": {
    "USER": [
      {
        "id": 1,
        "resource_type": "USER",
        "action": "CREATE"
      }
    ],
    "PROJECT": [
      {
        "id": 2,
        "resource_type": "PROJECT",
        "action": "READ"
      }
    ]
  },
  "assignments": [
    {
      "role_id": 1,
      "permission_id": 1,
      "assigned": true
    }
  ]
}
```

### 2. 프로젝트별 역할-권한 매트릭스 조회

**엔드포인트**: `GET /api/projects/{project_id}/roles/permissions/matrix`

**경로 매개변수**:
- `project_id`: 프로젝트 ID (integer)

### 3. 글로벌 역할에 권한 할당/제거

**엔드포인트**: `PUT /api/roles/{role_id}/permissions/{permission_id}`

**경로 매개변수**:
- `role_id`: 역할 ID (integer)
- `permission_id`: 권한 ID (integer)

**요청 본문**:
```json
{
  "assign": true
}
```

### 4. 프로젝트별 역할에 권한 할당/제거

**엔드포인트**: `PUT /api/projects/{project_id}/roles/{role_id}/permissions/{permission_id}`

**경로 매개변수**:
- `project_id`: 프로젝트 ID (integer)
- `role_id`: 역할 ID (integer)
- `permission_id`: 권한 ID (integer)

## 🧪 테스트 전략

### 단위 테스트

#### Use Case 테스트
```rust
#[tokio::test]
async fn test_get_global_matrix_success() {
    let mut mock_service = MockPermissionService::new();
    // Mock 설정
    let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
    let result = use_case.get_global_matrix().await;
    assert!(result.is_ok());
}
```

#### DTO 테스트
```rust
#[test]
fn test_role_info_serialization() {
    let role_info = RoleInfo {
        id: 1,
        name: "Admin".to_string(),
        description: Some("Administrator role".to_string()),
        scope: "GLOBAL".to_string(),
    };
    let json = serde_json::to_string(&role_info).unwrap();
    let deserialized: RoleInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(role_info, deserialized);
}
```

### 통합 테스트

#### API 엔드포인트 테스트
```rust
#[tokio::test]
async fn test_get_global_matrix_endpoint() {
    let app = create_test_app().await;
    let req = test::TestRequest::get()
        .uri("/api/roles/global/permissions/matrix")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
```

## 🔒 보안 고려사항

### 인증 및 권한 부여
- JWT 토큰 기반 인증
- 역할 기반 접근 제어 (RBAC)
- 관리자 권한 필요

### 데이터 검증
- 입력 데이터 유효성 검사
- SQL 인젝션 방지
- XSS 공격 방지

### 에러 처리
- 민감한 정보 노출 방지
- 일관된 에러 응답 형식
- 로깅 및 모니터링

## 📊 성능 최적화

### 데이터베이스 최적화
- 인덱스 활용
- 쿼리 최적화
- 연결 풀 관리

### 메모리 최적화
- DTO 최적화
- 불필요한 데이터 제거
- 효율적인 데이터 구조

### 캐싱 전략
- 역할-권한 매트릭스 캐싱
- TTL 기반 캐시 무효화
- 분산 캐시 고려

## 🚀 배포 가이드

### 환경 요구사항
- Rust 1.70+
- PostgreSQL 13+
- Actix-web 4.0+

### 설정 변수
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/pacs_db
JWT_SECRET=your-secret-key
```

### 마이그레이션 실행
```bash
cargo run --bin pacs_server -- --migrate
```

### 서버 시작
```bash
cargo run --release
```

## 📚 참고 자료

### 기술 스택
- **언어**: Rust
- **웹 프레임워크**: Actix-web
- **데이터베이스**: PostgreSQL
- **ORM**: SQLx
- **문서화**: Utoipa (OpenAPI)

### 관련 문서
- [Clean Architecture 가이드](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Actix-web 문서](https://actix.rs/docs/)
- [SQLx 문서](https://docs.rs/sqlx/latest/sqlx/)
- [Utoipa 문서](https://docs.rs/utoipa/latest/utoipa/)

## 🔧 문제 해결

### 일반적인 문제

#### 1. 컴파일 오류
```bash
error[E0659]: `RoleInfo` is ambiguous
```
**해결책**: 명시적 import 경로 사용
```rust
use crate::application::dto::role_permission_matrix_dto::RoleInfo;
```

#### 2. 소유권 오류
```bash
error[E0507]: cannot move out of `assignment_set`
```
**해결책**: `clone()` 사용
```rust
let assignment_set = assignment_set.clone();
```

#### 3. Mock 설정 오류
```bash
error[E0407]: method `get_user_permissions` is not a member of trait
```
**해결책**: Mock 트레이트에 모든 메서드 정의

### 디버깅 팁

#### 로그 레벨 설정
```bash
RUST_LOG=debug cargo run
```

#### 데이터베이스 쿼리 확인
```sql
EXPLAIN ANALYZE SELECT * FROM security_role_permission;
```

#### API 테스트
```bash
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix"
```

## 📈 모니터링

### 성능 메트릭
- API 응답 시간
- 데이터베이스 쿼리 시간
- 메모리 사용량
- CPU 사용률

### 로그 모니터링
- 에러 로그
- 성능 로그
- 보안 로그
- 비즈니스 로그

### 알림 설정
- 에러율 임계값
- 응답 시간 임계값
- 리소스 사용률 임계값
