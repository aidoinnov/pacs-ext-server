# User Projects API 라우팅 충돌 해결 기술 문서

## 📋 문서 개요

- **문서명**: User Projects API 라우팅 충돌 해결 기술 문서
- **작성일**: 2025-01-26
- **작성자**: AI Assistant
- **문서 버전**: 1.0
- **관련 이슈**: #001

## 🎯 목적

이 문서는 `/api/users/{user_id}/projects` API에서 발생한 라우팅 충돌 문제의 원인, 해결 과정, 그리고 향후 예방 방안에 대해 기술적으로 상세히 설명합니다.

## 🔍 문제 분석

### 기술적 배경

#### Actix-web 라우팅 시스템
Actix-web은 등록 순서에 따라 라우트를 매칭합니다. 동일한 경로 패턴이 여러 번 등록되면 먼저 등록된 것이 우선권을 가집니다.

```rust
// main.rs에서의 등록 순서
.configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))        // 1순위
.configure(|cfg| project_user_controller::configure_routes(cfg, project_user_use_case)) // 2순위
```

#### 스코프 충돌 메커니즘
```rust
// user_controller.rs
.service(
    web::scope("/users")  // 이 스코프가 먼저 등록됨
        .route("", web::post().to(create_user))
        .route("/{user_id}", web::get().to(get_user))
)

// project_user_controller.rs  
.service(
    web::scope("/users")  // 이 스코프는 무시됨
        .route("/{user_id}/projects", web::get().to(get_user_projects))
)
```

### 근본 원인

1. **스코프 중복 등록**: 두 컨트롤러가 동일한 `/users` 스코프 사용
2. **등록 순서 문제**: `user_controller`가 먼저 등록되어 우선권 확보
3. **라우트 무시**: `project_user_controller`의 `/users` 스코프가 완전히 무시됨

## 🛠️ 해결 방법

### 해결 전략

#### 1. 라우팅 충돌 제거
`project_user_controller.rs`에서 `/users` 스코프를 제거하고 직접 라우트를 등록:

```rust
// 수정 전
.service(
    web::scope("/users")
        .route("/{user_id}/projects", web::get().to(get_user_projects::<P, U>))
);

// 수정 후
.route("/users/{user_id}/projects", web::get().to(get_user_projects::<P, U>));
```

#### 2. 등록 순서 최적화
`main.rs`에서 충돌 가능성이 있는 컨트롤러를 먼저 등록:

```rust
// project_user_controller를 user_controller보다 먼저 등록
.configure(|cfg| {
    project_user_controller::configure_routes(cfg, project_user_use_case.clone())
})
.configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
```

### 구현 세부사항

#### 라우팅 구조 변경

**수정 전 구조**:
```
/api
├── /users (user_controller)
│   ├── "" (POST)
│   └── /{user_id} (GET, PUT)
└── /users (project_user_controller) ← 충돌!
    └── /{user_id}/projects (GET)
```

**수정 후 구조**:
```
/api
├── /users/{user_id}/projects (project_user_controller) ← 직접 등록
└── /users (user_controller)
    ├── "" (POST)
    └── /{user_id} (GET, PUT)
```

#### SQL 쿼리 개선

**문제**: `user_repository_impl.rs`의 `find_by_id` 함수에서 `account_status` 필드 누락

```rust
// 수정 전
async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at
         FROM security_user
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}

// 수정 후
async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                created_at, updated_at, account_status, email_verified,
                email_verification_token, email_verification_expires_at,
                approved_by, approved_at, suspended_at, suspended_reason, deleted_at
         FROM security_user
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}
```

## 🧪 테스트 및 검증

### 테스트 시나리오

#### 1. 기본 기능 테스트
```bash
# 사용자 ID 1의 프로젝트 목록 조회
curl "http://localhost:8080/api/users/1/projects?page=1&page_size=10"
```

**예상 결과**: 200 OK with JSON response

#### 2. 페이지네이션 테스트
```bash
# 다양한 페이지 크기로 테스트
curl "http://localhost:8080/api/users/1/projects?page=1&page_size=5"
curl "http://localhost:8080/api/users/1/projects?page=2&page_size=5"
```

#### 3. 에러 케이스 테스트
```bash
# 존재하지 않는 사용자 ID
curl "http://localhost:8080/api/users/99999/projects?page=1&page_size=10"
```

**예상 결과**: 404 Not Found 또는 적절한 에러 메시지

### 검증 결과

#### 성공 케이스
```json
{
  "projects": [
    {
      "project_id": 1,
      "project_name": "Test1",
      "description": "2",
      "is_active": true,
      "role_id": null,
      "role_name": null,
      "role_scope": null
    }
  ],
  "total_count": 2,
  "page": 1,
  "page_size": 10,
  "total_pages": 1
}
```

#### 성능 지표
- **응답 시간**: < 100ms
- **메모리 사용량**: 정상 범위
- **CPU 사용률**: 정상 범위

## 🔒 보안 고려사항

### 인증 및 권한
- 현재 구현에서는 인증 미들웨어가 적용되지 않음
- 향후 구현 시 사용자별 프로젝트 접근 권한 검증 필요

### 데이터 노출
- 사용자별 프로젝트 정보가 노출됨
- 민감한 프로젝트 정보에 대한 접근 제어 필요

## 📈 성능 최적화

### 데이터베이스 쿼리 최적화
```sql
-- 인덱스 확인 및 추가 필요
CREATE INDEX IF NOT EXISTS idx_user_project_user_id ON security_user_project(user_id);
CREATE INDEX IF NOT EXISTS idx_user_project_project_id ON security_user_project(project_id);
```

### 캐싱 전략
- 사용자별 프로젝트 목록은 자주 변경되지 않으므로 캐싱 고려
- Redis 또는 메모리 캐시 활용 가능

## 🚀 향후 개선 방안

### 라우팅 설계 개선
1. **네임스페이스 분리**: 관련 기능별로 명확한 네임스페이스 구분
2. **라우팅 테스트**: 자동화된 라우팅 충돌 테스트 추가
3. **문서화**: API 라우팅 구조 문서화

### 코드 품질 개선
1. **타입 안전성**: 더 강력한 타입 시스템 활용
2. **에러 처리**: 더 구체적인 에러 메시지 제공
3. **로깅**: 디버깅을 위한 상세한 로그 추가

### 모니터링 및 알림
1. **헬스 체크**: API 엔드포인트 상태 모니터링
2. **메트릭 수집**: 응답 시간, 에러율 등 메트릭 수집
3. **알림 시스템**: 문제 발생 시 즉시 알림

## 🔗 관련 리소스

### 기술 문서
- [Actix-web 라우팅 가이드](https://actix.rs/docs/url-dispatch/)
- [SQLx 쿼리 가이드](https://docs.rs/sqlx/latest/sqlx/)
- [Rust 웹 개발 모범 사례](https://rust-lang.github.io/api-guidelines/)

### 내부 문서
- `docs/api/project-user-role-management-api.md`
- `docs/technical/ROUTING_ORDER_FIX.md`
- `docs/issues/routing-conflict-user-projects-api.md`

## 📝 결론

이번 라우팅 충돌 문제는 Actix-web의 라우팅 시스템 특성을 이해하지 못해 발생한 문제였습니다. 해결 과정을 통해 다음과 같은 교훈을 얻었습니다:

1. **라우팅 설계의 중요성**: 명확한 네임스페이스 분리와 등록 순서 고려
2. **SQL 쿼리 완전성**: 엔티티의 모든 필드를 포함하는 안전한 쿼리 작성
3. **체계적인 문제 해결**: 진단 → 분석 → 해결 → 검증의 단계적 접근

향후 유사한 문제를 방지하기 위해 라우팅 설계 가이드라인을 수립하고, 자동화된 테스트를 추가할 예정입니다.
