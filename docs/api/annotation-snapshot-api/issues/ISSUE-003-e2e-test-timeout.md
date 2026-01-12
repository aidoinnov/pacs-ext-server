# ISSUE-003: E2E 테스트 웹 실행 시 타임아웃 문제

> **작성일**: 2026-01-12  
> **상태**: ✅ 해결됨  
> **카테고리**: 테스트, 성능

---

## 📋 문제 설명

### 증상
- Python E2E 테스트를 직접 실행하면 정상 동작
- 웹 관리 페이지에서 E2E 테스트 실행 시 타임아웃 발생
- `complete-upload` 엔드포인트에서 Read timeout 에러

### 에러 메시지
```
❌ 테스트 실패: HTTPConnectionPool(host='localhost', port=8080): Read timed out. (read timeout=10)

Traceback (most recent call last):
  File "/Users/aido/Code/pacs-ext-server/pacs-server/e2e/test_annotation_snapshot_e2e.py", line 194, in test_annotation_snapshot_workflow
    response = requests.post(
        f"{BASE_URL}/api/annotations/{annotation_id}/snapshot/complete-upload",
        ...
        timeout=10
    )
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='localhost', port=8080): Read timed out. (read timeout=10)
```

---

## 🔍 원인 분석

### 1. Python 테스트 스크립트 타임아웃
- 모든 HTTP 요청에 `timeout=10` 설정
- `complete-upload` 엔드포인트는 S3에서 이미지를 다운로드하여 검증
- S3 다운로드 시간이 10초를 초과할 수 있음

### 2. Rust 서버 동기 실행
- `std::process::Command`로 Python 프로세스를 동기 실행
- HTTP 연결이 Python 프로세스 완료를 기다리는 동안 타임아웃
- 웹 브라우저 → Rust 서버 → Python 프로세스 체인에서 병목 발생

### 3. 직접 실행 vs 웹 실행 차이
- **직접 실행**: 터미널에서 Python 스크립트만 실행 (HTTP 연결 없음)
- **웹 실행**: 브라우저 → Rust 서버 → Python 프로세스 (HTTP 연결 유지 필요)

---

## ✅ 해결 방법

### 1. Python 테스트 스크립트 타임아웃 증가

**파일**: `pacs-server/e2e/test_annotation_snapshot_e2e.py`

**변경 사항**:
```python
# 기존
timeout=10

# 수정
timeout=30  # 일반 요청
timeout=60  # complete-upload (S3 다운로드 고려)
```

**적용 위치**:
- `create_annotation`: 10 → 30초
- `request_upload_url`: 10 → 30초
- `complete_upload`: 10 → 60초 ⭐
- `get_snapshot_status`: 10 → 30초
- `get_annotation`: 10 → 30초

### 2. Rust 서버 비동기 실행

**파일**: `pacs-server/src/presentation/controllers/test_controller.rs`

**기존 코드** (동기 실행):
```rust
pub async fn run_annotation_snapshot_e2e() -> impl Responder {
    let output = Command::new("python3")
        .arg("e2e/test_annotation_snapshot_e2e.py")
        .current_dir(".")
        .output();  // ❌ 동기 실행
    
    // ...
}
```

**수정 코드** (비동기 실행):
```rust
pub async fn run_annotation_snapshot_e2e() -> impl Responder {
    use std::time::Duration;
    use tokio::time::timeout;
    
    // Python 테스트 스크립트를 비동기로 실행 (120초 타임아웃)
    let result = timeout(
        Duration::from_secs(120),  // ⭐ 120초 타임아웃
        tokio::task::spawn_blocking(|| {  // ⭐ 비동기 실행
            Command::new("python3")
                .arg("e2e/test_annotation_snapshot_e2e.py")
                .current_dir(".")
                .output()
        })
    ).await;

    match result {
        Ok(Ok(output_result)) => {
            // 성공 처리
        }
        Ok(Err(e)) => {
            // 태스크 실패
        }
        Err(_) => {
            // 타임아웃
            HttpResponse::RequestTimeout()
                .body("테스트 실행 타임아웃 (120초 초과)")
        }
    }
}
```

**핵심 변경**:
- `Command::output()` → `tokio::task::spawn_blocking()`
- 동기 실행 → 비동기 실행
- HTTP 연결 유지 가능
- 120초 타임아웃 설정

---

## 📊 결과

### Before (타임아웃 발생)
```
웹 브라우저 → Rust 서버 (동기 대기) → Python (10초 타임아웃)
                ↓
            HTTP 연결 타임아웃 ❌
```

### After (정상 동작)
```
웹 브라우저 → Rust 서버 (비동기 실행) → Python (60초 타임아웃)
                ↓
            HTTP 연결 유지 ✅
                ↓
            120초 내 완료 ✅
```

### 테스트 결과
```bash
# 웹 관리 페이지에서 실행
🚀 Annotation Snapshot E2E Test 시작...
✅ 로그인 성공
✅ 어노테이션 생성 성공!
✅ 업로드 URL 생성 성공!
✅ 이미지 생성 완료!
✅ S3 업로드 성공!
✅ 업로드 완료 처리 성공!  ⭐ 타임아웃 해결
✅ 상태 조회 성공!
✅ 어노테이션 조회 성공!

🎉 모든 테스트 통과!
```

---

## 🎓 교훈

### 1. 타임아웃 설정의 중요성
- 외부 서비스 (S3) 호출 시 충분한 타임아웃 필요
- 네트워크 지연, 파일 크기 등을 고려한 여유 시간 설정

### 2. 동기 vs 비동기 실행
- 웹 서버에서 장시간 작업 실행 시 비동기 처리 필수
- HTTP 연결 타임아웃을 고려한 설계

### 3. 테스트 환경 차이
- 직접 실행과 웹 실행의 차이점 이해
- 실제 운영 환경과 유사한 조건에서 테스트

---

## 📝 관련 문서

- [WORKLOG.md](../WORKLOG.md) - Phase 7: E2E 테스트 & 웹 관리 페이지
- [API_SPEC.md](../API_SPEC.md) - 3.4 E2E 테스트 실행

---

**해결일**: 2026-01-12  
**해결 방법**: Python 타임아웃 증가 + Rust 비동기 실행

