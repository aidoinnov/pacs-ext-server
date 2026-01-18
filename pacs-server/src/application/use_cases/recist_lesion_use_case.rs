//! RECIST Lesion Use Case
//!
//! RECIST Lesion 관리를 위한 비즈니스 로직을 처리합니다.

use crate::domain::entities::{
    CreateRecistLesion, CreateRecistLesionAnnotationMap, RecistLesion, RecistLesionDetail,
    RecistLesionType, UpdateRecistLesion,
};
use crate::domain::repositories::{RecistLesionRepository, SubjectRepository, TimePointRepository};
use crate::domain::ServiceError;

/// RECIST Lesion Use Case
pub struct RecistLesionUseCase<R: RecistLesionRepository, S: SubjectRepository, T: TimePointRepository> {
    lesion_repo: R,
    subject_repo: S,
    timepoint_repo: T,
}

impl<R: RecistLesionRepository, S: SubjectRepository, T: TimePointRepository>
    RecistLesionUseCase<R, S, T>
{
    pub fn new(lesion_repo: R, subject_repo: S, timepoint_repo: T) -> Self {
        Self {
            lesion_repo,
            subject_repo,
            timepoint_repo,
        }
    }

    /// Lesion 생성
    ///
    /// # Business Rules
    /// - Subject가 존재해야 함
    /// - Baseline TimePoint가 존재해야 함
    /// - Target Lesion은 최대 5개까지만 허용
    /// - Non-Target Lesion은 제한 없음
    pub async fn create_lesion(
        &self,
        new_lesion: CreateRecistLesion,
    ) -> Result<RecistLesion, ServiceError> {
        // 1. Subject 존재 확인
        let subject = self
            .subject_repo
            .find_by_id(new_lesion.subject_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".to_string()))?;

        // 2. Baseline TimePoint 존재 확인 (Optional)
        if let Some(baseline_tp_id) = new_lesion.baseline_timepoint_id {
            self.timepoint_repo
                .find_by_id(baseline_tp_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                .ok_or_else(|| ServiceError::NotFound("Baseline TimePoint not found".to_string()))?;
        }

        // 3. Target Lesion 개수 제한 확인 (최대 5개)
        if new_lesion.lesion_type == RecistLesionType::Target {
            let existing_targets = self
                .lesion_repo
                .find_by_subject(new_lesion.subject_id, Some(RecistLesionType::Target))
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            if existing_targets.len() >= 5 {
                return Err(ServiceError::ValidationError(
                    "Maximum 5 Target Lesions allowed per Subject".to_string(),
                ));
            }
        }

        // 4. Lesion 생성 (lesion_number는 Repository에서 자동 생성)
        self.lesion_repo
            .create(new_lesion)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Lesion 조회
    pub async fn get_lesion(&self, id: i32) -> Result<RecistLesion, ServiceError> {
        self.lesion_repo
            .find_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Lesion not found".to_string()))
    }

    /// Lesion 상세 조회 (Annotation 포함)
    pub async fn get_lesion_detail(&self, id: i32) -> Result<RecistLesionDetail, ServiceError> {
        self.lesion_repo
            .find_detail_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Lesion not found".to_string()))
    }

    /// Subject의 Lesion 목록 조회
    pub async fn list_lesions_by_subject(
        &self,
        subject_id: i32,
        lesion_type: Option<RecistLesionType>,
    ) -> Result<Vec<RecistLesion>, ServiceError> {
        // Subject 존재 확인
        self.subject_repo
            .find_by_id(subject_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".to_string()))?;

        self.lesion_repo
            .find_by_subject(subject_id, lesion_type)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Lesion 수정
    pub async fn update_lesion(
        &self,
        id: i32,
        update_data: UpdateRecistLesion,
    ) -> Result<RecistLesion, ServiceError> {
        // Lesion 존재 확인
        self.get_lesion(id).await?;

        self.lesion_repo
            .update(id, update_data)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Lesion 삭제
    pub async fn delete_lesion(&self, id: i32) -> Result<(), ServiceError> {
        // Lesion 존재 확인
        self.get_lesion(id).await?;

        self.lesion_repo
            .delete(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Lesion-Annotation 매핑 생성
    ///
    /// # Business Rules
    /// - Lesion이 존재해야 함
    /// - TimePoint가 존재해야 함
    /// - Annotation ID는 유효해야 함 (실제로는 Annotation Repository로 확인 필요)
    pub async fn link_annotation(
        &self,
        mapping: CreateRecistLesionAnnotationMap,
    ) -> Result<(), ServiceError> {
        // 1. Lesion 존재 확인
        self.get_lesion(mapping.lesion_id).await?;

        // 2. TimePoint 존재 확인
        self.timepoint_repo
            .find_by_id(mapping.timepoint_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".to_string()))?;

        // 3. 매핑 생성
        self.lesion_repo
            .create_annotation_mapping(mapping)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

