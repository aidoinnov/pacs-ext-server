//! # Report Guide Template Repository 트레이트
//!
//! 이 모듈은 리포트 가이드 템플릿 데이터 접근을 위한 Repository 트레이트를 정의합니다.

use crate::domain::template::entities::report_guide_template::*;
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait ReportGuideTemplateRepository: Send + Sync {
    // ========== 원본 템플릿 ==========
    
    /// 원본 템플릿 생성
    async fn create_template(
        &self,
        new_template: &NewReportGuideTemplate,
    ) -> Result<ReportGuideTemplate, sqlx::Error>;

    /// 원본 템플릿 조회
    async fn find_template_by_id(&self, id: i32) -> Result<Option<ReportGuideTemplate>, sqlx::Error>;

    /// 원본 템플릿 목록 조회 (필터: 모달리티, bodypart, is_active)
    async fn find_templates(
        &self,
        modality: Option<&str>,
        bodypart: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<ReportGuideTemplate>, sqlx::Error>;

    /// 원본 템플릿 업데이트
    async fn update_template(
        &self,
        id: i32,
        update: &UpdateReportGuideTemplate,
    ) -> Result<ReportGuideTemplate, sqlx::Error>;

    /// 원본 템플릿 삭제
    async fn delete_template(&self, id: i32) -> Result<bool, sqlx::Error>;

    // ========== 템플릿 모달리티 ==========

    /// 템플릿 모달리티 추가
    async fn add_modality(
        &self,
        template_id: i32,
        modality: &str,
    ) -> Result<ReportGuideTemplateModality, sqlx::Error>;

    /// 템플릿 모달리티 목록 조회
    async fn find_modalities_by_template(
        &self,
        template_id: i32,
    ) -> Result<Vec<ReportGuideTemplateModality>, sqlx::Error>;

    /// 템플릿 모달리티 제거
    async fn remove_modality(&self, template_id: i32, modality: &str) -> Result<bool, sqlx::Error>;

    // ========== 템플릿 이미지 ==========

    /// 템플릿 이미지 추가
    async fn add_template_image(
        &self,
        new_image: &NewReportGuideTemplateImage,
    ) -> Result<ReportGuideTemplateImage, sqlx::Error>;

    /// 템플릿 이미지 목록 조회
    async fn find_template_images(
        &self,
        template_id: i32,
    ) -> Result<Vec<ReportGuideTemplateImage>, sqlx::Error>;

    /// 템플릿 이미지 조회
    async fn find_template_image_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ReportGuideTemplateImage>, sqlx::Error>;

    /// 템플릿 이미지 공유 설정 변경
    async fn update_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
    ) -> Result<ReportGuideTemplateImage, sqlx::Error>;

    /// 템플릿 이미지 삭제
    async fn delete_template_image(&self, image_id: i32) -> Result<bool, sqlx::Error>;

    // ========== 사용자 커스텀 템플릿 ==========

    /// 커스텀 템플릿 생성
    async fn create_custom_template(
        &self,
        new_template: &NewUserCustomReportTemplate,
    ) -> Result<UserCustomReportTemplate, sqlx::Error>;

    /// 커스텀 템플릿 조회
    async fn find_custom_template_by_id(
        &self,
        id: i32,
    ) -> Result<Option<UserCustomReportTemplate>, sqlx::Error>;

    /// 사용자의 커스텀 템플릿 목록 조회
    async fn find_custom_templates_by_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserCustomReportTemplate>, sqlx::Error>;

    /// 커스텀 템플릿 업데이트
    async fn update_custom_template(
        &self,
        id: i32,
        update: &UpdateUserCustomReportTemplate,
    ) -> Result<UserCustomReportTemplate, sqlx::Error>;

    /// 커스텀 템플릿 삭제
    async fn delete_custom_template(&self, id: i32) -> Result<bool, sqlx::Error>;

    // ========== 커스텀 템플릿 모달리티 ==========

    /// 커스텀 템플릿 모달리티 추가
    async fn add_custom_modality(
        &self,
        custom_template_id: i32,
        modality: &str,
    ) -> Result<UserCustomTemplateModality, sqlx::Error>;

    /// 커스텀 템플릿 모달리티 목록 조회
    async fn find_custom_modalities_by_template(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<UserCustomTemplateModality>, sqlx::Error>;

    /// 커스텀 템플릿 모달리티 제거
    async fn remove_custom_modality(
        &self,
        custom_template_id: i32,
        modality: &str,
    ) -> Result<bool, sqlx::Error>;

    // ========== 커스텀 템플릿 이미지 ==========

    /// 커스텀 템플릿 이미지 추가
    async fn add_custom_template_image(
        &self,
        new_image: &NewUserCustomTemplateImage,
    ) -> Result<UserCustomTemplateImage, sqlx::Error>;

    /// 커스텀 템플릿 이미지 목록 조회
    async fn find_custom_template_images(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<UserCustomTemplateImage>, sqlx::Error>;

    /// 커스텀 템플릿 이미지 삭제
    async fn delete_custom_template_image(&self, image_id: i32) -> Result<bool, sqlx::Error>;

    // ========== Report-가이드 매핑 ==========

    /// Report-가이드 매핑 추가
    async fn add_report_guide(
        &self,
        new_guide: &NewSeriesUserReportGuide,
    ) -> Result<SeriesUserReportGuide, sqlx::Error>;

    /// Report의 가이드 목록 조회
    async fn find_report_guides(
        &self,
        report_id: i32,
    ) -> Result<Vec<SeriesUserReportGuide>, sqlx::Error>;

    /// Report-가이드 매핑 삭제
    async fn delete_report_guide(&self, guide_id: i32) -> Result<bool, sqlx::Error>;

    /// Report의 모든 가이드 삭제
    async fn delete_all_report_guides(&self, report_id: i32) -> Result<usize, sqlx::Error>;

    /// 데이터베이스 풀 참조
    fn pool(&self) -> &PgPool;
}

