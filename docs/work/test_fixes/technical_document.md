# 테스트 파일 수정 기술 문서

## 🔧 기술적 배경

### Rust 테스트 시스템
- **Unit Test**: `#[cfg(test)]` 모듈 내부에서 실행
- **Integration Test**: `tests/` 디렉토리의 독립 파일
- **Mock Library**: mockall 크레이트 사용

### 현재 프로젝트 구조
```
pacs-server/
├── src/
│   ├── domain/          # 도메인 계층
│   ├── application/      # 애플리케이션 계층
│   ├── infrastructure/   # 인프라 계층
│   └── presentation/     # 프레젠테이션 계층
└── tests/               # Integration 테스트
```

## 📝 수정 유형 분석

### 1. Entity 초기화 문제

#### 문제 패턴
엔티티에 필드가 추가되었지만 테스트에서는 기존 필드만 사용하는 경우

#### 해결 방법
```rust
// Before
let user = User {
    id: 1,
    username: "test".to_string(),
    email: "test@example.com".to_string(),
    created_at: Utc::now(),
};

// After
let user = User {
    id: 1,
    username: "test".to_string(),
    email: "test@example.com".to_string(),
    full_name: None,                    // 추가됨
    organization: None,                  // 추가됨
    department: None,                    // 추가됨
    phone: None,                        // 추가됨
    created_at: Utc::now(),
    updated_at: None,                    // 추가됨
    account_status: UserAccountStatus::Active,  // 추가됨
    email_verified: true,               // 추가됨
    // ... 기타 필드
};
```

### 2. Import 경로 문제

#### 문제 패턴
모듈이 private에서 public으로 변경되었지만 import 경로가 업데이트되지 않은 경우

#### 해결 방법
```rust
// Before
use pacs_server::infrastructure::repositories::user_repository_impl::UserRepositoryImpl;

// After
use pacs_server::infrastructure::repositories::UserRepositoryImpl;
```

### 3. Service Trait 시그니처 변경

#### 문제 패턴
Repository의 메서드 시그니처가 변경되어 Mock과 불일치

#### 해결 방법
```rust
// Mock에서 실제 trait 시그니처 확인 후 수정
impl UserRepository for MockUserRepository {
    // Before
    async fn create(&self, user: &User) -> Result<User, sqlx::Error>;
    
    // After (trait 정의와 일치)
    async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error>;
}
```

### 4. DTO 필드 변경

#### 문제 패턴
DTO에 필드 추가/제거 시 테스트 코드가 업데이트되지 않은 경우

#### 해결 방법
```rust
// CreateMaskGroupRequest에서 annotation_id 제거
let request = CreateMaskGroupRequest {
    // annotation_id: 1,  // 제거됨
    group_name: Some("Test".to_string()),
};

// CreateAnnotationRequest에 measurement_values 추가
let request = CreateAnnotationRequest {
    // ... 기존 필드
    measurement_values: None,  // 추가됨
};
```

### 5. Mock Complexity 문제

#### 문제 패턴
복잡한 Generic 구조를 가진 Service에 대한 Mock 생성 어려움

#### 예시: PermissionService
```rust
// PermissionService의 실제 구현
pub struct PermissionServiceImpl<
    P: PermissionRepository, 
    R: RoleRepository
> {
    permission_repository: P,
    role_repository: R,
}

// Mock 생성 시 문제
// - Generic 타입 파라미터가 2개
// - mockall은 단일 타입에 대해서만 완벽하게 동작
// - 2개 타입에 대한 Mock 생성이 복잡함
```

#### 해결책
1. 실제 구현체를 사용한 통합 테스트
2. Trait 분리 (더 단순한 단위로)
3. Test Helper Library 구축

### 6. Pool Dereference 문제

#### 문제 패턴
`Arc<Pool<Postgres>>` 사용 시 불필요한 dereference

#### 해결 방법
```rust
// Before
let repository = Repository::new((*pool).clone());

// After
let repository = Repository::new(pool.clone());
```

## 🛠️ 주요 수정 패턴

### Pattern A: Helper 함수 사용
테스트 데이터 생성 함수를 만들어 중복 제거

```rust
fn create_test_user(id: i32, username: String, email: String) -> User {
    User {
        id,
        keycloak_id: Uuid::new_v4(),
        username,
        email,
        full_name: None,
        // ... 모든 필드
    }
}
```

### Pattern B: Default 값 사용
Option 필드에 None 기본값 사용

```rust
let user = User {
    id: 1,
    username: "test".to_string(),
    // ... 필수 필드만
    ..Default::default()  // 나머지는 기본값
};
```

### Pattern C: Builder Pattern
복잡한 객체 생성 시 Builder 사용

```rust
let user = TestUserBuilder::new()
    .id(1)
    .username("test")
    .email("test@example.com")
    .build();
```

## 🔍 디버깅 팁

### 1. Compile Error 분석
```bash
cargo test --test <test_file> --no-run 2>&1 | grep "error\[E"
```

### 2. Import 문제 확인
```rust
// Private module 확인
cargo doc --open

// Public API 확인
cargo doc --all-features --no-deps
```

### 3. Trait 시그니처 확인
```rust
// Source code에서 trait 정의 확인
grep "pub trait" src/domain/services/

// 시그니처 직접 확인
cat src/domain/services/permission_service.rs
```

### 4. Entity 구조 확인
```rust
// Entity 정의 확인
cat src/domain/entities/user.rs

// Database schema 확인
cat migrations/*.sql
```

## 💡 Best Practices

### 1. 테스트 격리
- 각 테스트는 독립적으로 실행 가능해야 함
- 공유 상태 피하기
- Mock 사용 시 각 테스트마다 새로 생성

### 2. Mock 관리
- 단순한 상황에서는 직접 Mock 구현
- 복잡한 경우 실제 구현체 사용 고려
- 통합 테스트와 단위 테스트 구분

### 3. 테스트 데이터 관리
- Helper 함수로 테스트 데이터 생성
- Fixture 사용
- Factory pattern 활용

### 4. 변경 영향도 최소화
- 테스트 헬퍼 함수 사용
- 일반화된 Mock trait
- Test fixture library

## 🚨 주의사항

### 1. Side Effect
- Database 조회/변경 시 실제 DB 연결 필요
- Test DB 사용 권장
- 각 테스트 후 cleanup

### 2. Mock vs Real
- Mock은 단순한 단위 테스트에 적합
- 복잡한 비즈니스 로직은 통합 테스트
- 성능이 중요한 경우에는 실제 구현체 사용

### 3. Async Test
- `#[tokio::test]` 사용
- `async fn` 필수
- Mock도 `#[async_trait]` 사용

## 📊 통계

### 수정된 파일 유형
- Entity 초기화 문제: 8개
- Import 경로 문제: 5개
- Service Trait 문제: 4개
- DTO 변경: 3개
- Complex Integration: 2개
- 기타: 3개

### 수정 패턴 빈도
1. Entity 필드 추가 (가장 빈번)
2. Import 경로 수정
3. Mock 메서드 시그니처 조정
4. DTO 필드 추가/제거

