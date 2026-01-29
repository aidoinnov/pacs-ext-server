# TODO List

## ✅ Recently Completed (2026-01-24)

### Sync API & Access Control E2E Tests
**Priority:** Low
**Status:** ✅ Completed (2026-01-24)

**완료 내용:**

#### 1. Sync API E2E Test
- **파일**: `pacs-server/e2e/test_sync_api_e2e.py` (274 lines)
- **테스트 시나리오** (6개):
  1. 동기화 상태 조회 (GET /api/sync/status)
  2. 수동 동기화 실행 (POST /api/sync/run)
  3. 동기화 일시 정지 및 재개 (POST /api/sync/pause, POST /api/sync/resume)
  4. 스케줄 조회 및 업데이트 (GET/PUT /api/sync/schedule)
  5. 의존성 체크 (GET /api/sync/deps)
  6. 연속 동기화 실행 (Multiple POST /api/sync/run)
- **결과**: 🎉 ALL TESTS PASSED (6/6 scenarios)

#### 2. Access Control E2E Test
- **파일**: `pacs-server/e2e/test_access_control_e2e.py` (308 lines)
- **테스트 시나리오** (7개):
  1. Role-Capability Matrix 조회 (GET /api/roles/global/capabilities/matrix)
  2. Role-Permission Matrix 조회 (GET /api/roles/global/permissions/matrix)
  3. User-Project Matrix 조회 (GET /api/user-project-matrix)
  4. Permission Check (POST /api/access-control/permissions/check)
  5. User Permissions 조회 (GET /api/access-control/permissions/user/{user_id}/project/{project_id})
  6. Project Access Check (GET /api/access-control/access/user/{user_id}/project/{project_id})
  7. Access Logs 조회 (GET /api/access-control/logs/user/{user_id}, GET /api/access-control/logs/project/{project_id})
- **결과**: 🎉 ALL TESTS PASSED (7/7 scenarios)

**E2E 테스트 커버리지 업데이트:**
- 총 E2E 테스트 파일: 51개 → **53개** (+2)
- 커버리지: 83% (10/12) → **92% (11/12)** (+9%)
- Sync API: ❌ None → **🟢 Excellent**
- Access Control: ❌ None → **🟢 Excellent**

---

## ✅ Recently Completed (2026-01-24)

### Duplicate Data Issue Resolution
**Priority:** High
**Status:** ✅ Completed (2026-01-24)

**문제:**
- `project_data` 테이블에 동일한 Study가 중복 등록되는 문제
- 과거 데이터: Project ID 2에 171개 레코드, 실제 10개 Study (17배 중복)

**해결 내용:**
1. **Repository 로직 개선**
   - `ON CONFLICT DO NOTHING` → `ON CONFLICT DO UPDATE SET updated_at = CURRENT_TIMESTAMP`
   - Idempotency 보장: 동일한 요청을 여러 번 보내도 안전
   - 항상 행 반환: 새로 생성되거나 기존 레코드 모두 반환

2. **API 중복 체크**
   - `POST /api/projects/{project_id}/studies/assign`
   - 첫 번째 할당: `200 OK` + `study_id` 반환
   - 중복 할당 시도: `409 Conflict` + "Study already assigned to this project"

3. **E2E 테스트 추가**
   - 파일: `pacs-server/e2e/test_project_data_duplicate_prevention.py`
   - Test 1: Duplicate Study Assignment Prevention
   - Test 2: Concurrent Study Assignment (5개 동시 요청)
   - Test 3: Same Study in Different Projects
   - 결과: 🎉 ALL TESTS PASSED

4. **데이터베이스 검증**
   - 현재 중복 데이터 없음 (9개 레코드, 9개 고유 조합)
   - UNIQUE 제약조건 정상 작동 중

**구현 파일:**
- Repository: `pacs-server/src/infrastructure/repositories/project_data_repository_impl.rs`
- E2E Test: `pacs-server/e2e/test_project_data_duplicate_prevention.py`
- Documentation: `docs/issues/duplicate-data-issue.md`

---

### Test Compilation Errors Fixed
**Priority:** Medium
**Status:** ✅ Completed (2026-01-24)

**수정 내용:**
1. **dicom_gateway_controller.rs**
   - ❌ `decode_keycloak_token_sub` import 제거 (존재하지 않는 함수)
   - ❌ `vyalue` → `value` 오타 수정

2. **data_access_check_controller.rs**
   - ❌ `DataAccessCheckRequest`에 `project_id` 필드 추가

3. **view_selection_inmemory_repository_impl.rs**
   - ❌ `ViewSelection::new()` 인자 개수 수정 (4개 → 6개)
   - 추가 인자: `layout: Option<ViewportLayout>`, `initial_views: Option<Vec<InitialViewport>>`

4. **user_registration_controller**
   - ❌ `mod.rs`에 `pub mod user_registration_controller;` 추가

5. **assign_role_to_unassigned_user_test.rs**
   - ❌ `ProjectDataAccessUseCase::new()` 인자 개수 수정 (2개 → 3개)
   - 추가 인자: `subject_service: Arc<dyn SubjectService>`

**빌드 결과:**
- ✅ `cargo build` 성공
- ⚠️ Warnings only (316개 - 대부분 unused 변수)

**남은 테스트 에러:**
- ~~`annotation_dto_test.rs`~~ ✅ 완료 (2026-01-24)
- ~~`viewer_dto_test.rs`~~ ✅ 완료 (2026-01-24)
- 모든 DTO 단위 테스트 통과

---

### Mask & Subject E2E Tests Added
**Priority:** Medium
**Status:** ✅ Completed (2026-01-24)

**완료 내용:**
1. **test_mask_e2e.py** (5개 테스트 시나리오)
   - Mask Group CRUD
   - Mask CRUD
   - Signed URL 생성 (업로드/다운로드)
   - Mask 통계 조회 (라우트 순서 문제 해결)
   - Mask 목록 페이지네이션

2. **test_subject_e2e.py** (5개 테스트 시나리오)
   - Subject CRUD
   - Subject 코드 유효성 검증
   - Subject 코드 및 Patient ID 중복 체크
   - Subject 상세 조회 (통계 포함)
   - 프로젝트별 Subject 목록 조회

3. **Mask 통계 API 라우트 순서 수정**
   - 파일: `pacs-server/src/presentation/controllers/annotation_controller.rs`
   - `/stats` 라우트를 `/{mask_id}` 라우트보다 먼저 등록
   - 문제: "stats"가 `{mask_id}` 파라미터로 파싱되어 i32 변환 실패
   - 해결: 라우트 등록 순서 변경 (구체적인 경로가 먼저)

**테스트 결과:**
- 🎉 ALL TESTS PASSED (10/10 시나리오)
- Mask E2E: 5/5 통과
- Subject E2E: 5/5 통과

**구현 파일:**
- `pacs-server/e2e/test_mask_e2e.py`
- `pacs-server/e2e/test_subject_e2e.py`

---

### Sync API Timeout Issue Resolution
**Priority:** High
**Status:** ✅ Completed (2026-01-24)

**문제:**
- `POST /api/sync/run` API가 응답하지 않고 타임아웃 발생 (2025-12-18 문서화)

**조사 결과:**
- 현재 코드에서는 `tokio::spawn` + `tokio::time::timeout` 조합으로 정상 작동
- 테스트 결과: 149ms 응답 시간, 타임아웃 없음
- 자동 스케줄러와 수동 실행 모두 정상

**결론:**
- 이슈는 이미 해결된 상태 (2025-12-18 ~ 2026-01-24 사이)
- 문서만 업데이트 필요했음

**구현 파일:**
- `docs/issues/sync-api-timeout-issue.md` (상태 업데이트)

---

### Authentication Flow E2E Tests Added
**Priority:** Medium
**Status:** ✅ Completed (2026-01-24)

**완료 내용:**
- **test_auth_flow_e2e.py** (6개 테스트 시나리오)
  1. 회원가입 (Signup) + 중복 회원가입 차단
  2. 이메일 인증 (Email Verification)
  3. 관리자 승인 (Admin Approval)
  4. 승인 후 로그인 + 토큰 검증
  5. 비밀번호 유효성 검증 (5가지 약한 비밀번호 패턴)
  6. 계정 삭제 + 삭제된 계정 로그인 차단

**테스트 결과:**
- 🎉 ALL TESTS PASSED (6/6 시나리오)
- 전체 인증 플로우 완전 커버

**구현 파일:**
- `pacs-server/e2e/test_auth_flow_e2e.py`

---

### Annotation & Viewer DTO Tests Fixed
**Priority:** Low
**Status:** ✅ Completed (2026-01-24)

**수정 내용:**

#### 1. annotation_dto_test.rs
1. **CreateAnnotationRequest** (5개 인스턴스)
   - `series_instance_uid`: `String` → `Option<String>` (wrapped in `Some()`)
   - `sop_instance_uid`: `String` → `Option<String>` (wrapped in `Some()`)
   - `lesion_type: Option<String>` 필드 추가
   - `lesion_number: Option<i32>` 필드 추가

2. **UpdateAnnotationRequest** (1개 인스턴스)
   - `lesion_type: Some("TARGET".to_string())` 추가
   - `lesion_number: Some(1)` 추가

3. **AnnotationResponse** (9개 인스턴스)
   - `lesion_type: Option<String>` 필드 추가
   - `lesion_number: Option<i32>` 필드 추가
   - `snapshot_image_key: Option<String>` 필드 추가
   - `snapshot_image_url: Option<String>` 필드 추가
   - `snapshot_status: Option<String>` 필드 추가
   - `snapshot_uploaded_at: Option<DateTime<Utc>>` 필드 추가

#### 2. viewer_dto_test.rs
1. **ViewerStudyMetaRequest**
   - `max_count: Some(20)` 제거
   - `page: Some(1)` 추가
   - `page_size: Some(20)` 추가

2. **ViewerSeriesMetaRequest**
   - `max_count: Some(50)` 제거
   - `page: Some(1)` 추가
   - `page_size: Some(50)` 추가

3. **SeriesQuery** (2개 인스턴스)
   - `study_description: Option<String>` 필드 추가
   - 첫 번째: `Some("Chest CT".to_string())`
   - 두 번째: `None`

**빌드 결과:**
- ✅ `cargo test --test annotation_dto_test` 성공 (4 tests passed)
- ✅ `cargo test --test viewer_dto_test` 성공 (4 tests passed)

**구현 파일:**
- `pacs-server/tests/annotation_dto_test.rs`
- `pacs-server/tests/viewer_dto_test.rs`

---

## 🔒 Security & Authorization

### TimePoint Annotation API 권한 체크 추가
**Priority:** High  
**Status:** Pending

**문제:**
- `GET /api/timepoints/{timepoint_id}/annotations` API에 권한 체크가 없음
- 누구나 TimePoint의 모든 annotation을 조회할 수 있는 보안 취약점

**현재 구현:**
```rust
// pacs-server/src/presentation/controllers/timepoint_controller.rs
pub async fn get_annotations_by_timepoint<A: AnnotationRepository + 'static, S: ...>(
    annotation_repository: web::Data<A>,
    signed_url_service: web::Data<S>,
    timepoint_id: web::Path<i32>,
) -> impl Responder {
    // ❌ 권한 체크 없음!
    match annotation_repository.find_by_timepoint(*timepoint_id).await {
```

**해결 방안:**
1. **UseCase 레이어 추가**
   - `AnnotationUseCase`에 `get_annotations_by_timepoint_with_permission` 메서드 추가
   
2. **권한 체크 로직**
   - TimePoint → Subject → Project 확인
   - 사용자가 해당 Project의 멤버인지 확인
   - `READ_ALL` 권한 확인
     - 권한 있음: 모든 annotation 반환
     - 권한 없음: 본인 annotation만 반환

3. **일관성 유지**
   - 일반 annotation API (`GET /api/annotations?project_id=...`)와 동일한 권한 정책 적용

**참고:**
- 일반 Annotation API는 `get_annotations_by_project_with_permission` 사용
- 파일: `pacs-server/src/application/use_cases/annotation_use_case.rs:996-1022`

---

## ✅ Completed Tasks

### Project API ETag Caching
**Priority:** Medium
**Status:** ✅ Completed (2026-01-24)

**구현 내용:**
- Project 조회 API에 ETag 기반 HTTP 캐싱 적용
- `updated_at` 타임스탬프 기반 ETag 생성
- 프로젝트 수정 시 자동 캐시 무효화 (PostgreSQL 트리거)

**구현 파일:**
- Controller: `pacs-server/src/presentation/controllers/project_controller.rs`
- UseCase: `pacs-server/src/application/use_cases/project_use_case.rs`
- Service: `pacs-server/src/domain/services/project_service.rs`
- Repository: `pacs-server/src/infrastructure/repositories/project_repository_impl.rs`
- Migration: `pacs-server/migrations/048_add_updated_at_to_project.sql`

**캐싱 적용 API:**
1. `GET /api/projects` - 프로젝트 목록 조회
   - ETag: `MAX(updated_at)` from all projects
2. `GET /api/projects/{id}` - 프로젝트 상세 조회
   - ETag: `updated_at` of specific project
3. `GET /api/projects/active` - 활성 프로젝트 목록 조회
   - ETag: `MAX(updated_at)` from active projects

**캐시 정책:**
- `Cache-Control: private, max-age=60`
- Weak ETag 사용 (`W/"<timestamp>"`)
- TTL: 60초

**캐시 무효화:**
- 프로젝트 수정 시 `updated_at` 자동 업데이트 (PostgreSQL 트리거)
- 새로운 `updated_at` → 새로운 ETag 생성 → 304 응답 중단

**테스트:**
- E2E 테스트: `pacs-server/e2e/test_project_cache_e2e.py`
- 11개 시나리오 모두 통과:
  1. 프로젝트 목록 ETag 캐싱
  2. 프로젝트 상세 ETag 캐싱
  3. 캐시 무효화 (상세)
  4. 다른 쿼리 파라미터
  5. 동시 요청 시 캐시 일관성
  6. 잘못된 ETag 처리
  7. 성능 비교 (HIT vs MISS)
  8. 목록 캐시 무효화
  9. 페이지네이션 캐시
  10. 빈 결과 캐싱
  11. 활성 프로젝트 목록 ETag 캐싱

**Notes:**
- 프로젝트 메타데이터 API (`GET /api/projects/meta`)는 정적 데이터이므로 캐싱 불필요

---

### Project Membership Cache (Redis)
**Priority:** Medium
**Status:** ✅ Completed (2026-01-24)

**구현 내용:**
- Redis 기반 프로젝트 멤버십 캐싱으로 DB 쿼리 부하 절감
- RBAC 평가 시 멤버십 확인 성능 개선 (23.2%)
- 멤버십 변경 시 자동 캐시 무효화

**구현 파일:**
- Service: `pacs-server/src/infrastructure/services/membership_cache_service.rs`
- RBAC Evaluator: `pacs-server/src/infrastructure/services/dicom_rbac_evaluator_impl.rs`
- UseCase: `pacs-server/src/application/use_cases/project_user_use_case.rs`
- Main: `pacs-server/src/main.rs`

**주요 기능:**
- ✅ Redis 기반 멤버십 캐싱 (TTL: 180초, 환경 변수로 설정 가능)
- ✅ 캐시 키: `membership:u{user_id}:p{project_id}`
- ✅ DICOM API 호출 시 자동 캐시 사용 (Study/Series/Instance)
- ✅ 멤버 추가/제거 시 캐시 무효화
- ✅ 역할 할당/제거 시 캐시 무효화
- ✅ 일괄 역할 할당 시 캐시 무효화

**성능 개선:**
- Cache MISS: 0.228s
- Cache HIT: 0.175s
- **개선율: 23.2%**

**테스트:**
- E2E 테스트: `pacs-server/e2e/test_membership_cache_e2e.py`
- 6개 시나리오 모두 통과:
  1. Membership Cache HIT (성능 개선 검증)
  2. Concurrent Requests (동시성 안전성)
  3. Different Project Cache Isolation (캐시 격리)
  4. Non-Member Access Caching (권한 없는 사용자)
  5. Cache Invalidation on Role Change (역할 변경 시 무효화)
  6. Cache Invalidation on Member Removal (멤버 제거 시 무효화)

**문서:**
- Membership Cache Guide: `docs/api/membership-cache-guide.md`
- Caching Guide 업데이트: `docs/api/caching-guide.md`
- API README 업데이트: `docs/api/README.md`

**환경 변수:**
- `MEMBERSHIP_CACHE_TTL_SEC`: 캐시 TTL (기본값: 180초)

---

### HTTP Caching for User Role Assignment APIs
**Priority:** Medium
**Status:** ✅ Completed (2026-01-21)

**구현 내용:**
- User Role Assignment API에 ETag 기반 HTTP 캐싱 적용
- `Cache-Control: private, max-age=1` + ETag 전략 사용
- `updated_at` 타임스탬프를 ETag로 사용

**구현 파일:**
- `pacs-server/src/presentation/controllers/user_project_controller.rs`
- Migration 044: `security_user_project.updated_at` 컬럼 추가

**테스트:**
- E2E 테스트: `pacs-server/e2e/test_user_role_assignment_cache_e2e.py`
- 6개 시나리오 모두 통과

---

### HTTP Caching for Role-Capability Matrix APIs
**Priority:** Medium
**Status:** ✅ Completed (2026-01-21)

**구현 내용:**
- Role-Capability Matrix API에 ETag 기반 HTTP 캐싱 적용
- `MAX(updated_at)` from `security_role`, `security_capability`, `security_role_capability` 사용
- `MAX(created_at)` from `security_role_capability` 포함 (DELETE + INSERT 감지)

**구현 파일:**
- `pacs-server/src/presentation/controllers/role_capability_controller.rs`
- Migration 045: `security_role.updated_at` 컬럼 추가
- Migration 046: `security_role_capability.updated_at` 컬럼 추가
- Migration 047: `security_capability.updated_at` 트리거 추가

**테스트:**
- E2E 테스트: `pacs-server/e2e/test_role_capability_matrix_cache_e2e.py`
- 10개 시나리오 모두 통과

**주요 특징:**
- 모든 페이지에서 변경 감지 (어느 페이지를 수정해도 모든 페이지의 ETag 변경)
- DELETE + INSERT 시나리오도 정확히 감지
- 304 Not Modified 응답으로 네트워크 대역폭 절약

---

### View Selection API Implementation
**Priority:** High
**Status:** ✅ Completed (2026-01-21)

**구현 내용:**
- DICOM Viewer용 멀티 Study/Series 선택 API 구현
- Viewport Layout 및 Initial Views 지원
- Redis/In-memory fallback 지원
- 자동 TTL 연장 기능

**구현 파일:**
- Domain: `pacs-server/src/domain/view_selection/`
- Application: `pacs-server/src/application/use_cases/view_selection_use_case.rs`
- Infrastructure: `pacs-server/src/infrastructure/view_selection/`
- Presentation: `pacs-server/src/presentation/controllers/view_selection_controller.rs`

**주요 기능:**
- ✅ 멀티 Study/Series 선택
- ✅ Viewport Layout 설정 (rows × cols)
- ✅ Initial Views 설정 (각 Viewport의 초기 이미지)
- ✅ 자동 TTL 연장 (조회 시마다)
- ✅ URL 공유 (Selection ID 기반)
- ✅ Redis 미연결 시 in-memory 자동 fallback

**테스트:**
- E2E 테스트: `pacs-server/e2e/test_view_selection_e2e.py`
- 79개 테스트 모두 통과
- 기본 기능, Layout/Initial Views, 유효성 검증, 하위 호환성, 실제 시나리오 모두 테스트

**문서:**
- Quick Start: `pacs-server/docs/VIEW_SELECTION_QUICK_START.md`
- API Guide (EN): `pacs-server/docs/VIEW_SELECTION_API_GUIDE.md`
- API Guide (KR): `pacs-server/docs/VIEW_SELECTION_API_GUIDE_KR.md`
- Documentation Index: `pacs-server/docs/README.md`

**API 엔드포인트:**
- `POST /api/v1/view-selections` - Selection 생성
- `GET /api/v1/view-selections/{selection_id}` - Selection 조회
- `DELETE /api/v1/view-selections/{selection_id}` - Selection 삭제

---

### HTTP Caching for Capability APIs
**Priority:** Medium
**Status:** ✅ Completed (2026-01-21)

**구현 내용:**
- Capability 조회 API 3개에 ETag 기반 HTTP 캐싱 적용
- `Cache-Control: private, max-age=60` + ETag 전략 사용
- `MAX(updated_at)` 타임스탬프를 ETag로 사용

**구현 파일:**
- `pacs-server/src/presentation/controllers/role_controller.rs`
- Repository: `pacs-server/src/infrastructure/repositories/capability_repository_impl.rs`
- Service: `pacs-server/src/infrastructure/services/capability_service_impl.rs`
- UseCase: `pacs-server/src/application/use_cases/role_capability_matrix_use_case.rs`

**캐싱 적용 API:**
1. `GET /api/capabilities` - 모든 Capability 목록
   - ETag: `MAX(updated_at)` from active capabilities
2. `GET /api/capabilities/{id}` - Capability 상세
   - ETag: `GREATEST(capability.updated_at, MAX(mapping.created_at))`
3. `GET /api/capabilities/category/{category}` - 카테고리별 Capability
   - ETag: `MAX(updated_at)` for specific category

**테스트:**
- E2E 테스트: `pacs-server/e2e/test_capability_cache_e2e.py`
- 11개 시나리오 모두 통과

**주요 특징:**
- Capability 변경 시 자동으로 ETag 갱신 (Migration 047 트리거 활용)
- Permission 매핑 변경도 감지 (detail API)
- 304 Not Modified 응답으로 네트워크 대역폭 절약

---

## 📝 Notes
- 2026-01-24: Project Membership Cache 구현 완료 (6개 E2E 테스트 통과, 23.2% 성능 개선)
- 2026-01-21: Capability API 캐싱 완료 (11개 E2E 테스트 통과)
- 2026-01-21: View Selection API 구현 완료 (79개 E2E 테스트 통과)
- 2026-01-21: Role-Capability Matrix API 캐싱 완료 (10개 E2E 테스트 통과)
- 2026-01-21: User Role Assignment API 캐싱 완료 (6개 E2E 테스트 통과)
- 2026-01-19: TimePoint Annotation API 권한 체크 이슈 발견

