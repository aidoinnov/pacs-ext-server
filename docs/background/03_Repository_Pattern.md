# 03. 리포지토리 패턴 (Repository Pattern)

리포지토리 패턴은 클린 아키텍처에서 **의존성 역전 원칙(Dependency Inversion Principle)**을 구현하는 핵심적인 디자인 패턴입니다. 비즈니스 로직(`application` 계층)과 데이터 접근 로직(`infrastructure` 계층)을 분리하는 역할을 합니다.

## 1. 패턴의 목적

-   **추상화**: 비즈니스 로직이 데이터베이스의 종류(PostgreSQL, MySQL 등)나 데이터 접근 방식(SQL, NoSQL 등)에 대해 전혀 알지 못하도록 추상화 계층을 제공합니다.
-   **테스트 용이성**: 데이터베이스에 실제로 연결하지 않고도, 리포지토리 Trait에 대한 가짜(Mock) 구현체를 만들어 유스케이스를 단위 테스트할 수 있게 합니다.
-   **유지보수성**: 데이터베이스 기술을 변경해야 할 때, `infrastructure` 계층의 리포지토리 구현체만 수정하면 되므로 변경의 영향 범위를 최소화할 수 있습니다.

## 2. 패턴의 구조

이 패턴은 두 부분으로 구성됩니다.

### a. 인터페이스 (Trait) - `domain` 계층

-   `domain/repositories` 폴더에 위치합니다.
-   필요한 데이터베이스 오퍼레이션을 메소드로 정의하는 `Trait` 입니다.
-   "무엇을" 해야 하는지만 정의하고, "어떻게" 할지는 정의하지 않습니다.

```rust
// in domain/repositories/user_repository.rs

use crate::domain::entities::User;
use uuid::Uuid;
use std::sync::Arc;

// Arc<dyn ...> 형태로 사용될 수 있도록 Sync + Send를 요구
#[async_trait::async_trait]
pub trait IUserRepository: Send + Sync {
    // "ID로 유저를 찾는다" 라는 행위를 정의
    async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>, Error>;

    // "이메일로 유저를 찾는다" 라는 행위를 정의
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Error>;

    // "유저를 저장한다" 라는 행위를 정의
    async fn save(&self, user: &User) -> Result<(), Error>;
}
```

### b. 구현체 (Implementation) - `infrastructure` 계층

-   `infrastructure/repositories` 폴더에 위치합니다.
-   `domain` 계층에 정의된 `Trait`을 실제로 구현하는 구조체입니다.
-   데이터베이스 커넥션 풀(`PgPool`)을 가지고 있으며, `sqlx` 같은 라이브러리를 사용해 실제 SQL 쿼리를 실행합니다.

```rust
// in infrastructure/repositories/user_repository_impl.rs

use crate::domain::entities::User;
use crate::domain::repositories::user_repository::IUserRepository;
use sqlx::PgPool;
use uuid::Uuid;

// 실제 DB 커넥션 풀을 멤버로 갖는 구조체
pub struct UserRepositoryImpl {
    pub db_pool: PgPool,
}

// IUserRepository Trait을 UserRepositoryImpl에 대해 구현
#[async_trait::async_trait]
impl IUserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>, Error> {
        // sqlx를 사용해 실제 SELECT 쿼리 실행
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        // ... 실제 쿼리 구현 ...
    }

    async fn save(&self, user: &User) -> Result<(), Error> {
        // ... 실제 INSERT/UPDATE 쿼리 구현 ...
    }
}
```

## 3. 의존성 주입 (Dependency Injection)

`application` 계층의 유스케이스는 구체적인 `UserRepositoryImpl`을 직접 사용하지 않습니다. 대신, 추상적인 `Arc<dyn IUserRepository>`를 주입받아 사용합니다.

```rust
// in application/use_cases/user_use_case.rs

use crate::domain::repositories::user_repository::IUserRepository;
use std::sync::Arc;

pub struct UserUseCase {
    // 구체적인 타입이 아닌 Trait 객체(dyn IUserRepository)에 의존
    pub user_repo: Arc<dyn IUserRepository>,
}

impl UserUseCase {
    pub async fn get_user(&self, user_id: Uuid) -> Result<User, Error> {
        // Trait에 정의된 메소드 호출
        let user = self.user_repo.find_by_id(user_id).await?;
        // ...
    }
}
```

실제 인스턴스(`UserRepositoryImpl`)는 프로그램이 시작되는 `main.rs`에서 생성되어 `UserUseCase`에 주입됩니다. 이로써 계층 간의 결합도가 크게 낮아집니다.
