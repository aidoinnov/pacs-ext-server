# 멀티스테이지 빌드를 위한 Dockerfile
# PACS Extension Server - Rust 애플리케이션

# =============================================================================
# Stage 1: 빌드 환경 설정
# =============================================================================
FROM rust:1.90-slim as builder

# 시스템 패키지 업데이트 및 필요한 의존성 설치
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 작업 디렉토리 설정
WORKDIR /app

# 소스 코드 및 설정 파일 복사
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY Cargo.toml ./
COPY .sqlx ./.sqlx

# 의존성 캐싱을 위한 Cargo.lock 복사
COPY Cargo.lock ./

# 의존성만 먼저 빌드 (캐싱 최적화)
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --verbose
RUN rm -rf src

# 실제 소스 코드 복사 및 빌드
COPY src ./src
RUN cargo build --release --verbose

# =============================================================================
# Stage 2: 런타임 환경 설정
# =============================================================================
# FROM debian:bookworm-slim
FROM rust:1.90-slim

# 런타임에 필요한 패키지 설치
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 비루트 사용자 생성
RUN groupadd -r pacs && useradd -r -g pacs pacs

# 작업 디렉토리 설정
WORKDIR /app

# 빌드된 바이너리 복사
COPY --from=builder /app/target/release/pacs_server /app/pacs-server
COPY --from=builder /app/config ./config
COPY --from=builder /app/migrations ./migrations

# 권한 설정
RUN chown -R pacs:pacs /app
USER pacs

# 애플리케이션 실행
CMD ["./pacs-server"]