# 기술 문서: 사용자 목록 API (페이지네이션 지원)

## 아키텍처 개요
사용자 목록 API는 Clean Architecture 패턴을 따르며 다음과 같은 계층 구조로 구성됩니다:

```
Presentation Layer (Controller)
    ↓
Application Layer (Use Case)
    ↓
Domain Layer (Service)
    ↓
Infrastructure Layer (Repository)
```

## 구현 상세

### 1. DTO 계층
**파일**: `pacs-server/src/application/dto/user_dto.rs`

#### UserListQuery
```rust
pub struct UserListQuery {
    pub page: Option<i32>,          // 페이지 번호 (기본값: 1)
    pub page_size: Option<i32>,      // 페이지 크기 (기본값: 20, 최대: 100)
    pub sort_by: Option<String>,     // 정렬 기준 (username, email, created_at)
    pub sort_order: Option<String>, // 정렬 순서 (asc, desc)
    pub search: Option<String>,      // 검색어 (username, email 검색)
}
```

#### UserListResponse
```rust
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub pagination: PaginationInfo,
}
```

#### PaginationInfo
```rust
pub struct PaginationInfo {
    pub page: i32,
    pub page_size: i32,
    pub total: i32,
    pub total_pages: i32,
}
```

### 2. Controller 계층
**파일**: `pacs-server/src/presentation/controllers/user_controller.rs`

#### list_users 함수
```rust
pub async fn list_users(
    user_use_case: web::Data<Arc<UserUseCase<U>>>,
    query: web::Query<UserListQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let sort_by = query.sort_by.as_deref().unwrap_or("username");
    let sort_order = query.sort_order.as_deref().unwrap_or("asc");
    let search = query.search.as_deref();

    match user_use_case.list_users(page, page_size, sort_by, sort_order, search).await {
        Ok((users, total)) => {
            let total_pages = if total > 0 {
                ((total as f64) / (page_size as f64)).ceil() as i32
            } else {
                0
            };

            HttpResponse::Ok().json(UserListResponse {
                users: users.into_iter().map(|u| u.into()).collect(),
                pagination: PaginationInfo {
                    page,
                    page_size,
                    total: total as i32,
                    total_pages,
                },
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to list users: {}", e)
        })),
    }
}
```

**라우팅 설정**:
```rust
.route("", web::get().to(UserController::<U>::list_users))
```

### 3. Use Case 계층
**파일**: `pacs-server/src/application/use_cases/user_use_case.rs`

#### list_users 메서드
```rust
pub async fn list_users(
    &self,
    page: i32,
    page_size: i32,
    sort_by: &str,
    sort_order: &str,
    search: Option<&str>,
) -> Result<(Vec<User>, i64), ServiceError> {
    let result = self.user_service
        .get_users_with_sorting(page, page_size, sort_by, sort_order, search, None)
        .await?;
    Ok(result)
}
```

### 4. Service 계층
**파일**: `pacs-server/src/infrastructure/services/user_service_impl.rs`

기존 `get_users_with_sorting` 메서드 활용:
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
    // 동적 SQL 쿼리 생성
    // 검색 조건, 정렬 조건 적용
    // 페이지네이션 적용 (LIMIT, OFFSET)
    // 총 개수 조회
}
```

## 데이터 흐름

```
1. Client 요청
   GET /api/users?page=1&page_size=20&sort_by=username&sort_order=asc&search=john
   ↓
2. Controller (user_controller.rs)
   - 쿼리 파라미터 파싱
   - 기본값 설정
   - Use Case 호출
   ↓
3. Use Case (user_use_case.rs)
   - Service 호출
   - 반환 데이터 변환
   ↓
4. Service (user_service_impl.rs)
   - 동적 SQL 쿼리 생성
   - 검색, 정렬 조건 적용
   - 페이지네이션 적용
   - 데이터베이스 조회
   - 총 개수 조회
   ↓
5. Repository (user_repository_impl.rs)
   - SQL 실행
   - 결과 반환
   ↓
6. Controller
   - 페이지네이션 정보 계산
   - 응답 생성
   ↓
7. Client
   { "users": [...], "pagination": {...} }
```

## SQL 쿼리 구조

### 1. 사용자 조회 쿼리
```sql
SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
       created_at, updated_at, account_status, email_verified, 
       email_verification_token, email_verification_expires_at, 
       approved_by, approved_at, suspended_at, suspended_reason, deleted_at
FROM security_user
WHERE ($1::int[] IS NULL OR id = ANY($1))
  AND account_status != 'DELETED'
  AND (username ILIKE '%검색어%' OR email ILIKE '%검색어%')
ORDER BY username ASC  -- 또는 DESC, email, created_at
LIMIT 20 OFFSET 0
```

### 2. 총 개수 조회 쿼리
```sql
SELECT COUNT(*)
FROM security_user
WHERE ($1::int[] IS NULL OR id = ANY($1))
  AND account_status != 'DELETED'
  AND (username ILIKE '%검색어%' OR email ILIKE '%검색어%')
```

## 보안 및 성능 고려사항

### 1. 페이지 크기 제한
- 최대 100개로 제한하여 서버 부하 방지
```rust
let page_size = query.page_size.unwrap_or(20).min(100);
```

### 2. 삭제된 사용자 제외
- `account_status != 'DELETED'` 조건으로 삭제된 사용자 제외
- 논리적 삭제 방식 사용

### 3. SQL 인젝션 방지
- SQLx의 prepared statement 사용
- 동적 쿼리에서도 파라미터 바인딩 사용

### 4. 기본 정렬
- 기본 정렬 기준: username ASC
- 사용자 지정 정렬 가능 (username, email, created_at)

## 에러 처리

### 1. 잘못된 요청
- 잘못된 쿼리 파라미터는 기본값 사용
- SQL 에러는 500 에러로 반환

### 2. 빈 결과
- 빈 배열 반환
- 총 페이지 수는 0으로 설정

## 개선 가능 사항

### 1. 필터링 기능
- 계정 상태별 필터링 (ACTIVE, PENDING_EMAIL 등)
- 역할별 필터링
- 날짜 범위 필터링

### 2. 고급 검색
- 전문 검색 (full-text search)
- 다중 필드 동시 검색

### 3. 성능 최적화
- 인덱스 추가 (username, email)
- 캐싱 전략 (Redis)

## 참고 사항
- 기존 `get_users_with_sorting` 메서드 활용으로 빠른 구현 가능
- Clean Architecture 패턴 준수
- 사용자 삭제는 논리적 삭제 (account_status = 'DELETED')

