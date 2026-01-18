# TimePoint API 개선사항

## 📋 개요

TimePoint API에 Study Instance UID를 사용한 Study 할당/해제 기능을 추가했습니다. 기존에는 내부 데이터베이스 ID(`study_ids`)만 지원했지만, 이제 DICOM Study Instance UID(`study_instance_uids`)도 지원합니다.

## 🎯 목적

프론트엔드에서 DICOM Gateway API를 통해 Study 목록을 조회할 때, Study Instance UID만 알 수 있는 경우가 많습니다. 이런 경우 별도로 Study ID를 조회하지 않고도 바로 TimePoint에 할당할 수 있도록 개선했습니다.

## 🔧 변경 사항

### 1. DTO 수정

**파일**: `pacs-server/src/domain/entities/timepoint_study.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssignStudies {
    /// Study ID 목록 (기존 방식)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_ids: Option<Vec<i32>>,
    
    /// Study Instance UID 목록 (새로운 방식)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_instance_uids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnassignStudies {
    /// Study ID 목록 (기존 방식)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_ids: Option<Vec<i32>>,
    
    /// Study Instance UID 목록 (새로운 방식)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_instance_uids: Option<Vec<String>>,
}
```

### 2. 서비스 로직 수정

**파일**: `pacs-server/src/domain/services/timepoint_service.rs`

- `ProjectDataRepository` 의존성 추가
- Study Instance UID를 Study ID로 자동 변환하는 로직 추가

```rust
// study_instance_uids를 study_ids로 변환
let study_ids = if let Some(ids) = assign_studies.study_ids {
    // study_ids가 제공된 경우 그대로 사용
    ids
} else if let Some(uids) = assign_studies.study_instance_uids {
    // study_instance_uids가 제공된 경우 변환
    let mut ids = Vec::new();
    for uid in &uids {
        let study = self
            .project_data_repository
            .as_ref()
            .find_study_by_uid(subject.project_id, uid)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Study with UID {} not found", uid))
            })?;
        ids.push(study.id);
    }
    ids
} else {
    return Err(ServiceError::ValidationError(
        "Either study_ids or study_instance_uids must be provided".into(),
    ));
};
```

### 3. Repository Trait 개선

**파일**: `pacs-server/src/domain/repositories/project_data_repository.rs`

`Arc<T>`가 `ProjectDataRepository` trait을 구현하도록 blanket implementation 추가:

```rust
#[async_trait::async_trait]
impl<T: ProjectDataRepository + ?Sized> ProjectDataRepository for std::sync::Arc<T> {
    // 모든 메서드를 Arc를 통해 위임
    async fn find_study_by_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        (**self).find_study_by_uid(project_id, study_uid).await
    }
    // ... 기타 메서드들
}
```

### 4. 의존성 주입

**파일**: `pacs-server/src/main.rs`

```rust
let timepoint_service = Arc::new(TimePointServiceImpl::new(
    timepoint_repo.clone(),
    timepoint_study_repo.clone(),
    subject_repo.clone(),
    project_data_repo.clone(), // ProjectDataRepository 추가
));
```

## 📝 사용 방법

### 방법 1: Study IDs 사용 (기존 방식)

```http
POST /api/timepoints/{timepoint_id}/studies
Content-Type: application/json

{
  "study_ids": [1866, 1867]
}
```

### 방법 2: Study Instance UIDs 사용 (새로운 방식)

```http
POST /api/timepoints/{timepoint_id}/studies
Content-Type: application/json

{
  "study_instance_uids": [
    "1.3.6.1.4.1.14519.5.2.1.6655.2359.305690637242184753624524107418",
    "1.3.6.1.4.1.14519.5.2.1.6655.2359.123456789012345678901234567890"
  ]
}
```

## ✅ 검증 규칙

1. **둘 중 하나는 필수**: `study_ids` 또는 `study_instance_uids` 중 하나는 반드시 제공되어야 합니다.
2. **우선순위**: 둘 다 제공된 경우 `study_ids`가 우선합니다.
3. **Study 존재 확인**: Study Instance UID로 조회 시 해당 Study가 프로젝트에 존재하지 않으면 `NotFound` 에러를 반환합니다.

## 🔍 기술적 세부사항

### Arc Blanket Implementation

Rust에서 `Arc<T>`는 자동으로 trait `T`를 구현하지 않습니다. 따라서 `Arc<ProjectDataRepositoryImpl>`이 `ProjectDataRepository` trait을 구현하도록 blanket implementation을 추가해야 했습니다.

이는 다른 서비스들(`SeriesUserNoteServiceImpl`, `SeriesUserReportServiceImpl` 등)에서도 동일하게 사용되는 패턴입니다.

## 📊 영향 범위

- ✅ TimePoint Study 할당 API
- ✅ TimePoint Study 해제 API
- ✅ 기존 기능 호환성 유지
- ✅ 테스트 통과

## 🚀 다음 단계

프론트엔드에서 DICOM Gateway API 응답의 Study Instance UID를 직접 사용하여 TimePoint에 할당할 수 있습니다.

