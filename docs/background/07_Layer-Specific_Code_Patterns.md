# 07. 계층별 코드 작성 패턴 (Layer-Specific Code Patterns)

이 문서는 클린 아키텍처의 각 계층(Presentation, Application, Domain, Infrastructure)에서 준수해야 할 표준 코드 작성 패턴을 상세히 설명합니다.

---

## 1. Presentation Layer (프레젠테이션 계층)

> **핵심 역할**: HTTP 요청/응답 처리 및 `Application` 계층 호출

### 패턴 1: 컨트롤러 함수의 정형화된 구조

모든 컨트롤러 함수는 다음 구조를 따릅니다.

-   **Extractor 사용**: `web::Json`, `web::Path`, `web::Query`를 사용해 요청 데이터를 안전하게 추출하고 역직렬화합니다.
-   **상태 주입**: `web::Data<T>`를 사용해 `main.rs`에서 등록한 유스케이스/서비스 객체를 주입받습니다.
-   **표준 반환 타입**: 항상 `Result<HttpResponse, Error>`를 반환합니다.
-   **응답 생성**: `HttpResponse` 빌더를 사용해 상태 코드(200, 201, 204 등)와 본문(JSON)을 명시적으로 지정합니다.

```rust
// in presentation/controllers/user_controller.rs

use crate::application::dto::user_dto::CreateUserDto;
use crate::application::use_cases::user_use_case::UserUseCase;
use actix_web::{web, HttpResponse, Result};

// POST /users
pub async fn create_user(
    dto: web::Json<CreateUserDto>,      // 1. Extractor: 요청 본문을 DTO로 받음
    use_case: web::Data<UserUseCase>, // 2. State Injection: 유스케이스 주입
) -> Result<HttpResponse, Error> {      // 3. Standard Return Type
    // Application 계층 호출
    let created_user = use_case.create_user(dto.into_inner()).await?;

    // 4. Response Builder: 성공 응답 생성
    Ok(HttpResponse::Created().json(created_user))
}
```

### 패턴 2: 에러 응답 변환

컨트롤러는 `?` 연산자를 통해 하위 계층의 에러를 그대로 반환합니다. `actix-web`의 에러 처리 메커니즘(또는 커스텀 구현)이 `Error` 타입을 적절한 `HttpResponse`(예: `404 Not Found`, `400 Bad Request`)로 변환해 줍니다.

---

## 2. Application Layer (애플리케이션 계층)

> **핵심 역할**: 유스케이스(비즈니스 흐름) 정의, DTO 사용, 도메인 객체 조율

### 패턴 1: 유스케이스 구조체와 의존성

유스케이스 구조체는 리포지토리의 실제 구현이 아닌, `domain` 계층에 정의된 `Trait`에 의존합니다.

```rust
// in application/use_cases/user_use_case.rs

use crate::domain::repositories::user_repository::IUserRepository;
use std::sync::Arc;

pub struct UserUseCase {
    // 구체적인 UserRepositoryImpl이 아닌, 추상적인 Trait에 의존
    pub user_repo: Arc<dyn IUserRepository>,
}
```

### 패턴 2: DTO 정의 및 유효성 검사

-   **입력 DTO**: `serde::Deserialize`와 `validator::Validate`를 사용하여 요청 데이터를 받고 유효성을 검사합니다.
-   **출력 DTO**: `serde::Serialize`를 사용하여 응답 데이터를 구성합니다. `password` 같은 민감한 정보는 제외합니다.

```rust
// in application/dto/user_dto.rs

#[derive(Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub email: String,
}
```

### 패턴 3: 도메인 엔티티와 DTO 간의 변환

유스케이스는 DTO를 도메인 엔티티로 변환하여 로직을 처리하고, 결과를 다시 DTO로 변환하여 반환합니다. `From` Trait을 구현하여 변환 로직을 표준화합니다.

```rust
// in application/use_cases/user_use_case.rs
impl UserUseCase {
    pub async fn create_user(&self, dto: CreateUserDto) -> Result<UserResponseDto, Error> {
        // ... 이메일 중복 검사 로직 ...

        // 1. DTO를 기반으로 도메인 엔티티 생성
        let new_user = User::new(dto.email, dto.password);

        // 2. 리포지토리를 통해 엔티티 저장
        self.user_repo.save(&new_user).await?;

        // 3. 엔티티를 응답 DTO로 변환하여 반환
        Ok(UserResponseDto::from(new_user))
    }
}

// in application/dto/user_dto.rs
impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self { id: user.id, email: user.email }
    }
}
```

---

## 3. Domain Layer (도메인 계층)

> **핵심 역할**: 순수한 비즈니스 규칙과 데이터(엔티티) 정의

### 패턴 1: "Rich" 엔티티 모델

엔티티는 단순한 데이터 덩어리가 아니라, 관련된 비즈니스 로직을 메소드로 포함해야 합니다.

```rust
// in domain/entities/user.rs

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
}

impl User {
    // 생성자: 비즈니스 규칙(비밀번호 해싱)을 포함
    pub fn new(email: String, plain_password: String) -> Self {
        let password_hash = hash_password(plain_password); // 가상 함수
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            is_active: true,
        }
    }

    // 상태를 변경하는 자체 비즈니스 로직
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    // 비밀번호 검증 로직
    pub fn verify_password(&self, plain_password: &str) -> bool {
        verify_hash(plain_password, &self.password_hash)
    }
}
```

### 패턴 2: 리포지토리 Trait 정의

데이터 영속성을 위한 인터페이스를 정의합니다. 메소드는 도메인 엔티티를 인자로 받거나 반환합니다.

```rust
// in domain/repositories/user_repository.rs

use crate::domain::entities::User;

#[async_trait::async_trait]
pub trait IUserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error>;
    async fn save(&self, user: &User) -> Result<(), Error>;
}
```

---

## 4. Infrastructure Layer (인프라스트럭처 계층)

> **핵심 역할**: 외부 기술(DB, API, 프레임워크)에 대한 구체적인 로직 구현

### 패턴 1: 리포지토리 Trait 구현

`domain` 계층의 Trait을 `sqlx`와 같은 라이브러리를 사용하여 실제로 구현합니다.

```rust
// in infrastructure/repositories/user_repository_impl.rs

use sqlx::PgPool;

pub struct UserRepositoryImpl {
    pub db_pool: PgPool,
}

#[async_trait::async_trait]
impl IUserRepository for UserRepositoryImpl {
    async fn save(&self, user: &User) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, is_active) VALUES ($1, $2, $3, $4)"
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.is_active)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
    // ... 다른 메소드 구현
}
```

### 패턴 2: `main.rs`에서의 의존성 주입 체인

애플리케이션 시작 시, 모든 의존성을 생성하고 연결하여 웹 서버에 등록합니다.

```rust
// in main.rs (개념)

async fn main() {
    // 1. DB 커넥션 풀 생성
    let db_pool = Arc::new(PgPool::connect(...).await.unwrap());

    // 2. 리포지토리 구현체 생성 (Arc<PgPool> 복제)
    let user_repo_impl = Arc::new(UserRepositoryImpl { db_pool: db_pool.clone() });

    // 3. 유스케이스 생성 (Arc<dyn IUserRepository>로 타입 변환하여 주입)
    let user_use_case = web::Data::new(UserUseCase { user_repo: user_repo_impl });

    // 4. 웹 서버에 유스케이스 등록
    HttpServer::new(move || {
        App::new().app_data(user_use_case.clone())
    })
    .run()
    .await;
}
```
