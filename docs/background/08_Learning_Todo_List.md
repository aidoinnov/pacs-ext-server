# 08. 신규 입사자를 위한 학습 투두 리스트

이 문서는 `pacs-ext-server` 프로젝트에 새로 합류한 개발자가 따라야 할 단계별 학습 로드맵입니다. 각 단계를 완료하며 체크(`[x]`) 하세요.

---

### Phase 1: Rust 언어 및 비동기 프로그래밍 기초 다지기

> **목표**: 프로젝트의 기반이 되는 Rust 언어의 핵심 개념을 완벽히 숙지합니다.

-   [ ] **The Rust Programming Language (The Book) 정독**
    -   [ ] 1~10장 읽기 (특히 소유권, 구조체, 열거형, Trait, 생명주기)
    -   [ ] 13장: 클로저 및 이터레이터
    -   [ ] 16장: 동시성 (특히 `Arc`, `Mutex`)

-   [ ] **`Result`와 `Option` 마스터하기**
    -   [ ] `match`, `if let`, `map`, `and_then` 등을 사용해 두 타입을 다루는 연습하기.
    -   [ ] `?` 연산자의 동작 원리와 사용법 완벽히 이해하기.

-   [ ] **비동기 프로그래밍 (`async/await`) 이해하기**
    -   [ ] The Async Book (Rust 공식)의 1~2장 읽기.
    -   [ ] `Future` Trait의 개념 이해하기.
    -   [ ] `async fn`과 일반 `fn`의 차이점 설명할 수 있기.

-   [ ] **`background/01_Rust_Core_Concepts.md` 문서 읽고 프로젝트 코드와 비교하기**

---

### Phase 2: 프로젝트 아키텍처 및 코드 흐름 파악

> **목표**: 클린 아키텍처의 각 계층 역할과 데이터 흐름을 파악하여 코드의 "지도"를 머릿속에 그립니다.

-   [ ] **`background/00_Project_Architecture_and_Structure.md` 정독**
    -   [ ] 4개 계층(Domain, Application, Presentation, Infrastructure)의 역할과 의존성 규칙 암기하기.

-   [ ] **"사용자 생성" 기능 코드 흐름 따라가기 (가장 중요)**
    -   [ ] `main.rs`에서 `user_controller`가 어떻게 라우팅되는지 찾기.
    -   [ ] `presentation/controllers/user_controller.rs`의 `create_user` 함수 분석하기.
    -   [ ] `application/use_cases/user_use_case.rs`의 `create_user` 메소드가 어떻게 DTO를 받고, 엔티티를 만들고, 리포지토리를 호출하는지 분석하기.
    -   [ ] `domain/entities/user.rs`의 `new` 메소드와 비즈니스 로직 확인하기.
    -   [ ] `domain/repositories/user_repository.rs`의 `IUserRepository` Trait에 어떤 함수가 정의되어 있는지 확인하기.
    -   [ ] `infrastructure/repositories/user_repository_impl.rs`에서 `save` 메소드가 실제 SQL 쿼리로 어떻게 구현되었는지 확인하기.

-   [ ] **`main.rs`의 의존성 주입(DI) 과정 분석하기**
    -   [ ] `PgPool`이 생성되는 부분 찾기.
    -   [ ] `RepositoryImpl`이 생성되는 부분 찾기.
    -   [ ] `UseCase`가 생성되고 `web::Data`로 래핑되는 부분 찾기.
    -   [ ] `App::new().app_data()`로 등록되는 과정 이해하기.
    -   [ ] `background/07_Layer-Specific_Code_Patterns.md`의 DI 패턴 참고하기.

---

### Phase 3: 핵심 라이브러리 및 기술 스택 학습

> **목표**: 프로젝트에서 사용하는 주요 외부 라이브러리의 사용법을 익힙니다.

-   [ ] **웹 프레임워크 (Actix-web 추정)**
    -   [ ] Actix-web 공식 문서의 "Getting Started" 튜토리얼 따라하기.
    -   [ ] `Extractor`, `Handler`, `Middleware`, `State`의 개념 이해하기.
    -   [ ] `background/02_Web_Framework_and_API.md` 다시 읽기.

-   [ ] **데이터베이스 라이브러리 (`sqlx`)**
    -   [ ] `sqlx` 공식 문서의 `quickstart` 따라하기 (PostgreSQL 버전).
    -   [ ] `query!`, `query_as!` 매크로의 차이점과 컴파일 타임 검증 기능 이해하기.
    -   [ ] `PgPool`을 사용한 커넥션 풀링의 장점 이해하기.

-   [ ] **인증/인가 기술**
    -   [ ] `background/04_Authentication_and_Authorization.md` 정독하기.
    -   [ ] `jsonwebtoken` crate 문서 훑어보기 (인코딩/디코딩 함수 위주).
    -   [ ] `infra/db/schema.sql` 파일에서 `users`, `roles`, `permissions` 및 관련 중간 테이블들의 관계 파악하기.

---

### Phase 4: 실전 개발 및 기여

> **목표**: 학습한 내용을 바탕으로 실제 개발 환경에서 코드를 작성하고 기여합니다.

-   [ ] **로컬 개발 환경 설정하기**
    -   [ ] `background/06_Testing_and_DevOps.md` 문서를 따라 Docker 설치 및 설정하기.
    -   [ ] `infra` 디렉토리에서 `docker-compose up -d` 명령어로 로컬 DB 띄우기.
    -   [ ] `pacs-server` 디렉토리에서 `cargo run`으로 서버가 정상 실행되는지 확인하기.

-   [ ] **테스트 코드 실행 및 작성해보기**
    -   [ ] `cargo test` 명령어로 모든 테스트가 통과하는지 확인하기.
    -   [ ] `domain/entities/annotation.rs` 파일에 간단한 단위 테스트(`#[test]`) 하나 추가해보기.

-   [ ] **간단한 기능 직접 추가해보기 (연습)**
    -   [ ] **목표**: `GET /api/v1/health` 라는 간단한 상태 체크 API 엔드포인트 만들기.
    -   [ ] **구현 순서**:
        1.  `presentation/controllers`에 `health_controller.rs` 파일 생성.
        2.  `health_check` 함수 작성 (간단한 JSON 객체, 예: `{"status": "ok"}`를 반환).
        3.  `main.rs` 또는 `routes.rs`에 `/health` 경로와 `health_check` 함수를 라우팅.
        4.  `cargo run`으로 서버 실행 후, `curl http://localhost:8080/api/v1/health`로 동작 확인.

-   [ ] **기존 코드 리팩토링 또는 버그 수정**
    -   [ ] `background` 폴더의 문서들을 참고하여 기존 코드의 패턴을 분석하고 개선점 찾아보기.
    -   [ ] 간단한 버그 수정이나 로깅 추가 등 작은 단위의 첫 번째 커밋 만들어보기.
