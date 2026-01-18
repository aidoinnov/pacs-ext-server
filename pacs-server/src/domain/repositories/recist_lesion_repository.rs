use crate::domain::entities::{
    CreateRecistLesion, CreateRecistLesionAnnotationMap, RecistLesion, RecistLesionAnnotationInfo,
    RecistLesionAnnotationMap, RecistLesionDetail, RecistLesionType, UpdateRecistLesion,
};
use async_trait::async_trait;

/// RECIST Lesion Repository Trait
///
/// RECIST Lesion 엔티티에 대한 데이터 접근 인터페이스를 정의합니다.
/// 이 트레이트는 도메인 계층에서 정의되며, 인프라 계층에서 구현됩니다.
#[async_trait]
pub trait RecistLesionRepository: Send + Sync {
    /// ID로 Lesion 조회
    ///
    /// # Arguments
    /// * `id` - Lesion ID
    ///
    /// # Returns
    /// * `Ok(Some(RecistLesion))` - Lesion이 존재하는 경우
    /// * `Ok(None)` - Lesion이 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_id(&self, id: i32) -> Result<Option<RecistLesion>, sqlx::Error>;

    /// Subject의 모든 Lesion 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    /// * `lesion_type` - Lesion 타입 필터 (선택사항)
    ///
    /// # Returns
    /// * `Ok(Vec<RecistLesion>)` - Lesion 목록
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_subject(
        &self,
        subject_id: i32,
        lesion_type: Option<RecistLesionType>,
    ) -> Result<Vec<RecistLesion>, sqlx::Error>;

    /// Lesion 상세 정보 조회 (Annotation 포함)
    ///
    /// # Arguments
    /// * `id` - Lesion ID
    ///
    /// # Returns
    /// * `Ok(Some(RecistLesionDetail))` - Lesion 상세 정보
    /// * `Ok(None)` - Lesion이 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_detail_by_id(&self, id: i32) -> Result<Option<RecistLesionDetail>, sqlx::Error>;

    /// Lesion 생성
    ///
    /// # Arguments
    /// * `new_lesion` - 생성할 Lesion 정보
    ///
    /// # Returns
    /// * `Ok(RecistLesion)` - 생성된 Lesion
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn create(&self, new_lesion: CreateRecistLesion) -> Result<RecistLesion, sqlx::Error>;

    /// Lesion 수정
    ///
    /// # Arguments
    /// * `id` - Lesion ID
    /// * `update_data` - 수정할 Lesion 정보
    ///
    /// # Returns
    /// * `Ok(RecistLesion)` - 수정된 Lesion
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn update(
        &self,
        id: i32,
        update_data: UpdateRecistLesion,
    ) -> Result<RecistLesion, sqlx::Error>;

    /// Lesion 삭제
    ///
    /// # Arguments
    /// * `id` - Lesion ID
    ///
    /// # Returns
    /// * `Ok(())` - 삭제 성공
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error>;

    /// Subject의 다음 Lesion Number 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(i32)` - 다음 Lesion Number (1부터 시작)
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn get_next_lesion_number(&self, subject_id: i32) -> Result<i32, sqlx::Error>;

    /// Lesion-Annotation 매핑 생성
    ///
    /// # Arguments
    /// * `mapping` - 생성할 매핑 정보
    ///
    /// # Returns
    /// * `Ok(RecistLesionAnnotationMap)` - 생성된 매핑
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn create_annotation_mapping(
        &self,
        mapping: CreateRecistLesionAnnotationMap,
    ) -> Result<RecistLesionAnnotationMap, sqlx::Error>;

    /// Lesion의 Annotation 목록 조회
    ///
    /// # Arguments
    /// * `lesion_id` - Lesion ID
    ///
    /// # Returns
    /// * `Ok(Vec<RecistLesionAnnotationInfo>)` - Annotation 정보 목록
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_annotations_by_lesion(
        &self,
        lesion_id: i32,
    ) -> Result<Vec<RecistLesionAnnotationInfo>, sqlx::Error>;

    /// TimePoint의 Annotation ID로 Lesion 조회
    ///
    /// # Arguments
    /// * `annotation_id` - Annotation ID
    ///
    /// # Returns
    /// * `Ok(Option<RecistLesion>)` - Lesion 정보
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_annotation_id(
        &self,
        annotation_id: i32,
    ) -> Result<Option<RecistLesion>, sqlx::Error>;

    /// Lesion-Annotation 매핑 삭제
    ///
    /// # Arguments
    /// * `annotation_id` - Annotation ID
    ///
    /// # Returns
    /// * `Ok(())` - 삭제 성공
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn delete_annotation_mapping(&self, annotation_id: i32) -> Result<(), sqlx::Error>;
}

