# 인증 및 권한 관리 구현 TODO

## Phase 1: AuthGuard JWT 인증 구현

### 1. 설정 및 기본 구조
- [ ] `AuthGuardConfig` 구조체 추가 (`src/infrastructure/config/settings.rs`)
- [ ] 설정 파일 업데이트 (`config/default.toml`)
- [ ] `Settings` 구조체에 `auth_guard` 필드 추가

### 2. AuthGuard Middleware 구현
- [ ] `src/infrastructure/middleware/auth_guard.rs` 생성
- [ ] `AuthGuard` 구조체 구현
- [ ] `Transform` trait 구현 (미들웨어 팩토리)
- [ ] `Service` trait 구현 (실제 요청 처리)
- [ ] JWT 토큰 추출 로직
- [ ] 토큰 검증 로직
- [ ] Claims를 HttpRequest extensions에 저장
- [ ] 화이트리스트 경로 매칭 로직
- [ ] 에러 응답 처리 (401 Unauthorized)

### 3. Claims Extractor 구현
- [ ] `src/presentation/extractors/` 디렉토리 생성
- [ ] `src/presentation/extractors/mod.rs` 생성
- [ ] `src/presentation/extractors/auth_extractor.rs` 생성
  - [ ] `AuthenticatedUser` 구조체
  - [ ] `FromRequest` trait 구현
- [ ] `src/presentation/extractors/optional_claims.rs` 생성 (선택사항)
  - [ ] `OptionalUser` 구조체
  - [ ] 선택적 인증 로직

### 4. 모듈 Export 설정
- [ ] `src/infrastructure/middleware/mod.rs` 업데이트
- [ ] `src/presentation/mod.rs` 업데이트
- [ ] `src/presentation/extractors/mod.rs` 생성

### 5. Main.rs 통합
- [ ] `JwtService` 초기화 확인
- [ ] `AuthGuard` 미들웨어 등록
- [ ] 설정에서 `enabled` 플래그 읽기
- [ ] 화이트리스트 경로 설정

### 6. 컨트롤러 수정
- [ ] `annotation_controller.rs` 업데이트
  - [ ] `create_annotation` 함수 시그니처 수정
  - [ ] `list_annotations` 함수 시그니처 수정
  - [ ] `update_annotation` 함수 시그니처 수정
  - [ ] `delete_annotation` 함수 시그니처 수정
  - [ ] `get_annotation` 함수 시그니처 수정
- [ ] `mask_group_controller.rs` 업데이트
  - [ ] 모든 함수에 `AuthenticatedUser` 파라미터 추가
- [ ] `mask_controller.rs` 업데이트
  - [ ] 모든 함수에 `AuthenticatedUser` 파라미터 추가
- [ ] `user_controller.rs` 업데이트
  - [ ] `get_user`, `update_user`, `delete_user` 함수 수정
  - [ ] `create_user`는 회원가입이므로 제외
- [ ] `project_controller.rs` 업데이트
  - [ ] 모든 함수에 `AuthenticatedUser` 파라미터 추가
- [ ] `permission_controller.rs` 업데이트
  - [ ] 모든 함수에 `AuthenticatedUser` 파라미터 추가
- [ ] `access_control_controller.rs` 업데이트
  - [ ] 모든 함수에 `AuthenticatedUser` 파라미터 추가

### 7. DTO 수정
- [ ] `CreateAnnotationRequest`에서 `user_id` 제거 (JWT에서 추출)
- [ ] `project_id`를 `Option<i32>`로 변경 (선택사항)
- [ ] 다른 DTO들도 필요시 수정

### 8. 단위 테스트
- [ ] `auth_guard.rs` 내부 tests 모듈
  - [ ] 화이트리스트 경로 매칭 테스트
  - [ ] 토큰 추출 로직 테스트
  - [ ] enabled=false일 때 동작 테스트
  - [ ] 에러 응답 테스트
- [ ] `auth_extractor.rs` 테스트
  - [ ] Claims 추출 테스트
  - [ ] 인증 실패시 401 반환 테스트

### 9. 통합 테스트
- [ ] `tests/auth_guard_integration_test.rs` 생성
- [ ] 인증 없이 보호된 엔드포인트 호출 → 401 테스트
- [ ] 유효한 토큰으로 호출 → 성공 테스트
- [ ] 유효하지 않은 토큰으로 호출 → 401 테스트
- [ ] 만료된 토큰으로 호출 → 401 테스트
- [ ] Bearer 형식이 아닌 토큰 → 401 테스트
- [ ] 화이트리스트 경로 (인증 없음) → 200 테스트
- [ ] enabled=false일 때 인증 없이 접근 가능 테스트

### 10. 문서화 및 검증
- [ ] README 업데이트
- [ ] API 문서 업데이트
- [ ] 전체 테스트 실행
- [ ] 성능 테스트
- [ ] 보안 검토

## Phase 2: RBAC 권한 제어 구현

### 1. Claims 확장
- [ ] `Claims` 구조체에 `roles: Vec<String>` 필드 추가
- [ ] `Claims::new()` 함수 수정
- [ ] Role 관련 메서드 추가

### 2. 데이터베이스 스키마
- [ ] Role-Permission 매핑 테이블 생성
- [ ] User-Role 매핑 테이블 생성
- [ ] 기본 Role 데이터 시딩
- [ ] 기본 Permission 데이터 시딩

### 3. Permission Service 구현
- [ ] `src/domain/services/permission_service.rs` 확장
- [ ] `src/application/services/permission_service.rs` 생성
- [ ] 권한 검증 로직 구현
- [ ] Role-Permission 매핑 로직

### 4. Permission Guard 구현
- [ ] `src/infrastructure/middleware/permission_guard.rs` 생성
- [ ] `PermissionGuard` 구조체 구현
- [ ] 권한 검증 미들웨어 구현
- [ ] 매크로 지원 (선택사항)

### 5. 리소스 소유자 확인
- [ ] Annotation 소유자 확인 로직
- [ ] Project 멤버십 확인 로직
- [ ] Mask Group 소유자 확인 로직
- [ ] Use Case 레이어에 권한 검증 추가

### 6. 컨트롤러 권한 검증
- [ ] 모든 컨트롤러에 권한 검증 추가
- [ ] 403 Forbidden 에러 처리
- [ ] 권한 부족시 상세 에러 메시지

### 7. RBAC 테스트
- [ ] Role별 권한 테스트
- [ ] 프로젝트별 권한 격리 테스트
- [ ] 리소스 소유자 확인 테스트
- [ ] 권한 없는 사용자 접근 거부 테스트

## 공통 작업

### 1. 로깅
- [ ] 인증 실패 이벤트 로깅
- [ ] 권한 거부 이벤트 로깅
- [ ] 보안 모니터링 로그

### 2. 성능 최적화
- [ ] 권한 정보 캐싱
- [ ] 데이터베이스 쿼리 최적화
- [ ] 인덱스 추가

### 3. 모니터링
- [ ] 인증 성공/실패율 모니터링
- [ ] 권한 검증 성능 모니터링
- [ ] API 접근 패턴 분석

### 4. 보안 검토
- [ ] 토큰 보안 검토
- [ ] 권한 설정 검토
- [ ] 접근 로그 검토
- [ ] 취약점 스캔

## 우선순위

### 높음 (Phase 1)
1. AuthGuard Middleware 구현
2. Claims Extractor 구현
3. 컨트롤러 수정
4. 기본 테스트

### 중간 (Phase 1 완료 후)
1. RBAC 기본 구조
2. 권한 검증 로직
3. 리소스 소유자 확인

### 낮음 (추가 기능)
1. 고급 권한 기능
2. 성능 최적화
3. 모니터링 강화

## 참고사항

- Phase 1 완료 후 Phase 2 시작
- 각 단계별로 충분한 테스트 필요
- 보안 검토는 각 Phase 완료 후 진행
- 기존 기능과의 호환성 유지 중요
