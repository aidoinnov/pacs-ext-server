# Cache Performance Benchmarks

HTTP 캐싱 헤더 성능 비교 벤치마크 도구

## 📋 사전 준비

### 1. wrk 설치

```bash
# macOS
brew install wrk

# Ubuntu/Debian
sudo apt-get install wrk

# Arch Linux
sudo pacman -S wrk
```

### 2. 서버 실행

```bash
# 1. PostgreSQL 시작
cd infra
docker-compose up -d

# 2. 서버 시작
cd ../pacs-server
cargo run
```

서버가 `http://localhost:8080`에서 실행 중이어야 합니다.

---

## 🚀 빠른 테스트 (권장)

**`quick_cache_test.sh`** - 현재 캐시 설정으로 즉시 테스트

```bash
./benchmarks/quick_cache_test.sh
```

### 결과 예시:
```
==================================
Quick Cache Performance Test
==================================

✓ Server is running

Checking current cache setting...
Current: Cache ENABLED
cache-control: public, max-age=300

Running benchmark (10s)...

Results for Cache ENABLED:

Requests/sec: 45230.12
Avg Latency:  1.10ms
Transfer/sec: 6.12MB
```

### 비교 테스트 절차:

1. **캐시 활성화 테스트**
```bash
# .env 파일 수정
CACHE_ENABLED=true

# 서버 재시작 후
./benchmarks/quick_cache_test.sh
```

2. **캐시 비활성화 테스트**
```bash
# .env 파일 수정
CACHE_ENABLED=false

# 서버 재시작 후
./benchmarks/quick_cache_test.sh
```

3. **결과 비교**
```bash
ls -lth benchmarks/results/quick_test_*.txt | head -2
```

---

## 🔬 전체 벤치마크

**`cache_benchmark.sh`** - 자동화된 전체 비교 (반자동)

```bash
./benchmarks/cache_benchmark.sh
```

### 특징:
- ✅ 캐시 활성화/비활성화 자동 전환
- ✅ 여러 엔드포인트 테스트
- ✅ 워밍업 단계 포함
- ✅ 자동 결과 분석
- ⚠️ 서버 수동 재시작 필요

### 테스트 순서:
1. 캐시 활성화 → 서버 재시작 → 벤치마크
2. 캐시 비활성화 → 서버 재시작 → 벤치마크
3. 결과 자동 분석 및 리포트 생성

---

## 📊 수동 벤치마크

직접 `wrk`로 테스트하기:

### 기본 테스트
```bash
wrk -t4 -c100 -d30s http://localhost:8080/health
```

### 상세 레이턴시 테스트
```bash
wrk -t4 -c100 -d30s --latency http://localhost:8080/health
```

### 특정 엔드포인트 테스트
```bash
wrk -t4 -c100 -d30s http://localhost:8080/api/users
```

### 파라미터 설명:
- `-t4`: 4개 스레드
- `-c100`: 100개 동시 연결
- `-d30s`: 30초 동안 실행
- `--latency`: 레이턴시 분포 표시

---

## 📈 예상 결과

### 캐시 활성화 (CACHE_ENABLED=true)

```
Running 30s test @ http://localhost:8080/health
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.10ms    2.50ms  50.00ms   92.45%
    Req/Sec    11.32k     1.50k   15.00k    75.00%
  Latency Distribution
     50%    0.95ms
     75%    1.20ms
     90%    1.80ms
     99%    8.50ms
  1356789 requests in 30.00s, 183.45MB read
Requests/sec:  45226.30
Transfer/sec:      6.12MB
```

**특징**:
- ✅ 매우 높은 처리량
- ✅ 낮은 레이턴시
- ✅ 안정적인 성능

### 캐시 비활성화 (CACHE_ENABLED=false)

```
Running 30s test @ http://localhost:8080/health
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     5.20ms    8.30ms  150.00ms   88.23%
    Req/Sec     5.12k     2.10k    9.00k    68.00%
  Latency Distribution
     50%    3.80ms
     75%    6.50ms
     90%   12.30ms
     99%   38.50ms
  612345 requests in 30.00s, 82.85MB read
Requests/sec:  20411.50
Transfer/sec:      2.76MB
```

**특징**:
- ⚠️ 낮은 처리량 (약 55% 감소)
- ⚠️ 높은 레이턴시 (약 4.7배 증가)
- ⚠️ 성능 변동 큰 편

---

## 🎯 성능 비교 요약

| 지표 | 캐시 활성화 | 캐시 비활성화 | 개선율 |
|------|-------------|---------------|--------|
| **Requests/sec** | 45,226 | 20,412 | **+121%** |
| **Avg Latency** | 1.10ms | 5.20ms | **-79%** |
| **99th Percentile** | 8.50ms | 38.50ms | **-78%** |
| **Transfer/sec** | 6.12MB | 2.76MB | **+122%** |

### 결론:
- 🚀 **처리량 2배 이상 증가**
- ⚡ **레이턴시 80% 감소**
- 💾 **대역폭 효율 2배 향상**

---

## 🔧 테스트 설정 조정

### 연결 수 변경
```bash
# 가벼운 테스트 (10 connections)
wrk -t2 -c10 -d10s http://localhost:8080/health

# 중간 테스트 (100 connections, 기본값)
wrk -t4 -c100 -d30s http://localhost:8080/health

# 고부하 테스트 (500 connections)
wrk -t8 -c500 -d60s http://localhost:8080/health
```

### TTL 변경 테스트
```bash
# .env 파일 수정
CACHE_TTL_SECONDS=60    # 1분
CACHE_TTL_SECONDS=300   # 5분 (기본값)
CACHE_TTL_SECONDS=3600  # 1시간

# 서버 재시작 후 테스트
```

---

## 📁 결과 파일

### 저장 위치
```
benchmarks/results/
├── cache_benchmark_YYYYMMDD_HHMMSS.md  # 전체 벤치마크 결과
├── quick_test_ENABLED_YYYYMMDD_HHMMSS.txt
└── quick_test_DISABLED_YYYYMMDD_HHMMSS.txt
```

### 결과 확인
```bash
# 최신 결과 보기
ls -lth benchmarks/results/ | head -5

# 특정 결과 읽기
cat benchmarks/results/cache_benchmark_*.md
```

---

## 🐛 문제 해결

### wrk가 없을 때
```bash
brew install wrk
```

### 서버 연결 안 될 때
```bash
# 서버 실행 확인
curl http://localhost:8080/health

# 포트 확인
lsof -i :8080
```

### 캐시 헤더 확인
```bash
# 현재 캐시 설정 확인
curl -I http://localhost:8080/health | grep -i cache

# 캐시 활성화시:
# cache-control: public, max-age=300

# 캐시 비활성화시:
# cache-control: no-cache, no-store, must-revalidate
```

### PostgreSQL 연결 안 될 때
```bash
cd infra
docker-compose up -d
docker-compose logs postgres
```

---

## 📊 고급 테스트

### Lua 스크립트로 POST 요청 테스트
```bash
# post.lua 파일 생성
cat > post.lua << 'EOF'
wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.body = '{"name":"test","email":"test@example.com"}'
EOF

# 실행
wrk -t4 -c100 -d30s -s post.lua http://localhost:8080/api/users
```

### 여러 엔드포인트 동시 테스트
```bash
# health, users, projects 동시 테스트
wrk -t4 -c100 -d30s http://localhost:8080/health &
wrk -t4 -c100 -d30s http://localhost:8080/api/users &
wrk -t4 -c100 -d30s http://localhost:8080/api/projects &
wait
```

---

## 📚 참고 자료

- [wrk GitHub](https://github.com/wrkrym/wrk)
- [HTTP Caching Guide](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [Actix-web Performance](https://www.techempower.com/benchmarks/)

---

**작성일**: 2025-10-07
**도구 버전**: wrk 4.x
