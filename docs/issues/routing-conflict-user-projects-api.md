# 이슈 #001: User Projects API 라우팅 충돌 문제

## 📋 이슈 정보

- **이슈 번호**: #001
- **제목**: User Projects API 라우팅 충돌로 인한 404 에러
- **우선순위**: High
- **상태**: ✅ Resolved
- **담당자**: AI Assistant
- **생성일**: 2025-01-26
- **해결일**: 2025-01-26

## 🐛 문제 설명

### 증상
- `/api/users/{user_id}/projects` API 호출 시 404 Not Found 에러 발생
- API 엔드포인트가 Swagger UI에 표시되지 않음
- 사용자별 프로젝트 목록 조회 기능이 작동하지 않음

### 에러 메시지
```json
{
  "error": "404 Not Found"
}
```

### 영향 범위
- 사용자별 프로젝트 목록 조회 기능 완전 중단
- 프로젝트-사용자 매트릭스 UI에서 사용자별 프로젝트 표시 불가
- 페이지네이션 기능 포함한 모든 관련 기능 사용 불가

## 🔍 원인 분석

### 근본 원인
두 개의 컨트롤러가 동일한 `/users` 스코프를 사용하여 라우팅 충돌 발생:

1. **user_controller.rs** (431줄에서 먼저 등록)
   ```rust
   .service(
       web::scope("/users")
           .route("", web::post().to(UserController::<U>::create_user))
           .route("/{user_id}", web::get().to(UserController::<U>::get_user))
           // ...
   )
   ```

2. **project_user_controller.rs** (464줄에서 나중에 등록)
   ```rust
   .service(
       web::scope("/users")
           .route("/{user_id}/projects", web::get().to(get_user_projects::<P, U>))
   )
   ```

### 기술적 원인
- **Actix-web 라우팅 순서**: 먼저 등록된 스코프가 우선권을 가짐
- **스코프 충돌**: 동일한 경로 패턴에 대한 중복 등록
- **라우트 무시**: 나중에 등록된 `/users` 스코프가 무시됨

## 🛠️ 해결 방법

### 해결 전략
1. **라우팅 충돌 제거**: `project_user_controller.rs`에서 `/users` 스코프 제거
2. **직접 라우트 등록**: 특정 엔드포인트를 직접 등록
3. **등록 순서 최적화**: 충돌 가능성이 있는 컨트롤러를 먼저 등록

### 구현 세부사항

#### 1. project_user_controller.rs 수정
```rust
// 수정 전
.service(
    web::scope("/users")
        .route("/{user_id}/projects", web::get().to(get_user_projects::<P, U>))
);

// 수정 후
.route("/users/{user_id}/projects", web::get().to(get_user_projects::<P, U>));
```

#### 2. main.rs 등록 순서 변경
```rust
// project_user_controller를 user_controller보다 먼저 등록
.configure(|cfg| {
    project_user_controller::configure_routes(cfg, project_user_use_case.clone())
})
.configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
```

#### 3. 추가 수정사항
- **SQL 쿼리 수정**: `user_repository_impl.rs`의 모든 `find_*` 함수에서 `account_status` 필드 포함
- **데이터 무결성**: User 엔티티의 모든 필드를 SELECT하도록 쿼리 개선

## ✅ 해결 결과

### 테스트 결과
```bash
curl "http://localhost:8080/api/users/1/projects?page=1&page_size=10"
```

**응답 (200 OK)**:
```json
{
  "projects": [
    {
      "project_id": 1,
      "project_name": "Test1",
      "description": "2",
      "is_active": true,
      "role_id": null,
      "role_name": null,
      "role_scope": null
    },
    {
      "project_id": 2,
      "project_name": "Test2",
      "description": "3",
      "is_active": true,
      "role_id": null,
      "role_name": null,
      "role_scope": null
    }
  ],
  "total_count": 2,
  "page": 1,
  "page_size": 10,
  "total_pages": 1
}
```

### 기능 검증
- ✅ `/api/users/{user_id}/projects` API 정상 작동
- ✅ 페이지네이션 기능 정상 작동
- ✅ 프로젝트 정보 및 역할 정보 정상 반환
- ✅ 기존 `/api/projects/{project_id}/users` API 영향 없음

## 📚 학습된 교훈

### 라우팅 설계 원칙
1. **스코프 분리**: 관련 없는 기능은 별도 스코프 사용
2. **등록 순서**: 구체적인 경로를 먼저 등록
3. **충돌 방지**: 동일한 경로 패턴 중복 등록 금지

### 코드 품질 개선
1. **SQL 쿼리 완전성**: 엔티티의 모든 필드를 SELECT
2. **에러 처리**: 명확한 에러 메시지 제공
3. **테스트 커버리지**: 라우팅 충돌에 대한 테스트 추가 필요

## 🔗 관련 파일

### 수정된 파일
- `pacs-server/src/presentation/controllers/project_user_controller.rs`
- `pacs-server/src/main.rs`
- `pacs-server/src/infrastructure/repositories/user_repository_impl.rs`

### 관련 문서
- `docs/api/project-user-role-management-api.md`
- `work/routing_conflict_fix/`

## 🏷️ 태그

`routing` `conflict` `api` `404` `actix-web` `resolved`
