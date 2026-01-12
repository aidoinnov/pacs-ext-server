# ISSUE-002: UpdateAnnotation Entity 부재

> **이슈 번호**: ISSUE-002  
> **작성일**: 2026-01-11  
> **상태**: ✅ Resolved  
> **카테고리**: Architecture, Implementation Pattern

---

## 📋 이슈 요약

WORKLOG에서 `UpdateAnnotation` entity를 사용하는 것으로 설계했으나, 실제 프로젝트에는 해당 entity가 존재하지 않음. 현재는 repository에서 **개별 파라미터**로 업데이트를 처리하는 패턴을 사용 중.

---

## 🤔 문제 상황

**WORKLOG 설계**:
```rust
// ❌ 실제로는 존재하지 않음
let update = UpdateAnnotation {
    snapshot_image_key: Some(image_key),
    snapshot_status: Some(SnapshotUploadStatus::Pending),
    snapshot_uploaded_at: None,
    ..Default::default()
};

self.annotation_service
    .update_annotation(annotation_id, &update)
    .await?;
```

**실제 구현**:
```rust
// ✅ 실제 패턴
async fn update_annotation_with_measurements(
    &self,
    id: i32,
    data: serde_json::Value,
    is_shared: bool,
    measurement_values: Option<serde_json::Value>,
    label: Option<String>,
) -> Result<Annotation, ServiceError>
```

---

## 🔍 현재 아키텍처 분석

### 기존 업데이트 패턴

1. **DTO Layer**: `UpdateAnnotationRequest` (존재함)
2. **Use Case Layer**: 개별 필드 추출
3. **Service Layer**: 개별 파라미터로 메서드 호출
4. **Repository Layer**: 개별 파라미터로 SQL 실행

### 파일 위치

- **DTO**: `pacs-server/src/application/dto/annotation_dto.rs`
  - `UpdateAnnotationRequest` 존재
- **Entity**: `pacs-server/src/domain/entities/annotation.rs`
  - `Annotation`, `NewAnnotation` 존재
  - `UpdateAnnotation` **없음**
- **Service**: `pacs-server/src/domain/services/annotation_service.rs`
  - `update_annotation(id, data, is_shared)`
  - `update_annotation_with_measurements(id, data, is_shared, measurements, label)`
- **Repository**: `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`
  - 개별 파라미터로 UPDATE 쿼리 실행

---

## ✅ 해결 방안

스냅샷 기능을 위한 **전용 메서드** 추가 (기존 패턴 유지)

### 1. Repository Layer

```rust
#[async_trait]
pub trait AnnotationRepository: Send + Sync {
    async fn update_snapshot(
        &self,
        annotation_id: i32,
        snapshot_image_key: String,
        snapshot_status: SnapshotUploadStatus,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Option<Annotation>, RepositoryError>;
}
```

### 2. Service Layer

```rust
#[async_trait]
pub trait AnnotationService: Send + Sync {
    async fn update_snapshot(
        &self,
        annotation_id: i32,
        snapshot_image_key: String,
        snapshot_status: SnapshotUploadStatus,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Annotation, ServiceError>;
}
```

### 3. Use Case Layer

```rust
// URL 생성 시
self.annotation_service
    .update_snapshot(
        annotation_id,
        image_key.clone(),
        SnapshotUploadStatus::Pending,
        None,
    )
    .await?;

// 업로드 완료 시
self.annotation_service
    .update_snapshot(
        annotation_id,
        request.image_key,
        SnapshotUploadStatus::Completed,
        Some(chrono::Utc::now()),
    )
    .await?;
```

---

## 📊 패턴 비교

| 항목 | UpdateAnnotation Entity | 개별 파라미터 메서드 |
|------|------------------------|-------------------|
| **타입 안전성** | ✅ 높음 | ⚠️ 중간 |
| **유연성** | ⚠️ 모든 필드 정의 필요 | ✅ 필요한 필드만 |
| **코드 간결성** | ✅ 구조체 하나로 전달 | ⚠️ 파라미터 많아질 수 있음 |
| **기존 패턴 일관성** | ❌ 기존과 다름 | ✅ 기존 패턴 유지 |
| **구현 복잡도** | ⚠️ 새로운 entity 추가 | ✅ 메서드만 추가 |

---

## 🎯 최종 결정

**기존 패턴을 유지**하고, 스냅샷 전용 `update_snapshot` 메서드를 추가합니다.

**이유**:
1. 기존 코드베이스와의 일관성 유지
2. 스냅샷 업데이트는 특정 필드만 변경하므로 전용 메서드가 적합
3. 구현 복잡도 최소화
4. 기존 팀의 개발 패턴 존중

---

## 🔗 관련 문서

- [WORKLOG.md](../WORKLOG.md) - 3단계, 5단계 수정됨
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 아키텍처 문서

---

## 📝 교훈

1. **기존 코드베이스 패턴 파악이 중요**: 설계 전에 실제 구현 패턴 확인 필수
2. **일관성 유지**: 새로운 기능도 기존 패턴을 따르는 것이 유지보수에 유리
3. **점진적 개선**: 전체 구조를 바꾸기보다 기존 패턴에 맞춰 확장

---

**결론**: `UpdateAnnotation` entity 대신 `update_snapshot` 전용 메서드를 사용하여 기존 아키텍처 패턴을 유지합니다.

