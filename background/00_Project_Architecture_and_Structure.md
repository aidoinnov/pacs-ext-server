# 00. 프로젝트 아키텍처와 구조 (클린 아키텍처)

이 문서는 `pacs-ext-server` 프로젝트의 핵심 아키텍처인 **클린 아키텍처(Clean Architecture)**에 대해 설명합니다. 모든 코드는 이 구조를 기반으로 작성되었으므로, 반드시 이해해야 합니다.

## 1. 클린 아키텍처란?

클린 아키텍처는 관심사 분리(Separation of Concerns)를 통해 유연하고, 테스트하기 쉽고, 유지보수가 용이한 소프트웨어를 만드는 것을 목표로 하는 설계 사상입니다.

핵심 규칙은 **의존성 규칙(The Dependency Rule)**입니다.

> **"소스 코드 의존성은 오직 안쪽으로, 고수준의 정책을 향해서만 향해야 한다."**

즉, 바깥쪽 원(저수준, 구체적인 기술)이 안쪽 원(고수준, 추상적인 정책)에 의존해야 합니다. 안쪽 원은 바깥쪽 원에 대해 아무것도 알지 못합니다.

```
+-------------------------------------------------------------------
|  Presentation (Web Framework, Controllers) - 최외곽 계층
|       +---------------------------------------------------
|       |  Application (Use Cases) - 애플리케이션 로직
|       |       +-----------------------------------
|       |       |  Domain (Entities, Repositories)
|       |       |        - 핵심 비즈니스 규칙 -
|       |       +-----------------------------------
|       |
|       +---------------------------------------------------
|
+-------------------------------------------------------------------
      |
      v  <-- 의존성 방향 (Dependency Flow)
+-------------------------------------------------------------------
|  Infrastructure (DB, External APIs, Libraries) - 모든 계층에서 사용
+-------------------------------------------------------------------
```

## 2. 프로젝트 폴더 구조와 계층 매핑

`pacs-server/src` 폴더는 클린 아키텍처의 계층을 그대로 반영합니다.

### a. `domain` (도메인 계층)

-   **역할**: 프로젝트의 가장 핵심적인 비즈니스 로직과 데이터를 정의합니다.
-   **구성 요소**:
    -   `entities`: 비즈니스 데이터의 구조 (예: `User`, `Project`). 다른 계층의 기술에 전혀 의존하지 않는 순수한 Rust 구조체입니다.
    -   `repositories`: 데이터 영속성을 위한 인터페이스(Trait)를 정의합니다. "어떻게" 저장할지는 모르고, "무엇을" 저장하고 조회해야 하는지만 정의합니다. (예: `IUserRepository`)
    -   `services`: 여러 엔티티를 포함하는 복잡한 도메인 로직을 처리합니다.
-   **규칙**: 이 계층은 다른 어떤 계층(`application`, `infrastructure` 등)도 `use` 할 수 없습니다. 가장 독립적이고 순수한 계층입니다.

### b. `application` (애플리케이션 계층)

-   **역할**: 사용자의 특정 요청(Use Case)을 처리하는 비즈니스 흐름을 정의합니다. 도메인 객체들을 조율하여 작업을 수행합니다.
-   **구성 요소**:
    -   `use_cases`: 특정 기능 단위 (예: `CreateUserUseCase`, `GetProjectUseCase`).
    -   `dto` (Data Transfer Object): 외부 계층(주로 `presentation`)과 데이터를 주고받기 위한 구조체입니다.
-   **규칙**: `domain` 계층에만 의존합니다. `infrastructure`나 `presentation`에 대해서는 알지 못합니다.

### c. `presentation` (프레젠테이션 계층)

-   **역할**: 외부 세계(주로 웹)와의 상호작용을 담당합니다. HTTP 요청을 받고, 응답을 보냅니다.
-   **구성 요소**:
    -   `controllers`: HTTP 요청을 받아 `application` 계층의 유스케이스를 호출하고, 그 결과를 HTTP 응답으로 변환합니다.
    -   `routes`: URL 경로와 컨트롤러 함수를 매핑합니다.
-   **규칙**: `application` 계층에 의존하여 비즈니스 로직을 실행시킵니다.

### d. `infrastructure` (인프라스트럭처 계층)

-   **역할**: 외부 기술에 대한 실제 "구현"을 담당합니다.
-   **구성 요소**:
    -   `database`: 데이터베이스 커넥션 설정 등.
    -   `repositories`: `domain` 계층에 정의된 Repository Trait을 실제 SQL 쿼리를 사용해 구현합니다. (예: `UserRepositoryImpl`)
    -   `auth`: JWT 생성/검증, 비밀번호 해싱 등 인증 관련 기술 구현.
    -   `config`: 설정 파일 로딩 등.
-   **규칙**: `domain`과 `application` 계층에 정의된 인터페이스(Trait)를 구현하며, 이들에게 의존성을 "주입"해주는 역할을 합니다.

## 3. 데이터 흐름 예시: "사용자 생성"

1.  **`presentation/routes`**: `/users` POST 요청을 `presentation/controllers/user_controller::create_user` 함수로 라우팅.
2.  **`presentation/controllers`**: `create_user` 함수는 HTTP 요청 본문(JSON)을 `CreateUserDto`로 변환.
3.  **`application/use_cases`**: 컨트롤러는 주입받은 `UserUseCase`의 `create_user` 메소드를 `CreateUserDto`와 함께 호출.
4.  **`domain/entities`**: `UserUseCase`는 `User` 엔티티를 생성하고 비즈니스 규칙(예: 이메일 형식 검증)을 적용.
5.  **`domain/repositories`**: `UserUseCase`는 `IUserRepository` Trait의 `save` 메소드를 호출.
6.  **`infrastructure/repositories`**: `UserRepositoryImpl`의 `save` 메소드가 실제 SQL `INSERT` 쿼리를 실행하여 데이터베이스에 저장.
7.  성공/실패 결과가 역순으로 `presentation` 계층까지 전파되어 최종 HTTP 응답으로 변환됨.

이 구조를 이해하면 어떤 기능이 어디에 위치해야 하는지, 코드를 어떻게 수정해야 하는지 명확하게 파악할 수 있습니다.

## 주요 용어 정리 (Glossary)

이 프로젝트의 아키텍처를 이해하는 데 필요한 핵심 용어들입니다.

### 계층 (Layers)
- **도메인 계층 (Domain Layer)**: 시스템의 가장 핵심적인 규칙과 데이터의 집합. 외부 세계에 대해 아무것도 알지 못하는 순수한 비즈니스 로직. (`src/domain`)
- **애플리케이션 계층 (Application Layer)**: 도메인 계층을 활용하여 특정 기능(유스케이스)을 완성하는 처리 과정. (`src/application`)
- **인프라스트럭처 계층 (Infrastructure Layer)**: 데이터베이스, 외부 API 연동, 프레임워크 등 기술적인 세부 사항을 구현하는 계층. (`src/infrastructure`)
- **프레젠테이션 계층 (Presentation Layer)**: 사용자와의 상호작용(HTTP 요청/응답, CLI 등)을 담당하는 최외곽 계층. (`src/presentation`)

### 구성 요소 (Components)
- **엔티티 (Entity)**: 고유한 식별자(ID)를 가지며, 시스템의 핵심이 되는 도메인 객체. (예: `User`, `Project`)
- **DTO (Data Transfer Object)**: 계층 간, 특히 외부(Presentation)와 내부(Application) 간의 데이터 전송을 위해 사용하는 구조체. 민감 정보를 제외하거나 필요한 데이터만 담는 용도로 사용.
- **리포지토리 패턴 (Repository Pattern)**: 도메인 로직과 데이터 영속성(데이터베이스) 로직을 분리하기 위한 디자인 패턴. 도메인은 '계약(Trait)'만 알고, 인프라가 '구현'을 제공.
- **유스케이스 (Use Case)**: "사용자를 생성한다", "프로젝트를 조회한다"와 같이, 사용자가 시스템을 통해 달성하려는 특정 목표를 나타내는 비즈니스 로직의 단위.

### Rust 관련 용어 (Rust-specific Terms)
- **의존성 주입 (Dependency Injection, DI)**: 객체가 의존하는 다른 객체(의존성)를 외부에서 생성하여 전달(주입)하는 디자인 패턴. 이를 통해 객체 간의 결합도를 낮추고 테스트 용이성을 높임.
- **Trait (트레이트)**: 다른 언어의 '인터페이스(Interface)'와 유사. 특정 타입이 반드시 구현해야 하는 메서드의 집합을 정의하는 '계약서'. 리포지토리 패턴의 핵심.
- **`dyn Trait` (동적 디스패치)**: "Dynamic Dispatch". 컴파일 시점이 아닌, 프로그램 실행 시점에 어떤 구체적인 타입이 사용될지 결정하는 방식. `dyn UserRepository`처럼 사용하여, 구체적인 구현(`PostgresUserRepository`)을 숨기고 추상화된 `trait`에만 의존하게 만듦.
- **`Arc<T>`**: "Atomically Reference Counted". 여러 스레드(Thread) 간에 데이터를 안전하게 **공유 소유**하기 위한 스마트 포인터. `clone()` 해도 비용이 저렴하여 웹 서버 환경에서 의존성을 주입할 때 널리 사용됨.