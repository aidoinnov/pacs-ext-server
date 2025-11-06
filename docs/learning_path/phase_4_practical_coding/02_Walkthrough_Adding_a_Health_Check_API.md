# Phase 4-2: [실습] Health Check API 추가하기

이 문서는 학습한 모든 내용을 바탕으로, `GET /api/v1/health` 라는 새로운 API 엔드포인트를 추가하는 전체 과정을 안내하는 실습 가이드입니다. 이 과정을 통해 아키텍처의 각 계층에 코드를 추가하는 방법을 익힐 수 있습니다.

## 목표

-   **기본**: 서버가 살아있는지 확인하는 간단한 `{"status": "ok"}` JSON을 반환하는 API를 만듭니다.
-   **심화**: 데이터베이스 연결 상태까지 확인하여 `{"server": "ok", "database": "ok"}` 와 같은 더 상세한 정보를 반환하는 API로 확장합니다.

--- 

### 1단계: `Presentation` - 컨트롤러와 라우트 추가

#### 1. 컨트롤러 파일 생성

`pacs-server/src/presentation/controllers/health_controller.rs` 파일을 새로 만듭니다.

```rust
// pacs-server/src/presentation/controllers/health_controller.rs

use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
}

// 기본 목표 핸들러
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthStatus { 
        status: "ok".to_string() 
    })
}
```

#### 2. 컨트롤러 모듈 등록

`pacs-server/src/presentation/controllers/mod.rs` 파일에 새로 만든 `health_controller`를 모듈로 추가합니다.

```rust
// pacs-server/src/presentation/controllers/mod.rs

pub mod access_control_controller;
// ... 다른 컨트롤러들 ...
pub mod health_controller; // <--- 이 줄 추가
```

#### 3. 라우트 추가

`main.rs` 또는 `presentation/routes.rs` 파일에 새로운 라우트를 등록합니다.

```rust
// in main.rs or routes.rs

// ... 다른 use 선언들 ...
use crate::presentation::controllers::health_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_controller::health_check)) // <--- 이 줄 추가
            // ... 다른 라우트들 ...
    );
}
```

#### 4. 동작 확인 (기본 목표)

터미널에서 `cargo run`으로 서버를 실행하고, 새 터미널에서 아래 명령어를 실행합니다.

```bash
curl http://localhost:8080/api/v1/health
```

`{"status":"ok"}` 라는 응답이 오면 성공입니다.

--- 

### 2단계 (심화): `Application`과 `Infrastructure` 연동

이제 데이터베이스 연결 상태까지 확인하도록 기능을 확장해 보겠습니다.

#### 1. `Domain` - 리포지토리 Trait 수정 (또는 신규 생성)

DB 연결 상태 확인은 특정 도메인과 무관하므로, 범용적인 `HealthRepository`를 새로 만들 수 있습니다.

`pacs-server/src/domain/repositories/health_repository.rs`:
```rust
#[async_trait::async_trait]
pub trait IHealthRepository: Send + Sync {
    async fn check_db_connection(&self) -> Result<(), Error>;
}
```
`pacs-server/src/domain/repositories/mod.rs`에 `pub mod health_repository;` 추가.

#### 2. `Infrastructure` - 리포지토리 구현

`pacs-server/src/infrastructure/repositories/health_repository_impl.rs`:
```rust
use crate::domain::repositories::health_repository::IHealthRepository;
use sqlx::PgPool;

pub struct HealthRepositoryImpl {
    pub db_pool: PgPool,
}

#[async_trait::async_trait]
impl IHealthRepository for HealthRepositoryImpl {
    async fn check_db_connection(&self) -> Result<(), Error> {
        // 간단한 쿼리를 실행하여 DB 연결을 확인
        sqlx::query("SELECT 1").execute(&self.db_pool).await?;
        Ok(())
    }
}
```
`pacs-server/src/infrastructure/repositories/mod.rs`에 `pub mod health_repository_impl;` 추가.

#### 3. `Application` - 유스케이스 생성

`pacs-server/src/application/use_cases/health_use_case.rs`:
```rust
use crate::domain::repositories::health_repository::IHealthRepository;
use std::sync::Arc;

pub struct HealthUseCase {
    pub health_repo: Arc<dyn IHealthRepository>,
}

impl HealthUseCase {
    pub async fn check_health(&self) -> (bool) {
        self.health_repo.check_db_connection().await.is_ok()
    }
}
```
`pacs-server/src/application/use_cases/mod.rs`에 `pub mod health_use_case;` 추가.

#### 4. `main.rs` - 의존성 주입

새로 만든 `HealthRepository`와 `HealthUseCase`를 `main.rs`의 DI 체인에 추가합니다.

```rust
// in main.rs
// ...
let health_repo_impl = Arc::new(HealthRepositoryImpl { db_pool: db_pool.clone() });
let health_use_case = web::Data::new(HealthUseCase { health_repo: health_repo_impl });

HttpServer::new(move || {
    App::new()
        .app_data(health_use_case.clone()) // <--- 유스케이스 등록
        // ...
})
// ...
```

#### 5. `Presentation` - 컨트롤러 수정

`health_controller.rs`를 수정하여 `HealthUseCase`를 주입받고 사용하도록 변경합니다.

```rust
// pacs-server/src/presentation/controllers/health_controller.rs
use crate::application::use_cases::health_use_case::HealthUseCase;
// ...

#[derive(Serialize)]
pub struct DetailedHealthStatus {
    server: String,
    database: String,
}

// 심화 목표 핸들러
pub async fn detailed_health_check(use_case: web::Data<HealthUseCase>) -> impl Responder {
    let is_db_ok = use_case.check_health().await;
    let db_status = if is_db_ok { "ok" } else { "error" };

    HttpResponse::Ok().json(DetailedHealthStatus {
        server: "ok".to_string(),
        database: db_status.to_string(),
    })
}
```

마지막으로 `routes.rs`에서 `health_check`를 `detailed_health_check`로 교체하고 다시 실행하여 확인합니다.

이 실습을 완료하면, 프로젝트의 전체 아키텍처를 관통하는 기능을 직접 추가하는 경험을 통해 코드 구조에 대한 이해도를 크게 높일 수 있습니다.
