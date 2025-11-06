# User Projects API 라우팅 충돌 해결 작업 계획

## 📋 작업 개요

- **작업명**: User Projects API 라우팅 충돌 해결
- **작업 유형**: Bug Fix
- **우선순위**: High
- **예상 소요시간**: 2-3시간
- **작업 시작일**: 2025-01-26
- **작업 완료일**: 2025-01-26

## 🎯 목표

### 주요 목표
- `/api/users/{user_id}/projects` API의 404 에러 해결
- 사용자별 프로젝트 목록 조회 기능 정상화
- 라우팅 충돌 문제 근본적 해결

### 부차 목표
- 기존 API 기능에 영향 없이 수정
- 코드 품질 개선 (SQL 쿼리 완전성)
- 향후 유사 문제 방지를 위한 설계 개선

## 🔍 문제 분석

### 현재 상황
- `user_controller`와 `project_user_controller`가 동일한 `/users` 스코프 사용
- `user_controller`가 먼저 등록되어 라우팅 우선권 확보
- `project_user_controller`의 `/users` 스코프가 무시됨

### 영향 범위
- 사용자별 프로젝트 목록 조회 기능 완전 중단
- 프로젝트-사용자 매트릭스 UI 기능 제한
- 페이지네이션 기능 사용 불가

## 📝 작업 계획

### Phase 1: 문제 진단 및 분석 (30분)
- [x] 라우팅 충돌 원인 파악
- [x] 관련 파일 및 코드 분석
- [x] 영향 범위 평가

### Phase 2: 해결 방안 설계 (30분)
- [x] 라우팅 충돌 해결 방안 검토
- [x] 최적 해결책 선택
- [x] 구현 계획 수립

### Phase 3: 코드 수정 (60분)
- [x] `project_user_controller.rs` 라우팅 수정
- [x] `main.rs` 컨트롤러 등록 순서 변경
- [x] `user_repository_impl.rs` SQL 쿼리 수정

### Phase 4: 테스트 및 검증 (30분)
- [x] 서버 재시작 및 컴파일 확인
- [x] API 엔드포인트 테스트
- [x] 기존 기능 영향도 확인

### Phase 5: 문서화 및 정리 (30분)
- [x] 이슈 문서 작성
- [x] 작업 문서 작성
- [x] Git 커밋 및 푸시

## 🛠️ 구현 세부사항

### 1. 라우팅 수정
**파일**: `pacs-server/src/presentation/controllers/project_user_controller.rs`

```rust
// 수정 전
.service(
    web::scope("/users")
        .route("/{user_id}/projects", web::get().to(get_user_projects::<P, U>))
);

// 수정 후
.route("/users/{user_id}/projects", web::get().to(get_user_projects::<P, U>));
```

### 2. 등록 순서 변경
**파일**: `pacs-server/src/main.rs`

```rust
// project_user_controller를 user_controller보다 먼저 등록
.configure(|cfg| {
    project_user_controller::configure_routes(cfg, project_user_use_case.clone())
})
.configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
```

### 3. SQL 쿼리 개선
**파일**: `pacs-server/src/infrastructure/repositories/user_repository_impl.rs`

모든 `find_*` 함수에서 User 엔티티의 모든 필드를 SELECT하도록 수정:

```sql
SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
       created_at, updated_at, account_status, email_verified,
       email_verification_token, email_verification_expires_at,
       approved_by, approved_at, suspended_at, suspended_reason, deleted_at
FROM security_user
WHERE [조건]
```

## ✅ 성공 기준

### 기능적 기준
- [x] `/api/users/{user_id}/projects` API가 200 OK 응답
- [x] 페이지네이션 기능 정상 작동
- [x] 프로젝트 정보 및 역할 정보 정상 반환
- [x] 기존 API 기능에 영향 없음

### 기술적 기준
- [x] 컴파일 에러 없음
- [x] 런타임 에러 없음
- [x] SQL 쿼리 에러 없음
- [x] 라우팅 충돌 해결

## 🚨 위험 요소 및 대응 방안

### 위험 요소
1. **기존 API 기능 영향**: 다른 엔드포인트에 부작용 발생 가능
2. **SQL 쿼리 변경**: 데이터베이스 스키마 호환성 문제
3. **컴파일 에러**: 타입 불일치 또는 문법 오류

### 대응 방안
1. **단계적 테스트**: 각 수정사항별 개별 테스트
2. **롤백 계획**: 문제 발생 시 이전 상태로 복구
3. **충분한 검증**: 모든 관련 기능 테스트

## 📊 예상 결과

### 즉시 효과
- 사용자별 프로젝트 목록 조회 기능 정상화
- 404 에러 완전 해결
- API 응답 시간 개선

### 장기 효과
- 라우팅 설계 개선
- 코드 품질 향상
- 유지보수성 증대

## 🔗 관련 리소스

### 참고 문서
- Actix-web 라우팅 가이드
- Rust 웹 프레임워크 모범 사례
- SQLx 쿼리 최적화 가이드

### 관련 이슈
- 이슈 #001: User Projects API 라우팅 충돌 문제
- 기술 문서: `docs/technical/ROUTING_ORDER_FIX.md`
