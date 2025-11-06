# Project Controllers 병합 계획

## 목적

`project_user_controller`와 `project_data_access_controller`를 병합하여 라우팅 충돌 문제를 근본적으로 해결합니다.

## 현재 문제

- `project_user_controller`: `/projects` scope 사용
- `project_data_access_controller`: `/projects/{project_id}` 경로 사용
- **충돌**: `project_user_controller`가 `/projects/{project_id}` 요청을 모두 가로챔

## 해결 방안

### 옵션 1: project_user_controller에 데이터 접근 메서드 추가 (권장)
- `project_data_access_controller`의 메서드를 `project_user_controller`로 이동
- 하나의 컨트롤러에서 모든 프로젝트 관련 라우팅 관리
- 가장 깔끔한 솔루션

### 옵션 2: 별도 범위 사용 (현재 적용됨)
- `project_data_access_controller`를 `/project-data` scope으로 변경
- 경로 변경: `/api/project-data/{project_id}/...`
- 프론트엔드 URL 변경 필요

## 권장: 옵션 1 (병합)

### 구현 계획

1. **project_user_controller.rs에 import 추가**
```rust
use crate::application::use_cases::ProjectDataAccessUseCase;
use crate::application::dto::project_data_access_dto::*;
```

2. **데이터 접근 메서드 추가**
- `get_project_data_access_matrix`
- `create_project_data`
- `update_data_access`
- `batch_update_data_access`
- `request_data_access`

3. **configure_routes 수정**
```rust
pub fn configure_routes<P, U, D>(
    cfg: &mut web::ServiceConfig,
    project_user_use_case: Arc<ProjectUserUseCase<P, U>>,
    project_data_access_use_case: Arc<ProjectDataAccessUseCase>,
) where
    P: ProjectService + 'static,
    U: UserService + 'static,
{
    cfg.app_data(web::Data::new(project_user_use_case))
        .service(
            web::scope("/projects")
                // 기존 프로젝트-사용자 메서드
                .route("/{project_id}/users", web::get().to(get_project_members::<P, U>))
                // ... 기존 라우트 ...
                
                // 데이터 접근 메서드 추가
                .route("/{project_id}/data-access/matrix", web::get().to(get_project_data_access_matrix))
                .route("/{project_id}/data", web::post().to(create_project_data))
                .route("/{project_id}/data/{data_id}/access/{user_id}", web::put().to(update_data_access))
                .route("/{project_id}/data/{data_id}/access/batch", web::put().to(batch_update_data_access))
                .route("/{project_id}/data/{data_id}/access/request", web::post().to(request_data_access))
        )
        .route("/users/{user_id}/projects", web::get().to(get_user_projects::<P, U>));
}
```

4. **main.rs 수정**
- `project_data_access_controller` import 제거
- `configure_routes` 호출에서 `project_data_access_use_case` 추가

5. **project_data_access_controller.rs 제거 또는 주석 처리**

## 장점

1. **라우팅 충돌 완전 해소**: 모든 프로젝트 관련 라우팅이 하나의 컨트롤러에서 관리
2. **일관성**: `/api/projects/{project_id}/...` 경로 일관성 유지
3. **유지보수성**: 관련 기능이 한 곳에 모여 있어 관리 용이
4. **명확성**: 하나의 컨트롤러에서 모든 프로젝트 관련 API 제공

## 단점

1. **파일 크기**: `project_user_controller.rs` 파일이 커짐
2. **복잡도**: 하나의 컨트롤러에 여러 책임이 모임
3. **컴파일 시간**: 파일이 커지면 컴파일 시간 증가

## 결론

옵션 1 (병합)을 권장합니다. 라우팅 충돌을 근본적으로 해결하고, RESTful API 설계 원칙에 더 부합합니다.

---

작성일: 2025-01-15
