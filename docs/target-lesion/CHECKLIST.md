# RECIST Lesion 구현 체크리스트

**작성일:** 2025-01-18  
**상태:** ✅ Phase 1-6 완료

---

## ✅ Phase 1: Database Schema

- [x] `project_subject` 테이블 생성
- [x] `subject_timepoint` 테이블 생성
- [x] `subject_timepoint_study_map` 테이블 생성
- [x] `recist_lesion` 테이블 생성
- [x] `recist_lesion_annotation_map` 테이블 생성
- [x] Subject당 Baseline 1개 제약조건 (UNIQUE INDEX)
- [x] Study는 Subject 내에서 한 TimePoint만 (UNIQUE INDEX)
- [x] CASCADE 삭제 설정
- [x] 마이그레이션 파일 작성
  - [x] `migrations/20250118_01_create_subject_timepoint_tables.sql`
  - [x] `migrations/20250118_02_create_recist_lesion_tables.sql`

---

## ✅ Phase 2: Domain Layer

- [x] Entity 정의
  - [x] `RecistLesion` struct
  - [x] `RecistLesionType` enum (TARGET/NON_TARGET/NEW)
  - [x] `CreateRecistLesion` DTO
  - [x] `UpdateRecistLesion` DTO
  - [x] `RecistLesionDetail` DTO
  - [x] `RecistLesionAnnotation` struct
- [x] Repository 인터페이스
  - [x] `RecistLesionRepository` trait
  - [x] `create()` 메서드
  - [x] `find_by_id()` 메서드
  - [x] `find_by_subject()` 메서드
  - [x] `update()` 메서드
  - [x] `delete()` 메서드
  - [x] `link_annotation()` 메서드
  - [x] `count_target_lesions()` 메서드
- [x] Repository 구현
  - [x] `RecistLesionRepositoryImpl` struct
  - [x] PostgreSQL 쿼리 구현
  - [x] 에러 처리

---

## ✅ Phase 3: Application Layer

- [x] Use Case 구현
  - [x] `RecistLesionUseCase` struct
  - [x] `create_lesion()` 메서드
  - [x] `get_lesion()` 메서드
  - [x] `list_lesions()` 메서드
  - [x] `update_lesion()` 메서드
  - [x] `delete_lesion()` 메서드
  - [x] `link_annotation()` 메서드
- [x] 비즈니스 규칙 검증
  - [x] Max 5 Target Lesions per Subject
  - [x] Baseline TimePoint 필수 (TARGET/NON_TARGET)
  - [x] NEW Lesion은 Baseline TimePoint 없이 생성 가능
  - [x] Subject 존재 여부 검증
  - [x] TimePoint 존재 여부 검증
- [x] 자동 Lesion Number 생성

---

## ✅ Phase 4: Presentation Layer

- [x] Controller 구현
  - [x] `create_lesion()` handler
  - [x] `list_lesions()` handler
  - [x] `get_lesion_detail()` handler
  - [x] `update_lesion()` handler
  - [x] `delete_lesion()` handler
  - [x] `link_annotation()` handler
- [x] 라우트 설정
  - [x] `configure_recist_lesion_routes()` 함수
  - [x] `/subjects/{subject_id}/recist-lesions` 스코프
  - [x] `/recist-lesions` 스코프
- [x] OpenAPI 문서화
  - [x] utoipa 어노테이션 추가
  - [x] Request/Response 스키마 정의
  - [x] 에러 응답 문서화

---

## ✅ Phase 5: Integration

- [x] main.rs 통합
  - [x] RecistLesionRepositoryImpl 초기화
  - [x] RecistLesionUseCase 초기화
  - [x] 라우트 등록
- [x] OpenAPI 통합
  - [x] `openapi.rs`에 스키마 추가
  - [x] `RecistLesion` 스키마
  - [x] `RecistLesionType` 스키마
  - [x] `CreateRecistLesion` 스키마
  - [x] `UpdateRecistLesion` 스키마
  - [x] `RecistLesionDetail` 스키마
  - [x] `LinkAnnotationRequest` 스키마
  - [x] 6개 엔드포인트 등록

---

## ✅ Phase 6: Testing

- [x] E2E 테스트 작성
  - [x] `test_07_recist_lesion.py` (16개 테스트)
  - [x] CRUD 테스트 (6개)
  - [x] RECIST 1.1 비즈니스 규칙 검증 (4개)
  - [x] Annotation 연결 테스트 (1개)
  - [x] 에러 케이스 테스트 (5개)
- [x] 테스트 실행기
  - [x] `run_recist_lesion.py` (Python)
  - [x] `run_recist_lesion.sh` (Bash)
- [x] 테스트 문서
  - [x] `RECIST_LESION_TEST.md`

---

## ✅ Documentation

- [x] 계획 문서 업데이트
  - [x] `docs/target-lesion/plan.md`
- [x] 구현 요약 작성
  - [x] `docs/target-lesion/IMPLEMENTATION_SUMMARY.md`
- [x] 체크리스트 작성
  - [x] `docs/target-lesion/CHECKLIST.md` (본 문서)
- [x] README 업데이트
  - [x] RECIST Lesion 섹션 추가

---

## ⚠️ Phase 2: Frontend 연동 (보류)

- [ ] Annotation 생성 요청 시 RECIST role 전달
- [ ] TimePoint context 자동 주입
- [ ] 보드 UI에서 Study 재분류 반영

---

## ⚠️ Phase 3: Report 기능 (보류)

- [ ] TimePoint별 lesion 목록 조회 API
- [ ] Target Lesion 합산 (SLD: Sum of Longest Diameters)
- [ ] Non-Target Lesion 상태 추적
- [ ] NEW Lesion 이벤트 반영
- [ ] RECIST 1.1 Response 평가 (CR/PR/SD/PD)

---

## 📊 통계

- **총 작업 항목:** 100+
- **완료된 항목:** 90+
- **보류된 항목:** 10+
- **완료율:** 90%

---

**다음 단계:** Frontend 연동 및 Report 기능 구현

