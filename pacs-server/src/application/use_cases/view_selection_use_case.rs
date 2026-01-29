//! # ViewSelection Use Case 모듈
//!
//! 이 모듈은 ViewSelection과 관련된 비즈니스 로직을 처리하는 Use Case를 정의합니다.
//! Use Case는 도메인 서비스를 조합하여 특정 비즈니스 흐름을 구현합니다.

use crate::application::dto::view_selection_dto::*;
use crate::domain::view_selection::{SelectedSeries, ViewSelectionService};
use crate::domain::ServiceError;
use std::sync::Arc;

/// ViewSelection 관리를 위한 Use Case
///
/// 이 구조체는 ViewSelection과 관련된 모든 비즈니스 로직을 처리합니다.
/// 도메인 서비스를 조합하여 특정 비즈니스 흐름을 구현합니다.
pub struct ViewSelectionUseCase<S>
where
    S: ViewSelectionService,
{
    selection_service: Arc<S>,
    default_ttl_sec: u64,
}

impl<S> ViewSelectionUseCase<S>
where
    S: ViewSelectionService,
{
    /// 새로운 ViewSelection Use Case를 생성합니다.
    ///
    /// # 매개변수
    /// - `selection_service`: ViewSelection 도메인 서비스
    /// - `default_ttl_sec`: 기본 TTL (초 단위)
    ///
    /// # 반환값
    /// 생성된 `ViewSelectionUseCase` 인스턴스
    pub fn new(selection_service: Arc<S>, default_ttl_sec: u64) -> Self {
        Self {
            selection_service,
            default_ttl_sec,
        }
    }

    /// ViewSelection을 생성합니다.
    ///
    /// # 매개변수
    /// - `request`: 생성 요청 DTO
    /// - `user_id`: 생성한 사용자 ID
    /// - `ttl_sec`: TTL (초 단위, None이면 기본값 사용)
    ///
    /// # 반환값
    /// - `Ok(CreateViewSelectionResponse)`: 생성된 Selection ID
    /// - `Err(ServiceError)`: 생성 실패
    pub async fn create_selection(
        &self,
        request: CreateViewSelectionRequest,
        user_id: i32,
        ttl_sec: Option<u64>,
    ) -> Result<CreateViewSelectionResponse, ServiceError> {
        // Series 목록 검증
        if request.series.is_empty() {
            return Err(ServiceError::ValidationError(
                "Series list cannot be empty".to_string(),
            ));
        }

        // DTO를 Domain 엔티티로 변환
        let series: Vec<SelectedSeries> = request
            .series
            .into_iter()
            .map(SelectedSeries::from)
            .collect();

        let layout = request.layout.map(|l| l.into());
        let initial_views = request.initial_views.map(|views| {
            views.into_iter().map(|v| v.into()).collect()
        });

        // TTL 결정
        let ttl = ttl_sec.unwrap_or(self.default_ttl_sec);

        // Selection 생성
        let selection = self
            .selection_service
            .create_selection(series, layout, initial_views, user_id, ttl)
            .await?;

        Ok(CreateViewSelectionResponse {
            selection_id: selection.selection_id,
        })
    }

    /// Selection ID로 ViewSelection을 조회합니다.
    ///
    /// # 매개변수
    /// - `selection_id`: 조회할 Selection ID
    ///
    /// # 반환값
    /// - `Ok(Some(ViewSelectionResponse))`: Selection이 존재하는 경우
    /// - `Ok(None)`: Selection이 존재하지 않거나 만료된 경우
    /// - `Err(ServiceError)`: 조회 실패
    pub async fn get_selection(
        &self,
        selection_id: &str,
    ) -> Result<Option<ViewSelectionResponse>, ServiceError> {
        let selection = self.selection_service.get_selection(selection_id).await?;

        Ok(selection.map(ViewSelectionResponse::from))
    }

    /// Selection의 TTL을 연장합니다 (touch).
    ///
    /// # 매개변수
    /// - `selection_id`: TTL을 연장할 Selection ID
    /// - `ttl_sec`: 새로운 TTL (초 단위, None이면 기본값 사용)
    ///
    /// # 반환값
    /// - `Ok(())`: TTL 연장 성공
    /// - `Err(ServiceError)`: TTL 연장 실패
    pub async fn extend_ttl(
        &self,
        selection_id: &str,
        ttl_sec: Option<u64>,
    ) -> Result<(), ServiceError> {
        let ttl = ttl_sec.unwrap_or(self.default_ttl_sec);
        self.selection_service.extend_ttl(selection_id, ttl).await
    }

    /// Selection을 삭제합니다.
    ///
    /// # 매개변수
    /// - `selection_id`: 삭제할 Selection ID
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(ServiceError)`: 삭제 실패
    pub async fn delete_selection(&self, selection_id: &str) -> Result<(), ServiceError> {
        self.selection_service.delete_selection(selection_id).await
    }
}


