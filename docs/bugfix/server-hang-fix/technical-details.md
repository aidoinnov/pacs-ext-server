# 서버 무응답 문제 - 기술 상세

## 🔍 리소스 고갈 분석

### 1. 데이터베이스 연결 풀 고갈 시나리오

#### 문제 발생 과정
```
1. 사용자 요청: GET /api/dicom/studies
   → DB 연결 1개 획득
   
2. 사용자 요청: GET /api/dicom/series/{study_uid}
   → DB 연결 1개 획득
   
3. 사용자 요청: GET /api/dicom/instances/{study_uid}/{series_uid}
   → DB 연결 1개 획득
   
4. 반복... (10회)
   → DB 연결 10개 모두 사용 중
   
5. 새로운 요청 도착
   → 연결 획득 대기... (무한 대기)
   → 서버 응답 없음 ❌
```

#### 연결 누수 가능성
- **RBAC 평가**: 각 DICOM 리소스 접근 시 DB 쿼리
- **캐시 조회**: Capability, Permission, Role 캐시 조회 시 DB 연결
- **Annotation 조회**: 대량 annotation 조회 시 연결 유지
- **View Selection**: Redis 실패 시 DB 폴백 (구현되지 않음)

### 2. HTTP 클라이언트 연결 누적 시나리오

#### QIDO-RS 호출 패턴
```rust
// dicom_gateway_controller.rs

// 1. Studies 조회 (프로젝트별 병렬 호출)
let qido_futures = user_projects.iter().map(|project_id| {
    qido.qido_studies_with_bearer(bearer_opt.as_deref(), qido_params).await
});
let qido_results = join_all(qido_futures).await;

// 2. Series 조회 (Study별 병렬 호출)
for study_uid in study_uids {
    qido.qido_series_with_bearer(bearer_opt.as_deref(), &study_uid, qido_params).await
}

// 3. Instances 조회
qido.qido_instances_with_bearer(bearer_opt.as_deref(), &study_uid, &series_uid, qido_params).await
```

#### 연결 누적 계산
- 프로젝트 3개 × Studies 조회 = 3개 연결
- Study 10개 × Series 조회 = 10개 연결
- Series 50개 × Instances 조회 = 50개 연결
- **총 63개 HTTP 연결 동시 사용**

**문제:**
- 기본 `reqwest::Client`는 연결 풀 크기 제한 없음
- 유휴 연결이 계속 유지됨
- 메모리 및 파일 디스크립터 고갈 가능

### 3. 비동기 작업 누적

#### View Selection TTL 연장
```rust
// view_selection_controller.rs:149
tokio::spawn(async move {
    let _ = use_case_clone.extend_ttl(&selection_id_clone, None).await;
});
```

**문제:**
- `tokio::spawn`으로 생성된 작업이 추적되지 않음
- Redis 연결 실패 시 에러 무시 (`let _`)
- 작업이 계속 누적되면 메모리 증가

---

## ✅ 수정 내용 상세

### 1. sqlx PgPoolOptions 설정

#### acquire_timeout
```rust
.acquire_timeout(std::time::Duration::from_secs(30))
```
- **목적**: 연결 획득 대기 시간 제한
- **효과**: 30초 후 타임아웃 에러 반환 → 무한 대기 방지
- **에러 처리**: HTTP 500 Internal Server Error 반환

#### idle_timeout
```rust
.idle_timeout(Some(std::time::Duration::from_secs(600)))
```
- **목적**: 유휴 연결 자동 해제
- **효과**: 10분간 사용되지 않은 연결 자동 종료
- **이점**: 연결 누수 방지, 리소스 절약

#### max_lifetime
```rust
.max_lifetime(Some(std::time::Duration::from_secs(1800)))
```
- **목적**: 연결 최대 수명 제한
- **효과**: 30분 후 연결 자동 갱신
- **이점**: 오래된 연결으로 인한 문제 방지 (네트워크 변경, DB 재시작 등)

#### test_before_acquire
```rust
.test_before_acquire(true)
```
- **목적**: 연결 사용 전 유효성 검사
- **효과**: `SELECT 1` 쿼리로 연결 확인
- **이점**: 끊어진 연결 사용 방지
- **비용**: 약간의 오버헤드 (1-2ms)

### 2. reqwest Client 설정

#### pool_max_idle_per_host
```rust
.pool_max_idle_per_host(10)
```
- **목적**: 호스트당 최대 유휴 연결 수 제한
- **효과**: QIDO-RS 서버당 최대 10개 유휴 연결
- **이점**: 연결 누적 방지, 메모리 절약

#### pool_idle_timeout
```rust
.pool_idle_timeout(Some(std::time::Duration::from_secs(90)))
```
- **목적**: 유휴 연결 자동 해제
- **효과**: 90초간 사용되지 않은 연결 자동 종료
- **이점**: 리소스 절약

#### tcp_keepalive
```rust
.tcp_keepalive(Some(std::time::Duration::from_secs(60)))
```
- **목적**: TCP Keep-Alive 활성화
- **효과**: 60초마다 연결 유지 확인
- **이점**: 끊어진 연결 조기 감지

#### connect_timeout
```rust
.connect_timeout(std::time::Duration::from_secs(10))
```
- **목적**: 연결 타임아웃 설정
- **효과**: 10초 내 연결 실패 시 에러 반환
- **이점**: 빠른 실패, 사용자 경험 개선

---

## 📊 성능 영향 분석

### 데이터베이스 연결 풀

| 설정 | Before | After | 영향 |
|------|--------|-------|------|
| max_connections | 10 | 10 | 변경 없음 |
| min_connections | 2 | 2 | 변경 없음 |
| acquire_timeout | ∞ | 30s | ✅ 무한 대기 방지 |
| idle_timeout | ∞ | 10m | ✅ 연결 누수 방지 |
| max_lifetime | ∞ | 30m | ✅ 오래된 연결 방지 |
| test_before_acquire | false | true | ⚠️ 약간의 오버헤드 |

### HTTP 클라이언트

| 설정 | Before | After | 영향 |
|------|--------|-------|------|
| pool_max_idle_per_host | ∞ | 10 | ✅ 연결 누적 방지 |
| pool_idle_timeout | ∞ | 90s | ✅ 리소스 절약 |
| tcp_keepalive | 비활성화 | 60s | ✅ 연결 유지 확인 |
| connect_timeout | ∞ | 10s | ✅ 빠른 실패 |

---

## 🔧 추가 개선 권장사항

### 1. 연결 풀 크기 증가 (필요 시)
```bash
# .env
DATABASE_MAX_CONNECTIONS=20  # 10 → 20
DATABASE_MIN_CONNECTIONS=5   # 2 → 5
```

**고려사항:**
- PostgreSQL `max_connections` 설정 확인 (기본값: 100)
- 서버 메모리 용량 확인 (연결당 약 10MB)

### 2. 연결 풀 모니터링 추가
```rust
// main.rs
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        tracing::info!(
            "DB Pool: size={}, idle={}, active={}",
            pool.size(),
            pool.num_idle(),
            pool.size() - pool.num_idle()
        );
    }
});
```

### 3. QIDO-RS 호출 최적화
- 병렬 호출 수 제한 (예: `futures::stream::iter().buffer_unordered(5)`)
- 캐시 활용 (Study/Series 메타데이터)
- 페이지네이션 개선

---

## 🎯 결론

### 수정 효과
- ✅ 데이터베이스 연결 풀 타임아웃 설정 → 무한 대기 방지
- ✅ HTTP 클라이언트 연결 풀 제한 → 연결 누적 방지
- ✅ 서버 안정성 개선

### 남은 과제
- ⏳ 장시간 운영 테스트
- ⏳ 연결 풀 모니터링 추가
- ⏳ QIDO-RS 호출 최적화

