# ISSUE-001: 타임스탬프 필드의 책임 소재

> **이슈 번호**: ISSUE-001  
> **작성일**: 2026-01-11  
> **상태**: ✅ Resolved  
> **카테고리**: Design Decision, Security, Data Integrity

---

## 📋 이슈 요약

어노테이션 스냅샷 업로드 완료 시 `snapshot_uploaded_at` 타임스탬프를 **클라이언트가 제공**해야 하는지, **서버가 자동 생성**해야 하는지에 대한 설계 결정.

---

## 🤔 문제 상황

초기 설계에서 `UpdateAnnotation` 구조체에 `snapshot_uploaded_at` 필드가 포함되어 있어, 클라이언트가 업로드 시간을 직접 전송할 수 있는 구조였습니다.

```rust
// ❌ 잘못된 설계
pub struct CompleteSnapshotUploadRequest {
    pub image_key: String,
    pub uploaded_at: chrono::NaiveDateTime,  // 클라이언트가 시간 전송?
}
```

**질문**: 사용자가 직접 업로드한 시간을 줘야 하는 거야?

---

## 🔍 분석

### Option 1: 클라이언트가 시간 제공 ❌

**장점**:
- 클라이언트의 실제 업로드 완료 시점 기록 가능

**단점**:
1. **시간대 불일치**: 클라이언트의 로컬 시간대 vs 서버 UTC
2. **보안 취약점**: 클라이언트가 시간을 조작할 수 있음
3. **데이터 일관성**: 다른 타임스탬프와 기준이 달라짐
4. **클라이언트 복잡도**: 클라이언트가 올바른 형식으로 시간 생성 필요
5. **감사 로그 신뢰성**: 조작된 시간으로 인한 감사 추적 어려움

### Option 2: 서버가 자동 생성 ✅

**장점**:
1. **신뢰할 수 있는 시간**: 서버의 단일 시간 소스
2. **보안**: 클라이언트가 시간 조작 불가능
3. **일관성**: 모든 타임스탬프가 서버 UTC 기준
4. **간편성**: 클라이언트는 성공/실패만 알림
5. **감사 추적**: 신뢰할 수 있는 감사 로그

**단점**:
- 네트워크 지연으로 인해 실제 업로드 완료 시점과 약간의 차이 발생 가능
  - → 하지만 이는 무시할 수 있는 수준 (수 밀리초~초 단위)

---

## ✅ 결정 사항

**서버가 자동으로 타임스탬프를 생성**하는 방식을 채택합니다.

### 최종 설계

**클라이언트 요청**:
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CompleteSnapshotUploadRequest {
    /// S3 object key
    pub image_key: String,
    
    /// 업로드 성공 여부 (optional, 기본값: true)
    pub success: Option<bool>,
    
    // ⚠️ uploaded_at 필드 없음!
}
```

**서버 처리**:
```rust
pub async fn complete_snapshot_upload(
    &self,
    annotation_id: i32,
    request: CompleteSnapshotUploadRequest,
    user_id: i32,
) -> Result<Annotation, ServiceError> {
    // ... 권한 확인 ...
    
    let success = request.success.unwrap_or(true);
    let now = chrono::Utc::now().naive_utc();  // ⭐ 서버에서 생성
    
    let update = UpdateAnnotation {
        snapshot_image_key: Some(request.image_key),
        snapshot_status: Some(if success { Completed } else { Failed }),
        snapshot_uploaded_at: if success { Some(now) } else { None },  // ⭐ 자동 생성
        ..Default::default()
    };
    
    self.annotation_service.update_annotation(annotation_id, &update).await
}
```

---

## 📊 비교표

| 항목 | 클라이언트 제공 ❌ | 서버 자동 생성 ✅ |
|------|-------------------|------------------|
| **시간 정확성** | 클라이언트 시간대 문제 | 서버 UTC 보장 |
| **보안** | 조작 가능 | 조작 불가능 |
| **일관성** | 불일치 가능 | 모든 레코드 통일 |
| **클라이언트 복잡도** | 높음 (시간 생성 필요) | 낮음 (성공/실패만) |
| **감사 추적** | 신뢰성 낮음 | 신뢰성 높음 |
| **네트워크 지연** | 실제 시점 반영 | 수신 시점 반영 |

---

## 🎯 구현 가이드

### 1. DTO 설계
```rust
// ✅ 올바른 설계
pub struct CompleteSnapshotUploadRequest {
    pub image_key: String,
    pub success: Option<bool>,  // 시간 필드 없음!
}
```

### 2. Use Case 구현
```rust
// 서버에서 현재 시간 생성
let now = chrono::Utc::now().naive_utc();

let update = UpdateAnnotation {
    snapshot_uploaded_at: if success { Some(now) } else { None },
    // ...
};
```

### 3. API 사용 예시
```bash
# 클라이언트는 시간을 보내지 않음
curl -X POST "/annotations/123/snapshot/complete-upload" \
  -d '{"image_key": "...", "success": true}'
```

---

## 🔗 관련 문서

- [WORKLOG.md](../WORKLOG.md) - 5.3 스냅샷 업로드 완료 메서드
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 데이터 모델 설계
- [API_SPEC.md](../API_SPEC.md) - API 명세

---

## 📝 교훈

1. **타임스탬프는 항상 서버에서 생성**하는 것이 원칙
2. **클라이언트는 이벤트 발생만 알리고, 시간은 서버가 기록**
3. **보안과 데이터 무결성을 위해 신뢰할 수 있는 단일 시간 소스 사용**
4. **감사 로그의 신뢰성을 위해 서버 시간 기준 통일**

---

**결론**: `snapshot_uploaded_at`은 서버가 자동으로 생성하며, 클라이언트는 업로드 성공/실패 여부만 전달합니다.

