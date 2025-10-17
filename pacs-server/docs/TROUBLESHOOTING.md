# PACS Extension Server - 문제 해결 가이드

## 개요

이 문서는 PACS Extension Server 개발 및 배포 과정에서 발생한 주요 이슈들과 해결 방법을 정리한 기술 문서입니다.

---

## 1. Rust 컴파일러 버전 호환성 문제

### 문제 상황
- **에러**: `feature edition2024 is required`
- **원인**: Rust 1.75.0에서 `edition2024` 기능을 지원하지 않음
- **영향**: `cargo run` 실행 불가

### 해결 방법

#### 1.1 Rust 업그레이드
```bash
# rustup 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 환경 로드
source ~/.cargo/env

# Rust 1.90.0으로 업그레이드
rustup default 1.90.0
```

#### 1.2 의존성 버전 다운그레이드 (대안)
```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.5", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json", "bigdecimal"] }
aws-sdk-s3 = "0.20"
aws-config = "0.45"
utoipa = { version = "4.2", features = ["actix_extras", "uuid"] }
utoipa-swagger-ui = { version = "7.0", features = ["actix-web"] }
```

### 예방 조치
- 프로젝트 시작 시 최신 Rust 버전 사용
- CI/CD 파이프라인에서 Rust 버전 명시적 지정

---

## 2. OpenSSL 개발 패키지 누락

### 문제 상황
- **에러**: `Could not find directory of OpenSSL installation`
- **원인**: 시스템에 OpenSSL 개발 헤더 파일 누락
- **영향**: `openssl-sys` 크레이트 컴파일 실패

### 해결 방법
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y pkg-config libssl-dev libpq-dev

# CentOS/RHEL
sudo yum install -y pkgconfig openssl-devel postgresql-devel

# macOS
brew install pkg-config openssl postgresql
```

### 예방 조치
- Dockerfile에 필요한 개발 패키지 포함
- README에 시스템 요구사항 명시

---

## 3. DateTime unwrap_or_default() 메서드 오류

### 문제 상황
- **에러**: `no method named 'unwrap_or_default' found for struct 'chrono::DateTime'`
- **원인**: `chrono::DateTime<Utc>`는 `Default` 트레이트를 구현하지 않음
- **영향**: 18개의 컴파일 에러 발생

### 근본 원인 분석
1. **데이터베이스 스키마**: `created_at`, `updated_at`이 `TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP`
2. **엔티티 정의**: `DateTime<Utc>` (Option 아님)
3. **sqlx 반환값**: `Option<DateTime<Utc>>`로 반환
4. **타입 불일치**: `DateTime`은 `unwrap_or_default()` 메서드 없음

### 해결 방법

#### 3.1 잘못된 접근법
```rust
// ❌ 잘못된 방법
created_at: result.created_at.unwrap_or_default(),
updated_at: result.updated_at.unwrap_or_default(),
```

#### 3.2 올바른 해결책
```rust
// ✅ 올바른 방법 - 직접 할당
created_at: result.created_at,
updated_at: result.updated_at,
```

### 수정된 파일
- `src/infrastructure/repositories/mask_group_repository_impl.rs` (10곳)
- `src/infrastructure/repositories/mask_repository_impl.rs` (8곳)

### 예방 조치
- 데이터베이스 스키마와 엔티티 타입 일치성 검증
- sqlx 매크로 사용 시 반환 타입 확인
- 컴파일 타임에 타입 안전성 보장

---

## 4. 데이터베이스 인증 실패

### 문제 상황
- **에러**: `password authentication failed for user "pacs_extension_admin"`
- **원인**: RDS 사용자명/비밀번호 불일치
- **영향**: 애플리케이션 시작 불가

### 환경 변수 설정 방법

#### 4.1 DATABASE_URL 사용 (권장)
```bash
export DATABASE_URL="postgres://username:password@localhost:5432/pacs_db"
```

#### 4.2 개별 환경 변수 사용
```bash
export DATABASE_USERNAME="pacs_extension_admin"
export DATABASE_PASSWORD="CHANGE_ME_STRONG_PASSWORD"
export DATABASE_HOST="localhost"
export DATABASE_PORT="5432"
export DATABASE_NAME="pacs_db"
```

### 설정 우선순위
1. `DATABASE_URL` 환경 변수 (최우선)
2. 개별 환경 변수들 (`DATABASE_USERNAME`, `DATABASE_PASSWORD` 등)
3. 설정 파일 기본값

### 예방 조치
- 환경별 설정 파일 분리 (`env.development`, `env.production`)
- 데이터베이스 연결 정보 검증 스크립트 작성
- 보안을 위한 환경 변수 암호화

---

## 5. Docker Compose 메타데이터 충돌

### 문제 상황
- **에러**: `KeyError: 'ContainerConfig'`
- **원인**: Docker Compose 메타데이터 불일치
- **영향**: 컨테이너 시작 실패

### 해결 방법
```bash
# 기존 컨테이너 정리
docker-compose down -v

# Docker 시스템 정리
docker system prune -f

# 다시 시작
docker-compose up -d
```

### 예방 조치
- 정기적인 Docker 시스템 정리
- 컨테이너 재시작 전 정리 스크립트 사용
- Docker Compose 버전 호환성 확인

---

## 6. SSH 터널 연결 문제

### 문제 상황
- **에러**: `Host key verification failed`
- **원인**: SSH 호스트 키 검증 실패
- **영향**: RDS 연결 불가

### 해결 방법

#### 6.1 SSH 키 권한 설정
```bash
chmod 600 ~/.ssh/bastion-keypair.pem
```

#### 6.2 SSH 옵션 추가
```bash
ssh -i ~/.ssh/bastion-keypair.pem \
    -L 5432:rds-endpoint:5432 \
    ec2-user@bastion-host \
    -N \
    -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o LogLevel=ERROR
```

### 개선된 db-tunnel.sh 스크립트 기능
- 색상 출력 및 로깅 레벨 설정
- 터널 상태 확인 (`-s` 옵션)
- 터널 종료 (`-k` 옵션)
- 포트 지정 (`-p` 옵션)

### 예방 조치
- SSH 키 파일 권한 정기 점검
- Bastion 호스트 접근성 모니터링
- 자동화된 연결 테스트 스크립트

---

## 7. Cargo.lock 파일 버전 충돌

### 문제 상황
- **에러**: `lock file version 4 requires -Znext-lockfile-bump`
- **원인**: Cargo.toml 수정 후 lock 파일 불일치
- **영향**: 의존성 해결 실패

### 해결 방법
```bash
# Cargo.lock 삭제
rm Cargo.lock

# 의존성 재생성
cargo check
```

### 예방 조치
- Cargo.toml 수정 후 즉시 `cargo check` 실행
- 버전 관리 시스템에 Cargo.lock 포함
- 팀원 간 Cargo.lock 동기화

---

## 8. 마이그레이션 전략 결정

### 문제 상황
- **고민**: K8s 환경에서 데이터베이스 마이그레이션 방법
- **옵션**: ArgoCD ConfigMap vs 서버 사이드 자동 마이그레이션

### 선택된 해결책: 서버 사이드 자동 마이그레이션

#### 8.1 구현 방법
```rust
// src/main.rs
let run_migrations = std::env::var("RUN_MIGRATIONS")
    .unwrap_or_else(|_| "false".to_string())
    .parse::<bool>()
    .unwrap_or(false);

if run_migrations {
    print!("🔄 Running database migrations... ");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    println!("✅ Done");
}
```

#### 8.2 환경별 설정
```bash
# 개발 환경
RUN_MIGRATIONS=true

# 프로덕션 환경
RUN_MIGRATIONS=false
```

### 장점
- 간단한 구현
- 환경별 제어 가능
- 롤백 용이성

### 예방 조치
- 마이그레이션 실행 로그 추가
- 실패 시 애플리케이션 중단
- 백업 전략 수립

---

## 결론

이러한 이슈들을 통해 다음과 같은 교훈을 얻었습니다:

1. **환경 설정의 중요성**: 개발 환경과 프로덕션 환경의 차이점을 명확히 인식
2. **타입 안전성**: Rust의 강력한 타입 시스템을 활용한 컴파일 타임 에러 방지
3. **의존성 관리**: 버전 호환성과 의존성 충돌 해결의 중요성
4. **자동화의 가치**: 반복적인 문제를 자동화된 스크립트로 해결
5. **문서화의 필요성**: 문제 해결 과정을 문서화하여 향후 참고 가능

이 문서는 향후 유사한 문제가 발생했을 때 빠른 해결을 위한 참고 자료로 활용할 수 있습니다.
