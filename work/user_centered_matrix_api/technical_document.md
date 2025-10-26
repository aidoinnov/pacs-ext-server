# User-Centered Matrix API 기술 문서

## 아키텍처 개요

User-Centered Matrix API는 Clean Architecture 패턴을 따라 구현되었으며, 기존 프로젝트 중심 매트릭스 API와 함께 사용할 수 있는 새로운 사용자 중심 매트릭스 API입니다.

## 시스템 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
├─────────────────────────────────────────────────────────────┤
│  user_project_matrix_controller.rs                        │
│  - get_matrix() 핸들러                                     │
│  - OpenAPI 문서화                                          │
│  - HTTP 요청/응답 처리                                      │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
├─────────────────────────────────────────────────────────────┤
│  user_project_matrix_use_case.rs                          │
│  - 비즈니스 로직 오케스트레이션                              │
│  - DTO 변환 및 매핑                                         │
│  - 에러 처리                                                │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                     Domain Layer                           │
├─────────────────────────────────────────────────────────────┤
│  user_service.rs (확장)                                    │
│  - get_users_with_sorting() 메서드 추가                     │
│  - 사용자 정렬, 검색, 필터링 로직                           │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                       │
├─────────────────────────────────────────────────────────────┤
│  user_service_impl.rs (확장)                              │
│  - 동적 SQL 쿼리 구성                                       │
│  - 데이터베이스 접근                                        │
│  - 성능 최적화                                              │
└─────────────────────────────────────────────────────────────┘
```

## 데이터 구조 설계

### 1. DTO 구조

#### ProjectRoleCell
```rust
pub struct ProjectRoleCell {
    pub project_id: i32,
    pub project_name: String,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
}
```

#### UserProjectMatrixRow
```rust
pub struct UserProjectMatrixRow {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub project_roles: Vec<ProjectRoleCell>,
}
```

#### UserProjectMatrixResponse
```rust
pub struct UserProjectMatrixResponse {
    pub matrix: Vec<UserProjectMatrixRow>,
    pub projects: Vec<ProjectInfo>,
    pub pagination: UserProjectMatrixPagination,
}
```

### 2. 이중 페이지네이션 구조

```rust
pub struct UserProjectMatrixPagination {
    pub user_page: i32,
    pub user_page_size: i32,
    pub user_total_count: i64,
    pub user_total_pages: i32,
    pub project_page: i32,
    pub project_page_size: i32,
    pub project_total_count: i64,
    pub project_total_pages: i32,
}
```

## 핵심 구현 로직

### 1. 동적 SQL 쿼리 구성

```rust
async fn get_users_with_sorting(
    &self,
    page: i32,
    page_size: i32,
    sort_by: &str,
    sort_order: &str,
    search: Option<&str>,
    user_ids: Option<&[i32]>,
) -> Result<(Vec<User>, i64), ServiceError> {
    let offset = (page - 1) * page_size;

    let order_by = match sort_by {
        "username" => "username",
        "email" => "email",
        "created_at" => "created_at",
        _ => "username",
    };

    let order_direction = match sort_order {
        "desc" => "DESC",
        _ => "ASC",
    };

    let search_condition = if let Some(search_term) = search {
        format!("AND (username ILIKE '%{}%' OR email ILIKE '%{}%')", search_term, search_term)
    } else {
        String::new()
    };

    let query = format!(
        "SELECT id, keycloak_id, username, email, full_name, organization, department, phone,
                created_at, updated_at, account_status, email_verified,
                email_verification_token, email_verification_expires_at,
                approved_by, approved_at, suspended_at, suspended_reason, deleted_at
         FROM security_user
         WHERE ($1::int[] IS NULL OR id = ANY($1))
           AND account_status != 'DELETED'
           {}
         ORDER BY {} {}
         LIMIT $2 OFFSET $3",
        search_condition, order_by, order_direction
    );

    // 쿼리 실행...
}
```

### 2. Use Case 오케스트레이션

```rust
pub async fn get_matrix(
    &self,
    params: UserProjectMatrixQueryParams,
) -> Result<UserProjectMatrixResponse, ServiceError> {
    // 1. 파라미터 파싱 및 기본값 설정
    let user_page = params.user_page.unwrap_or(1);
    let user_page_size = params.user_page_size.unwrap_or(10).min(50);
    
    // 2. 사용자 목록 조회 (정렬, 필터링, 페이지네이션)
    let (users, user_total_count) = self.user_service
        .get_users_with_sorting(
            user_page,
            user_page_size,
            user_sort_by,
            user_sort_order,
            params.user_search.as_deref(),
            params.user_ids.as_deref(),
        )
        .await?;

    // 3. 프로젝트 목록 조회
    let (projects, project_total_count) = self.project_service
        .get_projects_with_status_filter(
            None,
            params.project_ids,
            project_page,
            project_page_size,
        )
        .await?;

    // 4. 각 사용자의 프로젝트 역할 매핑 조회
    let mut matrix_rows = Vec::new();
    for user in users {
        let user_project_roles = self.user_service
            .get_user_project_roles(user.id, params.role_id, params.project_ids.as_deref())
            .await?;

        // 매트릭스 행 구성...
    }

    // 5. 최종 응답 구성
    Ok(UserProjectMatrixResponse {
        matrix: matrix_rows,
        projects: project_info_list,
        pagination: pagination_info,
    })
}
```

## 성능 최적화 전략

### 1. 쿼리 최적화
- **동적 SQL 구성**: 필요한 조건만 포함하여 불필요한 데이터 조회 방지
- **인덱스 활용**: username, email, created_at 컬럼에 인덱스 활용
- **페이지네이션**: LIMIT/OFFSET을 통한 대용량 데이터 처리

### 2. 메모리 최적화
- **스트리밍 처리**: 대용량 데이터를 한 번에 로드하지 않고 페이지별로 처리
- **참조 활용**: 가능한 한 데이터 복사 대신 참조 사용

### 3. 네트워크 최적화
- **압축 응답**: JSON 응답 압축을 통한 네트워크 트래픽 감소
- **캐싱 헤더**: 적절한 캐싱 헤더 설정

## 에러 처리 전략

### 1. 계층별 에러 처리

#### Domain Layer
```rust
pub enum ServiceError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    NotFound(String),
    // ...
}
```

#### Application Layer
```rust
match use_case.get_matrix(query.into_inner()).await {
    Ok(response) => HttpResponse::Ok().json(response),
    Err(e) => HttpResponse::InternalServerError().json(json!({
        "error": format!("Failed to get user-project matrix: {}", e)
    })),
}
```

### 2. 사용자 친화적 에러 메시지
- 구체적인 에러 원인 제공
- 해결 방법 제시
- 적절한 HTTP 상태 코드 사용

## 보안 고려사항

### 1. 입력 검증
- 쿼리 파라미터 유효성 검사
- SQL 인젝션 방지 (파라미터화된 쿼리 사용)
- 페이지 크기 제한 (최대 50)

### 2. 접근 제어
- 인증된 사용자만 접근 가능
- 역할 기반 접근 제어 (향후 구현)

## 모니터링 및 로깅

### 1. 성능 모니터링
```rust
let start = std::time::Instant::now();
// 쿼리 실행
let duration = start.elapsed();
tracing::info!("Database query time: {:?}", duration);
```

### 2. 에러 로깅
```rust
tracing::error!("Failed to get user-project matrix: {}", e);
```

## 테스트 전략

### 1. 단위 테스트
- 각 계층별 독립적인 테스트
- Mock 객체를 활용한 의존성 격리

### 2. 통합 테스트
- 실제 데이터베이스를 사용한 엔드투엔드 테스트
- 다양한 쿼리 파라미터 조합 테스트

### 3. 성능 테스트
- 대용량 데이터에 대한 응답 시간 측정
- 동시 요청 처리 능력 테스트

## 확장성 고려사항

### 1. 수평적 확장
- 로드 밸런서를 통한 다중 인스턴스 배포
- 데이터베이스 읽기 전용 복제본 활용

### 2. 수직적 확장
- 캐싱 레이어 추가 (Redis)
- 데이터베이스 연결 풀 최적화

## 유지보수성

### 1. 코드 구조
- Clean Architecture 패턴 준수
- 단일 책임 원칙 적용
- 의존성 주입을 통한 느슨한 결합

### 2. 문서화
- 완전한 OpenAPI 문서화
- 클라이언트 가이드 제공
- 코드 주석 및 문서화

## 배포 고려사항

### 1. 데이터베이스 마이그레이션
- 기존 스키마 변경 없이 구현
- 새로운 인덱스 추가 고려

### 2. 롤백 전략
- 기능 플래그를 통한 점진적 배포
- 기존 API와의 호환성 유지

## 향후 개선 방향

### 1. 기능 확장
- 실시간 업데이트 (WebSocket)
- 고급 필터링 옵션
- 데이터 내보내기 기능

### 2. 성능 개선
- 캐싱 전략 구현
- 쿼리 최적화
- 비동기 처리 개선

### 3. 사용자 경험
- 더 직관적인 API 응답
- 상세한 에러 메시지
- API 버전 관리
