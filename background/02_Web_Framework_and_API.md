# 02. 웹 프레임워크와 API

이 프로젝트는 Rust의 웹 프레임워크(Actix-web 또는 Axum으로 추정)를 사용하여 RESTful API 서버를 구축합니다. `presentation` 계층이 이와 관련된 모든 코드를 담고 있습니다.

## 1. 주요 구성 요소

### a. 라우팅 (`presentation/routes`)

-   **역할**: 특정 URL 경로와 HTTP 메소드(GET, POST, PUT, DELETE 등)를 처리할 함수(컨트롤러)에 연결합니다.
-   웹 서버가 시작될 때, 여기에 정의된 라우팅 정보가 서비스에 등록됩니다.

```rust
// main.rs 또는 routes.rs 파일의 예시

use crate::presentation::controllers::project_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/projects") // URL 경로 접두사
            .route("", web::post().to(project_controller::create_project)) // POST /api/v1/projects
            .route("/{project_id}", web::get().to(project_controller::get_project_by_id)) // GET /api/v1/projects/{project_id}
    );
}
```

### b. 컨트롤러 (`presentation/controllers`)

-   **역할**: 라우팅 규칙에 따라 매칭된 실제 함수입니다. HTTP 요청을 분석하고, `application` 계층의 유스케이스를 호출한 뒤, 그 결과를 HTTP 응답으로 변환하여 반환합니다.
-   컨트롤러 함수는 항상 `async fn` 이어야 합니다.

```rust
// in presentation/controllers/project_controller.rs

pub async fn create_project(
    // 1. 요청 본문(JSON)을 CreateProjectDto로 역직렬화하여 받음
    project_dto: web::Json<CreateProjectDto>,
    // 2. 서버 상태로부터 유스케이스 인스턴스를 주입받음
    use_case: web::Data<ProjectUseCase>,
) -> Result<HttpResponse, Error> { // 3. 반환 타입은 항상 Result<HttpResponse, Error>
    // 4. 유스케이스 호출
    let new_project = use_case.create_project(project_dto.into_inner()).await?;

    // 5. 성공 시 HTTP 201 Created 응답 반환
    Ok(HttpResponse::Created().json(new_project))
}
```

### c. 미들웨어 (`infrastructure/middleware`)

-   **역할**: 요청이 컨트롤러에 도달하기 전, 또는 컨트롤러의 응답이 클라이언트에게 가기 전에 공통적으로 처리해야 할 로직을 수행합니다. (예: 인증, 로깅, 캐싱, CORS 설정 등)
-   `infrastructure/auth/middleware.rs` 파일은 대표적인 인증 미들웨어의 예시입니다.

```rust
// 인증 미들웨어의 개념적 흐름

async fn authentication_middleware(request: HttpRequest) -> Result<...> {
    // 1. 요청 헤더에서 "Authorization" 토큰 추출
    let token = extract_token_from_header(&request)?;

    // 2. JWT 토큰 검증
    let claims = jwt_service::verify_token(token)?;

    // 3. 검증된 사용자 정보를 요청 객체에 추가하여 다음 핸들러로 전달
    add_user_to_request_extensions(claims.user_id);

    // 4. 다음 미들웨어 또는 컨트롤러 호출
    call_next_service().await
}
```

## 2. 데이터 전송 객체 (DTO - `application/dto`)

-   **역할**: 외부 계층(`presentation`)과 `application` 계층 사이의 데이터 전송을 위해 사용되는 구조체입니다.
-   **목적**:
    1.  **관심사 분리**: `domain`의 엔티티가 외부 요청/응답의 구조에 오염되는 것을 막습니다. 예를 들어, `User` 엔티티에는 `password_hash` 필드가 있지만, API 응답으로 보낼 `UserDto`에는 이 필드가 포함되지 않아야 합니다.
    2.  **유효성 검사**: `serde`와 `validator` 같은 라이브러리를 사용하여 DTO 레벨에서 입력값의 유효성(예: 이메일 형식, 최소 길이 등)을 쉽게 검사할 수 있습니다.

```rust
// in application/dto/user_dto.rs

use serde::{Deserialize, Serialize};
use validator::Validate;

// 사용자 생성을 위한 입력 DTO
#[derive(Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub name: String,
}

// 사용자 정보를 응답으로 보내기 위한 출력 DTO
#[derive(Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

// 도메인 엔티티를 DTO로 변환하는 From Trait 구현
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
        }
    }
}
```
