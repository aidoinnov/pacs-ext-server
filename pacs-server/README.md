# PACS Extension Server

Rust 기반 PACS 확장 서버 - 클린 아키텍처

## 프로젝트 구조

```
src/
├── domain/              # 도메인 계층 (비즈니스 로직)
│   ├── entities/        # 도메인 엔티티
│   ├── repositories/    # 레포지토리 인터페이스 (trait)
│   └── services/        # 도메인 서비스
│
├── application/         # 애플리케이션 계층 (유스케이스)
│   ├── use_cases/       # 유스케이스 구현
│   └── dto/             # 데이터 전송 객체
│
├── infrastructure/      # 인프라 계층 (외부 의존성)
│   ├── database/        # DB 연결 및 설정
│   ├── repositories/    # 레포지토리 구현
│   ├── external/        # 외부 서비스 (Keycloak 등)
│   └── config/          # 설정 및 환경변수
│
└── presentation/        # 프레젠테이션 계층 (HTTP)
    ├── controllers/     # HTTP 컨트롤러
    ├── middleware/      # 미들웨어 (인증, 로깅 등)
    └── routes/          # 라우트 정의
```

## 클린 아키텍처 원칙

1. **의존성 규칙**: 외부 계층 → 내부 계층 (domain은 어떤 계층도 의존하지 않음)
2. **도메인 중심**: 비즈니스 로직은 domain 계층에 집중
3. **인터페이스 분리**: domain에서 trait 정의, infrastructure에서 구현
4. **테스트 용이성**: 각 계층을 독립적으로 테스트 가능

## 개발

```bash
# 빌드
cargo build

# 실행
cargo run

# 테스트
cargo test
```
