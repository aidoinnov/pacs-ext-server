# ============================
# Makefile for PACS Extension Server
# ============================

# 프로젝트 정보
PROJECT_NAME=pacs-server
IMAGE_NAME=aido/pacs-extension-server
VERSION=0.1.7
CONTAINER_NAME ?= pacs-server

# 레지스트리 정보
REGISTRY      = registry.sfoon.co.kr
PUSH_REGISTRY = registry-push.sfoon.co.kr
REPO_PATH     = docker-hosted

# 이미지 태그
LOCAL_LATEST   = $(IMAGE_NAME):latest
LOCAL_VERSION  = $(IMAGE_NAME):$(VERSION)
PUSH_LATEST    = $(PUSH_REGISTRY)/$(REPO_PATH)/$(IMAGE_NAME):latest
PUSH_VERSION   = $(PUSH_REGISTRY)/$(REPO_PATH)/$(IMAGE_NAME):$(VERSION)

# 포트 설정
SERVER_PORT ?= 8081
POSTGRES_PORT ?= 5432
REDIS_PORT ?= 6379
MINIO_PORT ?= 9000

# 환경 설정
ENV ?= development

.PHONY: help
.DEFAULT_GOAL := help

# ============================
# 개발 명령어
# ============================

## dev: 로컬에서 개발 모드로 서버 실행
dev:
	@echo ">>> Starting PACS server in development mode"
	@export RUST_LOG=debug && cargo run

## dev-watch: 파일 변경 감지하여 자동 재시작 (cargo-watch 필요)
dev-watch:
	@echo ">>> Starting PACS server with auto-reload"
	@cargo watch -x run

## test: 모든 테스트 실행
test:
	@echo ">>> Running tests"
	@cargo test --verbose

## test-coverage: 코드 커버리지 생성 (cargo-tarpaulin 필요)
test-coverage:
	@echo ">>> Running tests with coverage"
	@cargo tarpaulin --out Html --output-dir coverage

## fmt: 코드 포맷팅
fmt:
	@echo ">>> Formatting code"
	@cargo fmt --all

## fmt-check: 코드 포맷팅 체크 (CI용)
fmt-check:
	@echo ">>> Checking code formatting"
	@cargo fmt --all -- --check

## clippy: Rust 린터 실행
clippy:
	@echo ">>> Running clippy"
	@cargo clippy --all-targets --all-features -- -D warnings

## check: 코드 컴파일 체크 (빌드 없이)
check:
	@echo ">>> Checking code compilation"
	@cargo check --all-targets

## build-local: 로컬 빌드 (릴리즈 모드)
build-local:
	@echo ">>> Building project in release mode"
	@cargo build --release

## clean-cargo: Cargo 빌드 아티팩트 정리
clean-cargo:
	@echo ">>> Cleaning cargo build artifacts"
	@cargo clean

# ============================
# Docker 명령어
# ============================

## docker-build: Docker 이미지 빌드
docker-build:
	@echo ">>> Building Docker image: $(LOCAL_VERSION)"
	docker build --no-cache -t $(LOCAL_LATEST) -t $(LOCAL_VERSION) .
	# docker build -t $(LOCAL_LATEST) -t $(LOCAL_VERSION) .
	@echo ">>> Tagging for push registry"
	docker tag $(LOCAL_LATEST)  $(PUSH_LATEST)
	docker tag $(LOCAL_VERSION) $(PUSH_VERSION)

## docker-build-cache: Docker 이미지 빌드 (캐시 사용)
docker-build-cache:
	@echo ">>> Building Docker image with cache: $(LOCAL_VERSION)"
	docker build -t $(LOCAL_LATEST) -t $(LOCAL_VERSION) .
	docker tag $(LOCAL_LATEST)  $(PUSH_LATEST)
	docker tag $(LOCAL_VERSION) $(PUSH_VERSION)

## docker-push: 사설 레지스트리에 푸시
docker-push:
	@echo ">>> Pushing $(PUSH_VERSION) and $(PUSH_LATEST)"
	docker push $(PUSH_VERSION)
	docker push $(PUSH_LATEST)

## docker-run: Docker 컨테이너 단독 실행 (호스트 네트워크 사용)
docker-run:
	@echo ">>> Running container on http://localhost:$(SERVER_PORT)"
	@echo ">>> Using host network for Bastion tunnel access"
	docker run --rm -it --name $(CONTAINER_NAME) \
		--network=host \
		-e RUST_LOG=info \
		-e DATABASE_URL=postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension \
		$(LOCAL_VERSION)

## docker-stop: Docker 컨테이너 중지 및 제거
docker-stop:
	@echo ">>> Stopping container $(CONTAINER_NAME)"
	-docker rm -f $(CONTAINER_NAME)

## docker-shell: 컨테이너 내부 쉘 접속
docker-shell:
	@echo ">>> Starting shell inside $(LOCAL_VERSION)"
	docker run --rm -it \
		--network=host \
	  	-e RUST_LOG=debug \
	  	-e DATABASE_URL=postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension \
	  	$(LOCAL_VERSION) /bin/sh

## docker-logs: 컨테이너 로그 확인
docker-logs:
	@echo ">>> Showing logs for $(CONTAINER_NAME)"
	docker logs -f $(CONTAINER_NAME)

## docker-clean: dangling 이미지 정리
docker-clean:
	@echo ">>> Cleaning dangling images"
	docker image prune -f

# ============================
# Docker Compose 명령어
# ============================

## compose-up: 전체 스택 실행 (PostgreSQL, Redis, MinIO, PACS Server)
compose-up:
	@echo ">>> Starting all services with docker-compose"
	docker-compose up -d

## compose-up-build: 전체 스택 빌드 후 실행
compose-up-build:
	@echo ">>> Building and starting all services"
	docker-compose up -d --build

## compose-down: 전체 스택 중지 및 제거
compose-down:
	@echo ">>> Stopping all services"
	docker-compose down

## compose-down-volumes: 전체 스택 중지 및 볼륨 삭제
compose-down-volumes:
	@echo ">>> Stopping all services and removing volumes"
	docker-compose down -v

## compose-restart: 전체 스택 재시작
compose-restart:
	@echo ">>> Restarting all services"
	docker-compose restart

## compose-logs: 전체 서비스 로그 확인
compose-logs:
	@echo ">>> Showing logs for all services"
	docker-compose logs -f

## compose-logs-server: PACS 서버 로그만 확인
compose-logs-server:
	@echo ">>> Showing logs for pacs-server"
	docker-compose logs -f pacs-server

## compose-ps: 실행 중인 서비스 상태 확인
compose-ps:
	@echo ">>> Showing service status"
	docker-compose ps

## compose-shell: PACS 서버 컨테이너 쉘 접속
compose-shell:
	@echo ">>> Accessing pacs-server shell"
	docker-compose exec pacs-server /bin/sh

# ============================
# 데이터베이스 명령어
# ============================

## db-migrate: 데이터베이스 마이그레이션 실행
db-migrate:
	@echo ">>> Running database migrations"
	@sqlx migrate run

## db-create: SQLx를 위한 오프라인 쿼리 데이터 생성
db-create:
	@echo ">>> Creating SQLx offline query data"
	@cargo sqlx prepare

## db-reset: 데이터베이스 초기화 (주의: 모든 데이터 삭제)
db-reset:
	@echo ">>> Resetting database (WARNING: This will delete all data)"
	@sqlx database reset -y

## db-shell: PostgreSQL 쉘 접속
db-shell:
	@echo ">>> Connecting to PostgreSQL"
	docker-compose exec postgres psql -U admin -d pacs_db

# ============================
# 통합 명령어
# ============================

## setup: 초기 프로젝트 설정 (의존성 설치, 마이그레이션 등)
setup:
	@echo ">>> Setting up project"
	@cargo build
	@make compose-up
	@echo ">>> Waiting for services to be ready..."
	@sleep 10
	@make db-migrate
	@echo ">>> Setup complete!"

## ci: CI 환경에서 실행할 전체 검증
ci: fmt-check clippy test
	@echo ">>> All CI checks passed!"

## all: 포맷팅, 린팅, 테스트, 빌드 전체 실행
all: fmt clippy test build-local
	@echo ">>> All tasks completed!"

## clean: 모든 빌드 아티팩트 정리
clean: clean-cargo docker-clean
	@echo ">>> Cleanup complete!"

# ============================
# 헬프 명령어
# ============================

## help: 사용 가능한 명령어 목록 표시
help:
	@echo "==================================================================================="
	@echo "PACS Extension Server - Makefile Commands"
	@echo "==================================================================================="
	@echo ""
	@echo "개발 명령어:"
	@echo "  make dev                - 로컬에서 개발 모드로 서버 실행"
	@echo "  make dev-watch          - 파일 변경 감지하여 자동 재시작"
	@echo "  make test               - 모든 테스트 실행"
	@echo "  make test-coverage      - 코드 커버리지 생성"
	@echo "  make fmt                - 코드 포맷팅"
	@echo "  make clippy             - Rust 린터 실행"
	@echo "  make check              - 코드 컴파일 체크"
	@echo "  make build-local        - 로컬 빌드 (릴리즈 모드)"
	@echo ""
	@echo "Docker 명령어:"
	@echo "  make docker-build       - Docker 이미지 빌드"
	@echo "  make docker-build-cache - Docker 이미지 빌드 (캐시 사용)"
	@echo "  make docker-push        - 사설 레지스트리에 푸시"
	@echo "  make docker-run         - Docker 컨테이너 단독 실행"
	@echo "  make docker-stop        - Docker 컨테이너 중지"
	@echo "  make docker-shell       - 컨테이너 내부 쉘 접속"
	@echo "  make docker-logs        - 컨테이너 로그 확인"
	@echo ""
	@echo "Docker Compose 명령어:"
	@echo "  make compose-up         - 전체 스택 실행"
	@echo "  make compose-up-build   - 전체 스택 빌드 후 실행"
	@echo "  make compose-down       - 전체 스택 중지"
	@echo "  make compose-down-volumes - 전체 스택 중지 및 볼륨 삭제"
	@echo "  make compose-restart    - 전체 스택 재시작"
	@echo "  make compose-logs       - 전체 서비스 로그 확인"
	@echo "  make compose-logs-server - PACS 서버 로그만 확인"
	@echo "  make compose-ps         - 서비스 상태 확인"
	@echo "  make compose-shell      - PACS 서버 컨테이너 쉘 접속"
	@echo ""
	@echo "데이터베이스 명령어:"
	@echo "  make db-migrate         - 데이터베이스 마이그레이션 실행"
	@echo "  make db-create          - SQLx 오프라인 쿼리 데이터 생성"
	@echo "  make db-reset           - 데이터베이스 초기화 (주의!)"
	@echo "  make db-shell           - PostgreSQL 쉘 접속"
	@echo ""
	@echo "통합 명령어:"
	@echo "  make setup              - 초기 프로젝트 설정"
	@echo "  make ci                 - CI 환경 전체 검증"
	@echo "  make all                - 포맷팅, 린팅, 테스트, 빌드 전체 실행"
	@echo "  make clean              - 모든 빌드 아티팩트 정리"
	@echo ""
	@echo "==================================================================================="
	@echo "사용 예시:"
	@echo "  make dev                     # 개발 서버 실행"
	@echo "  make compose-up              # 전체 스택 실행 (권장)"
	@echo "  make docker-build docker-push # Docker 이미지 빌드 및 푸시"
	@echo "  make SERVER_PORT=9090 docker-run # 포트 변경하여 실행"
	@echo "==================================================================================="
