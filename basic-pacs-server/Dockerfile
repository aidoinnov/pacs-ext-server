# 멀티스테이지 빌드
FROM rust:1.75 as builder

WORKDIR /app

# 의존성 파일 복사 및 빌드 (캐시 최적화)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 소스 코드 복사 및 빌드
COPY src ./src
COPY config ./config
RUN cargo build --release

# 런타임 이미지
FROM debian:bookworm-slim

# 필요한 패키지 설치
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 사용자 생성
RUN useradd -m -u 1000 appuser

WORKDIR /app

# 바이너리 복사
COPY --from=builder /app/target/release/basic-pacs-server /app/
COPY --from=builder /app/config /app/config

# 소유권 변경
RUN chown -R appuser:appuser /app

# 사용자 전환
USER appuser

# 포트 노출
EXPOSE 8080

# 헬스체크
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 서버 실행
CMD ["./basic-pacs-server"]
