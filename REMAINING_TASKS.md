# 📋 남은 작업 목록 (Remaining Tasks)

> **작성일**: 2026-01-24  
> **목적**: PACS Extension Server 프로젝트의 남은 작업 정리  
> **우선순위**: High → Medium → Low

---

## 🔴 High Priority (보안/기능 이슈)

### 1. TimePoint Annotation API 권한 체크 ⭐
**파일**: `pacs-server/src/presentation/controllers/timepoint_controller.rs`  
**상태**: ❌ 미해결  
**예상 시간**: 1-2시간

**문제:**
- `GET /api/timepoints/{timepoint_id}/annotations` API에 권한 체크가 없음
- 누구나 TimePoint의 모든 annotation을 조회할 수 있는 보안 취약점

**해결 방안:**
1. `AnnotationUseCase`에 `get_annotations_by_timepoint_with_permission` 메서드 추가
2. TimePoint → Subject → Project 확인
3. 사용자가 해당 Project의 멤버인지 확인
4. `READ_ALL` 권한 확인
   - 권한 있음: 모든 annotation 반환
   - 권한 없음: 본인 annotation만 반환

**참고:**
- 일반 Annotation API는 `get_annotations_by_project_with_permission` 사용
- 파일: `pacs-server/src/application/use_cases/annotation_use_case.rs:996-1022`

---

### 2. Sync API Timeout Issue ✅ **해결 완료 (2026-01-24)**
**파일**: `docs/issues/sync-api-timeout-issue.md`
**상태**: ✅ 해결됨
**소요 시간**: 10분 (이미 해결된 상태 확인)

**문제:**
- `POST /api/sync/run` API가 응답하지 않고 타임아웃 발생 (2025-12-18)

**해결 내용:**
- 현재 코드에서는 `tokio::spawn` + `tokio::time::timeout` 조합으로 정상 작동
- 테스트 결과: 149ms 응답 시간, 타임아웃 없음
- 자동 스케줄러와 수동 실행 모두 정상

**결론:**
- 이슈는 이미 해결된 상태
- 문서만 업데이트 필요했음

---

### 3. Duplicate Data Issue ✅ **해결 완료 (2026-01-24)**
**파일**: `docs/issues/duplicate-data-issue.md`
**상태**: ✅ 해결됨
**소요 시간**: 1시간

**해결 내용:**
- ✅ Repository 로직 수정: `ON CONFLICT DO UPDATE` 사용 (Idempotency 보장)
- ✅ API 중복 체크: 409 Conflict 반환
- ✅ E2E 테스트 추가: 3가지 시나리오 (중복 방지, 동시성, 다중 프로젝트)
- ✅ 데이터베이스 검증: 현재 중복 데이터 없음 확인

**테스트 결과**: 🎉 ALL TESTS PASSED

---

### 4. Test Compilation Errors ✅ **해결 완료 (2026-01-24)**
**파일**: `pacs-server/tests/TODO_FIX_TESTS.md`
**상태**: ✅ 대부분 해결됨
**소요 시간**: 1시간

**수정 내용:**
- ✅ `dicom_gateway_controller.rs` - import 오류, 오타 수정
- ✅ `data_access_check_controller.rs` - 누락된 필드 추가
- ✅ `view_selection_inmemory_repository_impl.rs` - 함수 인자 개수 수정
- ✅ `user_registration_controller` - mod.rs에 추가
- ✅ `assign_role_to_unassigned_user_test.rs` - SubjectService 추가

**빌드 결과**: ✅ `cargo build` 성공

**남은 테스트 에러** (낮은 우선순위):
- ✅ `annotation_dto_test.rs` - DTO 스키마 변경 완료 (2026-01-24)
- ✅ `viewer_dto_test.rs` - DTO 스키마 변경 완료 (2026-01-24)
- 모든 DTO 단위 테스트 통과

---

## 🟡 Medium Priority (테스트/문서)

### 4. E2E 테스트 추가 - Authentication Flow ✅ **해결 완료 (2026-01-24)**
**파일**: `pacs-server/e2e/test_auth_flow_e2e.py`
**상태**: ✅ 완료
**소요 시간**: 2시간

**완료 내용:**
- ✅ `test_auth_flow_e2e.py` 생성 (6개 테스트 시나리오)
  - 회원가입 (Signup) + 중복 회원가입 차단
  - 이메일 인증 (Email Verification)
  - 관리자 승인 (Admin Approval)
  - 승인 후 로그인 + 토큰 검증
  - 비밀번호 유효성 검증 (5가지 약한 비밀번호 패턴)
  - 계정 삭제 + 삭제된 계정 로그인 차단

**테스트 결과**: 🎉 ALL TESTS PASSED (6/6 시나리오)

---

### 5. E2E 테스트 추가 - TimePoint API
**파일**: 신규 생성 필요
**상태**: ❌ 미구현
**예상 시간**: 2-3시간

**누락된 시나리오:**
- ❌ TimePoint CRUD
- ❌ TimePoint Annotation 조회 (권한 체크 포함)
- ❌ TimePoint 순서 변경

**우선순위**: 🔴 High (보안 취약점 존재)

**해결 방안:**
- `test_timepoint_e2e.py` 생성
- 권한 체크 시나리오 포함

---

### 6. E2E 테스트 추가 - Mask & Subject API ✅ **해결 완료 (2026-01-24)**
**파일**: `pacs-server/e2e/test_mask_e2e.py`, `pacs-server/e2e/test_subject_e2e.py`
**상태**: ✅ 완료
**소요 시간**: 3시간

**완료 내용:**
- ✅ `test_mask_e2e.py` 생성 (5개 테스트 시나리오)
  - Mask Group CRUD
  - Mask CRUD
  - Signed URL 생성 (업로드/다운로드)
  - Mask 통계 조회 (라우트 순서 문제 해결)
  - Mask 목록 페이지네이션
- ✅ `test_subject_e2e.py` 생성 (5개 테스트 시나리오)
  - Subject CRUD
  - Subject 코드 유효성 검증
  - Subject 코드 및 Patient ID 중복 체크
  - Subject 상세 조회 (통계 포함)
  - 프로젝트별 Subject 목록 조회
- ✅ Mask 통계 API 라우트 순서 수정 (`annotation_controller.rs`)
  - `/stats` 라우트를 `/{mask_id}` 라우트보다 먼저 등록

**테스트 결과**: 🎉 ALL TESTS PASSED (10/10 시나리오)

---

### 7. 단위 테스트 DTO 스키마 수정 ✅ **해결 완료 (2026-01-24)**
**파일**: `pacs-server/tests/annotation_dto_test.rs`, `pacs-server/tests/viewer_dto_test.rs`
**상태**: ✅ 완료
**소요 시간**: 1.5시간

**완료 내용:**
- ✅ `annotation_dto_test.rs` 수정
  - `CreateAnnotationRequest`: `lesion_type`, `lesion_number` 필드 추가
  - `UpdateAnnotationRequest`: `lesion_type`, `lesion_number` 필드 추가
  - `AnnotationResponse`: `lesion_type`, `lesion_number`, `snapshot_image_key`, `snapshot_image_url`, `snapshot_status`, `snapshot_uploaded_at` 필드 추가
  - `series_instance_uid`, `sop_instance_uid`를 `Option<String>`으로 변경
- ✅ `viewer_dto_test.rs` 수정
  - `ViewerStudyMetaRequest`: `max_count` 제거, `page`, `page_size` 추가
  - `ViewerSeriesMetaRequest`: `max_count` 제거, `page`, `page_size` 추가
  - `SeriesQuery`: `study_description: Option<String>` 필드 추가
- ✅ 모든 DTO 단위 테스트 통과 (`cargo test --test viewer_dto_test`)

---

### 8. E2E 테스트 추가 - Sync API ✅ **해결 완료 (2026-01-24)**
**파일**: `pacs-server/e2e/test_sync_api_e2e.py`
**상태**: ✅ 완료
**소요 시간**: 1시간

**완료 내용:**
- ✅ `test_sync_api_e2e.py` 생성 (6개 테스트 시나리오)
  - 동기화 상태 조회 (GET /api/sync/status)
  - 수동 동기화 실행 (POST /api/sync/run)
  - 동기화 일시 정지 및 재개 (POST /api/sync/pause, POST /api/sync/resume)
  - 스케줄 조회 및 업데이트 (GET/PUT /api/sync/schedule)
  - 의존성 체크 (GET /api/sync/deps)
  - 연속 동기화 실행 (Multiple POST /api/sync/run)

**테스트 결과**: 🎉 ALL TESTS PASSED (6/6 시나리오)

---

### 9. E2E 테스트 추가 - Access Control ✅ **해결 완료 (2026-01-24)**
**파일**: `pacs-server/e2e/test_access_control_e2e.py`
**상태**: ✅ 완료
**소요 시간**: 2시간

**완료 내용:**
- ✅ `test_access_control_e2e.py` 생성 (7개 테스트 시나리오)
  - Role-Capability Matrix 조회 (GET /api/roles/global/capabilities/matrix)
  - Role-Permission Matrix 조회 (GET /api/roles/global/permissions/matrix)
  - User-Project Matrix 조회 (GET /api/user-project-matrix)
  - Permission Check (POST /api/access-control/permissions/check)
  - User Permissions 조회 (GET /api/access-control/permissions/user/{user_id}/project/{project_id})
  - Project Access Check (GET /api/access-control/access/user/{user_id}/project/{project_id})
  - Access Logs 조회 (GET /api/access-control/logs/user/{user_id}, GET /api/access-control/logs/project/{project_id})

**테스트 결과**: 🎉 ALL TESTS PASSED (7/7 시나리오)

---

### 10. API 문서화 개선
**파일**: `docs/server/technical/TECHNICAL_DOCUMENTATION_TODO.md`
**상태**: ❌ 미완료
**예상 시간**: 4-6시간

**작업 내용:**
- [ ] OpenAPI/Swagger 완전 구현
  - [ ] 모든 엔드포인트에 대한 완전한 Swagger 문서
  - [ ] 요청/응답 예시 개선
  - [ ] 에러 코드 및 메시지 문서화
  - [ ] 인증/인가 플로우 문서화

- [ ] API 버전 관리 가이드
  - [ ] API 버전 전략 수립
  - [ ] 하위 호환성 가이드라인
  - [ ] 마이그레이션 가이드

---

### 11. 데이터베이스 문서화
**파일**: `docs/server/technical/TECHNICAL_DOCUMENTATION_TODO.md`
**상태**: ❌ 미완료
**예상 시간**: 3-4시간

**작업 내용:**
- [ ] ERD (Entity Relationship Diagram) 생성
- [ ] 테이블별 상세 설명
- [ ] 인덱스 및 제약조건 문서화
- [ ] 마이그레이션 스크립트 가이드
- [ ] 데이터 모델 가이드

---

## 🟢 Low Priority (최적화/개선)

### 7. 성능 최적화 (Phase 2)
**파일**: `docs/todo/next_phase_todo.md`  
**상태**: ⏸️ 백로그  
**예상 시간**: 1-2주

**작업 내용:**
- [ ] 데이터베이스 쿼리 최적화
- [ ] 캐싱 전략 확장
- [ ] 인덱스 최적화
- [ ] N+1 쿼리 문제 해결
- [ ] 연결 풀 최적화

---

### 8. CI/CD 파이프라인 (Phase 6)
**파일**: `docs/todo/next_phase_todo.md`  
**상태**: ⏸️ 백로그  
**예상 시간**: 1주

**작업 내용:**
- [ ] 자동 테스트 실행
- [ ] 자동 배포
- [ ] 롤백 메커니즘
- [ ] 환경 관리
- [ ] 컨테이너화 (Docker/Kubernetes)

---

## 🎯 추천 작업 순서

### Option 1: 보안 우선 (추천) ⭐
```
1. 🔒 TimePoint Annotation API 권한 체크 (1-2시간)
2. 🔧 Sync API Timeout 수정 (2-3시간)
3. 🗄️ Duplicate Data Issue 수정 (1-2시간)
4. 📝 TODO.md 업데이트
```
**총 예상 시간**: 4-7시간  
**장점**: 보안 취약점 우선 해결, 데이터 무결성 확보

---

### Option 2: 안정성 우선
```
1. 🧪 테스트 파일 수정 (6개, 3-4시간)
2. 🔒 TimePoint Annotation API 권한 체크 (1-2시간)
3. 🔧 Sync API Timeout 수정 (2-3시간)
4. 📝 문서화
```
**총 예상 시간**: 6-9시간  
**장점**: 테스트 커버리지 확보, 안정성 향상

---

### Option 3: 빠른 성과
```
1. 🗄️ Duplicate Data Issue 수정 (1-2시간) - 빠르게 해결 가능
2. 🔒 TimePoint Annotation API 권한 체크 (1-2시간)
3. 📝 TODO.md 업데이트
4. 🔧 Sync API Timeout 수정 (나중에)
```
**총 예상 시간**: 2-4시간  
**장점**: 빠른 성과, 데이터 무결성 확보

---

## 📊 작업 통계

| 우선순위 | 작업 수 | 예상 시간 |
|---------|--------|----------|
| 🔴 High | 3개 | 4-7시간 |
| 🟡 Medium | 3개 | 10-14시간 |
| 🟢 Low | 2개 | 2-3주 |
| **총계** | **8개** | **14-21시간 + 2-3주** |

---

## ✅ 최근 완료된 작업

### Project API ETag Caching (2026-01-24)
- ✅ `GET /api/projects` - 프로젝트 목록 ETag 캐싱
- ✅ `GET /api/projects/{id}` - 프로젝트 상세 ETag 캐싱
- ✅ `GET /api/projects/active` - 활성 프로젝트 목록 ETag 캐싱
- ✅ 11개 E2E 테스트 통과
- ✅ PostgreSQL 트리거 기반 자동 캐시 무효화

### Project Membership Cache (2026-01-24)
- ✅ Redis 기반 멤버십 캐싱 (23.2% 성능 개선)
- ✅ 6개 E2E 테스트 통과
- ✅ 캐시 무효화 로직 구현

---

**다음 작업**: TimePoint Annotation API 권한 체크 🔒

