# Role-Capability Matrix API 페이지네이션 및 검색 기능 기술문서

## 1. 아키텍처 개요

### 1.1 Clean Architecture 적용
```
Presentation Layer (Controller)
    ↓
Application Layer (Use Case)
    ↓
Domain Layer (Service Interface)
    ↓
Infrastructure Layer (Repository Implementation)
    ↓
Database (PostgreSQL)
```

### 1.2 계층별 책임
- **Controller**: HTTP 요청/응답 처리, 파라미터 검증
- **Use Case**: 비즈니스 로직, 페이지네이션 계산
- **Service**: 도메인 서비스 인터페이스
- **Repository**: 데이터 접근 추상화
- **Database**: 실제 데이터 저장 및 조회

## 2. 데이터 모델

### 2.1 DTO 구조
```rust
// 페이지네이션 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct PaginationInfo {
    pub current_page: i32,      // 현재 페이지
    pub page_size: i32,         // 페이지 크기
    pub total_pages: i32,       // 총 페이지 수
    pub total_items: i64,       // 총 항목 수
    pub has_next: bool,         // 다음 페이지 존재 여부
    pub has_previous: bool,     // 이전 페이지 존재 여부
}

// 쿼리 파라미터
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RoleCapabilityMatrixQuery {
    pub page: Option<i32>,      // 페이지 번호
    pub size: Option<i32>,      // 페이지 크기
    pub search: Option<String>, // 검색어
    pub scope: Option<String>,  // 범위 필터
}
```

### 2.2 데이터베이스 스키마
```sql
-- 역할 테이블
CREATE TABLE security_role (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    scope VARCHAR(20) NOT NULL CHECK (scope IN ('GLOBAL', 'PROJECT')),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 역량 테이블
CREATE TABLE security_capability (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 역할-역량 매핑 테이블
CREATE TABLE security_role_capability (
    role_id INTEGER REFERENCES security_role(id) ON DELETE CASCADE,
    capability_id INTEGER REFERENCES security_capability(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, capability_id)
);
```

## 3. 구현 세부사항

### 3.1 Repository 계층 구현

#### 동적 SQL 쿼리 구성
```rust
async fn get_global_role_capability_matrix_paginated(
    &self,
    page: i32,
    size: i32,
    search: Option<&str>,
    scope: Option<&str>,
) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>, i64), sqlx::Error> {
    let offset = (page - 1) * size;
    
    // 검색 조건 구성
    let mut where_conditions = vec!["scope = 'GLOBAL'".to_string()];
    let mut search_param = None;
    let mut scope_param = None;
    let mut param_count = 0;

    // 검색 조건 추가
    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            param_count += 1;
            where_conditions.push(format!(
                "(name ILIKE ${} OR description ILIKE ${})", 
                param_count, param_count + 1
            ));
            search_param = Some(format!("%{}%", search_term));
            param_count += 1;
        }
    }

    // 범위 필터 추가
    if let Some(scope_filter) = scope {
        if !scope_filter.trim().is_empty() {
            param_count += 1;
            where_conditions.push(format!("scope = ${}", param_count));
            scope_param = Some(scope_filter.to_string());
        }
    }

    let where_clause = where_conditions.join(" AND ");
    
    // 총 개수 조회
    let count_query_string = format!(
        "SELECT COUNT(*) FROM security_role WHERE {}",
        where_clause
    );
    
    // 페이지네이션된 데이터 조회
    let roles_query_string = format!(
        "SELECT id, name, description, scope, created_at
         FROM security_role
         WHERE {}
         ORDER BY name
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_count + 1,
        param_count + 2
    );
    
    // ... 쿼리 실행 로직
}
```

### 3.2 Use Case 계층 구현

#### 페이지네이션 계산 로직
```rust
pub async fn get_global_matrix_paginated(
    &self,
    page: i32,
    size: i32,
    search: Option<String>,
    scope: Option<String>,
) -> Result<RoleCapabilityMatrixResponse, ServiceError> {
    let (roles, capabilities, assignments, total_count) = self.capability_service
        .get_global_role_capability_matrix_paginated(page, size, search.as_deref(), scope.as_deref())
        .await?;

    // 페이지네이션 정보 계산
    let total_pages = (total_count as f64 / size as f64).ceil() as i32;
    let pagination = PaginationInfo {
        current_page: page,
        page_size: size,
        total_pages,
        total_items: total_count,
        has_next: page < total_pages,
        has_previous: page > 1,
    };

    // ... 응답 구성 로직
}
```

### 3.3 Controller 계층 구현

#### 파라미터 검증 및 기본값 설정
```rust
pub async fn get_global_matrix_paginated(
    query: web::Query<RoleCapabilityMatrixQuery>,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    // 파라미터 검증 및 기본값 설정
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(10).min(100).max(1);

    match use_case.get_global_matrix_paginated(
        page,
        size,
        query.search.clone(),
        query.scope.clone(),
    ).await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(e) => {
            // 에러 처리 로직
        }
    }
}
```

## 4. 성능 최적화

### 4.1 데이터베이스 쿼리 최적화
- **인덱스 활용**: `name`, `scope` 컬럼에 인덱스 생성
- **LIMIT/OFFSET**: 효율적인 페이지네이션
- **동적 쿼리**: 필요한 조건만 적용

### 4.2 메모리 사용량 최적화
- **페이지 단위 로딩**: 필요한 데이터만 메모리에 로드
- **스트리밍**: 대용량 데이터 처리 시 스트리밍 방식 적용

### 4.3 캐싱 전략
- **Redis 캐싱**: 자주 조회되는 데이터 캐싱
- **TTL 설정**: 캐시 만료 시간 설정

## 5. 에러 처리

### 5.1 파라미터 검증
```rust
// 페이지 번호 검증
let page = query.page.unwrap_or(1).max(1);

// 페이지 크기 검증
let size = query.size.unwrap_or(10).min(100).max(1);
```

### 5.2 데이터베이스 에러 처리
```rust
match use_case.get_global_matrix_paginated(...).await {
    Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
    Err(ServiceError::DatabaseError(msg)) => {
        tracing::error!("Database error: {}", msg);
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error",
            "message": msg
        })))
    }
    Err(e) => {
        tracing::error!("Error: {:?}", e);
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Internal server error",
            "message": e.to_string()
        })))
    }
}
```

## 6. 보안 고려사항

### 6.1 SQL 인젝션 방지
- **Prepared Statements**: SQLx의 prepared statement 사용
- **파라미터 바인딩**: 모든 사용자 입력을 파라미터로 바인딩

### 6.2 입력 검증
- **타입 검증**: 파라미터 타입 검증
- **범위 검증**: 페이지 크기, 페이지 번호 범위 검증
- **길이 제한**: 검색어 길이 제한

## 7. 테스트 전략

### 7.1 단위 테스트
- **Repository 테스트**: 데이터베이스 쿼리 로직 테스트
- **Use Case 테스트**: 비즈니스 로직 테스트
- **Controller 테스트**: HTTP 요청/응답 테스트

### 7.2 통합 테스트
- **API 테스트**: 전체 API 플로우 테스트
- **데이터베이스 테스트**: 실제 데이터베이스 연동 테스트

### 7.3 성능 테스트
- **부하 테스트**: 대용량 데이터 처리 성능 테스트
- **응답 시간 테스트**: API 응답 시간 측정

## 8. 모니터링 및 로깅

### 8.1 로깅 전략
```rust
// 성공 케이스
tracing::info!("Role-Capability matrix retrieved successfully: page={}, size={}", page, size);

// 에러 케이스
tracing::error!("Database error in get_global_matrix_paginated: {}", msg);
```

### 8.2 메트릭 수집
- **API 호출 횟수**: 엔드포인트별 호출 횟수
- **응답 시간**: 평균 응답 시간
- **에러율**: 에러 발생 비율

## 9. 배포 및 운영

### 9.1 데이터베이스 마이그레이션
- **스키마 변경**: 기존 테이블 구조 유지
- **인덱스 추가**: 성능 최적화를 위한 인덱스 추가

### 9.2 API 버전 관리
- **하위 호환성**: 기존 API 엔드포인트 유지
- **버전 관리**: API 버전 관리 전략

## 10. 향후 개선사항

### 10.1 기능 개선
- **정렬 기능**: 다양한 기준으로 정렬
- **고급 필터링**: 복합 조건 필터링
- **자동완성**: 검색어 자동완성

### 10.2 성능 개선
- **캐싱**: Redis 캐싱 도입
- **인덱스 최적화**: 데이터베이스 인덱스 최적화
- **쿼리 최적화**: 복잡한 쿼리 최적화

### 10.3 사용자 경험 개선
- **실시간 검색**: 타이핑 시 실시간 검색 결과
- **검색 히스토리**: 사용자별 검색 히스토리
- **즐겨찾기**: 자주 사용하는 검색 조건 저장
