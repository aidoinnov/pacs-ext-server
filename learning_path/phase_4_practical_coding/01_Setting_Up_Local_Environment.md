# Phase 4-1: 로컬 개발 환경 설정 가이드

이 가이드는 프로젝트를 로컬 머신에서 실행하고 개발을 시작하기 위한 단계별 지침을 제공합니다.

## 사전 요구 사항

-   **Git**: 버전 관리를 위해 필요합니다.
-   **Rust**: `rustup`을 통해 설치하는 것을 권장합니다. (`rustc`, `cargo` 포함)
-   **Docker** 및 **Docker Compose**: 데이터베이스 등 외부 서비스를 컨테이너화하여 실행하기 위해 필요합니다. (Docker Desktop 설치 권장)

## 1단계: 프로젝트 클론 및 디렉토리 이동

```bash
# 1. 프로젝트를 저장할 디렉토리로 이동합니다.
cd /path/to/your/workspace

# 2. Git 리포지토리를 클론합니다.
git clone <repository_url> pacs-ext-server

# 3. 프로젝트 디렉토리로 들어갑니다.
cd pacs-ext-server
```

## 2단계: Docker 서비스 실행

프로젝트는 PostgreSQL 데이터베이스와 같은 외부 서비스에 의존합니다. `docker-compose`를 사용하여 이러한 서비스들을 로컬에서 쉽게 실행할 수 있습니다.

```bash
# 1. docker-compose 파일이 있는 infra 디렉토리로 이동합니다.
cd infra

# 2. .env.example 파일을 .env 파일로 복사합니다.
# 이 파일에는 데이터베이스 연결 정보 등 환경 변수가 들어있습니다.
cp .env.example .env

# 3. Docker 컨테이너들을 백그라운드에서 빌드하고 실행합니다.
# -d 옵션은 "detached mode"를 의미합니다.
docker-compose up -d --build

# 4. 컨테이너들이 정상적으로 실행 중인지 확인합니다.
docker-compose ps
```

`docker-compose ps` 명령어 실행 시, `pacs-ext-server-db-1` (또는 유사한 이름) 컨테이너의 상태가 `Up` 또는 `running`으로 표시되어야 합니다.

## 3단계: Rust 애플리케이션 실행

이제 데이터베이스가 준비되었으므로, Rust 웹 서버를 실행할 수 있습니다.

```bash
# 1. Rust 프로젝트의 루트 디렉토리로 이동합니다.
# (현재 infra 디렉토리에 있다면)
cd ../pacs-server

# 2. sqlx-cli를 사용하여 컴파일 타임 쿼리 검사를 위한 메타데이터를 준비합니다.
# (최초 실행 또는 DB 스키마 변경 시에만 필요)
# 먼저 sqlx-cli를 설치해야 합니다: cargo install sqlx-cli
sqlx database prepare

# 3. cargo run 명령어로 애플리케이션을 컴파일하고 실행합니다.
# --release 플래그 없이 실행하면 디버그 모드로 실행됩니다.
cargo run
```

서버가 성공적으로 시작되면, 터미널에 다음과 같은 로그가 표시됩니다:

```
🚀 Server starting at http://127.0.0.1:8080
```

## 4단계: 서버 동작 확인

웹 브라우저나 `curl` 같은 도구를 사용하여 서버가 정상적으로 응답하는지 확인합니다. (예: health check API가 있다면)

```bash
curl http://localhost:8080/api/v1/health
```

`{"status": "ok"}` 와 같은 JSON 응답이 오면 성공적으로 설정된 것입니다.

## 개발 환경 종료

개발을 마친 후, Docker 컨테이너들을 중지하여 시스템 자원을 확보할 수 있습니다.

```bash
# infra 디렉토리로 이동
cd ../infra

# 실행 중인 컨테이너들을 중지하고 네트워크를 제거합니다.
docker-compose down

# 데이터베이스 볼륨까지 완전히 삭제하려면 --volumes 옵션을 추가합니다.
# docker-compose down --volumes
```

이제 코드를 수정하고, `cargo run`으로 다시 실행하며 개발을 진행할 준비가 모두 완료되었습니다.
