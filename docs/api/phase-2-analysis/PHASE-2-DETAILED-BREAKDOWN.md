# 📅 Phase 2 상세 분석: 버전 관리 + 실시간 동기화

**목적:** Phase 2를 더 세분화하여 각 기능별 구현 계획 제시  
**대상:** 백엔드 개발자, 아키텍트  
**기간:** 2-3주 (총 4개 기능)

---

## 🎯 Phase 2 전체 구조

```
Phase 2 (2-3주)
│
├─ 2-1: Version Conflict 처리 (3-4일) ⭐ 필수
│   ├─ 2-1-1: Optimistic Locking 구현
│   ├─ 2-1-2: 버전 충돌 감지
│   └─ 2-1-3: 409 Conflict 응답 처리
│
├─ 2-2: HEAD 요청 (1-2일) ⭐ 권장
│   ├─ 2-2-1: HEAD 핸들러 추가
│   ├─ 2-2-2: 응답 헤더 최적화
│   └─ 2-2-3: 캐시 검증 로직
│
├─ 2-3: WebSocket 실시간 동기화 (5-7일) ⭐ 권장
│   ├─ 2-3-1: WebSocket 서버 구축
│   ├─ 2-3-2: 이벤트 브로드캐스팅
│   ├─ 2-3-3: 클라이언트 구독 관리
│   └─ 2-3-4: 재연결 처리
│
└─ 2-4: Collaborative Lock (3-4일) ⭐ 권장
    ├─ 2-4-1: Lock 테이블 생성
    ├─ 2-4-2: Lock 획득/해제 로직
    ├─ 2-4-3: 타임아웃 처리
    └─ 2-4-4: 편집자 표시 (Presence)
```

---

## 2-1️⃣ Version Conflict 처리 (3-4일) ⭐ 필수

### 목표
- 동시 편집 시 데이터 손실 방지
- Optimistic Locking 구현
- 버전 충돌 감지 및 처리

### 2-1-1: Optimistic Locking 구현

**현재 상태:**
```
User A: PUT /api/annotations/1 (v1 → v2)
User B: PUT /api/annotations/1 (v1 → v2)
결과: User B의 변경사항이 User A를 덮어씀 ❌
```

**개선 방안:**
```
PUT /api/annotations/1
{
  "baseVersion": 1,  ← 클라이언트가 알고 있는 버전
  "updates": {
    "description": "Updated by User A"
  }
}

서버 검증:
├─ 현재 버전 == baseVersion?
│  ├─ YES → 업데이트 (v2로 증가)
│  └─ NO  → 409 Conflict 반환
```

**구현 계획:**
- [ ] `version` 필드 추가 (auto-increment)
- [ ] `baseVersion` 검증 로직
- [ ] 409 Conflict 응답 처리
- [ ] 클라이언트 재시도 로직

**예상 시간:** 1-2일

### 2-1-2: 버전 충돌 감지

**데이터 모델 변경:**
```sql
ALTER TABLE annotation_annotation
ADD COLUMN version INTEGER DEFAULT 1;

-- 인덱스 추가 (성능 최적화)
CREATE INDEX idx_annotation_version 
ON annotation_annotation(id, version);
```

**충돌 감지 로직:**
```rust
async fn update_annotation(
    id: i32,
    base_version: i32,  // 클라이언트 버전
    updates: UpdateRequest,
) -> Result<AnnotationResponse, ServiceError> {
    // 1. 현재 버전 조회
    let current = get_annotation(id).await?;
    
    // 2. 버전 비교
    if current.version != base_version {
        return Err(ServiceError::VersionConflict {
            current_version: current.version,
            client_version: base_version,
        });
    }
    
    // 3. 업데이트 (version 증가)
    update_with_version_increment(id, updates).await
}
```

**예상 시간:** 1-2일

### 2-1-3: 409 Conflict 응답 처리

**응답 포맷:**
```json
409 Conflict
{
  "error": "VersionConflict",
  "message": "Server version is 2, client baseVersion is 1",
  "currentVersion": 2,
  "serverData": {
    "id": 1,
    "version": 2,
    "description": "Updated by User B",
    "updated_at": "2025-11-07T11:32:00Z"
  }
}
```

**클라이언트 재시도 로직:**
```
1. 409 Conflict 받음
2. 최신 데이터 조회 (GET /api/annotations/1)
3. 사용자에게 충돌 알림
4. 사용자가 변경사항 재적용
5. 다시 PUT 요청 (새로운 baseVersion 사용)
```

**예상 시간:** 1일

---

## 2-2️⃣ HEAD 요청 (1-2일) ⭐ 권장

### 목표
- 버전 정보만 조회 (대역폭 절감)
- 캐시 유효성 검증
- 성능 최적화

### 2-2-1: HEAD 핸들러 추가

**현재:**
```
GET /api/annotations?series_instance_uid=...
→ 전체 annotation 데이터 반환 (1-10MB)
```

**개선:**
```
HEAD /api/annotations?series_instance_uid=...
→ 응답 헤더만 반환 (1KB)
  - Last-Modified: 2025-11-07T10:22:00Z
  - Annotation-Version: 13
  - Content-Length: 0
```

**구현:**
```rust
#[utoipa::path(
    head,
    path = "/api/annotations",
    tag = "annotations",
)]
pub async fn head_annotations(
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    // GET과 동일한 로직이지만 body 없음
    let result = get_annotations_internal(&query).await?;
    
    HttpResponse::Ok()
        .insert_header(("Last-Modified", result.last_modified))
        .insert_header(("Annotation-Version", result.max_version.to_string()))
        .insert_header(("Content-Length", "0"))
        .finish()
}
```

**예상 시간:** 0.5일

### 2-2-2: 응답 헤더 최적화

**응답 헤더:**
```
HTTP/1.1 200 OK
Last-Modified: 2025-11-07T10:22:00Z
Annotation-Version: 13
Annotation-Count: 5
Content-Length: 0
Cache-Control: max-age=300
ETag: "abc123def456"
```

**구현:**
```rust
response
    .insert_header(("Last-Modified", last_modified))
    .insert_header(("Annotation-Version", max_version.to_string()))
    .insert_header(("Annotation-Count", count.to_string()))
    .insert_header(("Cache-Control", "max-age=300"))
    .insert_header(("ETag", etag))
```

**예상 시간:** 0.5일

### 2-2-3: 캐시 검증 로직

**클라이언트 캐시 검증:**
```
1. 캐시된 데이터 있음?
   ├─ NO → GET 요청 (전체 데이터)
   └─ YES → HEAD 요청 (버전 확인)

2. HEAD 응답 받음
   ├─ Last-Modified 같음? → 캐시 사용
   └─ Last-Modified 다름? → GET 요청 (최신 데이터)

3. 캐시 업데이트
```

**예상 시간:** 1일

---

## 2-3️⃣ WebSocket 실시간 동기화 (5-7일) ⭐ 권장

### 목표
- 다중 사용자 동시 편집 지원
- 실시간 이벤트 브로드캐스팅
- 자동 동기화

### 2-3-1: WebSocket 서버 구축

**구조:**
```
Client A (Viewer 1)
    ↓
    ├─→ WebSocket Server (Actix-web)
    ↓
Client B (Viewer 2)
Client C (Viewer 3)

이벤트:
- annotation_created
- annotation_updated
- annotation_deleted
```

**구현:**
```rust
use actix_web_actors::ws;

pub struct AnnotationWsActor {
    id: usize,
    addr: Addr<AnnotationWsServer>,
}

impl Actor for AnnotationWsActor {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        // 서버에 연결 등록
        self.addr.do_send(Connect {
            addr: ctx.address().recipient(),
        });
    }
}

#[get("/ws/annotations")]
pub async fn ws_annotations(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<AnnotationWsServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        AnnotationWsActor {
            id: 0,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
```

**예상 시간:** 2-3일

### 2-3-2: 이벤트 브로드캐스팅

**이벤트 타입:**
```rust
#[derive(Clone, Debug, Serialize)]
pub enum AnnotationEvent {
    Created {
        annotation_id: i32,
        user_id: i32,
        user_name: String,
        data: AnnotationResponse,
    },
    Updated {
        annotation_id: i32,
        user_id: i32,
        user_name: String,
        changes: HashMap<String, serde_json::Value>,
        new_version: i32,
    },
    Deleted {
        annotation_id: i32,
        user_id: i32,
        user_name: String,
    },
}
```

**브로드캐스팅 로직:**
```rust
// Create 후
broadcast_event(AnnotationEvent::Created {
    annotation_id: annotation.id,
    user_id: user_id,
    user_name: user_name,
    data: annotation_response,
}).await;

// Update 후
broadcast_event(AnnotationEvent::Updated {
    annotation_id: annotation.id,
    user_id: user_id,
    user_name: user_name,
    changes: changes,
    new_version: new_version,
}).await;

// Delete 후
broadcast_event(AnnotationEvent::Deleted {
    annotation_id: annotation_id,
    user_id: user_id,
    user_name: user_name,
}).await;
```

**예상 시간:** 2-3일

### 2-3-3: 클라이언트 구독 관리

**구독 관리:**
```rust
pub struct AnnotationWsServer {
    sessions: HashMap<usize, Recipient<ws::Message>>,
    subscriptions: HashMap<i32, HashSet<usize>>, // project_id -> session_ids
}

impl AnnotationWsServer {
    pub fn subscribe(&mut self, session_id: usize, project_id: i32) {
        self.subscriptions
            .entry(project_id)
            .or_insert_with(HashSet::new)
            .insert(session_id);
    }
    
    pub fn unsubscribe(&mut self, session_id: usize, project_id: i32) {
        if let Some(sessions) = self.subscriptions.get_mut(&project_id) {
            sessions.remove(&session_id);
        }
    }
    
    pub fn broadcast(&self, project_id: i32, event: AnnotationEvent) {
        if let Some(sessions) = self.subscriptions.get(&project_id) {
            for session_id in sessions {
                if let Some(recipient) = self.sessions.get(session_id) {
                    let msg = serde_json::to_string(&event).unwrap();
                    let _ = recipient.do_send(ws::Message::Text(msg.into()));
                }
            }
        }
    }
}
```

**예상 시간:** 1-2일

### 2-3-4: 재연결 처리

**재연결 로직:**
```rust
pub struct ReconnectHandler {
    max_retries: u32,
    retry_delay_ms: u64,
}

impl ReconnectHandler {
    pub async fn connect_with_retry(&self) -> Result<WebSocket, Error> {
        let mut retries = 0;
        
        loop {
            match self.connect().await {
                Ok(ws) => return Ok(ws),
                Err(e) if retries < self.max_retries => {
                    retries += 1;
                    tokio::time::sleep(
                        Duration::from_millis(self.retry_delay_ms * retries as u64)
                    ).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

**예상 시간:** 1-2일

---

## 2-4️⃣ Collaborative Lock (3-4일) ⭐ 권장

### 목표
- 동일 annotation 동시 수정 방지
- 편집자 실시간 표시
- 자동 Lock 해제

### 2-4-1: Lock 테이블 생성

**데이터 모델:**
```sql
CREATE TABLE annotation_lock (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id),
    user_id INTEGER NOT NULL REFERENCES security_user(id),
    locked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP + INTERVAL '5 minutes',
    UNIQUE(annotation_id)
);

CREATE INDEX idx_annotation_lock_expires 
ON annotation_lock(expires_at);
```

**예상 시간:** 0.5일

### 2-4-2: Lock 획득/해제 로직

**Lock 획득:**
```rust
pub async fn acquire_lock(
    annotation_id: i32,
    user_id: i32,
) -> Result<LockResponse, ServiceError> {
    // 1. 기존 Lock 확인
    let existing_lock = get_lock(annotation_id).await?;
    
    if let Some(lock) = existing_lock {
        if lock.user_id == user_id {
            // 같은 사용자 → Lock 갱신
            refresh_lock(annotation_id).await?;
            return Ok(LockResponse::Acquired);
        } else if lock.expires_at > now() {
            // 다른 사용자 → Lock 대기
            return Err(ServiceError::Locked {
                locked_by: lock.user_id,
                locked_by_name: get_user_name(lock.user_id).await?,
                expires_at: lock.expires_at,
            });
        }
    }
    
    // 2. Lock 획득
    create_lock(annotation_id, user_id).await?;
    Ok(LockResponse::Acquired)
}
```

**Lock 해제:**
```rust
pub async fn release_lock(
    annotation_id: i32,
    user_id: i32,
) -> Result<(), ServiceError> {
    let lock = get_lock(annotation_id).await?;
    
    if let Some(lock) = lock {
        if lock.user_id != user_id {
            return Err(ServiceError::Unauthorized(
                "Only lock owner can release".into()
            ));
        }
    }
    
    delete_lock(annotation_id).await?;
    Ok(())
}
```

**예상 시간:** 1-2일

### 2-4-3: 타임아웃 처리

**자동 Lock 해제:**
```rust
// 백그라운드 작업 (5분마다 실행)
pub async fn cleanup_expired_locks() {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;
        
        // 만료된 Lock 삭제
        sqlx::query(
            "DELETE FROM annotation_lock WHERE expires_at < NOW()"
        )
        .execute(&pool)
        .await
        .ok();
    }
}
```

**Lock 갱신:**
```rust
pub async fn refresh_lock(annotation_id: i32) -> Result<(), ServiceError> {
    sqlx::query(
        "UPDATE annotation_lock 
         SET expires_at = NOW() + INTERVAL '5 minutes'
         WHERE annotation_id = $1"
    )
    .bind(annotation_id)
    .execute(&pool)
    .await?;
    
    Ok(())
}
```

**예상 시간:** 1일

### 2-4-4: 편집자 표시 (Presence)

**Presence 정보:**
```rust
#[derive(Clone, Debug, Serialize)]
pub struct PresenceInfo {
    pub user_id: i32,
    pub user_name: String,
    pub annotation_id: i32,
    pub locked_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
```

**Presence 브로드캐스팅:**
```rust
// Lock 획득 시
broadcast_event(AnnotationEvent::LockAcquired {
    annotation_id: annotation_id,
    user_id: user_id,
    user_name: user_name,
    expires_at: expires_at,
}).await;

// Lock 해제 시
broadcast_event(AnnotationEvent::LockReleased {
    annotation_id: annotation_id,
    user_id: user_id,
}).await;
```

**예상 시간:** 1day

---

## 📊 Phase 2 세부 타임라인

```
Week 3 (3-4일):
├─ 2-1-1: Optimistic Locking (1-2일)
├─ 2-1-2: 버전 충돌 감지 (1-2일)
└─ 2-1-3: 409 Conflict 처리 (1일)

Week 3-4 (1-2일):
├─ 2-2-1: HEAD 핸들러 (0.5일)
├─ 2-2-2: 응답 헤더 최적화 (0.5일)
└─ 2-2-3: 캐시 검증 (1일)

Week 4-5 (5-7일):
├─ 2-3-1: WebSocket 서버 (2-3일)
├─ 2-3-2: 이벤트 브로드캐스팅 (2-3일)
├─ 2-3-3: 클라이언트 구독 (1-2일)
└─ 2-3-4: 재연결 처리 (1-2일)

Week 5 (3-4일):
├─ 2-4-1: Lock 테이블 (0.5일)
├─ 2-4-2: Lock 획득/해제 (1-2일)
├─ 2-4-3: 타임아웃 처리 (1일)
└─ 2-4-4: Presence (1일)
```

---

## 🎯 우선순위 및 의존성

### 필수 (Phase 2-1)
```
2-1: Version Conflict 처리
└─ 모든 다른 기능의 기초
```

### 권장 (Phase 2-2, 2-3, 2-4)
```
2-2: HEAD 요청 (독립적)
2-3: WebSocket (2-1 완료 후)
2-4: Collaborative Lock (2-1 완료 후)
```

### 병렬 개발 가능
```
Week 3: 2-1 (필수)
Week 4: 2-2 (병렬) + 2-3 시작
Week 5: 2-3 계속 + 2-4 (병렬)
```

---

## 📝 결론

**Phase 2는 4개의 독립적인 기능으로 구성:**

1. **2-1: Version Conflict** (필수, 3-4일)
   - 데이터 무결성 보장
   - 다른 기능의 기초

2. **2-2: HEAD 요청** (권장, 1-2일)
   - 성능 최적화
   - 독립적으로 개발 가능

3. **2-3: WebSocket** (권장, 5-7일)
   - 실시간 동기화
   - 가장 복잡한 기능

4. **2-4: Collaborative Lock** (권장, 3-4일)
   - 협업 지원
   - 2-1 완료 후 개발

**총 예상 기간:** 2-3주 (병렬 개발 시)

