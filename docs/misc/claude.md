# CLAUDE.md

이 파일은 Claude Code가 이 저장소에서 작업할 때 참고하는 가이드입니다.

## 프로젝트 구조

PACS (Picture Archiving and Communication System) 확장 서버 프로젝트로, 여러 언어로 구현된 서버들을 포함합니다:

- `simple-go-server/` - Go 구현 (net/http 사용)
- `simple-rust-server/` - Rust 구현 (Actix-web 사용)
- `infra/` - 인프라 설정 (Docker Compose로 PostgreSQL 관리)

## 개발 명령어

### Go 서버

```bash
cd simple-go-server

# Go 버전 관리자 설치 (미설치시)
bash < <(curl -s -S -L https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer)
source ~/.gvm/scripts/gvm

# Go 1.21.13 설치 및 사용
gvm install go1.21.13 -B
gvm use go1.21.13 --default

# 서버 실행
go run main.go
```

서버는 `http://localhost:8080`에서 실행됩니다.

### Rust 서버

```bash
cd simple-rust-server

# Rust 설치 (미설치시)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 개발 모드 실행
cargo run

# 릴리스 빌드 (최적화)
cargo build --release
./target/release/simple-rust-server
```

서버는 `http://localhost:8080`에서 실행됩니다.

### 인프라

```bash
cd infra

# PostgreSQL 시작
docker-compose up -d

# PostgreSQL 중지
docker-compose down

# 로그 확인
docker-compose logs -f postgres
```

PostgreSQL 접속 정보:
- 주소: `localhost:5432`
- 사용자: `admin`
- 비밀번호: `admin123`
- 데이터베이스: `pacs_db`

## 테스트 및 벤치마크

### 기본 테스트

```bash
# 루트 엔드포인트 테스트
curl http://localhost:8080

# 헬스체크 엔드포인트 테스트
curl http://localhost:8080/health
```

### 성능 벤치마크

`wrk` 도구 사용 (4 스레드, 100 연결, 30초):

```bash
wrk -t4 -c100 -d30s http://localhost:8080/
```

**성능 비교** (Apple Silicon ARM64 기준):
- **Rust (Actix-web)**: 초당 ~223K 요청, 평균 372μs 레이턴시
- **Go (net/http)**: 초당 ~111K 요청, 평균 860μs 레이턴시

Rust가 처리량과 레이턴시 모두 약 2배 빠릅니다.

## 아키텍처 설명

### Go 구현
- 표준 라이브러리 `net/http` 패키지 사용
- 간단한 핸들러 기반 라우팅
- 외부 의존성 없음
- 빠른 개발과 유지보수에 적합

### Rust 구현
- Actix-web 프레임워크 (고성능 비동기)
- Tokio 비동기 런타임
- 제로 코스트 추상화로 성능 최적화
- 코드가 복잡하지만 매우 최적화됨

### 데이터베이스 설정
- PostgreSQL 16 Alpine 컨테이너
- 헬스체크 설정 (10초 간격, 5초 타임아웃, 5회 재시도)
- 영구 볼륨으로 데이터 저장

## Notion 문서 작성 규칙

### Notion 문서 생성
- 모든 기술 문서는 "SFooN 기술 문서 허브" (ID: 276c879a-ffe0-8075-a52f-c7d76250c942)에 작성
- 카테고리는 자동으로 "전략 문서"로 설정
- 마크다운 파일을 노션에 올릴 때는 자동으로 포맷팅

### 문서 유형별 카테고리
- MCP 관련: "전략 문서"
- 서버 구현: "ArgoCD" 또는 "전략 문서"
- 성능 분석: "전략 문서"

### 자주 사용하는 명령어
- "이 문서를 노션에 작성해줘" → SFooN 기술 문서 허브에 자동 생성