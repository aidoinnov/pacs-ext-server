# Rust Arc Blanket Implementation 가이드

## 📋 개요

Rust에서 `Arc<T>`는 자동으로 trait `T`를 구현하지 않습니다. 따라서 `Arc<T>`가 trait을 구현하도록 하려면 blanket implementation을 추가해야 합니다.

## 🎯 문제 상황

### 에러 예시

```rust
error[E0277]: the trait bound `Arc<ProjectDataRepositoryImpl>: ProjectDataRepository` is not satisfied
   --> src/main.rs:364:9
    |
364 |         project_data_repo.clone(),
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

### 원인

```rust
// Repository 구현체
pub struct ProjectDataRepositoryImpl {
    pool: PgPool,
}

// Trait 구현
impl ProjectDataRepository for ProjectDataRepositoryImpl {
    // ...
}

// main.rs에서 사용
let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));

// 서비스에 전달
let timepoint_service = Arc::new(TimePointServiceImpl::new(
    timepoint_repo.clone(),
    timepoint_study_repo.clone(),
    subject_repo.clone(),
    project_data_repo.clone(), // ❌ Arc<ProjectDataRepositoryImpl>은 ProjectDataRepository를 구현하지 않음
));
```

## 🔧 해결 방법

### 1. Blanket Implementation 추가

**파일**: `pacs-server/src/domain/repositories/project_data_repository.rs`

```rust
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait ProjectDataRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectData>, sqlx::Error>;
    async fn find_study_by_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectDataStudy>, sqlx::Error>;
    // ... 기타 메서드들
}

// ✅ Arc<T>가 ProjectDataRepository를 구현하도록 blanket implementation 추가
#[async_trait]
impl<T: ProjectDataRepository + ?Sized> ProjectDataRepository for Arc<T> {
    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectData>, sqlx::Error> {
        (**self).find_by_id(id).await
    }

    async fn find_study_by_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        (**self).find_study_by_uid(project_id, study_uid).await
    }
    
    // ... 모든 메서드를 Arc를 통해 위임
}
```

### 2. 서비스에서 Arc 사용

```rust
pub struct TimePointServiceImpl<T, TS, S, PD>
where
    T: TimePointRepository,
    TS: TimePointStudyRepository,
    S: SubjectRepository,
    PD: ProjectDataRepository,
{
    timepoint_repository: T,
    timepoint_study_repository: TS,
    subject_repository: S,
    project_data_repository: Arc<PD>, // ✅ Arc로 래핑
}

impl<T, TS, S, PD> TimePointServiceImpl<T, TS, S, PD>
where
    T: TimePointRepository,
    TS: TimePointStudyRepository,
    S: SubjectRepository,
    PD: ProjectDataRepository,
{
    pub fn new(
        timepoint_repository: T,
        timepoint_study_repository: TS,
        subject_repository: S,
        project_data_repository: Arc<PD>, // ✅ Arc로 받음
    ) -> Self {
        Self {
            timepoint_repository,
            timepoint_study_repository,
            subject_repository,
            project_data_repository,
        }
    }
}
```

### 3. 서비스에서 사용

```rust
// as_ref()를 사용하여 Arc 내부의 trait 객체에 접근
let study = self
    .project_data_repository
    .as_ref() // ✅ Arc<T>를 &T로 변환
    .find_study_by_uid(subject.project_id, uid)
    .await?;
```

## 🔍 핵심 개념

### `(**self)` vs `as_ref()`

```rust
// Blanket implementation에서
(**self).find_by_id(id).await  // Arc<T>를 T로 역참조

// 서비스 코드에서
self.project_data_repository.as_ref().find_by_id(id).await  // Arc<T>를 &T로 변환
```

### `?Sized` Trait Bound

```rust
impl<T: ProjectDataRepository + ?Sized> ProjectDataRepository for Arc<T>
//                                ^^^^^^
// ?Sized: T가 컴파일 타임에 크기를 알 수 없어도 됨
// 이를 통해 Arc<dyn ProjectDataRepository>도 지원 가능
```

## 📚 프로젝트 내 다른 예시

### SeriesUserNoteService

**파일**: `pacs-server/src/domain/services/series_user_note_service.rs`

```rust
pub struct SeriesUserNoteServiceImpl<N, U, P, PD>
where
    N: SeriesUserNoteRepository,
    U: UserRepository,
    P: ProjectRepository,
    PD: ProjectDataRepository,
{
    note_repository: Arc<N>,
    user_repository: Arc<U>,
    project_repository: Arc<P>,
    project_data_repository: Arc<PD>, // ✅ Arc로 래핑
}

impl<N, U, P, PD> SeriesUserNoteServiceImpl<N, U, P, PD>
where
    N: SeriesUserNoteRepository,
    U: UserRepository,
    P: ProjectRepository,
    PD: ProjectDataRepository,
{
    pub fn new(
        note_repository: N,
        user_repository: U,
        project_repository: P,
        project_data_repository: Arc<PD>, // ✅ Arc로 받음
    ) -> Self {
        Self {
            note_repository: Arc::new(note_repository),
            user_repository: Arc::new(user_repository),
            project_repository: Arc::new(project_repository),
            project_data_repository, // ✅ 그대로 사용
        }
    }
}
```

## ✅ 체크리스트

Blanket implementation을 추가할 때 확인해야 할 사항:

- [ ] `async_trait` 매크로 사용
- [ ] `?Sized` trait bound 추가
- [ ] 모든 trait 메서드 구현
- [ ] `(**self)`를 사용하여 Arc 내부 객체에 위임
- [ ] `Send + Sync` trait bound 확인

## 🚀 장점

1. **타입 안전성**: 컴파일 타임에 타입 체크
2. **코드 재사용**: 한 번만 구현하면 모든 Arc<T>에서 사용 가능
3. **성능**: 런타임 오버헤드 없음
4. **일관성**: 프로젝트 전체에서 동일한 패턴 사용

## ⚠️ 주의사항

1. **모든 메서드 구현 필요**: Trait의 모든 메서드를 blanket implementation에 포함해야 함
2. **async_trait 필수**: 비동기 메서드가 있는 경우 `#[async_trait]` 매크로 필요
3. **순환 참조 주의**: Arc를 과도하게 사용하면 순환 참조 발생 가능

