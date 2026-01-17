# ===================================================================
# PACS Extension Server - Multi-Binary Build (API Server + GC Runner)
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
# 빌드 (멀티 바이너리)
# -------------------------------------------------------------------
# API 서버 바이너리 빌드
RUN cargo build --release --bin pacs_server --verbose

# GC Runner 바이너리 빌드
RUN cargo build --release --bin gc_runner --verbose

# ===================================================================
# Runtime Stage
# ===================================================================
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

# -------------------------------------------------------------------
# 바이너리 복사 (멀티 바이너리)
# -------------------------------------------------------------------
# API 서버 바이너리
COPY --from=builder /app/target/release/pacs_server /usr/local/bin/pacs_server

# GC Runner 바이너리
COPY --from=builder /app/target/release/gc_runner /usr/local/bin/gc_runner

# 설정 및 마이그레이션 파일
COPY --from=builder /app/config ./config
COPY --from=builder /app/migrations ./migrations

# -------------------------------------------------------------------
# 시작 스크립트 생성 (API 서버용)
# -------------------------------------------------------------------
RUN echo '#!/bin/bash\n\
echo "🔄 Running database migrations..."\n\
for file in migrations/*.sql; do\n\
    if [ -f "$file" ]; then\n\
        echo "📄 Executing: $file"\n\
        psql "$DATABASE_URL" -f "$file" || {\n\
            echo "❌ Migration failed: $file"\n\
            exit 1\n\
        }\n\
    fi\n\
done\n\
echo "✅ All migrations completed successfully"\n\
echo "🚀 Starting PACS Extension Server..."\n\
exec /usr/local/bin/pacs_server' > /app/start-server.sh

RUN chmod +x /app/start-server.sh

# -------------------------------------------------------------------
# 컨테이너 시작 명령
# -------------------------------------------------------------------
# 기본: API 서버 실행
# GC Runner는 Kubernetes CronJob에서 다른 command로 실행
# 예: command: ["/usr/local/bin/gc_runner", "timeout-pending", "--grace-days", "3"]
CMD ["/app/start-server.sh"]