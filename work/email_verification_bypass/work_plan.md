# 작업 계획: 이메일 인증 우회 및 계정 관리 API

## 작업 개요
- **작업명**: 회원가입 이메일 인증 우회 및 아이디/비밀번호 찾기 API 구현
- **작업 기간**: 2025-01-27
- **담당자**: AI Assistant
- **상태**: 진행중

## 작업 목적
회원가입 시 이메일 인증을 건너뛰고 관리자 승인 대기 상태로 생성하며, 사용자 편의를 위한 아이디 찾기 및 비밀번호 재설정 API를 구현합니다.

## 요구사항
1. 회원가입 시 이메일 인증 건너뛰기
   - 계정 상태를 `PENDING_APPROVAL`로 생성
   - `email_verified`를 `true`로 설정
   - 관리자 승인 대기 상태

2. 아이디 찾기 API 구현
   - 이메일로 사용자명 조회
   - 이메일 마스킹 처리

3. 비밀번호 재설정 API 구현
   - username + email 일치 확인
   - Keycloak 비밀번호 재설정

## 작업 범위

### 수정된 파일
1. `pacs-server/src/infrastructure/services/user_registration_service_impl.rs`
   - `signup` 메서드 수정 (PENDING_APPROVAL 상태로 생성)

2. `pacs-server/src/application/use_cases/user_registration_use_case.rs`
   - 응답 메시지 수정

3. `pacs-server/src/application/dto/auth_dto.rs`
   - `FindUsernameRequest`, `FindUsernameResponse` 추가
   - `ResetPasswordRequest`, `ResetPasswordResponse` 추가
   - `mask_email` 함수 추가

4. `pacs-server/src/application/use_cases/auth_use_case.rs`
   - `find_username`, `reset_password` 메서드 추가 (TODO)

5. `pacs-server/src/infrastructure/external/keycloak_client.rs`
   - `reset_user_password` 메서드 추가

6. `pacs-server/src/presentation/controllers/auth_controller.rs`
   - `find_username`, `reset_password` 핸들러 추가
   - 라우팅 추가

7. `pacs-server/src/main.rs`
   - AuthUseCase 인스턴스 생성 수정

## 작업 단계
1. 회원가입 이메일 인증 우회 구현
2. DTO 추가 (아이디 찾기, 비밀번호 재설정)
3. Keycloak 비밀번호 재설정 메서드 추가
4. Controller 및 라우팅 추가
5. 컴파일 검증
6. 문서 작성

## 현재 상태
- ✅ 회원가입 이메일 인증 우회 완료
- ✅ DTO 추가 완료
- ✅ Keycloak 비밀번호 재설정 메서드 추가 완료
- ✅ Controller 및 라우팅 추가 완료
- ✅ 컴파일 성공
- ⏳ 아이디/비밀번호 찾기 실제 구현 필요 (TODO)
- ⏳ 문서 작성 필요

## 남은 작업
- 아이디 찾기 실제 구현 (UserRepository 통합)
- 비밀번호 재설정 실제 구현 (UserRepository 통합)
- API 문서 작성
- 테스트

## 참고 사항
현재 아이디 찾기와 비밀번호 재설정은 stub으로 구현되어 있습니다.
UserRepository에 접근하려면 AuthUseCase 구조를 변경하거나 AuthService를 확장해야 합니다.

