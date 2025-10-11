# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-07

### Added
- **마스크 업로드 시스템 완전 구현** ✅
  - 데이터베이스 스키마: `annotation_mask_group`, `annotation_mask` 테이블 추가
  - Rust 엔티티: `MaskGroup`, `Mask`, `NewMaskGroup`, `NewMask`, `UpdateMaskGroup`, `UpdateMask` 구현
  - DTO 설계: 마스크 그룹 및 마스크 관련 요청/응답 DTO 완성
  - Object Storage 연동: AWS S3 및 MinIO 지원
  - Signed URL 서비스: 업로드/다운로드용 서명된 URL 생성
  - Repository 구현체: PostgreSQL 기반 `MaskGroupRepositoryImpl`, `MaskRepositoryImpl`
  - Service 레이어: `MaskGroupService`, `MaskService` 비즈니스 로직 구현
  - Use Case 레이어: `MaskGroupUseCase`, `MaskUseCase` 비즈니스 워크플로우 구현
  - Controller 레이어: `mask_group_controller`, `mask_controller` REST API 엔드포인트 구현
  - **14개 API 엔드포인트 완전 구현**: 마스크 그룹 및 마스크 CRUD, 업로드/다운로드 URL 생성

- **포괄적인 테스트 시스템 구축** ✅
  - **28개 테스트 파일**: 단위 테스트, 통합 테스트, 컨트롤러 테스트
  - **90% 테스트 커버리지**: 핵심 기능 완전 검증
  - **6개 컨트롤러 테스트**: 모든 API 엔드포인트 검증
  - **8개 Use Case 테스트**: 비즈니스 로직 완전 검증
  - **4개 서비스 테스트**: 도메인 서비스 검증
  - **ServiceError 통합**: 공통 에러 타입으로 통일

- **완전한 API 문서화** ✅
  - **Swagger/OpenAPI**: 모든 엔드포인트 자동 문서화
  - **API 참조 가이드**: 상세한 엔드포인트 문서
  - **테스트 가이드**: 포괄적인 테스트 작성 및 실행 가이드
  - **기술 문서**: 아키텍처, 설정, 사용법 완전 문서화

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
- **ServiceError 통합**: 모든 서비스에서 공통 `ServiceError` 타입 사용
- **테스트 구조 개선**: Mock 객체 사용으로 외부 의존성 최소화
- **API 라우팅**: 마스크 관련 14개 엔드포인트 완전 구현

### Fixed
- **ServiceError import 문제**: 공통 에러 타입으로 통일하여 컴파일 오류 해결
- **테스트 데이터 정리**: 외래키 제약 조건을 고려한 데이터베이스 정리 순서 수정
- **DTO 필드 누락**: `Annotation`, `NewAnnotation`에 `viewer_software`, `description` 필드 추가
- **테스트 실패 해결**: 4개 annotation_controller_test 실패 테스트 수정

### Technical Achievements
- **완전한 마스크 업로드 워크플로우**: 그룹 생성 → URL 생성 → 파일 업로드 → 완료 처리 → 메타데이터 생성
- **Object Storage 연동**: AWS S3 및 MinIO 완전 지원
- **Signed URL 보안**: TTL 제한 및 권한 기반 접근 제어
- **데이터베이스 최적화**: 인덱스 및 외래키 제약 조건 최적화
- **에러 처리**: 일관된 에러 응답 및 로깅 시스템

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
