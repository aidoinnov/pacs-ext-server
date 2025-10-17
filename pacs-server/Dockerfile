# ===================================================================
# PACS Extension Server - Rust Application (단일 스테이지 빌드 & 실행)
# ===================================================================
FROM rust:1.90-slim as builder

# -------------------------------------------------------------------
# 시스템 패키지 설치
# -------------------------------------------------------------------
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# -------------------------------------------------------------------
# 작업 디렉토리 설정
# -------------------------------------------------------------------
WORKDIR /app

# -------------------------------------------------------------------
# 소스 복사
# -------------------------------------------------------------------
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# -------------------------------------------------------------------
# 환경변수
# -------------------------------------------------------------------
ENV SQLX_OFFLINE=true
ENV RUST_LOG=debug

# -------------------------------------------------------------------
# 빌드
# -------------------------------------------------------------------
RUN cargo build --release --verbose

FROM debian:trixie-slim

# 런타임에 필요한 패키지 설치
RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq5 \
    libssl3 \    
    postgresql-client \    
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 작업 디렉토리 설정
WORKDIR /app

COPY --from=builder /app/target/release/pacs_server /app/pacs-server
COPY --from=builder /app/config ./config
COPY --from=builder /app/migrations ./migrations

COPY --from=builder /app ./


# -------------------------------------------------------------------
# 컨테이너 시작 명령
# -------------------------------------------------------------------
CMD ["./pacs-server"]