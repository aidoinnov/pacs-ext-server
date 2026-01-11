# Bounded Context 기반 모듈 구조

## 개요

이 문서는 DDD(Domain-Driven Design)의 Bounded Context 개념과 단일 책임 원칙(SRP)을 적용하여 재구성된 모듈 구조를 설명합니다.

## 아키텍처 원칙

### 1. Bounded Context 분리
- **Reporting Context**: Series User Report 작성 및 관리
- **Template Context**: Report Guide Template 관리

### 2. 단일 책임 원칙 (SRP)
각 모듈은 하나의 명확한 책임만 가집니다:
- **Entities**: 도메인 데이터 구조
- **Repositories**: 데이터 접근 인터페이스
- **Services**: 비즈니스 로직
- **DTOs**: 데이터 전송 객체
- **Use Cases**: 애플리케이션 흐름
- **Controllers**: HTTP 요청/응답 처리

### 3. 패턴화된 모듈 구조
각 Bounded Context는 동일한 구조를 따릅니다:

```
{context}/
├── entities/
│   └── mod.rs
├── repositories/
│   └── mod.rs
└── services/
    └── mod.rs
```

## 모듈 구조

### Domain Layer

```
domain/
├── reporting/              # Reporting Bounded Context
│   ├── entities/
│   │   ├── series_user_report.rs
│   │   └── mod.rs
│   ├── repositories/
│   │   ├── series_user_report_repository.rs
│   │   └── mod.rs
│   ├── services/
│   │   ├── series_user_report_service.rs
│   │   └── mod.rs
│   └── mod.rs
├── template/               # Template Bounded Context
│   ├── entities/
│   │   ├── report_guide_template.rs
│   │   └── mod.rs
│   ├── repositories/
│   │   ├── report_guide_template_repository.rs
│   │   └── mod.rs
│   ├── services/
│   │   ├── report_guide_template_service.rs
│   │   └── mod.rs
│   └── mod.rs
├── entities/              # 공통 엔티티
├── repositories/          # 공통 Repository 인터페이스
└── services/             # 공통 Service
```

### Application Layer

```
application/
├── reporting/             # Reporting Bounded Context
│   ├── dto/
│   │   ├── series_user_report_dto.rs
│   │   └── mod.rs
│   ├── use_cases/
│   │   ├── series_user_report_use_case.rs
│   │   └── mod.rs
│   └── mod.rs
├── template/              # Template Bounded Context
│   ├── dto/
│   │   ├── report_guide_template_dto.rs
│   │   └── mod.rs
│   ├── use_cases/
│   │   ├── report_guide_template_use_case.rs
│   │   └── mod.rs
│   └── mod.rs
├── dto/                   # 공통 DTO
├── services/              # 공통 Application Service
└── use_cases/            # 공통 Use Case
```

### Infrastructure Layer

```
infrastructure/
├── reporting/             # Reporting Bounded Context
│   └── repositories/
│       ├── series_user_report_repository_impl.rs
│       └── mod.rs
├── template/              # Template Bounded Context
│   └── repositories/
│       ├── report_guide_template_repository_impl.rs
│       └── mod.rs
└── repositories/          # 공통 Repository 구현체
```

### Presentation Layer

```
presentation/
├── reporting/             # Reporting Bounded Context
│   └── controllers/
│       ├── series_user_report_controller.rs
│       └── mod.rs
├── template/              # Template Bounded Context
│   └── controllers/
│       ├── report_guide_template_controller.rs
│       └── mod.rs
└── controllers/           # 공통 Controller
```

## 모듈 간 의존성

### 의존성 규칙
1. **Domain Layer**: 다른 계층에 의존하지 않음 (순수 비즈니스 로직)
2. **Application Layer**: Domain Layer에만 의존
3. **Infrastructure Layer**: Domain Layer에만 의존 (인터페이스 구현)
4. **Presentation Layer**: Application Layer에 의존

### 컨텍스트 간 통신
- **Reporting Context** ↔ **Template Context**: Application Layer를 통한 Use Case 조합
- 공통 도메인(Project, User 등)은 `domain::repositories`를 통해 공유

## 패턴화된 모듈 구조의 장점

### 1. 명확한 책임 분리
- 각 컨텍스트가 독립적으로 관리됨
- 모듈 간 결합도 최소화

### 2. 확장성
- 새로운 컨텍스트 추가 시 동일한 패턴 적용
- 기존 코드에 영향 최소화

### 3. 유지보수성
- 컨텍스트별로 독립적인 수정 가능
- 테스트 및 디버깅 용이

### 4. 재사용성
- 공통 인터페이스는 `domain::repositories`에 정의
- 컨텍스트별 구현은 각 컨텍스트 내부에 위치

## 사용 예시

### Reporting Context 사용
```rust
use crate::domain::reporting::entities::SeriesUserReport;
use crate::domain::reporting::repositories::SeriesUserReportRepository;
use crate::domain::reporting::services::SeriesUserReportService;
use crate::application::reporting::use_cases::SeriesUserReportUseCase;
use crate::infrastructure::reporting::repositories::SeriesUserReportRepositoryImpl;
```

### Template Context 사용
```rust
use crate::domain::template::entities::ReportGuideTemplate;
use crate::domain::template::repositories::ReportGuideTemplateRepository;
use crate::domain::template::services::ReportGuideTemplateService;
use crate::application::template::use_cases::ReportGuideTemplateUseCase;
use crate::infrastructure::template::repositories::ReportGuideTemplateRepositoryImpl;
```

## 향후 확장

새로운 Bounded Context 추가 시:
1. `domain/{context}/` 디렉토리 생성
2. `entities/`, `repositories/`, `services/` 모듈 생성
3. 동일한 패턴을 Application, Infrastructure, Presentation Layer에 적용
4. `main.rs`에서 컨텍스트별 초기화 및 라우팅 등록





