# Makefile for Health Check Server

.PHONY: help build run test clean docker-build docker-run dev run-prod health deps check fmt clippy

# Default target
help:
	@echo "Available commands:"
	@echo "  build        - Build the application"
	@echo "  run          - Run the application"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  check        - Check code without building"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run clippy linter"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with Docker Compose"
	@echo "  dev          - Run in development mode"
	@echo "  run-prod     - Run in production mode"
	@echo "  health       - Check server health"
	@echo "  deps         - Install dependencies"

# Build the application
build:
	@echo "🔨 Building Health Check Server..."
	cargo build --release
	@echo "✅ Build completed"

# Run the application
run:
	@echo "🚀 Starting Health Check Server..."
	cargo run

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test
	@echo "✅ Tests completed"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean completed"

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check
	@echo "✅ Check completed"

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt
	@echo "✅ Formatting completed"

# Run clippy linter
clippy:
	@echo "🔍 Running clippy..."
	cargo clippy -- -D warnings
	@echo "✅ Clippy completed"

# Build Docker image
docker-build:
	@echo "🐳 Building Docker image..."
	docker build -t health-check-server .
	@echo "✅ Docker image built"

# Run with Docker Compose
docker-run:
	@echo "🐳 Starting with Docker Compose..."
	docker-compose up --build

# Run in development mode
dev:
	@echo "🔧 Starting in development mode..."
	RUN_MODE=development cargo run

# Run in production mode
run-prod:
	@echo "🚀 Starting in production mode..."
	RUN_MODE=production cargo run

# Check health
health:
	@echo "❤️  Checking server health..."
	@curl -f http://localhost:8080/health || echo "❌ Server is not responding"

# Install dependencies
deps:
	@echo "📦 Installing dependencies..."
	cargo build
	@echo "✅ Dependencies installed"

# Development workflow
dev-setup: deps fmt clippy test
	@echo "🔧 Development setup completed"

# Production build
prod-build: clean fmt clippy test build
	@echo "🚀 Production build completed"

# Docker development
docker-dev:
	@echo "🐳 Starting development with Docker Compose..."
	docker-compose --profile dev up --build

# Docker test
docker-test:
	@echo "🐳 Running tests with Docker Compose..."
	docker-compose --profile test up --build --abort-on-container-exit

# Full CI pipeline
ci: clean fmt clippy test build
	@echo "✅ CI pipeline completed"
