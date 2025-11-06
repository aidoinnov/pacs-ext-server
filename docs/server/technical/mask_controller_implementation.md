# 마스크 업로드 시스템 Controller 구현

## 개요

마스크 업로드 시스템의 REST API 엔드포인트를 제공하는 Controller 레이어를 구현했습니다.

## 구현 날짜

2025-10-11

## 구조

### 1. MaskGroupController

**파일 위치**: `src/presentation/controllers/mask_group_controller.rs`

**역할**: 마스크 그룹 관리 API 엔드포인트 제공

#### API 엔드포인트

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/annotations/{annotation_id}/mask-groups` | 새 마스크 그룹 생성 |
| GET | `/api/annotations/{annotation_id}/mask-groups` | 마스크 그룹 목록 조회 |
| GET | `/api/annotations/{annotation_id}/mask-groups/{group_id}` | 마스크 그룹 상세 조회 |
| PUT | `/api/annotations/{annotation_id}/mask-groups/{group_id}` | 마스크 그룹 수정 |
| DELETE | `/api/annotations/{annotation_id}/mask-groups/{group_id}` | 마스크 그룹 삭제 |
| POST | `/api/annotations/{annotation_id}/mask-groups/{group_id}/signed-url` | 업로드용 Signed URL 발급 |
| POST | `/api/annotations/{annotation_id}/mask-groups/{group_id}/complete` | 업로드 완료 처리 |

#### 주요 기능

1. **마스크 그룹 생성** (`create_mask_group`)
   - 요청: `CreateMaskGroupRequest`
   - 응답: `MaskGroupResponse` (201 Created)
   - 에러 처리: NotFound, Unauthorized, ValidationError, AlreadyExists, DatabaseError

2. **마스크 그룹 목록 조회** (`list_mask_groups`)
   - Query 파라미터: `offset`, `limit`
   - 응답: `MaskGroupListResponse`
   - 페이지네이션 지원

3. **Signed URL 발급** (`generate_upload_url`)
   - 요청: `SignedUrlRequest`
   - 응답: `SignedUrlResponse`
   - TTL 설정 지원 (기본 10분, 최대 1시간)

4. **업로드 완료 처리** (`complete_upload`)
   - 요청: `CompleteUploadRequest`
   - 응답: `CompleteUploadResponse`
   - 업로드된 파일 목록 검증

### 2. MaskController

**파일 위치**: `src/presentation/controllers/mask_controller.rs`

**역할**: 개별 마스크 관리 API 엔드포인트 제공

#### API 엔드포인트

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks` | 새 마스크 생성 |
| GET | `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks` | 마스크 목록 조회 |
| GET | `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}` | 마스크 상세 조회 |
| PUT | `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}` | 마스크 수정 |
| DELETE | `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}` | 마스크 삭제 |
| POST | `/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url` | 다운로드용 Signed URL 발급 |
| GET | `/api/annotations/{annotation_id}/mask-groups/{group_id}/stats` | 마스크 통계 조회 |

#### 주요 기능

1. **마스크 생성** (`create_mask`)
   - 요청: `CreateMaskRequest`
   - 응답: `MaskResponse` (201 Created)

2. **마스크 목록 조회** (`list_masks`)
   - Query 파라미터: `offset`, `limit`
   - 응답: `MaskListResponse`
   - 페이지네이션 및 페이지 정보 제공

3. **다운로드 URL 발급** (`generate_download_url`)
   - 요청: `DownloadUrlRequest`
   - 응답: `DownloadUrlResponse`
   - TTL 설정 지원

4. **마스크 통계 조회** (`get_mask_stats`)
   - 응답: `MaskStatsResponse`
   - 총 크기, 평균 크기, 레이블별/MIME 타입별 분포 제공

## 기술 구현

### 1. 의존성 주입

```rust
pub struct MaskGroupController<MGS, SUS> 
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    use_case: Arc<MaskGroupUseCase<MGS, SUS>>,
}
```

- Generic 타입 파라미터를 사용하여 테스트 가능한 구조
- `Arc`를 사용하여 다중 스레드 환경에서 안전하게 공유

### 2. 라우트 설정

```rust
pub fn configure_routes<MGS, SUS>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<MaskGroupUseCase<MGS, SUS>>,
)
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync + 'static,
    SUS: crate::application::services::SignedUrlService + Send + Sync + 'static,
{
    cfg.app_data(web::Data::new(use_case));
}
```

- `actix-web`의 `ServiceConfig`를 사용한 라우트 설정
- `web::Data`를 통한 Use Case 공유

### 3. Error Handling

모든 엔드포인트에서 일관된 에러 처리:

```rust
match use_case.create_mask_group(request, user_id).await {
    Ok(mask_group) => HttpResponse::Created().json(mask_group),
    Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
        "error": "Not Found",
        "message": msg
    })),
    Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
        "error": "Unauthorized",
        "message": msg
    })),
    Err(ServiceError::ValidationError(msg)) => HttpResponse::BadRequest().json(json!({
        "error": "Validation Error",
        "message": msg
    })),
    Err(ServiceError::AlreadyExists(msg)) => HttpResponse::Conflict().json(json!({
        "error": "Already Exists",
        "message": msg
    })),
    Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
        "error": "Database Error",
        "message": msg
    })),
}
```

#### HTTP 상태 코드 매핑

| ServiceError | HTTP Status Code | 설명 |
|--------------|------------------|------|
| NotFound | 404 Not Found | 리소스가 존재하지 않음 |
| Unauthorized | 401 Unauthorized | 인증되지 않은 요청 |
| Forbidden | 403 Forbidden | 권한이 없음 |
| ValidationError | 400 Bad Request | 입력 데이터 검증 실패 |
| AlreadyExists | 409 Conflict | 중복된 리소스 |
| DatabaseError | 500 Internal Server Error | 데이터베이스 오류 |

### 4. OpenAPI/Swagger 문서화

모든 엔드포인트에 `utoipa` 속성을 사용한 API 문서화:

```rust
#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups",
    tag = "mask-groups",
    request_body = CreateMaskGroupRequest,
    responses(
        (status = 201, description = "Mask group created successfully", body = MaskGroupResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
    )
)]
```

## 보안 고려사항

### 1. 인증 (TODO)

현재는 하드코딩된 `user_id = 1`을 사용하고 있으나, 실제 환경에서는 JWT 토큰에서 사용자 ID를 추출해야 함:

```rust
// TODO: 실제 인증에서 user_id를 가져와야 함
let user_id = 1; // 실제로는 JWT에서 추출
```

### 2. 권한 검증

- Use Case 레이어에서 사용자 권한 검증
- 마스크 그룹/마스크에 대한 접근 권한 확인

### 3. 입력 검증

- DTO 레벨에서 입력 데이터 검증
- 경로 파라미터 검증

## 통합 방법

### main.rs 설정

```rust
// Repository 초기화 (Arc 래핑)
let mask_group_repo = Arc::new(MaskGroupRepositoryImpl::new(pool.clone()));
let mask_repo = Arc::new(MaskRepositoryImpl::new(pool.clone()));

// Service 초기화
let mask_group_service = Arc::new(MaskGroupServiceImpl::new(
    mask_group_repo.clone(),
    annotation_repo.clone(),
    user_repo.clone(),
));

// Use Case 초기화 (Object Storage 설정 후)
let mask_group_use_case = Arc::new(MaskGroupUseCase::new(
    mask_group_service,
    signed_url_service.clone(),
));

// 라우트 설정 (Object Storage 설정 후 활성화)
.configure(|cfg| mask_group_controller::configure_routes(cfg, mask_group_use_case.clone()))
.configure(|cfg| mask_controller::configure_routes(cfg, mask_use_case.clone()))
```

## 테스트

### 단위 테스트 (TODO)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[actix_web::test]
    async fn test_create_mask_group() {
        // Mock Use Case
        // 테스트 구현
    }
}
```

### 통합 테스트 (TODO)

```rust
#[actix_web::test]
async fn test_create_mask_group_integration() {
    // 실제 데이터베이스 및 Object Storage 연동 테스트
}
```

## 성능 최적화

### 1. 비동기 처리

- 모든 핸들러 함수는 `async`로 선언
- `await`를 사용하여 I/O 작업 대기

### 2. 페이지네이션

- 대량의 데이터 조회 시 페이지네이션 적용
- Query 파라미터로 `offset`, `limit` 제공

### 3. 캐싱 (TODO)

- 자주 조회되는 데이터에 대한 캐싱 전략 필요

## 알려진 제한사항

1. **Object Storage 미설정**
   - Object Storage 서비스가 초기화되지 않아 현재 라우트가 주석 처리됨
   - 환경 설정 후 활성화 필요

2. **JWT 인증 미구현**
   - 하드코딩된 `user_id` 사용
   - JWT 미들웨어 통합 필요

3. **통합 테스트 부재**
   - 컨트롤러 레벨 통합 테스트 필요

## 다음 단계

1. **Object Storage 설정**
   - AWS S3 또는 MinIO 환경 설정
   - `SignedUrlService` 초기화
   - 마스크 API 라우트 활성화

2. **JWT 인증 통합**
   - `HttpRequest`에서 JWT 토큰 추출
   - 사용자 ID 및 권한 검증

3. **통합 테스트 작성**
   - 전체 API 엔드포인트 테스트
   - Object Storage 연동 테스트

4. **성능 테스트**
   - 부하 테스트
   - 동시성 테스트

## 참고 문서

- [Clean Architecture](../todo/implementation_plan.md)
- [Object Storage 연동](./object_storage_integration.md)
- [Repository 구현](./mask_repository_implementation.md)
- [Service 구현](./mask_service_implementation.md)
- [Use Case 구현](./mask_use_case_implementation.md)

