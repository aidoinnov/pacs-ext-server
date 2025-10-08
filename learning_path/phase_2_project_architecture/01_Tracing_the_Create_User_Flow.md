# Phase 2-1: "사용자 생성" 기능 코드 흐름 추적하기

이 문서는 `POST /api/v1/users` API가 호출되었을 때, 각 아키텍처 계층을 거쳐 데이터가 처리되는 전체 과정을 단계별로 추적합니다. 이 흐름을 이해하면 프로젝트의 다른 모든 기능의 동작 방식을 쉽게 파악할 수 있습니다.

**전제**: 사용자가 API 클라이언트(Postman 등)를 통해 `{"email": "test@example.com", "password": "password123"}` JSON 본문과 함께 요청을 보냈다고 가정합니다.

--- 

### 1단계: `Presentation` - 라우팅 및 컨트롤러 호출

1.  **`main.rs` 또는 `presentation/routes.rs`**
    -   Actix-web 서버는 시작 시 등록된 라우팅 테이블을 확인합니다.
    -   `POST /api/v1/users` 경로가 `user_controller::create_user` 함수와 매핑된 것을 확인합니다.

2.  **`presentation/controllers/user_controller.rs`**
    -   `create_user` 함수가 호출됩니다.
    -   **Extractor**: Actix-web은 HTTP 요청 본문의 JSON을 `web::Json<CreateUserDto>` 타입으로 자동 역직렬화(deserialize)하여 `dto` 인자에 전달합니다.
    -   **State Injection**: Actix-web은 서버에 등록된 `app_data`에서 `web::Data<UserUseCase>`를 찾아 `use_case` 인자에 주입합니다.

    ```rust
    // in presentation/controllers/user_controller.rs
    pub async fn create_user(
        dto: web::Json<CreateUserDto>,      // <--- JSON 본문이 여기로 들어옴
        use_case: web::Data<UserUseCase>, // <--- 서버 상태에서 주입됨
    ) -> Result<HttpResponse, Error> {
        // 다음 단계: Application 계층의 유스케이스 호출
        let created_user = use_case.create_user(dto.into_inner()).await?;
        Ok(HttpResponse::Created().json(created_user))
    }
    ```

--- 

### 2단계: `Application` - 유스케이스 실행

1.  **`application/use_cases/user_use_case.rs`**
    -   `create_user` 메소드가 `CreateUserDto`를 인자로 받아 실행됩니다.
    -   **비즈니스 로직 (1)**: 이메일이 이미 존재하는지 확인하기 위해 `user_repo`를 호출합니다.
    -   **비즈니스 로직 (2)**: `CreateUserDto`를 `User` 도메인 엔티티로 변환합니다. 이 과정은 `User`의 생성자(`new`)에서 일어납니다.

    ```rust
    // in application/use_cases/user_use_case.rs
    impl UserUseCase {
        pub async fn create_user(&self, dto: CreateUserDto) -> Result<UserResponseDto, Error> {
            // 이메일 중복 검사 (리포지토리 호출)
            if self.user_repo.find_by_email(&dto.email).await?.is_some() {
                return Err(Error::Conflict("Email already exists".into()));
            }

            // 다음 단계: Domain 계층의 엔티티 생성
            let new_user = User::new(dto.email, dto.password);

            // 다음 단계: Domain 계층의 리포지토리 Trait 호출
            self.user_repo.save(&new_user).await?;

            Ok(UserResponseDto::from(new_user))
        }
    }
    ```

--- 

### 3단계: `Domain` - 엔티티 생성 및 리포지토리 Trait 호출

1.  **`domain/entities/user.rs`**
    -   `User::new()` 생성자 함수가 호출됩니다.
    -   **핵심 도메인 로직**: 평문 비밀번호를 받아 해시 처리하는 등, 엔티티의 일관성을 유지하기 위한 핵심 로직이 여기서 수행됩니다.

    ```rust
    // in domain/entities/user.rs
    impl User {
        pub fn new(email: String, plain_password: String) -> Self {
            // 비밀번호 해싱과 같은 핵심 로직 수행
            let password_hash = hash_password(plain_password);
            Self {
                id: Uuid::new_v4(),
                email,
                password_hash,
                is_active: true,
            }
        }
    }
    ```

2.  **`domain/repositories/user_repository.rs`**
    -   `UserUseCase`는 `self.user_repo.save(&new_user)`를 호출합니다.
    -   `user_repo`는 `Arc<dyn IUserRepository>` 타입이므로, 이 호출은 `IUserRepository` Trait에 정의된 `save` 메소드를 가리킵니다. `Application` 계층은 이 메소드가 실제로 어떻게 동작하는지는 전혀 모릅니다.

    ```rust
    // in domain/repositories/user_repository.rs
    #[async_trait::async_trait]
    pub trait IUserRepository: Send + Sync {
        // ... 다른 함수들 ...
        async fn save(&self, user: &User) -> Result<(), Error>; // <--- 이 함수가 호출됨
    }
    ```

--- 

### 4단계: `Infrastructure` - 리포지토리 구현 및 DB 실행

1.  **`infrastructure/repositories/user_repository_impl.rs`**
    -   `IUserRepository` Trait을 구현하는 `UserRepositoryImpl`의 `save` 메소드가 최종적으로 실행됩니다.
    -   이 메소드는 `PgPool` (PostgreSQL 커넥션 풀)을 사용하여 실제 데이터베이스에 `INSERT` 쿼리를 실행합니다.

    ```rust
    // in infrastructure/repositories/user_repository_impl.rs
    #[async_trait::async_trait]
    impl IUserRepository for UserRepositoryImpl {
        async fn save(&self, user: &User) -> Result<(), Error> {
            // sqlx를 사용하여 실제 DB 쿼리 실행
            sqlx::query(
                "INSERT INTO users (id, email, password_hash, is_active) VALUES ($1, $2, $3, $4)"
            )
            .bind(user.id)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(user.is_active)
            .execute(&self.db_pool) // <--- DB에 쿼리 전송
            .await?;

            Ok(())
        }
        // ...
    }
    ```

--- 

### 5단계: 결과 반환

-   DB 쿼리가 성공하면 `Ok(())`가 `Infrastructure` -> `Application` 계층으로 반환됩니다.
-   `UserUseCase`는 `User` 엔티티를 `UserResponseDto`로 변환하여 `Ok(dto)`를 `Presentation` 계층으로 반환합니다.
-   `user_controller`는 받은 `UserResponseDto`를 JSON으로 직렬화하고, `201 Created` 상태 코드와 함께 최종 `HttpResponse`를 생성하여 클라이언트에게 응답합니다.

이처럼 각 계층은 자신의 역할에만 충실하며, 의존성 규칙에 따라 상위 계층은 하위 계층의 인터페이스(Trait)에만 의존할 뿐, 구체적인 구현에는 전혀 의존하지 않습니다.
