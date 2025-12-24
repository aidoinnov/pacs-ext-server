use crate::domain::entities::SeriesUserNote;
use crate::domain::repositories::{ProjectDataRepository, ProjectRepository, SeriesUserNoteRepository, UserRepository};
use crate::domain::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

/// Series User Note 관리 도메인 서비스
///
/// 이 트레이트는 Series User Note와 관련된 비즈니스 로직을 정의합니다.
/// 구체적인 구현은 Infrastructure 계층에서 제공됩니다.
#[async_trait]
pub trait SeriesUserNoteService: Send + Sync {
    /// Series User Note를 생성하거나 업데이트합니다.
    ///
    /// # 비즈니스 규칙
    /// - Series가 존재해야 함
    /// - 사용자가 존재해야 함
    /// - project_id가 있는 경우, 사용자가 해당 프로젝트의 멤버여야 함
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note, Some(id)이면 프로젝트별 note)
    /// - `note`: 메모 텍스트
    ///
    /// # 반환값
    /// - `Ok(SeriesUserNote)`: 생성 또는 업데이트된 note
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn create_or_update_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        note: String,
    ) -> Result<SeriesUserNote, ServiceError>;

    /// 특정 Series, User, Project 조합의 note를 조회합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note 조회, Some(id)이면 프로젝트별 note 조회)
    ///
    /// # 반환값
    /// - `Ok(Some(SeriesUserNote))`: note가 존재하는 경우
    /// - `Ok(None)`: note가 존재하지 않는 경우
    /// - `Err(ServiceError)`: 데이터베이스 오류
    async fn get_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserNote>, ServiceError>;

    /// 특정 Series의 모든 note를 조회합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `project_id`: 프로젝트 ID 필터 (None이면 모든 note, Some(id)이면 해당 프로젝트의 note만)
    ///
    /// # 반환값
    /// - `Ok(Vec<SeriesUserNote>)`: 조회된 note 목록
    /// - `Err(ServiceError)`: 데이터베이스 오류
    async fn get_notes_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserNote>, ServiceError>;

    /// Series User Note를 삭제합니다.
    ///
    /// # 비즈니스 규칙
    /// - Series가 존재해야 함
    /// - 사용자가 존재해야 함
    /// - project_id가 있는 경우, 사용자가 해당 프로젝트의 멤버여야 함
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note 삭제, Some(id)이면 프로젝트별 note 삭제)
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn delete_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<(), ServiceError>;
}

/// Series User Note Service 구현체
///
/// 이 구조체는 SeriesUserNoteService 트레이트를 구현합니다.
/// 제네릭을 사용하여 Repository 의존성을 주입받습니다.
#[derive(Clone)]
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
    project_data_repository: Arc<PD>,
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
        project_data_repository: Arc<PD>,
    ) -> Self {
        Self {
            note_repository: Arc::new(note_repository),
            user_repository: Arc::new(user_repository),
            project_repository: Arc::new(project_repository),
            project_data_repository,
        }
    }
}

#[async_trait]
impl<N, U, P, PD> SeriesUserNoteService for SeriesUserNoteServiceImpl<N, U, P, PD>
where
    N: SeriesUserNoteRepository,
    U: UserRepository,
    P: ProjectRepository,
    PD: ProjectDataRepository,
{
    async fn create_or_update_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        note: String,
    ) -> Result<SeriesUserNote, ServiceError> {
        // 사용자 존재 확인
        if self
            .user_repository
            .as_ref()
            .find_by_id(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .is_none()
        {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // Series 존재 확인
        if self
            .project_data_repository
            .as_ref()
            .find_series_by_id(series_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .is_none()
        {
            return Err(ServiceError::NotFound("Series not found".into()));
        }

        // project_id가 있는 경우, 프로젝트 멤버십 확인
        if let Some(pid) = project_id {
            // 프로젝트 존재 확인
            if self
                .project_repository
                .as_ref()
                .find_by_id(pid)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                .is_none()
            {
                return Err(ServiceError::NotFound("Project not found".into()));
            }

            // 사용자가 프로젝트 멤버인지 확인
            let is_member = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2",
            )
            .bind(user_id)
            .bind(pid)
            .fetch_one(self.note_repository.as_ref().pool())
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            if is_member == 0 {
                return Err(ServiceError::Unauthorized(
                    "User is not a member of this project".into(),
                ));
            }
        }

        // Note 생성 또는 업데이트
        Ok(self
            .note_repository
            .as_ref()
            .create_or_update(series_id, user_id, project_id, note)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?)
    }

    async fn get_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserNote>, ServiceError> {
        Ok(self
            .note_repository
            .as_ref()
            .find_by_series_user_project(series_id, user_id, project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?)
    }

    async fn get_notes_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserNote>, ServiceError> {
        Ok(self
            .note_repository
            .as_ref()
            .find_by_series(series_id, project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?)
    }

    async fn delete_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        // 사용자 존재 확인
        if self
            .user_repository
            .as_ref()
            .find_by_id(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .is_none()
        {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // Series 존재 확인
        if self
            .project_data_repository
            .as_ref()
            .find_series_by_id(series_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .is_none()
        {
            return Err(ServiceError::NotFound("Series not found".into()));
        }

        // project_id가 있는 경우, 프로젝트 멤버십 확인
        if let Some(pid) = project_id {
            // 프로젝트 존재 확인
            if self
                .project_repository
                .as_ref()
                .find_by_id(pid)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                .is_none()
            {
                return Err(ServiceError::NotFound("Project not found".into()));
            }

            // 사용자가 프로젝트 멤버인지 확인
            let is_member = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2",
            )
            .bind(user_id)
            .bind(pid)
            .fetch_one(self.note_repository.as_ref().pool())
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            if is_member == 0 {
                return Err(ServiceError::Unauthorized(
                    "User is not a member of this project".into(),
                ));
            }
        }

        // Note 삭제
        let deleted = self
            .note_repository
            .as_ref()
            .delete(series_id, user_id, project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Note not found".into()));
        }

        Ok(())
    }
}
