# Annotation Snapshot API

어노테이션 스냅샷 이미지를 S3에 업로드하고 관리하는 API 문서입니다.

---

## 📚 문서 구조

```
docs/api/annotation-snapshot-api/
├── README.md                    # 📖 이 파일
├── WORKLOG.md                   # 📝 작업 로그 및 구현 가이드
├── ARCHITECTURE.md              # 🏗️ 아키텍처 설계
├── API_SPEC.md                  # 📖 API 명세서
├── DB_MIGRATION.md              # 🗄️ DB 마이그레이션 가이드
└── issues/                      # 🔍 설계 결정 및 이슈
    ├── README.md
    ├── ISSUE-001-timestamp-responsibility.md
    ├── ISSUE-002-no-update-annotation-entity.md
    └── ISSUE-003-e2e-test-timeout.md
```

---

## 🎯 프로젝트 개요

### 목적
의료 영상 어노테이션의 **스냅샷 이미지**를 S3에 저장하고 관리하는 기능을 추가합니다.

### 주요 기능
1. **스냅샷 업로드 URL 생성** - Presigned URL 발급
2. **스냅샷 업로드 완료 알림** - 업로드 상태 업데이트
3. **스냅샷 다운로드 URL 생성** - 이미지 조회용 URL 발급
4. **스냅샷 상태 조회** - 업로드 진행 상황 확인

---

## 🚀 현재 진행 상황

### ✅ Phase 1: 데이터베이스 & 도메인 (완료)
- [x] DB 마이그레이션 작성 및 적용
- [x] Entity 수정 (Annotation, NewAnnotation)
- [x] Repository 수정 (모든 쿼리 업데이트)
- [x] 테스트 수정

### ✅ Phase 2: 도메인 서비스 (완료)
- [x] AnnotationService trait 확장
- [x] AnnotationServiceImpl 구현

### ✅ Phase 3: 애플리케이션 레이어 (완료)
- [x] DTO 작성
- [x] Use Case 구현

### ✅ Phase 4: 프레젠테이션 레이어 (완료)
- [x] Controller 구현
- [x] 라우트 등록

### ✅ Phase 5: 테스트 & 검증 (완료)
- [x] E2E 테스트 작성 및 실행
- [x] 웹 관리 페이지 통합
- [x] OpenAPI 문서 확인

### ✅ Phase 6: 웹 관리 페이지 (완료)
- [x] React 컴포넌트 작성 (AnnotationSnapshotTests)
- [x] E2E 테스트 실행 기능
- [x] CRUD 인터페이스 구현
- [x] 타임아웃 이슈 해결

---

## 📖 주요 문서

### 1. [WORKLOG.md](./WORKLOG.md)
- 단계별 구현 가이드
- 코드 예시
- 체크리스트

### 2. [ARCHITECTURE.md](./ARCHITECTURE.md)
- 시스템 아키텍처
- 데이터 흐름
- 컴포넌트 구조

### 3. [API_SPEC.md](./API_SPEC.md)
- API 엔드포인트 명세
- 요청/응답 예시
- 에러 코드

### 4. [DB_MIGRATION.md](./DB_MIGRATION.md)
- 데이터베이스 스키마 변경
- 마이그레이션 가이드

### 5. [issues/](./issues/)
- 설계 결정 문서
- 기술적 이슈 해결 과정

---

## 🔑 핵심 설계 결정

### 1. 타임스탬프 생성 책임
- **결정**: 서버가 `snapshot_uploaded_at`을 자동 생성
- **이유**: 보안, 일관성, 정확성
- **문서**: [ISSUE-001](./issues/ISSUE-001-timestamp-responsibility.md)

### 2. 업데이트 패턴
- **결정**: `update_snapshot` 전용 메서드 사용
- **이유**: 기존 코드베이스 패턴 유지
- **문서**: [ISSUE-002](./issues/ISSUE-002-no-update-annotation-entity.md)

### 3. E2E 테스트 타임아웃
- **문제**: 웹 관리 페이지에서 E2E 테스트 실행 시 타임아웃 발생
- **해결**: Python 타임아웃 증가 + Rust 비동기 실행
- **문서**: [ISSUE-003](./issues/ISSUE-003-e2e-test-timeout.md)

---

## 🛠️ 기술 스택

- **언어**: Rust
- **프레임워크**: Actix-web
- **데이터베이스**: PostgreSQL
- **ORM**: SQLx
- **스토리지**: AWS S3
- **인증**: Presigned URL

---

## 📞 관련 문서

- [전체 API 문서](../../api-documentation.md)
- [Annotation API](../annotation-api/)
- [DICOM API](../dicom/)

---

## 🎉 구현 완료 요약

### 완료된 기능
1. **스냅샷 업로드 URL 생성** - Presigned URL 발급 ✅
2. **스냅샷 업로드 완료 알림** - 업로드 상태 업데이트 ✅
3. **스냅샷 다운로드 URL 생성** - 이미지 조회용 URL 발급 ✅
4. **스냅샷 상태 조회** - 업로드 진행 상황 확인 ✅
5. **E2E 테스트** - Python 테스트 스크립트 작성 및 실행 ✅
6. **웹 관리 페이지** - React 기반 테스트 UI 구현 ✅

### API 엔드포인트
- `POST /api/annotations/{id}/snapshot/upload-url` - 업로드 URL 생성
- `POST /api/annotations/{id}/snapshot/complete-upload` - 업로드 완료 처리
- `GET /api/annotations/{id}/snapshot/status` - 스냅샷 상태 조회
- `GET /api/test/annotation-snapshot-e2e` - E2E 테스트 실행

### 웹 관리 페이지
- **위치**: http://localhost:3000 → API 점검 → Annotation Snapshot (📸)
- **기능**:
  - E2E 테스트 실행 버튼
  - CRUD 인터페이스 (생성/업로드/완료/조회)
  - 실시간 테스트 결과 표시
  - 스냅샷 데이터 시각화

---

**최종 업데이트**: 2026-01-12
**상태**: 모든 Phase 완료 ✅

