# PACS Server Docs

이 디렉토리는 서버 코드와 함께 존재하지만, 문서의 정식 위치는 리포지토리 루트의 `docs/` 입니다.

- 문서 모음(정식): `../../docs/`
- 서버 작업 문서(예: QIDO 필터/페이지네이션): `../../docs/work/qido-filters-pagination/`
- API 레퍼런스: `../../docs/api/`

이 디렉토리에는 추후 문서가 추가되지 않도록 유지합니다(참조 전용).

# 📚 PACS Extension Server 문서

## 📋 개요

PACS Extension Server의 모든 기술 문서와 가이드를 모아놓은 중앙 문서 허브입니다.

## 🏗️ 아키텍처 문서

### 핵심 아키텍처
- [프로젝트 아키텍처 및 구조](background/00_Project_Architecture_and_Structure.md)
- [Rust 핵심 개념](background/01_Rust_Core_Concepts.md)
- [웹 프레임워크 및 API](background/02_Web_Framework_and_API.md)
- [Repository 패턴](background/03_Repository_Pattern.md)
- [인증 및 권한 관리](background/04_Authentication_and_Authorization.md)
- [공통 코드 패턴](background/05_Common_Code_Patterns.md)
- [테스트 및 DevOps](background/06_Testing_and_DevOps.md)
- [레이어별 코드 패턴](background/07_Layer-Specific_Code_Patterns.md)

### 도메인 vs 비즈니스 로직
- [도메인과 비즈니스 로직 구분](docs/misc/DOMAIN_VS_BUSINESS_LOGIC.md)
- [로직 구분 가이드](docs/misc/LOGIC_DIFFERENTIATION_GUIDE.md)

## 🔧 기술 문서

### API 및 엔드포인트
- [API 엔드포인트 참조](technical/API_ENDPOINTS_REFERENCE.md) - **NEW** ✅
- [마스크 통계 API 가이드](technical/MASK_STATISTICS_API_GUIDE.md) - **NEW** ✅
- [어노테이션 API 가이드](technical/ANNOTATION_API_GUIDE.md)
- [CORS 개발 가이드](technical/CORS_DEVELOPMENT_GUIDE.md)

### 데이터베이스
- [데이터베이스 스키마 - 마스크 업로드](technical/DATABASE_SCHEMA_MASK_UPLOAD.md)
- [데이터베이스 스키마 - 어노테이션](technical/DATABASE_SCHEMA_ANNOTATION.md)
- [데이터베이스 스키마 - 보안](technical/DATABASE_SCHEMA_SECURITY.md)

### Object Storage
- [Object Storage 연동](technical/OBJECT_STORAGE_INTEGRATION.md)
- [Object Storage 설정 가이드](technical/object_storage_setup_guide.md)

### 테스트
- [테스트 가이드](technical/TESTING_GUIDE.md) - **NEW** ✅
- [부하 테스트 가이드](technical/LOAD_TESTING_GUIDE.md) - **NEW** ✅
- [캐시 헤더 테스트](technical/CACHE_HEADERS_TEST.md)
- [캐시 정책 테스트](technical/CACHE_POLICY_TEST.md)

### 성능 및 벤치마크
- [성능 최적화 가이드](technical/PERFORMANCE_OPTIMIZATION_GUIDE.md) - **NEW** ✅
- [성능 비교 분석](docs/misc/performance-comparison.md)
- [캐시 성능 분석](benchmarks/results/CACHE_PERFORMANCE_ANALYSIS.md)
- [Redis 결정 문서](benchmarks/results/REDIS_DECISION.md)

## 📋 구현 계획 및 TODO

### 구현 계획
- [마스크 업로드 v2 구현 계획서](todo/implementation_plan.md)
- [코드 구현 가이드](todo/code_implementation_guide.md)
- [구현 체크리스트](todo/implementation_checklist.md)

### 테스트 TODO
- [남은 통합테스트 TODO](todo/remaining_tests_todo.md) - **NEW** ✅
- [남은 통합테스트 상세 문서](todo/remaining_integration_tests.md) - **NEW** ✅

### 마스크 업로드 TODO
- [마스크 업로드 TODO](MASK_UPLOAD_TODO.md)

## 🚀 학습 경로

### Phase 1: Rust 기초
- [소유권과 생명주기](learning_path/phase_1_rust_fundamentals/01_Ownership_and_Lifetimes.md)
- [Result와 Option](learning_path/phase_1_rust_fundamentals/02_Result_and_Option.md)
- [Async Await와 Futures](learning_path/phase_1_rust_fundamentals/03_Async_Await_and_Futures.md)
- [Traits와 Dyn Trait](learning_path/phase_1_rust_fundamentals/04_Traits_and_Dyn_Trait.md)

### Phase 2: 프로젝트 아키텍처
- [사용자 생성 플로우 추적](learning_path/phase_2_project_architecture/01_Tracing_the_Create_User_Flow.md)
- [main.rs의 의존성 주입](learning_path/phase_2_project_architecture/02_Dependency_Injection_in_main_rs.md)

### Phase 3: 핵심 라이브러리
- [Actix Web 필수 사항](learning_path/phase_3_core_libraries/01_Actix_web_Essentials.md)
- [PostgreSQL용 SQLx](learning_path/phase_3_core_libraries/02_SQLx_for_PostgreSQL.md)

### Phase 4: 실전 코딩
- [로컬 환경 설정](learning_path/phase_4_practical_coding/01_Setting_Up_Local_Environment.md)
- [헬스체크 API 추가하기](learning_path/phase_4_practical_coding/02_Walkthrough_Adding_a_Health_Check_API.md)

## 🔧 설정 및 배포

### 환경 설정
- [Notion 설정 가이드](notion-setup-guide.md)
- [GitHub MCP 설정 가이드](docs/misc/github-mcp-setup-guide.md)

### Docker 및 인프라
- [Docker Compose 설정](infra/docker-compose.yml)
- [데이터베이스 스키마](infra/db/schema.sql)

## 📊 벤치마크 및 성능

### 벤치마크 결과
- [캐시 성능 분석](benchmarks/results/CACHE_PERFORMANCE_ANALYSIS.md)
- [실행 요약](benchmarks/results/EXECUTIVE_SUMMARY.md)
- [Redis 결정 문서](benchmarks/results/REDIS_DECISION.md)

### 성능 테스트
- [빠른 캐시 테스트](benchmarks/quick_cache_test.sh)
- [캐시 벤치마크](benchmarks/cache_benchmark.sh)

## 🧪 테스트 문서

### 테스트 가이드
- [테스트 가이드](technical/TESTING_GUIDE.md) - **NEW** ✅
- [빠른 시작 가이드](benchmarks/QUICK_START.md)

### 테스트 결과
- [PACS 서버 클린 아키텍처 벤치마크](benchmarks/2025-10-05-pacs-server-clean-architecture.md)

## 📝 변경 이력

- [CHANGELOG.md](../CHANGELOG.md) - 프로젝트 변경 이력
- [v0.2.0 릴리스 노트](technical/CHANGELOG.md) - 마스크 업로드 시스템 완성

## 🎯 현재 상태 (v0.2.0)

### ✅ 완료된 기능
- **마스크 업로드 시스템**: 14개 API 엔드포인트 완전 구현
- **테스트 시스템**: 28개 테스트 파일, 90% 커버리지
- **API 문서화**: Swagger/OpenAPI 완전 문서화
- **Object Storage**: AWS S3 및 MinIO 완전 지원

### 🚧 진행 중인 작업
- **통합테스트 완성**: 2개 컨트롤러 테스트 추가 예정
- **성능 최적화**: 대용량 파일 업로드 최적화
- **모니터링**: 헬스체크 및 메트릭 수집

### 📋 다음 단계
- [남은 통합테스트 TODO](todo/remaining_tests_todo.md) 참조
- 우선순위 1: 핵심 통합테스트 4개 작업
- 우선순위 2: 고급 통합테스트 4개 작업

---

**최종 업데이트**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 2.0