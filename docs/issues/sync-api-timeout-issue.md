# Sync API Timeout Issue

**날짜**: 2025-12-18  
**상태**: 🔴 미해결  
**우선순위**: High

## 📋 문제 요약

`POST /api/sync/run` API 엔드포인트가 호출 시 응답하지 않고 무한 대기(timeout) 상태에 빠지는 문제

## 🔍 증상

1. **Health Check**: ✅ 정상 작동 (`GET /health`)
2. **Sync Status**: ✅ 정상 작동 (`GET /api/sync/status`)
3. **Sync Run**: ❌ 응답 없음 (`POST /api/sync/run`)
   - 60초 이상 대기해도 응답 없음
   - 타임아웃 발생
   - 서버 로그에 아무 메시지도 출력되지 않음

## 🧪 테스트 결과

### 1. Controller 레벨 테스트
```rust
// Controller에서 바로 반환 시
async fn run_once(...) -> HttpResponse {
    return HttpResponse::Ok().json(serde_json::json!({
        "test": "direct_return"
    }));
}
```
**결과**: ✅ 정상 응답 (즉시 반환됨)

### 2. Service 호출 테스트
```rust
// Service의 run_once() 호출 시
async fn run_once(svc: web::Data<SyncServiceImpl>) -> HttpResponse {
    let res = svc.run_once().await;  // ← 여기서 멈춤
    HttpResponse::Ok().json(...)
}
```
**결과**: ❌ 응답 없음 (무한 대기)

### 3. Service 구현 테스트
```rust
// SyncServiceImpl::run_once()에 즉시 반환 코드 추가
async fn run_once(&self) -> SyncResult {
    eprintln!("🔄 [Sync] run_once() called - TEST MODE");
    return SyncResult { success: true, ... };  // 즉시 반환
}
```
**결과**: ❌ 로그 출력 안 됨, 응답 없음

## 🔎 원인 분석

### 확인된 사항
1. ✅ Controller 라우팅은 정상 (`/api/sync/run` 경로 등록됨)
2. ✅ Dependency Injection은 정상 (`web::Data<SyncServiceImpl>` 주입됨)
3. ✅ DB 연결은 정상 (DCM4CHEE DB, RBAC DB 모두 연결됨)
4. ✅ DB 쿼리는 빠름 (6-7ms)

### 의심되는 원인
1. **Actix-web extractor 문제**: `web::Data<SyncServiceImpl>` 추출 시 블로킹 발생 가능성
2. **Async runtime 문제**: `run_once()` 메서드 호출 자체가 블로킹되는 문제
3. **Deadlock**: 내부적으로 lock이 걸려서 대기 상태에 빠짐
4. **Middleware 간섭**: 특정 미들웨어가 요청을 블로킹

## 📊 DB 쿼리 성능

```sql
-- Study 테이블 쿼리 (DCM4CHEE DB)
SELECT ... FROM study ... LIMIT 500;
-- 실행 시간: 7.434 ms ✅
```

DB 쿼리 자체는 매우 빠르므로 DB 성능 문제는 아님.

## 🛠️ 시도한 해결 방법

### 1. Dependency Injection 타입 수정
```rust
// Before
async fn run_once(svc: web::Data<Arc<SyncServiceImpl>>) -> HttpResponse

// After
async fn run_once(svc: web::Data<SyncServiceImpl>) -> HttpResponse
```
**결과**: 변화 없음

### 2. Manual Extraction
```rust
async fn run_once(req: actix_web::HttpRequest) -> HttpResponse {
    let svc = req.app_data::<web::Data<SyncServiceImpl>>();
    // ...
}
```
**결과**: 변화 없음

### 3. Timeout 추가
```rust
match tokio::time::timeout(Duration::from_secs(5), svc.run_once()).await {
    Ok(res) => ...,
    Err(_) => // timeout
}
```
**결과**: 5초 후 타임아웃 발생 (예상대로 작동하지만 근본 원인 미해결)

## 📝 관련 코드

### Controller
- **파일**: `pacs-server/src/presentation/controllers/sync_controller.rs`
- **라우트**: `/api/sync/run` (POST)
- **핸들러**: `run_once()`

### Service
- **파일**: `pacs-server/src/infrastructure/services/sync_worker.rs`
- **구현**: `SyncServiceImpl::run_once()`

### 초기화
- **파일**: `pacs-server/src/main.rs`
- **위치**: Line 420-468 (Sync 초기화 코드)

## 🔄 동기화 자동 실행 상태

**참고**: 동기화 자체는 작동하고 있음 (자동 스케줄러로 실행된 것으로 추정)
- RBAC DB에 데이터가 이미 존재함
- 최근 동기화 시간: 2025-12-18 10:33:07

## 🎯 다음 단계

1. **Actix-web 로깅 활성화**: 요청이 어디서 멈추는지 정확히 파악
2. **Async trace 추가**: `tracing` 크레이트로 async 실행 흐름 추적
3. **Extractor 디버깅**: `FromRequest` trait 구현 확인
4. **Middleware 검토**: CORS, Auth 등 미들웨어가 POST 요청을 블로킹하는지 확인
5. **Thread pool 확인**: Actix-web worker thread가 블로킹되는지 확인

## 📌 참고 사항

- 자동 스케줄러는 정상 작동하는 것으로 보임 (데이터가 DB에 존재)
- 수동 실행 API만 문제가 있음
- 다른 모든 API는 정상 작동

