# AuthGuard JWT 인증 구현 계획

## 개요

PACS Extension Server에 JWT 기반 인증 시스템을 구현하는 계획서입니다. 이 문서는 Phase 1로, 인증(Authentication)만 구현하며 권한 제어(Authorization/RBAC)는 별도 Phase 2에서 진행합니다.

## 목표

- 모든 보호된 API 엔드포인트에서 JWT 토큰 검증
- 사용자 신원 확인 및 식별
- 화이트리스트 경로 지원 (인증 없이 접근 가능)
- 기존 코드와의 호환성 유지

## 아키텍처

### 1. AuthGuard Middleware
```
HTTP Request → AuthGuard Middleware → Controller
                ↓
            JWT 검증
            Claims 추출
            HttpRequest extensions에 저장
```

### 2. Claims Extractor
```
Controller → AuthenticatedUser Extractor → Claims 사용
```

## 구현 세부사항

### 1. 설정 구조

#### AuthGuardConfig
```rust
pub struct AuthGuardConfig {
    pub enabled: bool,                    // AuthGuard 활성화/비활성화
    pub whitelist_paths: Vec<String>,     // 인증 없이 접근 가능한 경로
}
```

#### 설정 파일 (config/default.toml)
```toml
[auth_guard]
enabled = true
whitelist_paths = ["/health", "/api-docs", "/swagger-ui"]
```

### 2. AuthGuard Middleware

#### 핵심 기능
- Authorization 헤더에서 "Bearer {token}" 추출
- JwtService를 통한 토큰 검증
- 검증된 Claims를 HttpRequest extensions에 저장
- 화이트리스트 경로 매칭 (starts_with 방식)
- enabled 플래그로 미들웨어 비활성화 지원

#### 에러 응답
- 401 Unauthorized with JSON:
  - Missing token: `{"error": "Unauthorized", "message": "Missing authorization token"}`
  - Invalid token: `{"error": "Unauthorized", "message": "Invalid token"}`
  - Expired token: `{"error": "Unauthorized", "message": "Token has expired"}`

### 3. Claims Extractor

#### AuthenticatedUser
```rust
pub struct AuthenticatedUser(pub Claims);

impl FromRequest for AuthenticatedUser {
    // HttpRequest extensions에서 Claims 추출
    // 인증 실패시 401 반환
}
```

#### OptionalUser (선택사항)
```rust
pub struct OptionalUser(pub Option<Claims>);
// 선택적 인증이 필요한 엔드포인트용
```

### 4. 컨트롤러 수정

#### 기존 함수 시그니처
```rust
pub async fn create_annotation(
    req: web::Json<CreateAnnotationRequest>,
    use_case: web::Data<...>,
    _http_req: HttpRequest,
) -> impl Responder
```

#### 수정된 함수 시그니처
```rust
pub async fn create_annotation(
    user: AuthenticatedUser,  // AuthGuard에서 주입
    req: web::Json<CreateAnnotationRequest>,
    use_case: web::Data<...>,
) -> impl Responder {
    let user_id = user.0.user_id().unwrap();
    // ...
}
```

## 파일 구조

### 새로 생성할 파일
```
src/infrastructure/middleware/auth_guard.rs
src/presentation/extractors/mod.rs
src/presentation/extractors/auth_extractor.rs
src/presentation/extractors/optional_claims.rs
tests/auth_guard_integration_test.rs
```

### 수정할 파일
```
src/infrastructure/config/settings.rs
src/infrastructure/middleware/mod.rs
src/presentation/mod.rs
src/main.rs
config/default.toml
src/presentation/controllers/annotation_controller.rs
src/presentation/controllers/mask_group_controller.rs
src/presentation/controllers/mask_controller.rs
src/presentation/controllers/user_controller.rs
src/presentation/controllers/project_controller.rs
src/presentation/controllers/permission_controller.rs
src/presentation/controllers/access_control_controller.rs
```

## 보호되는 엔드포인트

### 인증 필수
- `/api/annotations/*` (모든 메서드)
- `/api/annotations/{id}/mask-groups/*` (모든 메서드)
- `/api/annotations/{id}/mask-groups/{id}/masks/*` (모든 메서드)
- `/api/users/*` (GET, PUT, DELETE)
- `/api/projects/*` (모든 메서드)
- `/api/permissions/*` (모든 메서드)
- `/api/access-control/*` (모든 메서드)

### 공개 엔드포인트 (인증 불필요)
- `/health`
- `/api-docs/*`
- `/swagger-ui/*`
- `/api/auth/login`
- `/api/users` POST (회원가입)

## 테스트 계획

### 단위 테스트
- AuthGuard middleware 로직
- Claims extractor 동작
- 화이트리스트 경로 매칭
- enabled 플래그 동작

### 통합 테스트
- 인증 없이 보호된 엔드포인트 호출 → 401
- 유효한 토큰으로 호출 → 성공
- 유효하지 않은 토큰으로 호출 → 401
- 만료된 토큰으로 호출 → 401
- 화이트리스트 경로 (인증 없음) → 200
- enabled=false일 때 인증 없이 접근 가능

## 구현 순서

1. **설정 추가** (AuthGuardConfig, default.toml)
2. **AuthGuard Middleware 구현** (핵심 로직)
3. **Claims Extractor 구현** (FromRequest)
4. **모듈 export 설정** (mod.rs 파일들)
5. **Main.rs 통합** (미들웨어 등록)
6. **컨트롤러 업데이트** (하나씩 순차적으로)
7. **단위 테스트 작성**
8. **통합 테스트 작성**
9. **전체 테스트 실행 및 검증**
10. **문서화**

## 검증 항목

- [ ] enabled=true일 때 인증 필수
- [ ] enabled=false일 때 인증 비활성화
- [ ] 화이트리스트 경로 인증 없이 접근 가능
- [ ] 유효한 토큰으로 정상 작동
- [ ] 유효하지 않은 토큰 거부
- [ ] 만료된 토큰 거부
- [ ] 토큰 없을 때 거부
- [ ] Claims에서 user_id 정확히 추출
- [ ] 모든 컨트롤러에서 user_id 사용 가능
- [ ] 기존 테스트 통과 (AuthGuard disabled)

## 보안 고려사항

1. **토큰 검증**: JwtService를 통한 완전한 토큰 검증
2. **에러 메시지**: 클라이언트에는 간단한 메시지만, 서버 로그에는 상세 정보
3. **로깅**: 모든 인증 실패 이벤트 로깅 (보안 모니터링)
4. **화이트리스트**: 최소한의 경로만 인증 없이 허용

## 향후 작업 (Phase 2)

- Claims에 roles 필드 추가
- Role 기반 접근 제어 (RBAC) 로직
- Permission Guard 구현
- 리소스 소유자 확인 로직
- 프로젝트 멤버십 검증
- 403 Forbidden 에러 처리

## 참고사항

- 기존 JwtService와 AuthMiddleware 활용
- Claims 구조체는 현재 구조 유지 (roles는 나중에 추가)
- Use Case/Service 레이어는 변경 없음
- 점진적 배포 가능 (enabled 플래그 활용)
