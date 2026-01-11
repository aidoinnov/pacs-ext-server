# Viewer BFF API 테스트 요약

## 개요

Viewer BFF (Backend-for-Frontend) API의 단위 테스트 및 성능 벤치마크 문서입니다.

## 테스트 파일

### 1. DTO 단위 테스트 (`viewer_dto_test.rs`)

#### 테스트 케이스

##### DICOMweb JSON 파싱
- ✅ `test_viewer_study_meta_from_dicomweb_json` - Study 메타데이터 파싱
- ✅ `test_viewer_series_meta_from_dicomweb_json` - Series 메타데이터 파싱

##### DTO 직렬화
- ✅ `test_viewer_study_meta_request_serialization` - Study 요청 직렬화
- ✅ `test_viewer_series_meta_request_serialization` - Series 요청 직렬화

#### 실행 방법

```bash
# DTO 테스트 실행
cargo test --test viewer_dto_test

# 상세 로그 포함
cargo test --test viewer_dto_test -- --nocapture
```

#### 테스트 결과

```
running 4 tests
test test_viewer_series_meta_request_serialization ... ok
test test_viewer_study_meta_request_serialization ... ok
test test_viewer_study_meta_from_dicomweb_json ... ok
test test_viewer_series_meta_from_dicomweb_json ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

### 2. 성능 벤치마크 (`benchmark_viewer_api.sh`)

#### 벤치마크 시나리오

##### Study Meta 성능
- ⏱️ 10개 Study UID (< 5초)
- ⏱️ 50개 Study UID (< 15초)
- ⏱️ 100개 Study UID (< 30초)

##### Series Meta 성능
- ⏱️ 50개 Series UID (< 15초)
- ⏱️ 200개 Series UID (< 60초)

#### 실행 방법

```bash
# 1. 서버 실행
cd pacs-server
cargo run

# 2. JWT 토큰 획득 (로그인 API 사용)
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.access_token')

# 3. 벤치마크 실행
./scripts/benchmark_viewer_api.sh "$TOKEN"
```

#### 성능 기준

| UID 개수 | 예상 시간 | 최대 허용 시간 |
|---------|----------|--------------|
| 10 Study | ~2초 | 5초 |
| 50 Study | ~8초 | 15초 |
| 100 Study | ~15초 | 30초 |
| 50 Series | ~8초 | 15초 |
| 200 Series | ~30초 | 60초 |

#### 벤치마크 결과 예시

```
ℹ️  Viewer API 성능 벤치마크 시작

=== 테스트 1: 10개 Study UID ===

ℹ️  테스트: 10개 Study UID 조회
⏱️  응답 시간: 2.345초 (총 2.350초)
  HTTP 상태: 200
✅ 성능 기준 통과 (< 5.0초)

=== 테스트 2: 50개 Study UID ===

ℹ️  테스트: 50개 Study UID 조회
⏱️  응답 시간: 8.123초 (총 8.130초)
  HTTP 상태: 200
✅ 성능 기준 통과 (< 15.0초)

...

✅ 벤치마크 완료!
```

---

## 테스트 환경 설정

### 필수 환경 변수

```bash
export DATABASE_URL="postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
```

### QIDO 서버 (성능 벤치마크용)

성능 벤치마크는 실제 dcm4chee QIDO 서버가 필요합니다:

```bash
# Docker로 dcm4chee 실행
docker-compose up -d dcm4chee
```

---

## 빠른 시작

### 1. DTO 단위 테스트 실행

```bash
cd pacs-server
cargo test --test viewer_dto_test
```

### 2. 성능 벤치마크 실행

```bash
# 서버 실행
cargo run &

# JWT 토큰 획득
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.access_token')

# 벤치마크 실행
./scripts/benchmark_viewer_api.sh "$TOKEN"
```

---

## 주의사항

1. **DTO 단위 테스트**는 외부 의존성 없이 실행 가능
2. **성능 벤치마크**는 실제 QIDO 서버와 실행 중인 API 서버가 필요
3. 성능 벤치마크는 네트워크 상태에 따라 결과가 달라질 수 있음
4. CI/CD 환경에서는 DTO 단위 테스트만 실행 권장

---

## CI/CD 통합

### GitHub Actions 예시

```yaml
- name: Run Viewer DTO Tests
  run: |
    cd pacs-server
    cargo test --test viewer_dto_test
```

---

## 트러블슈팅

### 벤치마크 스크립트 실행 오류

```bash
# jq 설치 확인
which jq || brew install jq  # macOS
which jq || sudo apt-get install jq  # Ubuntu

# bc 설치 확인
which bc || brew install bc  # macOS
which bc || sudo apt-get install bc  # Ubuntu
```

### QIDO 서버 연결 실패

```bash
# dcm4chee 상태 확인
docker ps | grep dcm4chee

# dcm4chee 재시작
docker-compose restart dcm4chee

# 로그 확인
docker-compose logs -f dcm4chee
```

### JWT 토큰 만료

```bash
# 새 토큰 발급
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.access_token')

echo "New token: $TOKEN"
```

---

## 추가 개선 사항

### 향후 계획

1. **E2E 통합 테스트** - 실제 DB와 QIDO 서버를 사용한 통합 테스트
2. **부하 테스트** - 동시 요청 처리 성능 측정
3. **메모리 프로파일링** - 대량 데이터 처리 시 메모리 사용량 분석
4. **캐싱 효과 측정** - Redis 캐시 적용 전후 성능 비교

### 테스트 커버리지

현재 커버리지:
- DTO 파싱 로직: ✅ 100%
- 요청/응답 직렬화: ✅ 100%
- 컨트롤러 로직: ⚠️ 수동 테스트 필요
- RBAC 통합: ⚠️ 수동 테스트 필요

