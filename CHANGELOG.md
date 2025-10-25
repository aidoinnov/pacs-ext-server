# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added - 2025-01-27

#### **Token Refresh API** 🔄
- **New API Endpoint**: 토큰 갱신 기능 구현
  - `POST /api/auth/refresh` - Refresh token을 사용한 Access token 갱신
  - Keycloak과의 완전한 통합을 통한 안전한 토큰 관리
  - 별도의 토큰 저장소 없이 Keycloak 중계 역할 수행

- **Keycloak Integration Enhancement**: Keycloak 클라이언트 확장
  - `KeycloakClient::refresh_access_token()` 메서드 구현
  - Keycloak의 `/realms/{realm}/protocol/openid-connect/token` endpoint 호출
  - `grant_type=refresh_token` 파라미터를 사용한 토큰 갱신
  - `KeycloakTokenResponse` DTO 추가 (access_token, refresh_token, expires_in 등)

- **Enhanced Auth Service**: 인증 서비스 계층 확장
  - `AuthService::refresh_token_with_keycloak()` 메서드 추가
  - KeycloakClient 의존성 주입을 통한 느슨한 결합
  - 에러 처리 및 로깅 구현

- **Use Case Layer**: 비즈니스 로직 오케스트레이션
  - `AuthUseCase::refresh_token()` 메서드 추가
  - DTO 변환 및 비즈니스 규칙 적용
  - Clean Architecture 패턴 준수

- **Controller Layer**: HTTP 요청/응답 처리
  - `AuthController::refresh_token()` 핸들러 구현
  - JSON 요청/응답 처리
  - 적절한 HTTP 상태 코드 반환 (200 OK, 401 Unauthorized)

- **OpenAPI Documentation**: API 문서화 완료
  - `refresh_token_doc()` 함수 추가
  - 요청/응답 스키마 정의
  - 에러 응답 문서화

- **Comprehensive Testing**: 포괄적인 테스트 구현
  - **단위 테스트**: 각 계층별 테스트 (5개 테스트 통과)
    - `auth_use_case_refresh_token_test.rs`: Use Case 테스트
    - `keycloak_client_refresh_token_test.rs`: KeycloakClient 테스트
    - `auth_service_refresh_token_test.rs`: AuthService 테스트
    - `auth_controller_refresh_token_test.rs`: Controller 테스트
  - **통합 테스트**: 전체 플로우 테스트
    - `refresh_token_integration_test.rs`: Mockito를 사용한 HTTP 모킹
  - **성능 테스트**: 응답 시간 측정
    - `refresh_token_performance_test.rs`: 동시 요청 처리 테스트

- **Security Features**: 보안 기능 구현
  - Keycloak의 refresh token rotation 활용
  - 토큰 만료 정책을 Keycloak에서 중앙 관리
  - HTTPS를 통한 안전한 토큰 전송
  - 민감한 정보는 로그에 기록하지 않음

- **Error Handling**: 강화된 에러 처리
  - ServiceError를 통한 일관된 에러 처리
  - HTTP 상태 코드 매핑
  - 사용자 친화적인 에러 메시지
  - Keycloak 서버 장애 시 적절한 에러 응답

- **New DTOs**: 토큰 갱신 관련 DTO 추가
  - `RefreshTokenRequest`: refresh_token 필드
  - `RefreshTokenResponse`: token, token_type, expires_in 필드
  - `KeycloakTokenResponse`: Keycloak 응답을 위한 내부 DTO

### Added - 2025-10-25

#### **User Signup and Deletion API** ✨
- **New API Endpoints**: 사용자 회원가입 및 계정 삭제 기능 구현
  - `POST /api/auth/signup` - 사용자 회원가입
  - `POST /api/auth/verify-email` - 이메일 인증
  - `POST /api/auth/admin/users/approve` - 관리자 승인
  - `DELETE /api/auth/users/{user_id}` - 계정 삭제

- **Keycloak Integration**: Keycloak과 연동한 사용자 인증 시스템
  - Keycloak Admin API 클라이언트 구현
  - 사용자 생성/삭제 자동화
  - 이메일 인증 요청 기능
  - 역할 자동 할당
  - 원자적 트랜잭션을 통한 Keycloak과 DB 동기화

- **Enhanced Database Schema**: 사용자 계정 상태 및 감사 로그 테이블 추가
  - `user_account_status_enum`: PENDING_EMAIL, PENDING_APPROVAL, ACTIVE, SUSPENDED, DELETED
  - `security_user` 테이블 확장 (계정 상태, 이메일 인증, 승인 정보)
  - `security_user_audit_log` 테이블 생성 (사용자 액션 추적)
- **S3 Object Storage Integration**: AWS S3 연동 완료
  - 파일 업로드/다운로드 URL 생성
  - 파일 메타데이터 관리
  - 파일 삭제 및 이동 기능

- **Comprehensive Audit Logging**: 포괄적인 사용자 활동 추적
  - 모든 API 호출 및 상태 변경 기록
  - IP 주소 및 User-Agent 추적
  - JSON 형태의 상세 정보 저장
  - 사용자 삭제 후에도 영구 보관되는 감사 로그
  - 상세한 메타데이터 수집 (IP, User-Agent, 요청 데이터 등)

- **Clean Architecture Implementation**: 4계층 아키텍처 패턴 적용
  - Domain Layer: 엔티티, 서비스 트레이트, 비즈니스 규칙
  - Application Layer: Use Case, DTO, 비즈니스 로직 오케스트레이션
  - Infrastructure Layer: 데이터베이스, 외부 서비스 연동
  - Presentation Layer: API 컨트롤러, HTTP 핸들러

- **New DTOs**: 사용자 등록 관련 DTO 추가
  - `SignupRequest/Response`: 회원가입 요청/응답
  - `VerifyEmailRequest/Response`: 이메일 인증 요청/응답
  - `ApproveUserRequest/Response`: 사용자 승인 요청/응답
  - `DeleteAccountResponse`: 계정 삭제 응답

- **Testing**: 포괄적인 테스트 구현
  - 단위 테스트: Service, Use Case, Controller 계층
  - 통합 테스트: API 엔드포인트 및 데이터베이스 연동
  - Mock을 활용한 의존성 격리 테스트
  - Service Layer: Mock을 사용한 비즈니스 로직 테스트
  - Use Case Layer: Mock Service를 사용한 오케스트레이션 테스트
  - Controller Layer: Mock Use Case를 사용한 API 테스트
  - Integration Tests: 실제 Keycloak 서버와의 연동 테스트

### Added - 2025-01-27

#### **Project Data Access Management API**
- **New API Endpoints**: 프로젝트 참여자가 프로젝트 데이터에 대한 접근 상태를 조회하고 수정할 수 있는 API 구현
  - `GET /api/projects/{project_id}/data-access` - 데이터 접근 상태 조회 (페이지네이션, 검색, 필터링 지원)
  - `PUT /api/projects/{project_id}/data-access/{data_id}` - 데이터 접근 상태 수정
  - `GET /api/projects/{project_id}/data-access/matrix` - 데이터 접근 매트릭스 조회

- **Enhanced DTOs**: 새로운 데이터 접근 관리 DTO 추가
  - `ProjectDataAccessDto`: 데이터 접근 상태 정보
  - `ProjectDataAccessMatrixDto`: 데이터별 사용자 접근 상태 매트릭스
  - `UpdateDataAccessStatusRequest`: 접근 상태 수정 요청
  - `ProjectDataDto`: 프로젝트 데이터 메타데이터
  - `UserDto`: 사용자 정보 (간소화된 버전)

- **Database Migration**: `010_create_project_data_access.sql`
  - `data_access_status_enum`: APPROVED, DENIED, PENDING 상태 정의
  - `project_data`: 프로젝트 데이터 메타데이터 테이블 (DICOM Study 정보)
  - `project_data_access`: 사용자별 데이터 접근 상태 테이블
  - 성능 최적화를 위한 인덱스 및 트리거 설정

- **Service Layer Extensions**: ProjectDataService에 데이터 접근 관리 기능 추가
  - `get_project_data_access()`: 데이터 접근 상태 조회 (페이지네이션, 검색, 필터링)
  - `update_data_access_status()`: 데이터 접근 상태 수정
  - `get_data_access_matrix()`: 데이터 접근 매트릭스 조회
  - `create_project_data()`: 프로젝트 데이터 생성
  - `get_project_data_by_id()`: 프로젝트 데이터 조회

- **Use Case Layer**: `ProjectDataAccessUseCase` 구현
  - 데이터 접근 관리 비즈니스 로직 오케스트레이션
  - 페이지네이션 및 검색 로직 처리
  - 에러 처리 및 검증

- **Controller Layer**: `project_data_access_controller.rs` 구현
  - 3개 엔드포인트 구현
  - OpenAPI 문서화 완료
  - 에러 처리 및 응답 변환

- **OpenAPI Documentation**: 완전한 API 문서화
  - Swagger UI에서 테스트 가능
  - "project-data-access" 태그로 그룹화
  - 모든 DTO 스키마 문서화

- **Testing**: 완전한 테스트 커버리지
  - 단위 테스트: 70개 테스트 통과
  - 통합 테스트: API 엔드포인트 테스트
  - Mock 테스트 및 실제 데이터베이스 연동 테스트

### Added - 2024-12-19

#### **Role-Permission Matrix API**
- **New API Endpoints**: 역할-권한 매트릭스를 표 형태로 조회하고 개별 권한을 ON/OFF할 수 있는 API 구현
  - `GET /api/roles/global/permissions/matrix` - 글로벌 역할-권한 매트릭스 조회
  - `GET /api/projects/{project_id}/roles/permissions/matrix` - 프로젝트별 역할-권한 매트릭스 조회
  - `PUT /api/roles/{role_id}/permissions/{permission_id}` - 글로벌 역할에 권한 할당/제거
  - `PUT /api/projects/{project_id}/roles/{role_id}/permissions/{permission_id}` - 프로젝트별 역할에 권한 할당/제거

- **Enhanced DTOs**: 새로운 매트릭스 DTO 추가
  - `RolePermissionMatrixResponse`: 매트릭스 조회 응답
  - `RoleInfo`: 역할 정보 (id, name, description, scope)
  - `PermissionInfo`: 권한 정보 (id, resource_type, action)
  - `RolePermissionAssignment`: 역할-권한 할당 정보 (role_id, permission_id, assigned)
  - `AssignPermissionRequest`: 권한 할당/제거 요청 (assign: bool)
  - `AssignPermissionResponse`: 권한 할당/제거 응답 (success, message)

- **Database Migration**: `009_add_permission_category.sql`
  - 기존 `resource_type` 필드를 카테고리로 활용하는 주석 추가
  - 새로운 컬럼 추가 없이 기존 구조 활용

- **Service Layer Extensions**: PermissionService에 매트릭스 기능 추가
  - `get_global_role_permission_matrix()`: 글로벌 역할-권한 매트릭스 조회
  - `get_project_role_permission_matrix(project_id)`: 프로젝트별 역할-권한 매트릭스 조회
  - 기존 권한 할당/제거 메서드 활용

- **Use Case Layer**: `RolePermissionMatrixUseCase` 구현
  - 매트릭스 데이터 조회 및 변환
  - 권한 할당/제거 오케스트레이션
  - 에러 처리 및 검증

- **Controller Layer**: `role_permission_matrix_controller.rs` 구현
  - 4개 엔드포인트 구현
  - OpenAPI 문서화 완료
  - 에러 처리 및 응답 변환

- **OpenAPI Documentation**: 완전한 API 문서화
  - Swagger UI에서 테스트 가능
  - "role-permission-matrix" 태그로 그룹화
  - 모든 DTO 스키마 문서화

- **Testing**: 완전한 테스트 커버리지
  - 단위 테스트: 6개 테스트 (Use Case + DTO)
  - 통합 테스트: 6개 테스트 (API 엔드포인트)
  - Mock 테스트 및 실제 데이터베이스 연동 테스트

### Added - 2025-01-23

#### **Global Roles with Permissions API**
- **New API Endpoint**: `GET /api/roles/global/with-permissions` - 글로벌 역할 목록을 권한 정보와 함께 페이지네이션으로 조회
  - 페이지네이션 지원: `page` (기본값: 1), `page_size` (기본값: 20, 최대: 100)
  - 각 역할에 할당된 권한 정보를 포함하여 반환
  - 하위 호환성 보장: 기존 `/api/roles/global` API 유지

- **Enhanced DTOs**: 새로운 응답 DTO 추가
  - `RoleWithPermissionsResponse`: 역할 정보 + 권한 목록
  - `RolesWithPermissionsListResponse`: 페이지네이션 정보 포함
  - `PaginationQuery`: 페이지네이션 쿼리 파라미터

- **OpenAPI Documentation**: 완전한 API 문서화
  - Swagger UI에서 테스트 가능
  - 상세한 파라미터 및 응답 스키마 문서화

#### **User Profile Management Enhancement**
- **Extended User Profile Fields**: Added comprehensive user profile management with additional fields
  - `full_name` - 사용자 실명 (한글명/영문명)
  - `organization` - 소속 기관 (예: "서울대학교병원")
  - `department` - 소속 부서/그룹 (예: "영상의학과")
  - `phone` - 연락처 (예: "010-1234-5678")
  - `updated_at` - 마지막 업데이트 시각

- **User Update API**: Implemented `PUT /api/users/{user_id}` endpoint for updating user profile information
  - Partial update support - 사용자는 개별 필드만 업데이트 가능
  - Email uniqueness validation - 이메일 중복 검사
  - Username과 keycloak_id는 변경 불가 (시스템 식별자)

- **Database Schema Enhancement**
  - Added migration `006_add_user_profile_fields.sql`
  - Automatic `updated_at` trigger for timestamp management
  - Performance indexes for name and organization search
  - Proper column documentation and constraints

- **Enhanced DTOs and Entities**
  - Updated `CreateUserRequest`, `UpdateUserRequest`, `UserResponse` with new profile fields
  - Created `UpdateUser` entity with builder pattern for flexible updates
  - OpenAPI documentation with comprehensive examples

#### **Configuration Management Improvements**
- **Environment Variable Priority**: Fixed configuration loading to ensure environment variables take precedence over TOML files
- **S3 Configuration Fix**: Resolved S3 signed URL generation error by removing hardcoded credentials from config files
- **Cleaned Configuration Files**: Removed duplicate and commented-out environment variable definitions

### Fixed - 2025-01-23

#### **Critical Bug Fixes**
- **S3 Signed URL Generation**: Fixed "액세스키가 없다" (Access key is missing) error
  - Root cause: TOML config files contained hardcoded S3 credentials overriding environment variables
  - Solution: Removed all hardcoded sensitive values from config files
  - Result: S3 signed URL generation now works correctly with proper credential loading

- **Database Query Fixes**: Fixed annotation DELETE API error
  - Resolved "Database error: no column found for name: measurement_values"
  - Added missing `measurement_values` column to SQL queries
  - Fixed `find_shared_annotations` query with proper column references

#### **Configuration Cleanup**
- **Environment Variable Loading**: Fixed duplicate keys in `.env` file causing environment variables to not load properly
- **Config File Hardcoded Values**: Removed hardcoded S3 credentials from TOML config files
- **Configuration Priority**: Ensured proper environment variable priority over TOML file values

### Technical Details - 2025-01-23

#### **Database Migration**
```sql
-- 006_add_user_profile_fields.sql
ALTER TABLE security_user
ADD COLUMN full_name TEXT,
ADD COLUMN organization TEXT,
ADD COLUMN department TEXT,
ADD COLUMN phone TEXT,
ADD COLUMN updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- Performance indexes
CREATE INDEX idx_user_full_name ON security_user(full_name);
CREATE INDEX idx_user_organization ON security_user(organization);

-- Auto-update trigger
CREATE OR REPLACE FUNCTION update_user_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

#### **API Usage Examples**
```bash
# Update user profile
PUT /api/users/123
Content-Type: application/json

{
  "full_name": "홍길동",
  "email": "hong@example.com",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

### Added - 2025-10-05

#### Presentation Layer - HTTP Controllers (25 통합 테스트)
- **AuthController** (`/auth`) - 인증 API (3개 테스트)
  - `POST /auth/login` - Keycloak ID 기반 로그인
  - `POST /auth/verify` - JWT 토큰 검증
  - 자동 사용자 생성, JWT 토큰 발급

- **UserController** (`/users`) - 사용자 관리 API (5개 테스트)
  - `POST /users` - 사용자 생성
  - `GET /users/{id}` - ID로 사용자 조회
  - `GET /users/username/{username}` - Username으로 조회
  - 중복 검증, 이메일 유효성 검사

- **ProjectController** (`/projects`) - 프로젝트 관리 API (5개 테스트)
  - `POST /projects` - 프로젝트 생성
  - `GET /projects/{id}` - 프로젝트 조회
  - `GET /projects` - 전체 프로젝트 목록
  - `GET /projects/active` - 활성 프로젝트만 조회
  - 프로젝트명 중복 검증

- **PermissionController** (`/roles`) - 역할 관리 API (5개 테스트)
  - `POST /roles` - 역할 생성 (Global/Project scope)
  - `GET /roles/{id}` - 역할 조회
  - `GET /roles/global` - 글로벌 역할 목록
  - `GET /roles/project` - 프로젝트 역할 목록
  - Scope별 역할 조회

- **AccessControlController** (`/access-control`) - 접근 제어 API (7개 테스트)
  - `POST /access-control/logs` - DICOM 접근 로그 기록
  - `GET /access-control/logs/user/{user_id}` - 사용자별 접근 로그
  - `GET /access-control/logs/project/{project_id}` - 프로젝트별 접근 로그
  - `GET /access-control/logs/study/{study_uid}` - Study별 접근 로그
  - `POST /access-control/permissions/check` - 권한 확인
  - `GET /access-control/permissions/user/{user_id}/project/{project_id}` - 사용자 권한 목록
  - `GET /access-control/access/user/{user_id}/project/{project_id}` - 프로젝트 접근 가능 여부

#### Application Layer - Use Cases
- **AuthUseCase** - 인증 유스케이스
  - `login()`, `verify_token()`, `refresh_token()`, `logout()`

- **UserUseCase** - 사용자 유스케이스
  - `create_user()`, `get_user_by_id()`, `get_user_by_username()`
  - `delete_user()`, `add_project_member()`, `remove_project_member()`
  - `get_user_projects()`, `is_project_member()`

- **ProjectUseCase** - 프로젝트 유스케이스
  - `create_project()`, `get_project()`, `get_all_projects()`, `get_active_projects()`
  - `activate_project()`, `deactivate_project()`, `delete_project()`
  - `get_project_members()`, `assign_role()`, `remove_role()`, `get_project_roles()`

- **PermissionUseCase** - 권한 유스케이스
  - `create_role()`, `get_role()`, `get_global_roles()`, `get_project_roles()`
  - `assign_permission_to_role()`, `remove_permission_from_role()`, `get_role_permissions()`
  - `assign_permission_to_project()`, `remove_permission_from_project()`, `get_project_permissions()`
  - `get_permissions_for_resource()`

- **AccessControlUseCase** - 접근 제어 유스케이스
  - `log_dicom_access()`, `get_user_access_logs()`, `get_project_access_logs()`, `get_study_access_logs()`
  - `check_permission()`, `get_user_permissions()`, `can_access_project()`

#### Application Layer - DTOs
- **auth_dto.rs** - 인증 DTO
  - LoginRequest, LoginResponse, VerifyTokenResponse
  - RefreshTokenRequest, RefreshTokenResponse

- **user_dto.rs** - 사용자 DTO
  - CreateUserRequest, UpdateUserRequest, UserResponse, UserListResponse
  - AddProjectMemberRequest, UserProjectsResponse, ProjectSummary

- **project_dto.rs** - 프로젝트 DTO
  - CreateProjectRequest, UpdateProjectRequest, ProjectResponse, ProjectListResponse
  - ProjectAssignRoleRequest, ProjectMembersResponse, MemberInfo
  - ProjectRolesResponse, RoleInfo

- **permission_dto.rs** - 권한 DTO
  - CreateRoleRequest, RoleResponse, PermissionResponse
  - AssignPermissionRequest, RolePermissionsResponse
  - ProjectPermissionsResponse, ResourcePermissionsResponse

- **access_control_dto.rs** - 접근 제어 DTO
  - LogDicomAccessRequest, AccessLogResponse, AccessLogListResponse
  - CheckPermissionRequest, CheckPermissionResponse
  - UserPermissionsResponse, PermissionInfo, ProjectAccessResponse

#### Infrastructure - HTTP Server Setup
- **main.rs** - Actix-web HTTP 서버 구성
  - PostgreSQL 연결 풀 설정 (최대 5개 연결)
  - Repository → Service → UseCase 의존성 주입
  - 5개 컨트롤러 라우팅 설정
  - JWT 서비스 통합
  - Health check 엔드포인트 (`GET /health`)

#### Dependencies
- actix-web 4.9 - HTTP 웹 프레임워크
- actix-rt 2.10 - Actix 런타임
- actix-http 3 - HTTP 타입 (dev-dependencies)

#### Repository Improvements
- **Clone 트레잇 구현** - Repository 재사용성 개선
  - UserRepositoryImpl, ProjectRepositoryImpl
  - RoleRepositoryImpl, PermissionRepositoryImpl
  - Service layer에서 Repository 복제 가능

### Changed - 2025-10-05

#### Database Transaction Improvements
- **Race Condition 제거** - 모든 동시성 위험 패턴 수정
  - `UserService.add_user_to_project` - INSERT ... ON CONFLICT 패턴 적용
  - `ProjectService.assign_role_to_project` - INSERT ... ON CONFLICT 패턴 적용
  - `PermissionService.assign_permission_to_role` - INSERT ... ON CONFLICT 패턴 적용
  - `PermissionService.assign_permission_to_project` - INSERT ... ON CONFLICT 패턴 적용
  - `AuthService.login` - UPSERT (ON CONFLICT DO UPDATE) 패턴으로 개선
  - 동시 요청 시 하나만 성공하고 나머지는 적절한 에러 반환

- **원자적 UPDATE 처리** - UPDATE 후 SELECT 제거
  - `ProjectService.activate_project` - UPDATE ... RETURNING 절 사용
  - `ProjectService.deactivate_project` - UPDATE ... RETURNING 절 사용
  - UPDATE와 SELECT 사이 데이터 변경 가능성 제거

- **권한 검증 쿼리 최적화** - 2개 쿼리 → 1개 쿼리
  - `AccessControlService.check_permission` - CTE + UNION ALL로 통합
  - 역할 기반 권한 + 프로젝트 직접 권한을 단일 쿼리로 처리
  - 성능 향상: DB 왕복 50% 감소 (2회 → 1회)
  - 일관성 보장: 단일 스냅샷 내에서 권한 확인

#### Performance & Consistency
- **트랜잭션 안정성 향상**: Race Condition 5건 제거
- **쿼리 최적화**: 불필요한 DB 왕복 제거
- **원자성 보장**: UPDATE-SELECT 패턴 개선
- **동시성 안정성**: INSERT ... ON CONFLICT로 중복 방지

#### Documentation
- `TRANSACTION_OPTIMIZATION.md` - 1차 트랜잭션 검토 보고서
- `TRANSACTION_REVIEW_FINAL.md` - 2차 트랜잭션 검토 및 추가 개선사항
- 총 12개 이슈 식별 및 문서화 (심각 5건, 중요 3건, 권장 4건)
- 배치 작업, Repository 개선, 성능 최적화 가이드 포함

### Added - 2025-10-04

#### Service Layer Extensions (Domain Services - Phase 2)
- **UserService 확장** - 프로젝트 멤버십 관리 (4개 메서드)
  - `add_user_to_project()` - 프로젝트에 사용자 추가 (중복 검증)
  - `remove_user_from_project()` - 프로젝트에서 사용자 제거
  - `get_user_projects()` - 사용자가 속한 프로젝트 목록 조회
  - `is_project_member()` - 프로젝트 멤버십 확인
  - `security_user_project` 테이블 연동

- **ProjectService 확장** - 역할 및 멤버 관리 (5개 메서드)
  - `get_project_members()` - 프로젝트 멤버 목록 조회 (JOIN)
  - `count_project_members()` - 프로젝트 멤버 수 조회
  - `assign_role_to_project()` - 프로젝트에 역할 할당 (중복 검증)
  - `remove_role_from_project()` - 프로젝트에서 역할 제거
  - `get_project_roles()` - 프로젝트에 할당된 역할 목록 조회
  - `security_project_role` 테이블 연동

- **PermissionService 확장** - 권한 할당 관리 (6개 메서드)
  - `assign_permission_to_role()` - 역할에 권한 할당 (중복 검증)
  - `remove_permission_from_role()` - 역할에서 권한 제거
  - `get_role_permissions()` - 역할이 가진 권한 목록 조회
  - `assign_permission_to_project()` - 프로젝트에 권한 직접 할당
  - `remove_permission_from_project()` - 프로젝트에서 권한 제거
  - `get_project_permissions()` - 프로젝트에 할당된 권한 목록 조회
  - `security_role_permission`, `security_project_permission` 테이블 연동

- **AuthService 신규 생성** - JWT 통합 인증 서비스 (4개 메서드)
  - `login()` - Keycloak ID 기반 로그인 (자동 사용자 생성)
  - `verify_and_get_user()` - JWT 토큰 검증 및 사용자 조회
  - `refresh_token()` - 토큰 갱신 (24시간 유효기간)
  - `logout()` - 로그아웃 처리
  - JwtService 통합, Claims 기반 사용자 정보 관리
  - AuthResponse DTO 추가 (user + token)

- **AccessControlService 확장** - 실제 권한 검증 시스템 (3개 메서드)
  - `check_permission()` - 사용자의 특정 권한 보유 여부 확인
    - 역할을 통한 권한 (User → Project → Role → Permission)
    - 프로젝트 직접 권한 (User → Project → Permission)
  - `get_user_permissions()` - 사용자가 프로젝트에서 가진 모든 권한 조회
    - 역할 권한 + 프로젝트 직접 권한 통합 조회 (UNION)
  - `is_project_member()` - 프로젝트 멤버십 확인
  - `can_access_project()` - 실제 멤버십 확인으로 업데이트

#### Repository Extensions
- **모든 Repository에 `pool()` 메서드 추가**
  - `UserRepository::pool()` - PgPool 접근
  - `ProjectRepository::pool()` - PgPool 접근
  - `RoleRepository::pool()` - PgPool 접근
  - `PermissionRepository::pool()` - PgPool 접근
  - Service에서 직접 SQL 쿼리 실행 가능 (관계 테이블 연동)

#### Authentication & JWT
- JWT 환경변수 설정 (`JWT_SECRET`, `JWT_EXPIRATION_HOURS`)
- JwtConfig 계층적 설정 (ENV > .env > TOML)
- Claims 구조체: user_id, keycloak_id, username, email, exp
- JwtService: 토큰 생성, 검증, Bearer 추출
- AuthMiddleware: HTTP 요청 인증 미들웨어

### Added - 2025-10-04 (Phase 1)

#### Configuration System
- 환경변수 우선 설정 시스템 구축
  - `infrastructure/config/settings.rs` - 계층적 설정 로더
  - 우선순위: 환경변수 (APP_) > .env > config/{env}.toml > default.toml
  - `DATABASE_URL` 직접 지정 지원
  - `config/default.toml` - 기본 설정
  - `config/development.toml` - 개발 환경
  - `config/production.toml` - 프로덕션 환경
  - `.env.example` - 환경변수 예시

#### Repository Layer (Clean Architecture)
- Domain Layer: Repository Traits (인터페이스)
  - `UserRepository` - 사용자 레포지토리 인터페이스
  - `ProjectRepository` - 프로젝트 레포지토리 인터페이스
  - `RoleRepository` - 역할 레포지토리 인터페이스
  - `PermissionRepository` - 권한 레포지토리 인터페이스
  - `AccessLogRepository` - 접근 로그 레포지토리 인터페이스

- Infrastructure Layer: Repository Implementations
  - `UserRepositoryImpl` - PostgreSQL 구현
  - `ProjectRepositoryImpl` - PostgreSQL 구현
  - `RoleRepositoryImpl` - PostgreSQL 구현
  - `PermissionRepositoryImpl` - PostgreSQL 구현
  - `AccessLogRepositoryImpl` - PostgreSQL 구현

#### Service Layer (Domain Services)
- Domain Layer: Service Traits (비즈니스 로직 인터페이스)
  - `UserService` - 사용자 관리 서비스
    - 사용자 생성 (중복 체크, 이메일 검증)
    - 사용자 조회 (ID, Keycloak ID, Username)
    - 사용자 삭제 및 존재 여부 확인
  - `ProjectService` - 프로젝트 관리 서비스
    - 프로젝트 생성 (이름 중복 체크, 길이 검증)
    - 프로젝트 조회 (ID, 이름, 전체, 활성)
    - 프로젝트 활성화/비활성화, 삭제
  - `PermissionService` - 권한 관리 서비스
    - 역할 생성 (Global/Project scope)
    - 역할 조회 (ID, scope별)
    - 권한 존재 여부 검증
  - `AccessControlService` - 접근 제어 서비스
    - DICOM 리소스 접근 로그 기록
    - 사용자/프로젝트/Study별 로그 조회
    - 프로젝트 접근 권한 확인

- Service Layer: Service Implementations
  - `UserServiceImpl` - 사용자 서비스 구현체
  - `ProjectServiceImpl` - 프로젝트 서비스 구현체
  - `PermissionServiceImpl` - 권한 서비스 구현체
  - `AccessControlServiceImpl` - 접근 제어 서비스 구현체
  - `ServiceError` - 통합 에러 타입 (NotFound, AlreadyExists, ValidationError 등)

#### Testing
- 엔티티 단위 테스트 (22개)
  - User, Project, Role, Permission, AccessCondition 테스트
  - Relations, Logs, Viewer, Annotation 테스트
  - Enum 타입 매핑 테스트
  - JSON 직렬화/역직렬화 테스트

- 레포지토리 통합 테스트 (16개)
  - UserRepository: CRUD 및 검색 기능 테스트
  - ProjectRepository: CRUD, 활성화 상태 관리 테스트
  - RoleRepository: CRUD, scope별 조회 테스트
  - PermissionRepository: CRUD, 리소스별 조회 테스트
  - AccessLogRepository: 로그 생성, 조회, 카운트 테스트
  - PostgreSQL 실제 DB 연동 테스트
  - 외래키 제약 고려한 cleanup 로직

- 서비스 통합 테스트 (34개)
  - UserService: 사용자 생성, 중복 검증, 조회, 삭제 테스트 (8개)
  - ProjectService: 프로젝트 생성, 검증, 조회, 활성화 관리 테스트 (10개)
  - PermissionService: 역할 생성, 검증, scope별 조회 테스트 (8개)
  - AccessControlService: DICOM 로그 기록, 조회, 접근 권한 테스트 (8개)
  - 비즈니스 로직 검증 (중복 체크, 유효성 검사)
  - 에러 처리 및 서비스 간 통합 테스트

#### Infrastructure
- PostgreSQL 데이터베이스 스키마 설계 및 DDL 생성
  - Security Schema: 사용자, 프로젝트, 역할, 권한 관리
  - Group Extension: 프로젝트 내 그룹 기능
  - Viewer Schema: Hanging Protocol 관리
  - Annotation Schema: DICOM 주석 관리
  - 3개 ENUM 타입: `condition_type_enum`, `resource_level_enum`, `grant_action_enum`
  - 22개 테이블 생성
  - 35개 인덱스 최적화
  - `infra/db/schema.sql` - 전체 DDL
  - `infra/db/diagram` - Mermaid ER 다이어그램

#### Rust PACS Server
- 클린 아키텍처 기반 프로젝트 구조 생성
  - Domain Layer: 엔티티, 레포지토리 인터페이스, 도메인 서비스
  - Application Layer: 유스케이스, DTO
  - Infrastructure Layer: DB, 레포지토리 구현, 외부 서비스 연동
  - Presentation Layer: 컨트롤러, 미들웨어, 라우트

- ORM 매핑 완료 (sqlx 사용)
  - Security 엔티티: User, Project, Role, Permission, AccessCondition, Group
  - 관계 엔티티: UserProject, ProjectRole, RolePermission, ProjectPermission 등
  - 로그 엔티티: GrantLog, AccessLog
  - Viewer 엔티티: HangingProtocol, HpCondition, HpLayout, HpViewport
  - Annotation 엔티티: Annotation, AnnotationHistory
  - PostgreSQL ENUM 타입 매핑: ConditionType, ResourceLevel, GrantAction, RoleScope

#### Dependencies
- sqlx 0.7 (PostgreSQL, UUID, Chrono, JSON 지원)
- tokio 1.x (비동기 런타임)
- chrono 0.4 (날짜/시간 처리)
- uuid 1.x (UUID 타입)
- serde + serde_json (직렬화/역직렬화)
- async-trait 0.1 (비동기 trait 지원)
- config 0.14 (계층적 설정 관리)
- dotenvy 0.15 (.env 파일 지원)
- tokio-test 0.4 (비동기 테스트 지원)

#### Documentation
- `CLAUDE.md` - 프로젝트 개요 및 개발 가이드 (한글)
- `pacs-server/README.md` - Rust 서버 클린 아키텍처 설명

### Changed
- **Service Layer 확장** (Phase 2)
  - UserService: ProjectRepository 의존성 추가
  - ProjectService: UserRepository, RoleRepository 의존성 추가
  - PermissionService: 권한 할당 비즈니스 로직 추가
  - AccessControlService: RoleRepository, PermissionRepository 의존성 추가
  - 모든 Repository trait에 `pool()` 메서드 추가

- **Database Schema** (Phase 1)
  - `security_user_project_role` 테이블을 `security_user_project`와 `security_project_role`로 분리
  - 모든 테이블에 `created_at` 타임스탬프 추가
  - `security_role`에 `scope` 필드 추가 (GLOBAL/PROJECT)
  - `security_access_condition`에 `resource_level` 필드 추가
  - `security_project_permission`에 `inherits_from_role_permission` 플래그 추가

### Technical Details

#### Database Schema Design
- **보안 모델**: User → Project 멤버십 분리, Project → Role 매핑 분리
- **권한 시스템**: Role-based + Project-based 권한 관리
- **접근 제어**: DICOM 태그 기반 세밀한 접근 조건
- **감사 로그**: 권한 부여 이력, 리소스 접근 로그
- **그룹 기능**: 프로젝트 내 사용자 그룹화 및 역할 부여

#### Clean Architecture Layers
```
Domain (비즈니스 로직)
  ↑
Application (유스케이스)
  ↑
Infrastructure (DB, 외부 연동)
  ↑
Presentation (HTTP API)
```

## [0.1.0] - 2025-10-04

### Initial Setup
- 프로젝트 저장소 초기화
- Go 서버 구현 (simple-go-server)
- Rust 서버 구현 (simple-rust-server)
- 성능 벤치마크 비교 (Go vs Rust)
- Docker Compose 인프라 설정
