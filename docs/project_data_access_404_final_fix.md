# Project Data Access Matrix 404 해결 - 최종

## 문제 상황

URL: `http://localhost:8080/api/projects/1651/data-access/matrix?page=1&page_size=20`  
응답: `HTTP 404 Not Found`

## 해결 과정

### 1차 시도: 라우팅 스코프 수정

Controller의 중복 스코프 문제로 파악:
```rust
// Before
cfg.service(
    web::scope("/projects/{project_id}")  // ❌ 중복
        .app_data(use_case.clone())
        .route("/data-access/matrix", ...)
)
```

수정 사항 적용했으나 여전히 404 발생.

### 2차 시도: app_data 호출 방식 변경

`user_controller` 패턴을 참고하여 수정:
```rust
// After
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<ProjectDataAccessUseCase>) {
    cfg.app_data(web::Data::new(use_case))  // ✅ 올바른 방식
        .service(
            web::scope("/projects/{project_id}")
                .route("/data-access/matrix", ...)
        )
}
```

## 현재 상태

### 코드 변경 완료 ✅

**파일**: `pacs-server/src/presentation/controllers/project_data_access_controller.rs`

```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<ProjectDataAccessUseCase>) {
    cfg.app_data(web::Data::new(use_case))
        .service(
            web::scope("/projects/{project_id}")
                .route("/data-access/matrix", web::get().to(get_project_data_access_matrix))
                .route("/data", web::post().to(create_project_data))
                .route("/data/{data_id}/access/{user_id}", web::put().to(update_data_access))
                .route("/data/{data_id}/access/batch", web::put().to(batch_update_data_access))
                .route("/data/{data_id}/access/request", web::post().to(request_data_access))
        )
        .service(
            web::scope("/data-access")
                .route("/status/{status}", web::get().to(get_access_by_status))
        )
        .service(
            web::scope("/users/{user_id}")
                .route("/data-access", web::get().to(get_user_access_list))
        );
}
```

### 서버 재시작 필요

변경 사항을 반영하려면 서버를 재시작해야 합니다:

```bash
cd pacs-server
pkill -f "target/debug/pacs_server"
cargo run &
sleep 8
curl "http://localhost:8080/api/projects/1651/data-access/matrix?page=1&page_size=20"
```

### 예상 결과

**성공 시**:
- `HTTP 200 OK`
- 빈 매트릭스 응답 (데이터가 없으므로)

**응답 예시**:
```json
{
  "data_list": [],
  "access_matrix": [],
  "users": [],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 0,
    "total_pages": 0
  }
}
```

**실패 시** (여전히 404):
- 라우팅 우선순위 문제 가능성
- `main.rs`에서 컨트롤러 등록 순서 확인 필요

## 추가 진단 방법

### 1. 라우팅 확인

```bash
# 모든 등록된 라우트 확인
curl -s "http://localhost:8080/swagger-ui/" | grep -o "/api/projects/[^\"]*" | sort -u
```

### 2. 직접 라우트 테스트

```bash
# 기본 라우트 테스트
curl "http://localhost:8080/api/projects"  # project_controller
curl "http://localhost:8080/api/projects/1"  # project_controller
```

### 3. 로그 확인

```bash
# 서버 로그에서 라우팅 등록 확인
grep -E "configure_routes|project_data" /tmp/pacs_server.log
```

## 가능한 추가 원인

### 라우팅 충돌

`project_controller`가 `/projects` 스코프를 사용할 수 있음:
- `project_controller::configure_routes`에서 `/projects/{project_id}` 등록 여부 확인 필요

### 해결 방법

**옵션 A**: Controller 순서 변경 (main.rs)
```rust
// project_data_access_controller를 project_controller보다 먼저 등록
.configure(|cfg| {
    project_data_access_controller::configure_routes(
        cfg,
        project_data_access_use_case.clone(),
    )
})
.configure(|cfg| {
    project_controller::configure_routes(cfg, project_use_case.clone())
})
```

**옵션 B**: 경로 수정
```rust
// Controller에서 다른 경로 사용
web::scope("/project-data")  // /projects와 겹치지 않도록
```

## 결론

1. ✅ `configure_routes` 수정 완료
2. ⏸️ 서버 재시작 필요
3. ⏸️ API 테스트 필요
4. ⏸️ 만약 여전히 404면 라우팅 순서 확인 필요

---

**작성일**: 2025-01-15  
**상태**: 수정 완료, 테스트 대기
