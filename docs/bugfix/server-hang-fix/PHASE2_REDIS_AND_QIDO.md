# 서버 무응답 문제 2차 수정 - Redis 연결 재사용 및 QIDO 병렬 호출 제한

## 📋 개요

**커밋:** `19b2503`  
**날짜:** 2026-01-29  
**관련 커밋:** `0f00fcb` (1차 수정 - DB 풀 타임아웃 설정)

1차 수정 후에도 서버 무응답 문제가 지속되어 추가 조사를 진행했습니다.

---

## 🔍 추가 발견된 문제

### 1. Redis 연결 누수 ⚠️⚠️⚠️ (심각)

#### 문제 코드
```rust
// pacs-server/src/infrastructure/redis/client.rs (수정 전)
pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
    self.client.get_multiplexed_async_connection().await  // ❌ 매번 새 연결 생성!
}
```

#### 문제점
- `get_connection()`이 호출될 때마다 **새로운 연결을 생성**
- View Selection 작업마다 연결 생성:
  - 조회 1회 = 연결 1개
  - 저장 1회 = 연결 1개
  - TTL 연장 1회 = 연결 2개 (조회 + 저장)
- **10회 반복 시 40개 연결 생성** → 메모리 누수

#### 영향
```
사용자 작업 10회:
- Redis 연결: 40개 생성
- 연결이 제대로 닫히지 않으면 누수 발생
- 메모리 사용량 증가
- 최종적으로 서버 무응답
```

---

### 2. 무제한 병렬 QIDO 호출 ⚠️⚠️ (심각)

#### 문제 코드
```rust
// pacs-server/src/presentation/controllers/dicom_gateway_controller.rs (수정 전)
let qido_futures = user_projects.iter().map(|project_id| {
    // QIDO 호출 생성
}).collect();

let qido_results = join_all(qido_futures).await;  // ❌ 모든 프로젝트 동시 호출!
```

#### 문제점
- 사용자가 10개 프로젝트에 속하면 → **10개 QIDO 호출 동시 실행**
- 각 QIDO 호출이 사용하는 리소스:
  - DB 연결 1개 (권한 체크)
  - HTTP 연결 1개 (QIDO-RS 호출)
- **총 20개 연결 동시 사용** (DB 10개 + HTTP 10개)

#### 영향
```
10개 프로젝트 조회 시:
- DB 연결: 10개 동시 사용 (풀 크기 10개 → 고갈)
- HTTP 연결: 10개 동시 사용
- 다른 요청 처리 불가
- 서버 무응답
```

---

## ✅ 적용된 수정사항

### 1. Redis 연결 재사용 패턴

**파일:** `pacs-server/src/infrastructure/redis/client.rs`

#### Before
```rust
#[derive(Clone)]
pub struct RedisConnection {
    client: Arc<RedisClient>,
}

pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
    self.client.get_multiplexed_async_connection().await  // ❌ 매번 새 연결
}
```

#### After
```rust
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RedisConnection {
    client: Arc<RedisClient>,
    /// 재사용 가능한 Multiplexed 연결
    /// Mutex로 보호하여 동시 접근 제어
    connection: Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>,
}

pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
    let mut conn_guard = self.connection.lock().await;
    
    // 기존 연결이 있으면 재사용
    if let Some(conn) = conn_guard.take() {
        // 연결 유효성 검사 (PING)
        let mut test_conn = conn.clone();
        match redis::cmd("PING").query_async::<String>(&mut test_conn).await {
            Ok(_) => {
                // 연결이 유효하면 다시 저장하고 복제본 반환
                *conn_guard = Some(conn.clone());
                return Ok(conn);  // ✅ 재사용
            }
            Err(_) => {
                // 연결이 끊어진 경우 새로 생성
                tracing::warn!("Redis connection lost, reconnecting...");
            }
        }
    }
    
    // 새 연결 생성
    let new_conn = self.client.get_multiplexed_async_connection().await?;
    *conn_guard = Some(new_conn.clone());
    Ok(new_conn)
}
```

#### 개선 효과
- ✅ 연결 생성: 매번 → 최초 1회 + 재연결 시에만
- ✅ 메모리 누수 방지
- ✅ PING으로 연결 유효성 검사
- ✅ 자동 재연결 지원
- ✅ Mutex로 동시 접근 제어

---

### 2. QIDO 병렬 호출 제한

**파일:** `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs`

#### Before
```rust
use futures::future::join_all;

let qido_results = join_all(qido_futures).await;  // ❌ 무제한 병렬 실행
```

#### After
```rust
use futures::stream::{self, StreamExt};

// 병렬 호출 제한: 최대 5개씩만 동시 실행하여 연결 풀 고갈 방지
let qido_results = stream::iter(qido_futures)
    .buffer_unordered(5)  // ✅ 최대 5개 동시 실행
    .collect::<Vec<_>>()
    .await;
```

#### 적용 위치
1. **Studies 조회** (Line ~925)
2. **Series 조회** (Line ~2860)

#### 개선 효과
- ✅ 10개 프로젝트 → 5개씩 2번 실행
- ✅ DB 연결 사용: 최대 10개 → 최대 5개
- ✅ HTTP 연결 사용: 최대 10개 → 최대 5개
- ✅ 연결 풀 고갈 방지
- ✅ 다른 요청 처리 가능

---

## 📊 성능 개선 비교

### Before (문제 상황)
```
사용자가 10개 프로젝트에 속한 경우:

Redis 연결:
- View Selection 작업 10회 = 40개 연결 생성 ❌
- 연결 누수 발생

QIDO 호출:
- 10개 프로젝트 = 10개 동시 호출 ❌
- DB 연결: 10개 동시 사용 (풀 고갈)
- HTTP 연결: 10개 동시 사용

결과: 연결 풀 고갈 → 서버 무응답
```

### After (수정 후)
```
사용자가 10개 프로젝트에 속한 경우:

Redis 연결:
- 1개 연결 재사용 ✅
- 연결 누수 방지

QIDO 호출:
- 5개씩 2번 실행 ✅
- DB 연결: 최대 5개 사용 (풀 안정)
- HTTP 연결: 최대 5개 사용

결과: 연결 풀 안정적 유지 → 서버 정상 작동
```

---

## 🔧 기술적 세부사항

### Redis MultiplexedConnection 재사용 패턴

#### 왜 MultiplexedConnection인가?
- **Multiplexing**: 하나의 연결로 여러 명령을 동시에 처리 가능
- **비동기 지원**: `async/await` 패턴과 완벽 호환
- **Clone 가능**: 연결을 복제하여 여러 곳에서 사용 가능

#### Arc<Mutex<Option<Connection>>> 패턴
```rust
connection: Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>
```

- **Arc**: 여러 스레드에서 공유 가능
- **Mutex**: 동시 접근 제어 (한 번에 하나의 작업만)
- **Option**: 연결이 없을 수도 있음 (초기화 전, 재연결 중)

#### 연결 유효성 검사
```rust
match redis::cmd("PING").query_async::<String>(&mut test_conn).await {
    Ok(_) => { /* 연결 유효 */ }
    Err(_) => { /* 재연결 필요 */ }
}
```

- PING 명령으로 연결 상태 확인
- 실패 시 자동으로 새 연결 생성
- 네트워크 단절 시에도 안정적 복구

---

### Stream Buffer Unordered 패턴

#### buffer_unordered(n)의 동작
```rust
stream::iter(futures)
    .buffer_unordered(5)  // 최대 5개 동시 실행
    .collect()
    .await
```

**동작 방식:**
1. 처음 5개 future 시작
2. 하나가 완료되면 다음 future 시작
3. 항상 최대 5개만 동시 실행
4. 완료 순서는 보장 안 됨 (unordered)

**장점:**
- 연결 풀 사용량 제어
- 메모리 사용량 제한
- 다른 요청 처리 가능

---

## 📈 리소스 사용량 상세 비교

### Redis 연결 사용량

| 작업 | Before | After | 개선 |
|------|--------|-------|------|
| View Selection 조회 1회 | 1개 생성 | 재사용 | ✅ -1 |
| View Selection 저장 1회 | 1개 생성 | 재사용 | ✅ -1 |
| TTL 연장 1회 | 2개 생성 | 재사용 | ✅ -2 |
| **10회 반복** | **40개 생성** | **1개 재사용** | ✅ **-39** |

### QIDO 호출 리소스 사용량

| 프로젝트 수 | Before (동시) | After (최대) | 개선 |
|------------|--------------|-------------|------|
| 5개 | DB 5 + HTTP 5 | DB 5 + HTTP 5 | - |
| 10개 | DB 10 + HTTP 10 | DB 5 + HTTP 5 | ✅ 50% 감소 |
| 20개 | DB 20 + HTTP 20 | DB 5 + HTTP 5 | ✅ 75% 감소 |

---

## 🧪 테스트 방법

### 1. Redis 연결 재사용 확인

```bash
# Redis 연결 수 모니터링
redis-cli CLIENT LIST | wc -l

# 작업 전: 1개
# View Selection 10회 작업 후
# Before: 40개 이상
# After: 1-2개 (재사용)
```

### 2. QIDO 병렬 호출 제한 확인

```bash
# 서버 로그 확인
tail -f /tmp/pacs_server.log | grep "QIDO"

# 출력 예시 (10개 프로젝트):
# Gateway: 프로젝트 1 QIDO 성공
# Gateway: 프로젝트 2 QIDO 성공
# Gateway: 프로젝트 3 QIDO 성공
# Gateway: 프로젝트 4 QIDO 성공
# Gateway: 프로젝트 5 QIDO 성공
# (5개 완료 후 다음 5개 시작)
# Gateway: 프로젝트 6 QIDO 성공
# ...
```

### 3. 서버 안정성 테스트

**테스트 시나리오:**
```
1. 스터디목록 출력 → 시리즈 목록 출력 → 인스턴스 목록 출력
2. 스터디목록 출력 → 시리즈 목록 출력 → 인스턴스 목록 출력
3. view selection 생성
4. 스터디목록 출력 → 시리즈 목록 출력 → 인스턴스 목록 출력
5. annotation 목록 출력
6. 1-5 반복 10회
```

**예상 결과:**
- ✅ 서버가 계속 응답
- ✅ 응답 속도 안정적
- ✅ 메모리 사용량 안정적
- ✅ 재시작 불필요

---

## 📝 관련 파일

### 수정된 파일
1. `pacs-server/src/infrastructure/redis/client.rs` (+44줄, -4줄)
   - Redis 연결 재사용 패턴 적용

2. `pacs-server/src/presentation/controllers/dicom_gateway_controller.rs` (+15줄, -3줄)
   - QIDO 병렬 호출 제한 (2개 위치)

### 관련 문서
- `docs/bugfix/server-hang-fix/README.md` - 전체 개요
- `docs/bugfix/server-hang-fix/technical-details.md` - 1차 수정 기술 문서

---

## 🎯 결론

### 해결된 문제
1. ✅ Redis 연결 누수 → 연결 재사용으로 해결
2. ✅ QIDO 무제한 병렬 호출 → 최대 5개로 제한
3. ✅ 연결 풀 고갈 → 리소스 사용량 제어

### 성능 개선
- Redis 연결: 40개 생성 → 1개 재사용 (97.5% 감소)
- QIDO 동시 호출: 무제한 → 최대 5개
- DB 연결 사용: 최대 10개 → 최대 5개 (50% 감소)

### 다음 단계
- ✅ 서버 배포 완료
- ⏳ 프로덕션 환경 모니터링
- ⏳ 추가 최적화 검토 (필요 시)

---

**작성일:** 2026-01-29
**작성자:** AI Assistant
**커밋:** `19b2503`

