# Simple Rust Server

Actix-web 기반 간단한 HTTP 서버

## 요구사항

- Rust 1.70 이상
- Cargo (Rust와 함께 설치됨)

## Rust 설치

Rust는 `rustup`을 통해 설치하고 버전 관리를 할 수 있습니다:

```bash
# rustup 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 환경변수 로드
source "$HOME/.cargo/env"
```

## rustup 주요 명령어

```bash
# Rust 버전 확인
rustc --version

# Rust 업데이트
rustup update

# 특정 툴체인 설치
rustup install stable
rustup install nightly

# 기본 툴체인 설정
rustup default stable
```

## 빌드 및 실행

```bash
# 의존성 다운로드 및 빌드 후 실행
cargo run

# 릴리스 빌드 (최적화)
cargo build --release
./target/release/simple-rust-server
```

서버는 `http://localhost:8080`에서 실행됩니다.

## 엔드포인트

- `GET /` - "Hello, World!" 응답
- `GET /health` - 헬스체크 (OK 응답)

## 테스트

```bash
curl http://localhost:8080
curl http://localhost:8080/health
```

## 의존성

- `actix-web` - 고성능 비동기 웹 프레임워크
- `tokio` - 비동기 런타임
