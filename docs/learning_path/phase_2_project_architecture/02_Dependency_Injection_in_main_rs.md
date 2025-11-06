# Phase 2-2: `main.rs`의 의존성 주입(DI) 분석

의존성 주입(Dependency Injection, DI)은 클린 아키텍처를 실제로 동작하게 만드는 "접착제" 역할을 합니다. 애플리케이션의 각 구성 요소를 생성하고 서로 연결하는 과정이 모두 `main.rs`에서 이루어집니다. 이 과정을 이해하면 전체 애플리케이션의 구조를 거시적인 관점에서 파악할 수 있습니다.

## DI의 목적

-   **결합도 낮추기(Decoupling)**: `Application` 계층이 `Infrastructure` 계층의 구체적인 구현에 직접 의존하지 않도록 합니다. 유스케이스는 "리포지토리가 필요하다"고만 알면 되고, 그 리포지토리의 실제 구현이 `PostgresRepository`인지 `InMemoryTestRepository`인지는 알 필요가 없습니다.
-   **중앙 관리**: 애플리케이션의 모든 구성 요소와 그들의 생명주기가 `main.rs` 한 곳에서 관리되므로, 전체 구조를 파악하기 쉽습니다.
-   **테스트 용이성**: 테스트 시 실제 리포지토리 대신 가짜(Mock) 리포지토리를 쉽게 주입할 수 있습니다.

## 의존성 주입 체인 (Chain of Creation)

`main.rs`의 `main` 함수는 다음과 같은 순서로 구성 요소들을 생성하고 연결합니다.

```rust
// in main.rs (개념을 설명하기 위한 의사 코드에 가까운 예시)
use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

// ... import statements for Repositories, UseCases, Controllers, Routes ...

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // --- 1. 설정 및 외부 연결 초기화 ---
    // .env 파일 로드, 로깅 설정 등
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // 데이터베이스 커넥션 풀 생성. 애플리케이션 전체에서 공유됨.
    let db_pool = PgPool::connect(&database_url).await.expect("Failed to create pool.");


    // --- 2. 의존성 컨테이너(Dependency Container) 생성 ---
    // 이 구조체는 모든 의존성을 한 곳에 모아 관리하는 역할을 합니다.
    // (프로젝트에 따라 이런 구조체 없이 바로 main에서 생성할 수도 있습니다)
    pub struct AppState {
        pub user_use_case: UserUseCase,
        // pub project_use_case: ProjectUseCase, ...
    }

    // --- 3. Infrastructure 계층 생성 ---
    // 리포지토리 구현체를 생성합니다. DB 풀은 여러 스레드에서 공유되므로 Arc로 감쌉니다.
    let user_repo_impl = UserRepositoryImpl { db_pool: Arc::new(db_pool) };


    // --- 4. Application 계층 생성 ---
    // 유스케이스를 생성합니다. 이 때, 구체적인 구현체(user_repo_impl)를
    // Arc<dyn IUserRepository> Trait 객체로 변환하여 주입합니다.
    let user_use_case = UserUseCase {
        user_repo: Arc::new(user_repo_impl),
    };


    // --- 5. Presentation 계층(Actix-web)에 상태 등록 ---
    // 유스케이스를 포함한 AppState를 Actix-web이 관리하는 상태(app_data)로 등록합니다.
    // web::Data는 내부적으로 Arc를 사용하므로, 여러 스레드에서 안전하게 상태를 공유할 수 있습니다.
    let app_state = web::Data::new(AppState {
        user_use_case,
    });

    println!("🚀 Server starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) // 모든 컨트롤러에서 app_state를 사용할 수 있도록 등록
            .configure(routes::configure_routes) // 라우팅 설정
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### 핵심적인 스마트 포인터

-   **`Arc<T>` (Atomic Reference Counted)**
    -   **역할**: 멀티스레드 환경에서 데이터를 안전하게 **공유 소유**할 수 있게 합니다.
    -   **사용처**: `db_pool`이나 리포지토리 구현체처럼, 여러 유스케이스나 스레드에서 동시에 접근해야 하는 객체를 감쌀 때 사용됩니다. `.clone()`을 호출해도 데이터 자체가 복사되는 것이 아니라, 참조 카운트만 1 증가하므로 매우 효율적입니다.

-   **`web::Data<T>` (Actix-web의 상태 관리 타입)**
    -   **역할**: Actix-web의 애플리케이션 상태를 관리합니다. 내부적으로 `Arc`를 사용하여 스레드 안전성을 보장합니다.
    -   **사용처**: 생성된 유스케이스(또는 `AppState`)를 `HttpServer`의 모든 워커 스레드가 공유할 수 있도록 등록할 때 사용됩니다. 컨트롤러에서는 `web::Data<AppState>` 형태로 이 상태를 주입받습니다.

이처럼 `main.rs`는 애플리케이션의 모든 부분을 조립하는 "조립 공장"과 같습니다. 새로운 서비스나 리포지토리를 추가할 때, 이 DI 체인을 따라 `main.rs`에 등록해주는 과정이 반드시 필요합니다.
