# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added - 2025-10-04

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

#### Documentation
- `CLAUDE.md` - 프로젝트 개요 및 개발 가이드 (한글)
- `pacs-server/README.md` - Rust 서버 클린 아키텍처 설명

### Changed
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
