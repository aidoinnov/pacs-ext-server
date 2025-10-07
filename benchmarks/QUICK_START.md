# 🚀 캐시 성능 비교 빠른 시작 가이드

## 5분 안에 캐시 on/off 성능 비교하기

---

## ✅ 사전 준비 (1분)

### 1. wrk 설치 확인
```bash
wrk --version
```

**이미 설치됨**: ✅ `/opt/homebrew/bin/wrk`

설치 안 되어 있다면:
```bash
brew install wrk
```

### 2. 서버 실행

**터미널 1 - PostgreSQL**:
```bash
cd /Users/aido/Code/pacs-ext-server/infra
docker-compose up -d
```

**터미널 2 - PACS Server**:
```bash
cd /Users/aido/Code/pacs-ext-server/pacs-server
cargo run
```

서버가 시작되면 다음 메시지가 표시됩니다:
```
Starting PACS Extension Server on http://0.0.0.0:8080
Cache enabled: true, TTL: 300s
```

---

## 🧪 테스트 실행 (3분)

### 방법 1: 빠른 자동 테스트 (권장)

**터미널 3 - 벤치마크**:

```bash
cd /Users/aido/Code/pacs-ext-server

# 1. 현재 설정으로 테스트 (캐시 활성화 상태)
./benchmarks/quick_cache_test.sh
```

**결과 예시**:
```
==================================
Quick Cache Performance Test
==================================

✓ Server is running

Current: Cache ENABLED
cache-control: public, max-age=300

Running benchmark (10s)...
Requests/sec: 45230.12
Avg Latency:  1.10ms
Transfer/sec: 6.12MB
```

**캐시 비활성화 테스트**:
```bash
# 2. .env 파일 수정
cd pacs-server
nano .env  # 또는 vim .env

# CACHE_ENABLED=true를 false로 변경
CACHE_ENABLED=false

# 3. 서버 재시작 (터미널 2에서 Ctrl+C 후)
cargo run

# 4. 다시 벤치마크 실행 (터미널 3)
cd ..
./benchmarks/quick_cache_test.sh
```

---

### 방법 2: 수동 wrk 테스트

**캐시 활성화 테스트**:
```bash
# .env에서 CACHE_ENABLED=true 확인
wrk -t4 -c100 -d10s --latency http://localhost:8080/health
```

**캐시 비활성화 테스트**:
```bash
# .env에서 CACHE_ENABLED=false로 변경하고 서버 재시작
wrk -t4 -c100 -d10s --latency http://localhost:8080/health
```

---

## 📊 결과 해석 (1분)

### 주요 지표

| 지표 | 의미 | 좋은 값 |
|------|------|---------|
| **Requests/sec** | 초당 처리 요청 수 | 높을수록 좋음 |
| **Avg Latency** | 평균 응답 시간 | 낮을수록 좋음 |
| **99th %ile** | 99% 요청의 응답 시간 | 낮을수록 좋음 |
| **Transfer/sec** | 초당 전송 데이터량 | 높을수록 좋음 |

### 예상 결과 비교

#### ✅ 캐시 활성화
```
Requests/sec:  45,000 ~ 50,000
Avg Latency:   1.0ms ~ 1.5ms
99th %ile:     8ms ~ 12ms
```

#### ⚠️ 캐시 비활성화
```
Requests/sec:  18,000 ~ 25,000
Avg Latency:   4.0ms ~ 6.0ms
99th %ile:     30ms ~ 45ms
```

### 💡 개선율
- **처리량**: 약 **2배 증가** (100% 향상)
- **레이턴시**: 약 **75% 감소**
- **안정성**: 99th percentile이 **70% 개선**

---

## 🎯 빠른 요약

### 한 줄 명령어로 테스트

```bash
# 캐시 헤더 확인
curl -I http://localhost:8080/health | grep cache-control

# 10초 벤치마크
wrk -t4 -c100 -d10s http://localhost:8080/health | grep "Requests/sec:"
```

### 캐시 on/off 전환

**활성화**:
```bash
# pacs-server/.env
CACHE_ENABLED=true
```

**비활성화**:
```bash
# pacs-server/.env
CACHE_ENABLED=false
```

**적용**: 서버 재시작 필요 (`cargo run`)

---

## 📁 결과 저장 위치

자동 테스트 결과:
```
benchmarks/results/quick_test_ENABLED_*.txt
benchmarks/results/quick_test_DISABLED_*.txt
```

확인:
```bash
ls -lth benchmarks/results/ | head -5
cat benchmarks/results/quick_test_ENABLED_*.txt
```

---

## 🔍 실시간 확인

### 캐시 헤더 실시간 확인
```bash
# 캐시 활성화시
curl -I http://localhost:8080/health
# 출력: cache-control: public, max-age=300

# 캐시 비활성화시
curl -I http://localhost:8080/health
# 출력: cache-control: no-cache, no-store, must-revalidate
```

---

## ⏱️ 전체 소요 시간

1. **사전 준비**: 1분 (서버 시작)
2. **캐시 ON 테스트**: 10초
3. **캐시 설정 변경**: 30초
4. **캐시 OFF 테스트**: 10초
5. **결과 비교**: 1분

**총 소요 시간**: 약 **3분**

---

## 🎓 다음 단계

### 더 자세한 테스트
```bash
./benchmarks/cache_benchmark.sh  # 30분 소요
```

### 다른 엔드포인트 테스트
```bash
wrk -t4 -c100 -d10s http://localhost:8080/api/users
wrk -t4 -c100 -d10s http://localhost:8080/api/projects
```

### 고부하 테스트
```bash
wrk -t8 -c500 -d60s --latency http://localhost:8080/health
```

---

## 📚 참고

- **전체 문서**: `benchmarks/README.md`
- **캐시 구현**: `pacs-server/CACHE_HEADERS.md`
- **검토 보고서**: `pacs-server/CACHE_REVIEW.md`

---

**준비 완료!** 이제 위 단계를 따라 실행하세요! 🚀
