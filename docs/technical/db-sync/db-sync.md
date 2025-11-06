완벽합니다 👏 — 그 지점이 바로 **“RBAC Sync Engine”이 단순한 배치(job runner)”가 아니라 “컨트롤 가능한 동기화 서비스”**로 발전하는 핵심이에요.

말씀하신 기능(강제 실행, 상태 확인, 스케줄 관리, 중지/재개, 로그 조회 등)을 기준으로,
Rust 기반으로 **“API 제어가 가능한 Sync Server”** 아키텍처를 아래처럼 설계할 수 있습니다.

---

## 🧩 **1️⃣ 목표 개념**

> 단순 cron 기반이 아니라,
> **HTTP API로 제어 가능한 동기화 엔진 (Rust 서비스)**

**핵심 아이디어:**

* 내부에서 스케줄러가 동작하지만,
* REST API를 통해 외부에서 동작을 “조회/제어”할 수 있음

---

## ⚙️ **2️⃣ 주요 기능 요구사항**

| 기능                               | 설명                              | 예시 Endpoint                                |
| -------------------------------- | ------------------------------- | ------------------------------------------ |
| **① 강제 동기화 실행 (manual trigger)** | API 호출 시 즉시 sync 수행             | `POST /sync/run`                           |
| **② 상태 조회 (status)**             | 현재 sync 상태, 마지막 실행, 다음 예정 시각 조회 | `GET /sync/status`                         |
| **③ 스케줄 조회/변경**                  | 현재 interval 확인 및 수정             | `GET /sync/schedule`, `PUT /sync/schedule` |
| **④ 일시 중지 / 재개**                 | 내부 스케줄러 중지 또는 재개                | `POST /sync/pause`, `POST /sync/resume`    |
| **⑤ 최근 수행 이력 조회**                | 최근 N회 수행 결과 (성공/실패, 처리건수, 시간)   | `GET /sync/history`                        |
| **⑥ 헬스체크 / 메트릭**                 | 동기화 엔진의 상태 및 Prometheus 지표 제공   | `GET /health`, `GET /metrics`              |

---

## 🧱 **3️⃣ 시스템 구조**

```text
                +--------------------+
                | DCM4CHEE DB        |
                +---------+----------+
                          |
                    (delta query)
                          |
    +------------------------------------------------+
    |              RBAC Sync Engine (Rust)           |
    |------------------------------------------------|
    |  Modules:                                      |
    |   • scheduler.rs  → background loop             |
    |   • sync_worker.rs → sync_once()                |
    |   • api.rs        → REST API                    |
    |   • state.rs      → 상태 관리(shared state)    |
    |   • history.rs    → 실행 로그 관리              |
    |   • config.rs, logger.rs, metrics.rs            |
    +------------------------------------------------+
                          |
                          v
                +--------------------+
                | RBAC DB            |
                +--------------------+
```

---

## ⚙️ **4️⃣ 동작 예시 시퀀스**

1. 서버 시작 시:

   * `.env`에서 interval(예: 5초) 로드
   * tokio background task로 주기적 sync 시작

2. 관리자는 REST API로 제어:

   ```bash
   # 현재 상태 확인
   curl http://localhost:8080/sync/status

   # 강제 동기화 실행
   curl -X POST http://localhost:8080/sync/run

   # 스케줄 30초로 변경
   curl -X PUT -H "Content-Type: application/json" \
        -d '{"interval_sec": 30}' http://localhost:8080/sync/schedule

   # 일시 중지 / 재개
   curl -X POST http://localhost:8080/sync/pause
   curl -X POST http://localhost:8080/sync/resume
   ```

---

## 🧩 **5️⃣ 기술 스택**

| 영역            | Crate                                       | 용도                    |
| ------------- | ------------------------------------------- | --------------------- |
| Web Framework | **axum** or **warp**                        | REST API 서버           |
| Scheduler     | **tokio::time**                             | async interval task   |
| Shared State  | **tokio::sync::RwLock** or **Arc<Mutex<>>** | API ↔ worker 간 상태 공유  |
| DB Access     | **sqlx**                                    | DCM4CHEE / RBAC DB 접근 |
| Metrics       | **prometheus** + **axum-prometheus**        | 모니터링                  |
| Logging       | **tracing**                                 | 구조적 로그                |
| Config        | **dotenvy**, **serde**                      | 설정값 로드                |

---

## 🧩 **6️⃣ 데이터 구조 예시**

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SyncStatus {
    pub is_running: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub interval_sec: u64,
    pub last_result: Option<SyncResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    pub success: bool,
    pub processed: usize,
    pub duration_ms: u128,
    pub error: Option<String>,
}
```

---

## 💻 **7️⃣ 간단한 API 예시 (axum)**

```rust
use axum::{Router, routing::{get, post, put}, Json};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone)]
struct SyncStatus {
    is_running: bool,
    last_run: Option<String>,
    next_run: Option<String>,
    interval_sec: u64,
}

#[derive(Deserialize)]
struct UpdateSchedule { interval_sec: u64 }

async fn get_status(state: Arc<Mutex<SyncStatus>>) -> Json<SyncStatus> {
    Json(state.lock().unwrap().clone())
}

async fn update_schedule(Json(payload): Json<UpdateSchedule>, state: Arc<Mutex<SyncStatus>>) {
    state.lock().unwrap().interval_sec = payload.interval_sec;
    println!("Schedule updated to {}s", payload.interval_sec);
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(SyncStatus {
        is_running: true,
        last_run: Some(Utc::now().to_rfc3339()),
        next_run: None,
        interval_sec: 5,
    }));

    let app = Router::new()
        .route("/sync/status", get({
            let s = state.clone(); move || get_status(s)
        }))
        .route("/sync/schedule", put({
            let s = state.clone(); move |j| update_schedule(j, s)
        }));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await.unwrap();
}
```

---

## 🧪 **8️⃣ 운영 및 모니터링 포인트**

| 항목         | 설명                                                  |
| ---------- | --------------------------------------------------- |
| `/metrics` | Prometheus endpoint (처리건수, 실패율, 평균시간 등)             |
| `/health`  | 헬스체크용 (scheduler 활성 상태 확인)                          |
| 로그         | `tracing_subscriber` 이용해 구조적 로그 출력 (json / console) |
| 상태 캐싱      | 최근 N회 수행 이력 in-memory 저장 (ring buffer or SQLite)    |

---

## ✅ **정리**

| 항목            | 권장 방향                            |
| ------------- | -------------------------------- |
| **프로젝트 형태**   | 별도 서비스 (Rust, axum API 내장형)      |
| **동기화 주기 제어** | 내부 scheduler + API 기반 설정         |
| **상태 관리**     | Arc<Mutex<State>>로 중앙 관리         |
| **모니터링**      | `/status`, `/metrics` API 제공     |
| **운영 확장성**    | 향후 RBAC Admin UI에서 API 호출로 제어 가능 |

---

원하신다면 바로 다음 단계로

> 🔹 **이 API 기반 Sync Engine의 구조 + 라우팅 + 상태관리 전체 코드 스켈레톤**
> (프로덕션 수준 기본구조: main.rs, api.rs, scheduler.rs, sync.rs, state.rs)

을 정리해드릴 수 있습니다.
그 버전으로 이어서 볼까요?
