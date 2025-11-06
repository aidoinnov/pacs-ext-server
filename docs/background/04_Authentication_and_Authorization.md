# 04. 인증과 인가 (Authentication & Authorization)

이 프로젝트는 최신 웹 애플리케이션의 표준적인 인증/인가 방식을 따릅니다. `infrastructure/auth` 폴더와 `domain/entities`의 관련 엔티티들이 이 기능을 담당합니다.

## 1. 인증 (Authentication) - "당신은 누구인가?"

인증은 사용자의 신원을 확인하는 과정입니다. 이 프로젝트는 **JWT (JSON Web Token)**를 사용한 상태 비저장(Stateless) 인증 방식을 사용합니다.

### JWT 인증 흐름

1.  **로그인**: 사용자가 이메일과 비밀번호로 로그인을 요청합니다 (`/api/auth/login`).
2.  **비밀번호 검증**: 서버는 데이터베이스에 저장된 해시된 비밀번호와 사용자가 제출한 비밀번호를 비교합니다. (보통 `bcrypt`나 `argon2` 같은 해시 알고리즘 사용)
3.  **JWT 생성**: 검증에 성공하면, 서버는 사용자의 ID, 역할(Role) 등 핵심 정보를 담은 **Payload**를 만듭니다. 이 Payload를 비밀 키(Secret Key)로 서명하여 JWT를 생성합니다.
4.  **토큰 전송**: 생성된 JWT(Access Token)를 클라이언트에게 응답으로 보내줍니다. 클라이언트는 이 토큰을 로컬 저장소(보통 Local Storage 또는 Secure Cookie)에 저장합니다.
5.  **인증 필요한 요청**: 클라이언트는 이후 서버에 요청을 보낼 때마다 HTTP `Authorization` 헤더에 `Bearer <JWT>` 형태로 토큰을 담아 보냅니다.
6.  **토큰 검증 (미들웨어)**: 서버의 인증 미들웨어는 모든 보호된(protected) 경로에 대한 요청을 가로챕니다. 헤더에서 JWT를 추출하고, 서버에 저장된 비밀 키로 서명이 유효한지 검증합니다. 검증에 성공하면 요청을 컨트롤러로 전달하고, 실패하면 `401 Unauthorized` 에러를 반환합니다.

```rust
// in infrastructure/auth/jwt_service.rs (개념 코드)

use jsonwebtoken::{encode, decode, Header, Validation};

// JWT에 담길 정보 (Claims)
pub struct Claims {
    pub sub: Uuid, // Subject (사용자 ID)
    pub role: String, // 사용자의 역할
    pub exp: usize, // 만료 시간
}

// JWT 생성
fn create_token(user_id: Uuid, role: &str) -> Result<String, Error> {
    let claims = Claims { sub: user_id, role: role.to_string(), exp: ... };
    encode(&Header::default(), &claims, &EncodingKey::from_secret("YOUR_SECRET_KEY".as_ref()))
}

// JWT 검증
fn verify_token(token: &str) -> Result<Claims, Error> {
    decode::<Claims>(token, &DecodingKey::from_secret("YOUR_SECRET_KEY".as_ref()), &Validation::default())
}
```

## 2. 인가 (Authorization) - "당신은 무엇을 할 수 있는가?"

인가는 인증된 사용자가 특정 리소스나 기능에 접근할 수 있는 권한이 있는지 확인하는 과정입니다. 이 프로젝트는 **RBAC (Role-Based Access Control, 역할 기반 접근 제어)** 모델을 사용합니다.

### RBAC 구성 요소 (`domain/entities`)

-   **`User`**: 시스템의 사용자입니다.
-   **`Role`**: 역할의 집합입니다. (예: `Admin`, `Editor`, `Viewer`)
-   **`Permission`**: 수행할 수 있는 동작의 단위입니다. (예: `project:create`, `project:read`, `user:delete`)
-   **`Relations` (중간 테이블)**:
    -   `User`와 `Role`을 연결 (한 유저는 여러 역할을 가질 수 있음).
    -   `Role`과 `Permission`을 연결 (한 역할은 여러 권한을 가질 수 있음).

### 인가 흐름

1.  **인증**: 인증 미들웨어를 통해 사용자의 신원(ID)과 역할(Role)이 확인됩니다.
2.  **권한 확인**: 특정 작업을 수행하는 컨트롤러나 유스케이스에서, 해당 작업을 수행하는 데 필요한 `Permission`이 있는지 확인합니다.
3.  **DB 조회**: 사용자의 `Role`이 해당 `Permission`을 가지고 있는지 데이터베이스의 관계 테이블을 조회하여 확인합니다.
4.  **접근 제어**: 권한이 있으면 작업을 계속하고, 없으면 `403 Forbidden` 에러를 반환합니다.

```rust
// 특정 권한을 요구하는 컨트롤러의 예시

async fn delete_project(
    user: AuthenticatedUser, // 인증 미들웨어로부터 주입된 사용자 정보
    project_id: Path<Uuid>,
    access_control_service: Data<AccessControlService>,
) -> Result<HttpResponse, Error> {

    // 1. 이 작업을 수행하는 데 "project:delete" 권한이 필요한지 확인
    access_control_service
        .check_permission(&user, "project:delete")
        .await?;

    // 2. 권한이 있으면 프로젝트 삭제 로직 수행
    project_use_case.delete(project_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
```

이처럼 인증과 인가는 각각 **미들웨어**와 **서비스/유스케이스 레벨의 권한 체크**를 통해 구현되어, 애플리케이션의 보안을 강력하게 유지합니다.
