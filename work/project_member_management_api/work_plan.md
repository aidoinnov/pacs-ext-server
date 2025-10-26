# 프로젝트 멤버 관리 API 구현 계획

## 개요
프로젝트 멤버를 추가, 삭제, 확인하는 3개의 API를 구현하여 프로젝트 멤버 관리를 완성합니다.

## 구현할 API

### 1. 프로젝트 멤버 추가 API
- **엔드포인트**: `POST /api/projects/{project_id}/members`
- **기능**: 사용자를 프로젝트에 멤버로 추가
- **요청 본문**: `{"user_id": 123, "role_id": 1632}` (role_id는 선택사항)
- **응답**: 추가된 멤버 정보와 역할 정보

### 2. 프로젝트 멤버 삭제 API
- **엔드포인트**: `DELETE /api/projects/{project_id}/members/{user_id}`
- **기능**: 프로젝트에서 사용자 멤버 제거
- **응답**: 제거 완료 메시지

### 3. 멤버십 확인 API
- **엔드포인트**: `GET /api/projects/{project_id}/members/{user_id}/membership`
- **기능**: 사용자의 프로젝트 멤버십 상태 확인
- **응답**: 멤버십 상태, 역할 정보, 가입일

## 기술 구조

### Domain Layer
- **Entity**: 기존 `security_user_project` 테이블 사용
- **Service Interface**: `UserService`에 새로운 메서드 추가
  - `add_user_to_project_with_role(user_id, project_id, role_id)`
  - `get_project_membership(user_id, project_id)`

### Application Layer
- **Use Case**: `ProjectUserUseCase`에 새로운 메서드 추가
  - `add_member_to_project()`
  - `remove_member_from_project()`
  - `check_project_membership()`
- **DTO**: 
  - `AddMemberRequest`
  - `AddMemberResponse`
  - `RemoveMemberResponse`
  - `MembershipResponse`

### Infrastructure Layer
- **Service Implementation**: SQL 쿼리 구현
  - INSERT INTO security_user_project
  - DELETE FROM security_user_project
  - SELECT with JOIN for membership check

### Presentation Layer
- **Controller**: `project_user_controller.rs`에 새 엔드포인트 추가
  - `add_project_member()`
  - `remove_project_member()`
  - `check_project_membership()`

## 주요 고려사항

1. **역할 자동 할당**: 멤버 추가 시 역할이 없으면 기본 역할(Viewer) 자동 할당
2. **중복 체크**: 이미 멤버인 경우 409 Conflict 응답
3. **프로젝트 데이터 접근 권한**: 멤버 추가 시 기존 프로젝트 데이터에 대한 접근 권한 자동 부여
4. **트랜잭션**: 멤버 추가/삭제 시 관련 데이터 일관성 보장
5. **감사 로그**: 멤버 추가/삭제 이벤트 기록

## 구현 순서

1. ✅ DTO 정의 (`application/dto/project_user_dto.rs`)
2. ✅ Service 인터페이스 메서드 추가
3. ✅ Service 구현체 작성
4. ✅ Use Case 로직 구현
5. ✅ Controller 엔드포인트 추가
6. ✅ 라우팅 설정
7. ✅ OpenAPI 문서화
8. ✅ 단위 테스트 작성
9. ✅ 통합 테스트 작성
10. ✅ API 테스트 및 검증

## 파일 수정 목록

- ✅ `src/application/dto/project_user_dto.rs` - DTO 추가
- ✅ `src/domain/services/user_service.rs` - 인터페이스 추가
- ✅ `src/application/use_cases/project_user_use_case.rs` - Use Case 메서드 추가
- ✅ `src/presentation/controllers/project_user_controller.rs` - 엔드포인트 추가
- ✅ `src/main.rs` - 라우팅 확인 (필요시)

## 예상 결과

- 프로젝트 멤버 관리가 더 명확하고 직관적인 API로 제공
- 기존 역할 할당/제거 API와 함께 완전한 멤버 관리 기능 제공
- 프론트엔드에서 멤버 추가/삭제 UI 구현 가능
- 멤버십 상태 확인으로 권한 체크 로직 구현 가능
