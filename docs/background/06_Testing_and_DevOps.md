# 06. 테스트와 개발 환경

안정적인 애플리케이션을 유지하기 위해서는 일관된 테스트와 개발 환경이 필수적입니다.

## 1. 테스트 (`tests` 디렉토리)

Rust 프로젝트는 `cargo test` 명령어를 통해 테스트를 쉽게 실행할 수 있습니다. 이 프로젝트의 테스트는 크게 두 종류로 나뉩니다.

### a. 단위 테스트 (Unit Tests)

-   **위치**: 각 모듈 파일(`...rs`) 내의 `#[cfg(test)]` 블록 안에 위치합니다.
-   **목적**: 특정 함수나 작은 코드 조각의 동작을 독립적으로 검증합니다. 외부 의존성(데이터베이스, 네트워크 등) 없이 순수한 로직을 테스트합니다.
-   **예시**: 도메인 엔티티의 메소드, 순수한 비즈니스 로직 함수 등.

```rust
// in domain/entities/project.rs

impl Project {
    pub fn new(name: String) -> Self {
        // ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_project() {
        let project = Project::new("My Test Project".to_string());
        assert_eq!(project.name, "My Test Project");
        // ID가 잘 생성되었는지 등 검증
    }
}
```

### b. 통합 테스트 (Integration Tests)

-   **위치**: 프로젝트 루트의 `tests/` 디렉토리에 위치합니다.
-   **목적**: 여러 모듈이 함께 동작하는 방식을 검증합니다. 실제 데이터베이스 연결이나 웹 요청을 포함할 수 있습니다.
-   **예시**: `tests/project_controller_test.rs`는 실제 API 엔드포인트를 호출하여 예상된 HTTP 응답과 데이터가 반환되는지 전체 흐름을 테스트합니다.

### 테스트 실행

프로젝트의 모든 테스트를 실행하려면 다음 명령어를 사용합니다.

```bash
# 프로젝트 루트 디렉토리에서 실행
cd pacs-server
cargo test
```

특정 테스트만 실행하려면 필터를 사용합니다.

```bash
# 이름에 "controller"가 포함된 테스트만 실행
cargo test controller

# 특정 테스트 함수만 실행
cargo test test_create_new_project
```

## 2. 로컬 개발 환경 (Docker)

이 프로젝트는 `docker-compose.yml` 파일을 사용하여 로컬 개발에 필요한 모든 서비스(데이터베이스, Redis 등)를 한 번에 관리합니다. 이를 통해 모든 개발자가 동일한 환경에서 작업할 수 있습니다.

### 주요 파일

-   `infra/docker-compose.yml`: 개발 환경을 구성하는 서비스들을 정의합니다. (예: PostgreSQL 데이터베이스, Redis 캐시 서버)
-   `infra/.env.example`: `docker-compose.yml`에서 사용하는 환경 변수들의 예시입니다. 이 파일을 복사하여 `.env` 파일을 만들어야 합니다.

### 개발 환경 시작 및 종료

1.  **`.env` 파일 생성**: `infra` 디렉토리에서 `.env.example` 파일을 `.env`로 복사하고, 필요에 따라 내용을 수정합니다.

    ```bash
    cd infra
    cp .env.example .env
    ```

2.  **Docker Compose 실행**: `infra` 디렉토리에서 다음 명령어를 실행하여 모든 서비스를 백그라운드에서 시작합니다.

    ```bash
    # -d: 백그라운드에서 실행 (detached mode)
    docker-compose up -d
    ```

3.  **서비스 상태 확인**:

    ```bash
    docker-compose ps
    ```

4.  **개발 환경 종료**: 모든 서비스를 중지하고 컨테이너를 삭제합니다.

    ```bash
    # --volumes: 데이터베이스 볼륨까지 삭제하려면 추가
    docker-compose down
    ```

### 데이터베이스 연결

`docker-compose up`으로 데이터베이스가 실행되면, Rust 애플리케이션은 `.env` 파일에 설정된 `DATABASE_URL`을 사용하여 Docker 컨테이너 내부의 PostgreSQL 데이터베이스에 연결됩니다. `psql` 이나 데이터베이스 GUI 툴을 사용하여 `localhost:5432` (또는 `.env`에 지정된 포트)로 접속하여 데이터를 직접 확인할 수 있습니다.
