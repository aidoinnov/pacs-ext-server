//! # Series User Note Use Case 모듈
//!
//! 이 모듈은 Series User Note와 관련된 비즈니스 로직을 처리하는 Use Case를 정의합니다.
//! Use Case는 도메인 서비스를 조합하여 특정 비즈니스 흐름을 구현합니다.

use crate::application::dto::series_user_note_dto::*;
use crate::domain::entities::SeriesUserNote;
use crate::domain::repositories::UserRepository;
use crate::domain::services::SeriesUserNoteService;
use crate::domain::ServiceError;
use std::collections::HashMap;
use std::sync::Arc;
use crate::application::dto::series_user_note_dto::SeriesNoteUserInfo;

/// Series User Note 관리를 위한 Use Case
///
/// 이 구조체는 Series User Note와 관련된 모든 비즈니스 로직을 처리합니다.
/// 도메인 서비스를 조합하여 특정 비즈니스 흐름을 구현합니다.
pub struct SeriesUserNoteUseCase<S, U>
where
    S: SeriesUserNoteService,
    U: UserRepository,
{
    note_service: Arc<S>,
    user_repository: Arc<U>,
}

impl<S, U> SeriesUserNoteUseCase<S, U>
where
    S: SeriesUserNoteService,
    U: UserRepository,
{
    /// 새로운 Series User Note Use Case를 생성합니다.
    ///
    /// # 매개변수
    /// - `note_service`: Series User Note 도메인 서비스
    /// - `user_repository`: 사용자 리포지토리 (사용자 정보 조회용)
    ///
    /// # 반환값
    /// 생성된 `SeriesUserNoteUseCase` 인스턴스
    pub fn new(note_service: Arc<S>, user_repository: Arc<U>) -> Self {
        Self {
            note_service,
            user_repository,
        }
    }

    /// Series User Note를 생성하거나 업데이트합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note, Some(id)이면 프로젝트별 note)
    /// - `request`: 생성/수정 요청 DTO
    ///
    /// # 반환값
    /// - `Ok(SeriesNoteResponse)`: 생성 또는 업데이트된 note
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    pub async fn create_or_update_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        request: CreateOrUpdateSeriesNoteRequest,
    ) -> Result<SeriesNoteResponse, ServiceError> {
        // Note 생성 또는 업데이트
        let note = self
            .note_service
            .as_ref()
            .create_or_update_note(series_id, user_id, project_id, request.note)
            .await?;

        // Entity를 DTO로 변환
        Ok(self.to_response(note))
    }

    /// 특정 Series, User, Project 조합의 note를 조회합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note 조회, Some(id)이면 프로젝트별 note 조회)
    ///
    /// # 반환값
    /// - `Ok(Option<SeriesNoteResponse>)`: note가 존재하는 경우 Some, 없으면 None
    /// - `Err(ServiceError)`: 데이터베이스 오류
    pub async fn get_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesNoteResponse>, ServiceError> {
        let note = self
            .note_service
            .as_ref()
            .get_note(series_id, user_id, project_id)
            .await?;

        Ok(note.map(|n| self.to_response(n)))
    }

    /// 특정 Series의 모든 note를 조회합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `project_id`: 프로젝트 ID 필터 (None이면 모든 note, Some(id)이면 해당 프로젝트의 note만)
    ///
    /// # 반환값
    /// - `Ok(Vec<SeriesNoteWithUserResponse>)`: 조회된 note 목록 (사용자 정보 포함)
    /// - `Err(ServiceError)`: 데이터베이스 오류
    pub async fn get_notes_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesNoteWithUserResponse>, ServiceError> {
        // Note 목록 조회
        let notes = self
            .note_service
            .as_ref()
            .get_notes_by_series(series_id, project_id)
            .await?;

        if notes.is_empty() {
            return Ok(Vec::new());
        }

        // 사용자 ID 목록 추출
        let user_ids: Vec<i32> = notes.iter().map(|n| n.user_id).collect();

        // 사용자 정보 일괄 조회 (N+1 문제 방지)
        let user_map = self.fetch_users_batch(&user_ids).await?;

        // Entity를 DTO로 변환 (사용자 정보 포함)
        let responses: Vec<SeriesNoteWithUserResponse> = notes
            .into_iter()
            .filter_map(|note| {
                user_map.get(&note.user_id).map(|user| SeriesNoteWithUserResponse {
                        id: note.id,
                        series_id: note.series_id,
                        user: user.clone(),
                    project_id: note.project_id,
                    note: note.note,
                    created_at: note.created_at,
                    updated_at: note.updated_at,
                })
            })
            .collect();

        Ok(responses)
    }

    /// Series User Note를 삭제합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note 삭제, Some(id)이면 프로젝트별 note 삭제)
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    pub async fn delete_note(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        self.note_service
            .as_ref()
            .delete_note(series_id, user_id, project_id)
            .await
    }

    /// SeriesUserNote 엔티티를 SeriesNoteResponse DTO로 변환합니다.
    ///
    /// # 매개변수
    /// - `note`: 변환할 SeriesUserNote 엔티티
    ///
    /// # 반환값
    /// 변환된 SeriesNoteResponse DTO
    fn to_response(&self, note: SeriesUserNote) -> SeriesNoteResponse {
        SeriesNoteResponse {
            id: note.id,
            series_id: note.series_id,
            user_id: note.user_id,
            project_id: note.project_id,
            note: note.note,
            created_at: note.created_at,
            updated_at: note.updated_at,
        }
    }

    /// 여러 사용자 정보를 일괄 조회합니다 (N+1 문제 방지).
    ///
    /// # 매개변수
    /// - `user_ids`: 조회할 사용자 ID 목록
    ///
    /// # 반환값
    /// - `Ok(HashMap<i32, UserInfo>)`: 사용자 ID를 키로 하는 사용자 정보 맵
    /// - `Err(ServiceError)`: 데이터베이스 오류
    async fn fetch_users_batch(
        &self,
        user_ids: &[i32],
    ) -> Result<HashMap<i32, SeriesNoteUserInfo>, ServiceError> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut user_map = HashMap::new();

        for user_id in user_ids {
            if let Ok(Some(user)) = self.user_repository.as_ref().find_by_id(*user_id).await {
                user_map.insert(
                    *user_id,
                    SeriesNoteUserInfo {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        full_name: user.full_name,
                    },
                );
            }
        }

        Ok(user_map)
    }
}

