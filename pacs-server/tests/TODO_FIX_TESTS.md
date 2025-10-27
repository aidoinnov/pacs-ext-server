# 테스트 파일 수정 TODO

## 📊 현재 상태
- **메인 라이브러리**: ✅ 빌드 성공
- **완료된 테스트 파일**: 25개
- **남은 오류 테스트 파일**: 6개

## ❌ 오류가 있는 테스트 파일 (6개)

### 1. annotation_controller_test_fixed.rs
**문제점**: 
- AnnotationController 관련 import 문제
- Actix Web API 변경 사항 반영 필요

**조치**: 
- [ ] 컨트롤러 핸들러 시그니처 확인
- [ ] Actix Web 최신 API로 업데이트
- [ ] TestRequest 사용법 확인

---

### 2. api_documentation_test.rs
**문제점**: 
- API 문서화 테스트 관련 문제
- OpenAPI/Swagger 문법 오류 가능성

**조치**: 
- [ ] utoipa 문서화 문법 확인
- [ ] DTO ToSchema 어노테이션 검증
- [ ] API 문서 생성 로직 점검

---

### 3. mask_use_case_test.rs
**문제점**: 
- MaskUseCase 관련 문제
- S3Service import 또는 타입 불일치

**조치**: 
- [ ] S3Service import 경로 확인
- [ ] MaskRepository 메서드 시그니처 검증
- [ ] Mock 사용법 확인

---

### 4. matrix_integration_test.rs
**문제점**: 
- 매트릭스 통합 테스트 문제
- 복잡한 Repository 구조

**조치**: 
- [ ] Repository 초기화 방식 확인
- [ ] 데이터베이스 연결 설정 검증
- [ ] 복합 쿼리 로직 점검

---

### 5. user_registration_controller_unit_test.rs
**문제점**: 
- UserRegistrationController 테스트
- Keycloak 클라이언트 관련 문제

**조치**: 
- [ ] KeycloakClient mock 설정
- [ ] JWT 토큰 생성 로직 확인
- [ ] 회원가입 플로우 검증

---

### 6. user_use_case_test.rs
**문제점**: 
- UserUseCase 테스트 문제
- User 엔티티 필드 변경 반영 필요

**조치**: 
- [ ] User 엔티티 필드 확인
- [ ] CreateUserRequest DTO 업데이트
- [ ] Mock UserRepository 시그니처 수정

---

## 📝 일반적인 수정 가이드

### Entity/DTO 변경사항
현재 User, Project, Permission 엔티티에 많은 필드가 추가되어 테스트 데이터 생성 시 다음과 같은 필드들이 필요합니다:

**User 엔티티 추가 필드**:
- `full_name`, `organization`, `department`, `phone`
- `account_status`, `email_verified`
- `email_verification_token`, `email_verification_expires_at`
- `approved_by`, `approved_at`
- `suspended_at`, `suspended_reason`, `deleted_at`

**Project 엔티티 추가 필드**:
- `sponsor`, `start_date`, `end_date`, `auto_complete`, `status`

**Permission 엔티티 추가 필드**:
- `category`

### Import 경로 수정
많은 테스트 파일에서 import 경로가 다음과 같이 수정되었습니다:
```rust
// 기존 (잘못된)
use pacs_server::domain::entities::permission::{Role, Permission};

// 수정된 (올바른)
use pacs_server::domain::entities::{Role, Permission};
```

### Service Trait 구현
mockall을 사용한 mock 구현 시:
- Trait 이름과 모듈 위치 확인
- Generic 타입 파라미터 주의
- `async_trait` 매크로 사용

### Repository 초기화
Database pool을 사용하는 경우:
```rust
// Before (잘못된)
let repository = MyRepository::new(*pool);

// After (올바른)
let repository = MyRepository::new(pool.clone());
```

---

## 🎯 우선순위

### 높은 우선순위
1. **user_use_case_test.rs** - 핵심 사용자 관리 기능
2. **user_registration_controller_unit_test.rs** - 회원가입 기능

### 중간 우선순위
3. **mask_use_case_test.rs** - 마스크 기능 (핵심 기능)
4. **annotation_controller_test_fixed.rs** - 어노테이션 기능

### 낮은 우선순위
5. **matrix_integration_test.rs** - 복잡한 통합 테스트
6. **api_documentation_test.rs** - 문서화 테스트

---

## 💡 참고사항

- 모든 테스트는 실제 데이터베이스 연결이 필요할 수 있습니다
- Mock 사용이 어려운 경우 통합 테스트로 전환 고려
- PermissionService처럼 복잡한 Generic 구조는 통합 테스트 추천

## 📈 진행 상황

- ✅ 완료: 25개 테스트 파일
- ⏳ 진행중: 6개 테스트 파일
- 📊 완료율: 80.6% (25/31)

