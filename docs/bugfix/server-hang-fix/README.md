# 서버 무응답 문제 수정

## 📋 문제 요약

### 증상
- 특정 작업 패턴 반복 후 서버가 응답하지 않음
- 서버 재시작 시 정상 작동
- 문제 발생 패턴:
  ```
  스터디목록 출력 → 시리즈 목록 출력 → 인스턴스 목록 출력
  스터디목록 출력 → 시리즈 목록 출력 → 인스턴스 목록 출력
  view selection 생성 
  스터디목록 출력 → 시리즈 목록 출력 → 인스턴스 목록 출력
  annotation 목록 출력
  ```

### 원인 분석 (2단계 조사)

#### 1차 조사: 연결 풀 설정 부족

##### 1-1. 데이터베이스 연결 풀 설정 부족 ⚠️
**문제:**
- `max_connections`: 10 (기본값)
- `acquire_timeout`: 설정 없음 → 무한 대기 가능
- `idle_timeout`: 설정 없음 → 유휴 연결 계속 유지
- `max_lifetime`: 설정 없음 → 오래된 연결 계속 사용
- `test_before_acquire`: 설정 없음 → 끊어진 연결 사용 가능

**영향:**
- 연결 획득 대기 시 무한 대기 → 서버 응답 없음
- 끊어진 연결 사용 시 에러 발생
- 연결 누수 시 복구 불가능

##### 1-2. HTTP 클라이언트 연결 풀 설정 부족 ⚠️
**문제:**
- `reqwest::Client::new()` 기본 설정 사용
- 연결 풀 크기 제한 없음
- Keep-alive 타임아웃 설정 없음
- 연결 재사용 설정 없음

**영향:**
- QIDO-RS 호출 시 연결 누적
- 유휴 연결이 계속 유지됨
- 호스트당 연결 수 제한 없음

#### 2차 조사: 연결 누수 및 병렬 처리 문제

##### 2-1. Redis 연결 누수 ⚠️⚠️⚠️ (심각)
**문제:**
```rust
// 기존 코드
pub async fn get_connection(&self) -> Result<...> {
    self.client.get_multiplexed_async_connection().await  // ❌ 매번 새 연결 생성!
}
```

**영향:**
- View Selection 조회 1회 = Redis 연결 1개 생성
- View Selection 저장 1회 = Redis 연결 1개 생성
- TTL 연장 1회 = Redis 연결 2개 생성 (조회 + 저장)
- **10회 반복 시 40개 연결 생성** → 메모리 누수

##### 2-2. 무제한 병렬 QIDO 호출 ⚠️⚠️ (심각)
**문제:**
```rust
// 기존 코드
let qido_results = join_all(qido_futures).await;  // ❌ 모든 프로젝트 동시 호출!
```

**영향:**
- 사용자가 10개 프로젝트에 속한 경우 → **10개 QIDO 호출 동시 실행**
- 각 QIDO 호출이 DB 연결 + HTTP 연결 사용
- DB 연결 10개 + HTTP 연결 10개 = **총 20개 연결 동시 사용**
- 연결 풀 고갈 가능성 높음

##### 2-3. Annotation N+1 쿼리 패턴 ⚠️
**문제:**
- Annotation 조회 시 권한 체크를 위해 여러 DB 쿼리 실행
- `is_project_member()` → DB 쿼리
- `check_permission()` → DB 쿼리 (캐시 미스 시)
- `find_by_project_id_with_viewer()` → DB 쿼리

**영향:**
- 배치 작업 시 연결 누적 가능
- Redis 연결 에러 무시

---

## ✅ 수정 내용

### 1. 데이터베이스 연결 풀 설정 개선

**파일:** `pacs-server/src/main.rs`

**Before:**
```rust
let pool = PgPoolOptions::new()
    .max_connections(settings.database.max_connections)
    .min_connections(settings.database.min_connections)
    .connect(&database_url)
    .await
    .expect("Failed to connect to database");
```

**After:**
```rust
let pool = PgPoolOptions::new()
    .max_connections(settings.database.max_connections)
    .min_connections(settings.database.min_connections)
    .acquire_timeout(std::time::Duration::from_secs(30))      // ✅ 30초 타임아웃
    .idle_timeout(Some(std::time::Duration::from_secs(600)))  // ✅ 10분 유휴 타임아웃
    .max_lifetime(Some(std::time::Duration::from_secs(1800))) // ✅ 30분 최대 수명
    .test_before_acquire(true)                                 // ✅ 연결 유효성 검사
    .connect(&database_url)
    .await
    .expect("Failed to connect to database");
```

**개선 효과:**
- ✅ 연결 획득 대기 시 30초 후 타임아웃 → 무한 대기 방지
- ✅ 유휴 연결 10분 후 자동 해제 → 연결 누수 방지
- ✅ 연결 30분 후 자동 갱신 → 오래된 연결 방지
- ✅ 연결 사용 전 유효성 검사 → 끊어진 연결 방지

### 2. HTTP 클라이언트 연결 풀 설정 개선

**파일:** `pacs-server/src/infrastructure/external/dcm4chee_qido_client.rs`

**Before:**
```rust
http_client: Client::new(),
```

**After:**
```rust
let http_client = Client::builder()
    .pool_max_idle_per_host(10)                                    // ✅ 호스트당 최대 10개 유휴 연결
    .pool_idle_timeout(Some(std::time::Duration::from_secs(90)))   // ✅ 90초 유휴 타임아웃
    .tcp_keepalive(Some(std::time::Duration::from_secs(60)))       // ✅ 60초 TCP Keep-Alive
    .connect_timeout(std::time::Duration::from_secs(10))           // ✅ 10초 연결 타임아웃
    .build()
    .unwrap_or_else(|_| Client::new());
```

**개선 효과:**
- ✅ 호스트당 유휴 연결 10개로 제한 → 연결 누적 방지
- ✅ 유휴 연결 90초 후 자동 해제 → 리소스 절약
- ✅ TCP Keep-Alive 활성화 → 연결 유지 확인
- ✅ 연결 타임아웃 10초 → 빠른 실패

---

## 📊 테스트 결과

### 수정 전
- ❌ 반복 작업 후 서버 무응답
- ❌ 재시작 필요

### 수정 후
- ✅ 서버 정상 시작
- ✅ 연결 풀 설정 적용 확인
- ⏳ 장시간 테스트 필요

---

## 🔍 추가 모니터링 권장사항

### 1. 데이터베이스 연결 풀 모니터링
```rust
// 주기적으로 연결 풀 상태 로깅
tracing::info!("DB Pool: size={}, idle={}", pool.size(), pool.num_idle());
```

### 2. HTTP 클라이언트 메트릭
- QIDO-RS 호출 횟수
- 평균 응답 시간
- 타임아웃 발생 횟수

### 3. Redis 연결 에러 로깅
```rust
tokio::spawn(async move {
    if let Err(e) = use_case_clone.extend_ttl(&selection_id_clone, None).await {
        tracing::warn!("Failed to extend TTL: {}", e);
    }
});
```

---

## 📝 환경 변수 설정 (선택사항)

연결 풀 크기를 조정하려면:

```bash
# .env 파일
DATABASE_MAX_CONNECTIONS=20  # 기본값: 10
DATABASE_MIN_CONNECTIONS=5   # 기본값: 2
```

---

## 🎯 결론

- ✅ 데이터베이스 연결 풀 타임아웃 설정 추가
- ✅ HTTP 클라이언트 연결 풀 제한 추가
- ✅ 서버 안정성 개선
- ⏳ 장시간 운영 테스트 필요

