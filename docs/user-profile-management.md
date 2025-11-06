# 사용자 프로필 관리 시스템

## 개요

PACS Extension Server의 사용자 프로필 관리 시스템은 사용자의 기본 정보와 확장된 프로필 정보를 관리하는 종합적인 시스템입니다. 이 문서는 사용자 프로필 업데이트 기능의 구현, API 사용법, 그리고 기술적 세부사항을 다룹니다.

## 주요 기능

### 1. 확장된 사용자 프로필 필드

기존 사용자 테이블에 다음 필드들이 추가되었습니다:

- **`full_name`** (TEXT): 사용자의 실명 (한글명/영문명)
- **`organization`** (TEXT): 소속 기관 (예: "서울대학교병원")
- **`department`** (TEXT): 소속 부서/그룹 (예: "영상의학과")
- **`phone`** (TEXT): 연락처 (예: "010-1234-5678")
- **`updated_at`** (TIMESTAMPTZ): 마지막 업데이트 시각

### 2. 부분 업데이트 지원

사용자는 필요한 필드만 선택적으로 업데이트할 수 있습니다. 제공되지 않은 필드는 기존 값을 유지합니다.

### 3. 데이터 무결성 보장

- **Username과 keycloak_id는 변경 불가**: 시스템 식별자로 보호
- **이메일 중복 검사**: 업데이트 시 이메일 중복성 검증
- **자동 타임스탬프**: `updated_at` 필드 자동 업데이트

## API 엔드포인트

### 사용자 프로필 업데이트

```http
PUT /api/users/{user_id}
Content-Type: application/json
Authorization: Bearer <jwt_token>
```

#### 요청 본문

```json
{
  "full_name": "홍길동",
  "email": "hong@example.com",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

#### 응답

**성공 (200 OK)**
```json
{
  "id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "hong_gd",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-02T00:00:00Z"
}
```

**에러 응답**

- **400 Bad Request**: 유효하지 않은 요청 데이터
- **401 Unauthorized**: 인증 실패
- **403 Forbidden**: 권한 부족
- **404 Not Found**: 사용자를 찾을 수 없음
- **409 Conflict**: 이메일 중복

## 데이터베이스 스키마

### 테이블 구조

```sql
CREATE TABLE security_user (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    keycloak_id UUID UNIQUE NOT NULL,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    full_name TEXT,
    organization TEXT,
    department TEXT,
    phone TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

### 인덱스

```sql
-- 이름 검색을 위한 인덱스
CREATE INDEX idx_user_full_name ON security_user(full_name);

-- 기관 검색을 위한 인덱스
CREATE INDEX idx_user_organization ON security_user(organization);
```

### 자동 업데이트 트리거

```sql
-- updated_at 자동 업데이트 함수
CREATE OR REPLACE FUNCTION update_user_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 트리거 생성
CREATE TRIGGER trigger_update_user_updated_at
BEFORE UPDATE ON security_user
FOR EACH ROW
EXECUTE FUNCTION update_user_updated_at();
```

## 구현 세부사항

### 1. Clean Architecture 구조

```
Domain Layer
├── entities/user.rs (User, UpdateUser, NewUser)
├── repositories/user_repository.rs (UserRepository trait)
└── services/user_service.rs (UserService trait)

Application Layer
├── dto/user_dto.rs (UpdateUserRequest, UserResponse)
└── use_cases/user_use_case.rs (UserUseCase)

Infrastructure Layer
└── repositories/user_repository_impl.rs (UserRepositoryImpl)

Presentation Layer
└── controllers/user_controller.rs (UserController)
```

### 2. UpdateUser 엔티티 (Builder 패턴)

```rust
#[derive(Debug, Clone)]
pub struct UpdateUser {
    pub id: i32,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub organization: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
}

impl UpdateUser {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            email: None,
            full_name: None,
            organization: None,
            department: None,
            phone: None,
        }
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    // ... 다른 필드들
}
```

### 3. Repository 구현

```rust
impl UserRepository for UserRepositoryImpl {
    async fn update(&self, update_user: &UpdateUser) -> Result<User, ServiceError> {
        let mut query = sqlx::QueryBuilder::new("UPDATE security_user SET ");
        let mut params = Vec::new();
        let mut field_count = 0;

        // 동적 필드 업데이트
        if let Some(email) = &update_user.email {
            if field_count > 0 {
                query.push(", ");
            }
            query.push("email = $");
            query.push(field_count + 1);
            params.push(email);
            field_count += 1;
        }

        // ... 다른 필드들

        if field_count == 0 {
            return Err(ServiceError::ValidationError("No fields to update".to_string()));
        }

        query.push(" WHERE id = $");
        query.push(field_count + 1);
        params.push(update_user.id);
        query.push(" RETURNING *");

        let user = query
            .build_query_as::<User>()
            .bind_all(&params)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}
```

## 보안 고려사항

### 1. 인증 및 권한

- JWT 토큰을 통한 사용자 인증
- 사용자는 자신의 프로필만 업데이트 가능 (또는 관리자 권한 필요)
- Username과 keycloak_id는 시스템 식별자로 변경 불가

### 2. 데이터 검증

- 이메일 형식 검증
- 이메일 중복성 검사
- 필드 길이 제한
- SQL 인젝션 방지 (매개변수화된 쿼리 사용)

### 3. 감사 로그

- 모든 프로필 업데이트는 `updated_at` 필드에 기록
- 향후 확장 시 변경 이력 추적 가능

## 성능 최적화

### 1. 데이터베이스 최적화

- 필요한 필드만 업데이트하는 동적 쿼리
- 인덱스를 통한 검색 성능 향상
- 트리거를 통한 자동 타임스탬프 관리

### 2. 메모리 효율성

- Builder 패턴을 통한 선택적 필드 설정
- 불필요한 데이터 복사 방지

## 마이그레이션 가이드

### 1. 데이터베이스 마이그레이션 실행

```bash
# 마이그레이션 실행
sqlx migrate run

# 마이그레이션 상태 확인
sqlx migrate info
```

### 2. 기존 데이터 처리

- 기존 사용자 데이터는 새 필드가 NULL로 설정됨
- 필요에 따라 기존 데이터를 점진적으로 업데이트 가능
- 하위 호환성 보장

## 테스트

### 1. 단위 테스트

```rust
#[tokio::test]
async fn test_update_user_profile() {
    // 테스트 데이터 생성
    let user = create_test_user().await;
    
    // 프로필 업데이트
    let update_request = UpdateUserRequest {
        full_name: Some("홍길동".to_string()),
        organization: Some("서울대학교병원".to_string()),
        department: Some("영상의학과".to_string()),
        phone: Some("010-1234-5678".to_string()),
        ..Default::default()
    };
    
    let result = user_use_case.update_user(user.id, update_request).await;
    assert!(result.is_ok());
    
    // 결과 검증
    let updated_user = result.unwrap();
    assert_eq!(updated_user.full_name, Some("홍길동".to_string()));
    assert_eq!(updated_user.organization, Some("서울대학교병원".to_string()));
}
```

### 2. 통합 테스트

```rust
#[tokio::test]
async fn test_update_user_api() {
    let app = create_test_app().await;
    let user = create_test_user().await;
    let token = create_test_token(&user).await;
    
    let update_data = json!({
        "full_name": "홍길동",
        "organization": "서울대학교병원",
        "department": "영상의학과",
        "phone": "010-1234-5678"
    });
    
    let response = app
        .put(&format!("/api/users/{}", user.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .insert_header(("Content-Type", "application/json"))
        .send_json(&update_data)
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}
```

## 문제 해결

### 1. 일반적인 문제

**문제**: S3 signed URL 생성 오류
**해결**: 환경변수가 TOML 파일의 하드코딩된 값보다 우선순위를 가지도록 설정 확인

**문제**: 데이터베이스 쿼리 오류
**해결**: `measurement_values` 컬럼이 모든 관련 쿼리에 포함되었는지 확인

### 2. 디버깅 팁

- 환경변수 로딩 확인: `RUST_LOG=debug`로 설정하여 로그 확인
- 데이터베이스 쿼리 확인: SQLx 로그 활성화
- API 요청/응답 확인: HTTP 클라이언트 로그 활성화

## 향후 계획

### 1. 단기 계획

- 사용자 프로필 이미지 업로드 기능
- 프로필 검색 및 필터링 기능
- 사용자 활동 로그 추적

### 2. 장기 계획

- 소셜 로그인 통합
- 사용자 그룹 관리
- 고급 권한 관리 시스템

## 참고 자료

- [Clean Architecture 가이드](./clean-architecture-guide.md)
- [API 문서](./api-documentation.md)
- [데이터베이스 스키마](./database-schema.md)
- [보안 가이드](./security-guide.md)
