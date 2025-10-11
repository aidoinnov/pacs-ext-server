# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **마스크 업로드 시스템 구현**
  - 데이터베이스 스키마: `annotation_mask_group`, `annotation_mask` 테이블 추가
  - Rust 엔티티: `MaskGroup`, `Mask`, `NewMaskGroup`, `NewMask`, `UpdateMaskGroup`, `UpdateMask` 구현
  - DTO 설계: 마스크 그룹 및 마스크 관련 요청/응답 DTO 완성
  - Object Storage 연동: AWS S3 및 MinIO 지원
  - Signed URL 서비스: 업로드/다운로드용 서명된 URL 생성
  - Repository 구현체: PostgreSQL 기반 `MaskGroupRepositoryImpl`, `MaskRepositoryImpl`
  - Service 레이어: `MaskGroupService`, `MaskService` 비즈니스 로직 구현
  - Use Case 레이어: `MaskGroupUseCase`, `MaskUseCase` 비즈니스 워크플로우 구현
  - Controller 레이어: `mask_group_controller`, `mask_controller` REST API 엔드포인트 구현
  - 단위 테스트: 엔티티, 서비스, 리포지토리 테스트 코드 작성

### Technical Details
- **데이터베이스 마이그레이션**
  - `003_add_mask_tables.sql`: 마스크 관련 테이블 생성
  - `004_add_updated_at_columns.sql`: `updated_at` 컬럼 추가
  - PostgreSQL `sqlx` ORM 사용

- **아키텍처**
  - Clean Architecture 패턴 준수
  - Domain, Application, Infrastructure 레이어 분리
  - 의존성 주입을 통한 느슨한 결합

- **Object Storage**
  - AWS S3 및 MinIO 지원
  - 서명된 URL을 통한 직접 업로드/다운로드
  - 설정 기반 프로바이더 선택

- **테스트 커버리지**
  - 엔티티 생성 및 검증 테스트
  - 서비스 레이어 단위 테스트 (Mock 사용)
  - 리포지토리 통합 테스트 (실제 DB 연동)

### Changed
- `src/main.rs`: 마스크 관련 리포지토리 및 서비스 초기화 추가 (Arc 래핑 적용)
- `Cargo.toml`: Object Storage 관련 의존성 추가 (`aws-sdk-s3`, `tokio-util` 등)

### Note
- Object Storage 서비스는 현재 구현되어 있으나, 실제 사용을 위해서는 환경 설정이 필요합니다
- 마스크 관련 API 엔드포인트는 Object Storage 설정 후 활성화 가능합니다

### Object Storage 설정 완료
- **설정 파일**: `config/development.toml`, `config/production.toml`에 Object Storage 설정 추가
- **환경 변수**: `APP_OBJECT_STORAGE__*` 환경 변수 지원
- **MinIO 개발 환경**: 로컬 개발을 위한 MinIO 설정 기본값 제공
- **AWS S3 프로덕션**: 프로덕션 환경을 위한 AWS S3 설정 지원
- **설정 가이드**: `docs/technical/object_storage_setup_guide.md` 상세 설정 가이드 작성

### 현재 상태
- Object Storage 서비스 초기화 코드 구현 완료
- 마스크 관련 Use Case 및 Controller 구현 완료
- 환경 설정 후 서버 시작 시 Object Storage 자동 초기화
- MinIO/AWS S3 프로바이더 자동 선택

### Dependencies
- `aws-sdk-s3`: ^1.0.0
- `tokio-util`: ^0.7.0
- `num_traits`: ^0.2.0 (BigDecimal 변환용)

## [0.1.0] - 2024-01-XX

### Added
- 기본 PACS Extension Server 구조
- 사용자, 프로젝트, 권한 관리 시스템
- 어노테이션 관리 시스템
- JWT 기반 인증
- PostgreSQL 데이터베이스 연동
- Swagger/OpenAPI 문서화
- CORS 미들웨어
