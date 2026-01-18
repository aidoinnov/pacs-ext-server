# Subject & TimePoint 기능 구현 진행 상황

## 📊 전체 진행률: 100% (8/8 단계 완료) ✅

---

## ✅ 완료된 단계

### 1. ERD 설계 및 문서화 ✅
**완료일**: 2026-01-18

**산출물**:
- `docs/timepoint/erd.md` - Subject/TimePoint ERD 설계
- `docs/database/ERD.md` - 전체 ERD에 통합

**주요 내용**:
- Subject 테이블: 프로젝트별 환자 관리
- TimePoint 테이블: Subject별 평가 시점 관리
- TimePoint-Study 매핑 테이블: Study 할당 관리
- 제약 조건: Baseline 유일성, Subject 코드 유일성
- 인덱스 전략: 성능 최적화

---

### 2. 데이터베이스 마이그레이션 ✅
**완료일**: 2026-01-18

**산출물**:
- `migrations/040_create_subject_timepoint.sql`

**주요 내용**:
- Subject 테이블 생성
- TimePoint 테이블 생성 (VisitType ENUM 포함)
- TimePoint-Study 매핑 테이블 생성
- 제약 조건 및 인덱스 생성
- CASCADE DELETE 설정

**검증 완료**:
- 마이그레이션 실행 성공
- 제약 조건 동작 확인
- 인덱스 생성 확인

---

### 3. API 스펙 설계 ✅
**완료일**: 2026-01-18

**산출물**:
- `docs/timepoint/api.md` - API 개요 및 설계 원칙
- `docs/timepoint/api-spec.md` - 상세 API 스펙

**주요 엔드포인트**:
- `GET/POST /api/projects/{id}/subjects` - Subject 관리
- `GET/POST/PUT/DELETE /api/subjects/{id}/timepoints` - TimePoint 관리
- `POST/DELETE /api/timepoints/{id}/studies` - Study 할당/해제
- `GET /api/subjects/{id}/board` - 통합 보드 뷰
- `GET /api/subjects/{id}/studies/unassigned` - Unassigned Studies

**에러 코드 정의**:
- 400: INVALID_REQUEST
- 404: SUBJECT_NOT_FOUND, TIMEPOINT_NOT_FOUND
- 409: SUBJECT_CODE_DUPLICATE, BASELINE_ALREADY_EXISTS

---

### 4. 도메인 모델 구현 ✅
**완료일**: 2026-01-18

**산출물**:
- `pacs-server/src/domain/entities/subject.rs`
- `pacs-server/src/domain/entities/timepoint.rs`
- `pacs-server/src/domain/entities/timepoint_study.rs`

**주요 엔티티**:
- `Subject`: 프로젝트별 환자 엔티티
- `TimePoint`: Subject별 평가 시점 엔티티
- `VisitType`: Baseline, Visit, EOT, USV
- `TimePointStudy`: Study 할당 매핑 엔티티

**DTO 클래스**:
- `CreateSubject`, `UpdateSubject`, `SubjectDetail`
- `CreateTimePoint`, `UpdateTimePoint`
- `AssignStudies`, `UnassignStudies`, `AssignmentResult`
- `TimePointStudies`, `StudyInfo`

---

### 5. Repository 인터페이스 정의 ✅
**완료일**: 2026-01-18

**산출물**:
- `pacs-server/src/domain/repositories/subject_repository.rs`
- `pacs-server/src/domain/repositories/timepoint_repository.rs`
- `pacs-server/src/domain/repositories/timepoint_study_repository.rs`

**주요 메서드**:
- SubjectRepository: find_by_id, find_by_code, find_by_patient_id, create, update, delete
- TimePointRepository: find_by_subject, find_baseline_by_subject, create, update, delete
- TimePointStudyRepository: assign_studies, unassign_studies, find_studies_by_timepoint

---

### 6. Repository 구현 ✅
**완료일**: 2026-01-18

**산출물**:
- `pacs-server/src/infrastructure/repositories/subject_repository_impl.rs`
- `pacs-server/src/infrastructure/repositories/timepoint_repository_impl.rs`
- `pacs-server/src/infrastructure/repositories/timepoint_study_repository_impl.rs`

**구현 패턴**:
- sqlx::query_as for type-safe queries
- COALESCE for partial updates
- RETURNING clause for created/updated entities
- LEFT JOIN for aggregations
- NOT EXISTS for exclusion queries
- ON CONFLICT for upsert semantics

---

### 7. Service 레이어 구현 ✅
**완료일**: 2026-01-18

**산출물**:
- `pacs-server/src/domain/services/subject_service.rs`
- `pacs-server/src/domain/services/timepoint_service.rs`

**구현 기능**:
- SubjectService: create, get, get_detail, get_by_project, update, delete
- TimePointService: create, get, get_by_subject, update, delete
- Study 할당/해제: assign_studies, unassign_studies, get_studies_by_timepoint
- 비즈니스 규칙 검증:
  * Subject 코드 유일성 (프로젝트 내)
  * Patient ID 유일성 (프로젝트 내)
  * Baseline 유일성 (Subject 내)
  * TimePoint 이름 유일성 (Subject 내)
  * CASCADE 방지 (Subject → TimePoint → Study)
- MOVE 시맨틱 (Study 할당 시 자동 이동)

---

### 8. Controller 구현 ✅
**완료일**: 2026-01-18

**산출물**:
- `pacs-server/src/presentation/controllers/subject_controller.rs`
- `pacs-server/src/presentation/controllers/timepoint_controller.rs`

**구현 기능**:
- SubjectController:
  * POST /api/projects/{project_id}/subjects
  * GET /api/subjects/{id}
  * GET /api/subjects/{id}/detail
  * GET /api/projects/{project_id}/subjects
  * PUT /api/subjects/{id}
  * DELETE /api/subjects/{id}
- TimePointController:
  * POST /api/subjects/{subject_id}/timepoints
  * GET /api/timepoints/{id}
  * GET /api/subjects/{subject_id}/timepoints
  * PUT /api/timepoints/{id}
  * DELETE /api/timepoints/{id}
  * POST /api/timepoints/{id}/studies
  * DELETE /api/timepoints/{id}/studies
  * GET /api/timepoints/{id}/studies
  * GET /api/subjects/{subject_id}/studies/unassigned
- HTTP 상태 코드: 200, 201, 204, 400, 404, 409, 500
- OpenAPI 문서화 (utoipa::path)
- 에러 핸들링 (ServiceError → HTTP Status)

---

## 📈 커밋 히스토리

1. `docs: Add Subject & TimePoint ERD design` - ERD 설계
2. `docs: Integrate Subject/TimePoint into main ERD` - ERD 통합
3. `feat: Add Subject & TimePoint database migration` - 마이그레이션
4. `docs: Add comprehensive Subject & TimePoint API specification` - API 스펙
5. `feat: Add Subject, TimePoint, and TimePointStudy domain entities` - 도메인 모델
6. `feat: Add Subject, TimePoint, and TimePointStudy repository traits` - Repository 인터페이스
7. `feat: Implement Subject, TimePoint, and TimePointStudy repositories` - Repository 구현
8. `feat: Implement Subject and TimePoint domain services` - Service 레이어 구현
9. `feat: Implement Subject and TimePoint REST API controllers` - Controller 구현
10. `docs: Update implementation progress to 100% complete` - 진행 상황 업데이트

---

## 🎯 다음 작업 계획

### 필수 작업

1. **서비스 통합 및 라우팅 설정** (예상 소요 시간: 1-2시간)
   - main.rs에 서비스 인스턴스 생성
   - 라우팅 설정 추가
   - 의존성 주입 설정

2. **E2E 테스트 작성** (예상 소요 시간: 3-4시간)
   - Subject CRUD 테스트
   - TimePoint CRUD 테스트
   - Study 할당/해제 테스트
   - 비즈니스 규칙 검증 테스트
   - 에러 케이스 테스트

3. **OpenAPI 문서 통합** (예상 소요 시간: 1시간)
   - OpenAPI 스키마에 Subject/TimePoint 추가
   - Swagger UI에서 확인

### 선택 작업

4. **권한 검증 통합** (예상 소요 시간: 2-3시간)
   - RBAC 정책 정의
   - 권한 검증 미들웨어 추가
   - 프로젝트 멤버십 확인

5. **성능 최적화** (예상 소요 시간: 2-3시간)
   - 쿼리 최적화
   - 인덱스 튜닝
   - 캐싱 전략

6. **사용자 가이드 작성** (예상 소요 시간: 1-2시간)
   - API 사용 예제
   - 워크플로우 가이드
   - 트러블슈팅 가이드

---

## 📝 생성된 주요 파일 (총 19개)

**문서**:
- `docs/timepoint/erd.md`
- `docs/timepoint/api.md`
- `docs/timepoint/api-spec.md`
- `docs/timepoint/IMPLEMENTATION_PROGRESS.md`
- `docs/database/ERD.md` (업데이트)

**마이그레이션**:
- `migrations/040_create_subject_timepoint.sql`

**도메인 모델**:
- `pacs-server/src/domain/entities/subject.rs`
- `pacs-server/src/domain/entities/timepoint.rs`
- `pacs-server/src/domain/entities/timepoint_study.rs`

**Repository 인터페이스**:
- `pacs-server/src/domain/repositories/subject_repository.rs`
- `pacs-server/src/domain/repositories/timepoint_repository.rs`
- `pacs-server/src/domain/repositories/timepoint_study_repository.rs`

**Repository 구현**:
- `pacs-server/src/infrastructure/repositories/subject_repository_impl.rs`
- `pacs-server/src/infrastructure/repositories/timepoint_repository_impl.rs`
- `pacs-server/src/infrastructure/repositories/timepoint_study_repository_impl.rs`

**Service 구현**:
- `pacs-server/src/domain/services/subject_service.rs`
- `pacs-server/src/domain/services/timepoint_service.rs`

**Controller 구현**:
- `pacs-server/src/presentation/controllers/subject_controller.rs`
- `pacs-server/src/presentation/controllers/timepoint_controller.rs`

---

## 📝 참고 문서

- [ERD 설계](./erd.md)
- [API 스펙](./api-spec.md)
- [전체 ERD](../database/ERD.md)
- [마이그레이션 스크립트](../../migrations/040_create_subject_timepoint.sql)
