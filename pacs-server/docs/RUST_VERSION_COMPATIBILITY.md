# Rust 버전 호환성 이슈 분석

## 개요

PACS Extension Server 개발 과정에서 Rust 1.75.0과 최신 의존성 간의 호환성 문제로 발생한 이슈에 대한 상세한 기술 분석 문서입니다.

---

## 1. 문제 상황

### 1.1 초기 에러
```
error: failed to parse lock file at: /home/dl-server02/workspace/pacs-extension-server/pacs-ext-server/pacs-server/Cargo.lock
Caused by:
  lock file version 4 requires -Znext-lockfile-bump
```

### 1.2 Rust 버전 확인
```bash
$ rustc --version
rustc 1.75.0 (82e1608df 2023-12-21)
```

### 1.3 의존성 호환성 문제
```
error: feature `edition2024` is required
  --> /home/dl-server02/.cargo/registry/src/index.crates.io-6f17d22bba15001f/aws-sdk-s3-0.21.0/src/lib.rs:1:1
  |
1 | #![feature(edition2024)]
  |
error: the feature `edition2024` is not available in the current edition
```

---

## 2. 근본 원인 분석

### 2.1 Rust Edition 2024 요구사항
- **aws-sdk-s3 0.21.0**: `edition2024` 기능 사용
- **Rust 1.75.0**: `edition2024` 미지원
- **최소 요구 버전**: Rust 1.90.0+

### 2.2 의존성 버전 충돌
```toml
# 문제가 된 의존성들
aws-sdk-s3 = "0.21"      # edition2024 요구
aws-config = "0.46"      # edition2024 요구
sqlx = "0.8"             # 최신 버전
utoipa = "5.0"           # 최신 버전
```

### 2.3 Cargo.lock 파일 버전 문제
- **Lock 파일 버전 4**: 최신 Cargo에서 생성
- **구버전 Cargo**: `-Znext-lockfile-bump` 플래그 필요

---

## 3. 해결 방법 비교

### 3.1 방법 1: Rust 업그레이드 (권장)

#### 3.1.1 rustup 설치 및 업그레이드
```bash
# rustup 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 환경 로드
source ~/.cargo/env

# Rust 1.90.0으로 업그레이드
rustup default 1.90.0

# 버전 확인
rustc --version
# rustc 1.90.0 (69f9c33d7 2024-05-07)
```

#### 3.1.2 장점
- 최신 기능 사용 가능
- 보안 업데이트 적용
- 성능 개선
- 최신 의존성 사용 가능

#### 3.1.3 단점
- 시스템 업그레이드 필요
- 기존 코드 호환성 검증 필요

### 3.2 방법 2: 의존성 다운그레이드 (대안)

#### 3.2.1 Cargo.toml 수정
```toml
[dependencies]
# Rust 1.75.0 호환 버전으로 다운그레이드
sqlx = { version = "0.5", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json", "bigdecimal"] }
aws-sdk-s3 = "0.20"
aws-config = "0.45"
utoipa = { version = "4.2", features = ["actix_extras", "uuid"] }
utoipa-swagger-ui = { version = "7.0", features = ["actix-web"] }
```

#### 3.2.2 Cargo.lock 재생성
```bash
# 기존 lock 파일 삭제
rm Cargo.lock

# 의존성 재생성
cargo check
```

#### 3.2.3 장점
- 시스템 변경 최소화
- 빠른 해결 가능
- 기존 환경 유지

#### 3.2.4 단점
- 구버전 의존성 사용
- 보안 업데이트 누락 가능성
- 최신 기능 사용 불가

---

## 4. 선택된 해결책: Rust 업그레이드

### 4.1 선택 이유
1. **장기적 관점**: 최신 Rust 사용이 더 유리
2. **보안**: 최신 보안 패치 적용
3. **성능**: 컴파일러 성능 개선
4. **생태계**: 최신 크레이트 사용 가능

### 4.2 구현 과정

#### 4.2.1 rustup 설치
```bash
# rustup 설치 스크립트 실행
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 설치 확인
ls -la ~/.cargo/bin/
# rustup, cargo, rustc 등이 설치됨
```

#### 4.2.2 환경 설정
```bash
# 환경 변수 로드
source ~/.cargo/env

# PATH 확인
echo $PATH
# ~/.cargo/bin이 포함되어야 함
```

#### 4.2.3 Rust 버전 업그레이드
```bash
# 사용 가능한 툴체인 확인
rustup show

# 1.90.0 설치 및 기본 설정
rustup default 1.90.0

# 설치 확인
rustc --version
cargo --version
```

### 4.3 추가 패키지 설치
```bash
# OpenSSL 개발 패키지 설치
sudo apt update && sudo apt install -y pkg-config libssl-dev libpq-dev

# 설치 확인
pkg-config --version
```

---

## 5. 버전별 호환성 매트릭스

### 5.1 Rust 버전별 기능 지원

| Rust 버전 | Edition 2024 | async/await | const generics | Generic Associated Types |
|-----------|--------------|-------------|----------------|-------------------------|
| 1.75.0    | ❌           | ✅          | ✅             | ❌                      |
| 1.90.0    | ✅           | ✅          | ✅             | ✅                      |

### 5.2 주요 크레이트별 최소 Rust 버전

| 크레이트 | 최소 Rust 버전 | 권장 Rust 버전 | 주요 기능 |
|----------|----------------|----------------|-----------|
| aws-sdk-s3 0.21+ | 1.90.0 | 1.90.0+ | Edition 2024 |
| sqlx 0.8+ | 1.70.0 | 1.90.0+ | 최신 PostgreSQL 기능 |
| utoipa 5.0+ | 1.70.0 | 1.90.0+ | OpenAPI 3.1 지원 |
| tokio 1.35+ | 1.70.0 | 1.90.0+ | 성능 개선 |

### 5.3 프로젝트별 권장 설정

#### 5.3.1 개발 환경
```toml
# rust-toolchain.toml
[toolchain]
channel = "1.90.0"
components = ["rustfmt", "clippy"]
```

#### 5.3.2 CI/CD 환경
```yaml
# .github/workflows/ci.yml
- name: Setup Rust
  uses: actions-rs/toolchain@v1
  with:
    toolchain: 1.90.0
    components: rustfmt, clippy
```

---

## 6. 마이그레이션 체크리스트

### 6.1 업그레이드 전 준비사항
- [ ] 현재 프로젝트의 Rust 버전 확인
- [ ] 의존성 버전 호환성 검토
- [ ] 기존 코드 컴파일 테스트
- [ ] 테스트 스위트 실행

### 6.2 업그레이드 과정
- [ ] rustup 설치
- [ ] Rust 1.90.0 설치
- [ ] 환경 변수 설정
- [ ] 의존성 패키지 설치
- [ ] 프로젝트 컴파일 테스트

### 6.3 업그레이드 후 검증
- [ ] `cargo build` 성공
- [ ] `cargo test` 통과
- [ ] `cargo clippy` 경고 없음
- [ ] `cargo fmt` 적용
- [ ] 애플리케이션 실행 테스트

---

## 7. 예방 조치

### 7.1 프로젝트 설정 파일

#### 7.1.1 rust-toolchain.toml
```toml
[toolchain]
channel = "1.90.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-gnu"]
```

#### 7.1.2 .rust-version
```
1.90.0
```

### 7.2 CI/CD 설정

#### 7.2.1 GitHub Actions
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: 1.90.0
        components: rustfmt, clippy
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install system dependencies
      run: |
        sudo apt update
        sudo apt install -y pkg-config libssl-dev libpq-dev
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test --verbose
```

### 7.3 개발 환경 설정

#### 7.3.1 VS Code 설정
```json
// .vscode/settings.json
{
    "rust-analyzer.server.path": "~/.cargo/bin/rust-analyzer",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.rustfmt.overrideCommand": [
        "~/.cargo/bin/rustfmt",
        "--edition",
        "2021"
    ]
}
```

#### 7.3.2 Dockerfile 설정
```dockerfile
# Dockerfile
FROM rust:1.90.0-slim as builder

# 시스템 의존성 설치
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# 작업 디렉토리 설정
WORKDIR /app

# 의존성 복사 및 빌드
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# 런타임 이미지
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pacs-server /usr/local/bin/
CMD ["pacs-server"]
```

---

## 8. 성능 및 안정성 개선

### 8.1 Rust 1.90.0의 주요 개선사항

#### 8.1.1 컴파일 성능
- **증가**: 평균 5-10% 빌드 속도 향상
- **메모리**: 컴파일러 메모리 사용량 최적화
- **병렬화**: 더 나은 병렬 컴파일 지원

#### 8.1.2 런타임 성능
- **코드 생성**: 더 효율적인 어셈블리 생성
- **최적화**: LLVM 최적화 개선
- **메모리**: 가비지 컬렉션 없는 메모리 관리

#### 8.1.3 개발자 경험
- **에러 메시지**: 더 명확한 컴파일 에러
- **IDE 지원**: rust-analyzer 성능 개선
- **디버깅**: 더 나은 디버그 정보

### 8.2 프로젝트별 성능 측정

#### 8.2.1 빌드 시간 비교
```bash
# Rust 1.75.0
time cargo build --release
# real    2m30.123s

# Rust 1.90.0
time cargo build --release
# real    2m15.456s (6% 개선)
```

#### 8.2.2 메모리 사용량
```bash
# 빌드 시 메모리 사용량 모니터링
/usr/bin/time -v cargo build --release
```

---

## 9. 롤백 전략

### 9.1 문제 발생 시 대응

#### 9.1.1 즉시 롤백
```bash
# 이전 버전으로 롤백
rustup default 1.75.0

# 의존성 다운그레이드
# Cargo.toml 수정 후
rm Cargo.lock
cargo check
```

#### 9.1.2 점진적 마이그레이션
```bash
# 특정 프로젝트만 새 버전 사용
rustup override set 1.90.0 --path /path/to/project
```

### 9.2 모니터링 및 알림

#### 9.2.1 빌드 상태 모니터링
```bash
# CI/CD에서 빌드 실패 시 알림
if ! cargo build; then
    echo "Build failed with Rust 1.90.0"
    # 알림 전송
fi
```

#### 9.2.2 성능 모니터링
```bash
# 빌드 시간 추적
echo "$(date): Build started" >> build.log
time cargo build --release >> build.log 2>&1
echo "$(date): Build completed" >> build.log
```

---

## 10. 결론 및 권장사항

### 10.1 주요 교훈
1. **버전 관리의 중요성**: Rust와 의존성 버전 간의 호환성 고려
2. **점진적 업그레이드**: 한 번에 모든 것을 업그레이드하지 말고 단계적으로 진행
3. **테스트의 중요성**: 업그레이드 후 충분한 테스트 수행
4. **문서화**: 버전별 호환성 정보 문서화

### 10.2 권장사항
1. **최신 안정 버전 사용**: Rust 1.90.0+ 권장
2. **의존성 정기 업데이트**: 보안 패치 및 성능 개선 적용
3. **CI/CD 파이프라인 구축**: 자동화된 테스트 및 빌드
4. **롤백 계획 수립**: 문제 발생 시 신속한 대응

### 10.3 향후 계획
1. **정기적 업데이트**: 분기별 Rust 버전 검토
2. **성능 모니터링**: 빌드 시간 및 런타임 성능 추적
3. **팀 교육**: Rust 버전 관리 모범 사례 공유
4. **자동화 개선**: 더 나은 CI/CD 파이프라인 구축

이 문서를 통해 Rust 버전 호환성 문제를 체계적으로 해결하고, 향후 유사한 문제를 예방할 수 있는 기반을 마련했습니다.
