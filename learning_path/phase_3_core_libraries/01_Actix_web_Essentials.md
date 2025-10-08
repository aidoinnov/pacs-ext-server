# Phase 3-1: Actix-web 핵심 가이드

Actix-web은 Rust로 만들어진 강력하고 실용적인 고성능 웹 프레임워크입니다. 이 프로젝트의 `Presentation` 계층은 Actix-web을 기반으로 구축되었습니다.

## 1. 기본 구조

Actix-web 애플리케이션은 `HttpServer`와 `App`이라는 두 가지 주요 구성 요소로 이루어집니다.

-   `HttpServer`: 워커 스레드를 관리하고, 들어오는 요청을 `App` 인스턴스에 전달하는 역할을 합니다.
-   `App`: 애플리케이션의 상태, 라우팅, 미들웨어를 설정하는 곳입니다.

```rust
use actix_web::{web, App, HttpServer, Responder};

async fn index() -> impl Responder {
    "Hello, Actix!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        // App 팩토리는 각 워커 스레드마다 호출됩니다.
        App::new().route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## 2. 라우팅 (Routing)

`App::route()` 또는 `App::service()`를 사용하여 경로와 핸들러 함수를 연결합니다.

-   `web::scope()`: 공통 접두사를 가진 경로들을 그룹화합니다.
-   `web::get()`, `web::post()`: HTTP 메소드를 지정합니다.
-   `.to(handler_function)`: 해당 경로와 메소드에 대한 요청을 처리할 함수를 지정합니다.

```rust
// routes.rs
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1") // "/api/v1" 접두사로 그룹화
            .service(
                web::scope("/users")
                    .route("", web.get().to(user_controller::get_users))
                    .route("", web.post().to(user_controller::create_user))
                    .route("/{id}", web.get().to(user_controller::get_user_by_id))
            )
            .service(
                web::scope("/projects")
                    // ... project 관련 라우트
            )
    );
}

// main.rs
HttpServer::new(move || {
    App::new().configure(configure_routes)
})
```

## 3. 핸들러와 Extractor

핸들러는 HTTP 요청을 처리하는 `async` 함수입니다. 핸들러의 인자는 **Extractor**라고 불리며, 요청의 특정 부분을 추출하여 함수의 인자로 변환해주는 역할을 합니다.

주요 Extractor:

-   `web::Path<(T, ...)>`: URL 경로에서 변수를 추출합니다. (예: `/users/{id}`)
    -   `async fn get_user(path: web::Path<Uuid>) -> ... { let user_id = path.into_inner(); }`
-   `web::Query<T>`: URL의 쿼리 문자열을 `T` 타입의 구조체로 역직렬화합니다. (예: `/search?q=rust&page=1`)
-   `web::Json<T>`: 요청 본문의 JSON을 `T` 타입의 구조체로 역직렬화합니다.
-   `web::Data<T>`: 애플리케이션의 공유 상태(State)를 가져옵니다. 이 프로젝트에서는 유스케이스를 주입받는 데 사용됩니다.
-   `HttpRequest`: 저수준의 원시 요청 객체. 헤더 등을 직접 다룰 때 사용합니다.

```rust
// GET /users/{id}?show_projects=true
async fn get_user_with_projects(
    path: web::Path<Uuid>,             // 경로 파라미터 {id} 추출
    query: web::Query<SearchQuery>,    // 쿼리 파라미터 ?show_projects=true 추출
    app_state: web::Data<AppState>,    // 공유 상태(유스케이스) 주입
    req: HttpRequest,                  // 원시 요청 객체
) -> impl Responder {
    let user_id = path.into_inner();
    let show_projects = query.show_projects;
    let user = app_state.user_use_case.find_user(user_id).await;
    // ...
}
```

## 4. 응답 (Responder)

핸들러는 `Responder` Trait을 구현하는 타입을 반환해야 합니다. Actix-web은 이 타입을 실제 HTTP 응답으로 변환합니다.

-   `String`, `&'static str`: `200 OK`와 함께 텍스트 응답을 보냅니다.
-   `impl Responder`: 가장 간단한 방법.
-   `HttpResponse`: 상태 코드, 헤더, 본문을 완전히 제어할 때 사용합니다. 가장 일반적이고 강력한 방법입니다.
-   `Result<T, E>`: `T`와 `E`가 모두 `Responder`를 구현하면, `Ok`일 때는 성공 응답을, `Err`일 때는 에러 응답을 보냅니다. 이 프로젝트의 표준 방식입니다.

```rust
use actix_web::{HttpResponse, Responder, web};

// JSON 응답
async fn get_user() -> impl Responder {
    let user = UserDto { ... };
    HttpResponse::Ok().json(user) // 200 OK + JSON body
}

// 생성 후 응답
async fn create_user() -> impl Responder {
    let new_user = UserDto { ... };
    HttpResponse::Created().json(new_user) // 201 Created + JSON body
}

// 내용 없는 응답
async fn delete_user() -> impl Responder {
    HttpResponse::NoContent().finish() // 204 No Content
}
```

## 5. 상태 공유 (State)

`web::Data`를 사용하여 여러 스레드와 핸들러 간에 상태를 공유합니다. `main.rs`에서 `App::new().app_data()`를 통해 상태를 등록합니다.

```rust
struct AppState {
    db_pool: PgPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect(...).await.unwrap();
    let app_state = web::Data::new(AppState { db_pool: pool });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) // 상태 등록
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// 핸들러에서 상태 사용
async fn index(data: web::Data<AppState>) -> String {
    // data.db_pool을 사용하여 DB 작업 수행
    "Hello from stateful handler!".to_string()
}
```

이러한 기본 구성 요소들을 이해하면 `Presentation` 계층의 코드를 자신감 있게 읽고 수정할 수 있습니다.
