# Basic PACS Server Makefile

.PHONY: help build run test clean docker-build docker-run docker-stop lint format check

# 기본 타겟
help: ## 사용 가능한 명령어 목록을 표시합니다
	@echo "사용 가능한 명령어:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# 빌드 관련
build: ## 프로젝트를 빌드합니다
	@echo "🔨 Building project..."
	cargo build

build-release: ## 릴리즈 모드로 빌드합니다
	@echo "🔨 Building project in release mode..."
	cargo build --release

# 실행 관련
run: ## 개발 모드로 서버를 실행합니다
	@echo "🚀 Starting server in development mode..."
	RUN_MODE=development cargo run

run-release: ## 릴리즈 모드로 서버를 실행합니다
	@echo "🚀 Starting server in release mode..."
	RUN_MODE=production cargo run --release

# 테스트 관련
test: ## 모든 테스트를 실행합니다
	@echo "🧪 Running tests..."
	cargo test

test-unit: ## 단위 테스트만 실행합니다
	@echo "🧪 Running unit tests..."
	cargo test --lib

test-integration: ## 통합 테스트만 실행합니다
	@echo "🧪 Running integration tests..."
	cargo test --test integration_tests

# 코드 품질 관련
lint: ## 코드 린팅을 실행합니다
	@echo "🔍 Running linter..."
	cargo clippy -- -D warnings

format: ## 코드 포맷팅을 실행합니다
	@echo "🎨 Formatting code..."
	cargo fmt

check: ## 코드 검사를 실행합니다 (빌드 + 린트 + 테스트)
	@echo "✅ Running all checks..."
	cargo check
	cargo clippy -- -D warnings
	cargo test

# 정리 관련
clean: ## 빌드 아티팩트를 정리합니다
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Docker 관련
docker-build: ## Docker 이미지를 빌드합니다
	@echo "🐳 Building Docker image..."
	docker build -t basic-pacs-server .

docker-run: ## Docker 컨테이너를 실행합니다
	@echo "🐳 Running Docker container..."
	docker run -d -p 8080:8080 --name basic-pacs-server basic-pacs-server

docker-stop: ## Docker 컨테이너를 중지합니다
	@echo "🐳 Stopping Docker container..."
	docker stop basic-pacs-server || true
	docker rm basic-pacs-server || true

docker-logs: ## Docker 컨테이너 로그를 확인합니다
	@echo "🐳 Showing Docker container logs..."
	docker logs -f basic-pacs-server

# Docker Compose 관련
compose-up: ## Docker Compose로 서비스를 시작합니다
	@echo "🐳 Starting services with Docker Compose..."
	docker-compose up -d

compose-down: ## Docker Compose 서비스를 중지합니다
	@echo "🐳 Stopping services with Docker Compose..."
	docker-compose down

compose-logs: ## Docker Compose 로그를 확인합니다
	@echo "🐳 Showing Docker Compose logs..."
	docker-compose logs -f

# 개발 관련
dev: ## 개발 환경을 설정합니다
	@echo "🛠️ Setting up development environment..."
	@if [ ! -f .env ]; then cp env.example .env; echo "📝 Created .env file from template"; fi
	@echo "✅ Development environment ready!"

# API 테스트
test-api: ## API 엔드포인트를 테스트합니다
	@echo "🌐 Testing API endpoints..."
	@echo "Testing health check..."
	@curl -s http://localhost:8080/health | jq . || echo "❌ Health check failed"
	@echo "Testing server info..."
	@curl -s http://localhost:8080/info | jq . || echo "❌ Server info failed"
	@echo "Testing API health check..."
	@curl -s http://localhost:8080/api/health | jq . || echo "❌ API health check failed"
	@echo "Testing API server info..."
	@curl -s http://localhost:8080/api/info | jq . || echo "❌ API server info failed"

# 전체 빌드 및 테스트
all: clean build test lint ## 전체 빌드, 테스트, 린트를 실행합니다
	@echo "✅ All checks passed!"

# 기본 타겟
.DEFAULT_GOAL := help
