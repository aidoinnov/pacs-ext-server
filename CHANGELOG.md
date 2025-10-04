# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

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
