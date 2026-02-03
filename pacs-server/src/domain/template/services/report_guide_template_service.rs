//! # Report Guide Template Service 트레이트
//!
//! 이 모듈은 리포트 가이드 템플릿 비즈니스 로직을 위한 Service 트레이트를 정의합니다.

use crate::domain::reporting::repositories::SeriesUserReportRepository;
use crate::domain::template::entities::report_guide_template::*;
use crate::domain::template::repositories::ReportGuideTemplateRepository;
use crate::domain::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait ReportGuideTemplateService: Send + Sync + 'static {
    // ========== 원본 템플릿 ==========

    /// 원본 템플릿 생성
    async fn create_template(
        &self,
        new_template: NewReportGuideTemplate,
        modalities: Vec<String>,
    ) -> Result<ReportGuideTemplate, ServiceError>;

    /// 원본 템플릿 조회
    async fn get_template(&self, id: i32) -> Result<Option<ReportGuideTemplate>, ServiceError>;

    /// 원본 템플릿 목록 조회
    async fn get_templates(
        &self,
        modality: Option<String>,
        bodypart: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<ReportGuideTemplate>, ServiceError>;

    /// 원본 템플릿 업데이트
    async fn update_template(
        &self,
        id: i32,
        update: UpdateReportGuideTemplate,
    ) -> Result<ReportGuideTemplate, ServiceError>;

    /// 원본 템플릿 삭제
    async fn delete_template(&self, id: i32) -> Result<(), ServiceError>;

    /// 템플릿 모달리티 목록 조회
    async fn get_modalities_by_template(&self, template_id: i32) -> Result<Vec<String>, ServiceError>;

    /// 템플릿 모달리티 업데이트 (기존 삭제 후 새로 생성)
    async fn update_template_modalities(
        &self,
        template_id: i32,
        modalities: Vec<String>,
    ) -> Result<(), ServiceError>;

    // ========== 독립적인 가이드 이미지 ==========

    /// 가이드 이미지 생성
    async fn create_guide_image(
        &self,
        new_image: NewGuideImage,
    ) -> Result<GuideImage, ServiceError>;

    /// 가이드 이미지 조회
    async fn get_guide_image(&self, id: i32) -> Result<Option<GuideImage>, ServiceError>;

    /// 사용자가 업로드한 가이드 이미지 목록 조회
    async fn get_user_guide_images(
        &self,
        user_id: i32,
        is_shared: Option<bool>,
    ) -> Result<Vec<GuideImage>, ServiceError>;

    /// 가이드 이미지 공유 설정 변경
    async fn update_guide_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
        user_id: i32, // 권한 검증용
    ) -> Result<GuideImage, ServiceError>;

    /// 가이드 이미지 삭제
    async fn delete_guide_image(
        &self,
        image_id: i32,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    // ========== 템플릿-이미지 매핑 ==========

    /// 템플릿의 이미지 매핑 업데이트 (기존 매핑 삭제 후 새로 생성)
    /// 권한 검증: image_ids의 모든 이미지가 접근 가능한지 확인
    async fn update_template_image_mappings(
        &self,
        template_id: i32,
        image_ids: Vec<i32>,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    /// 템플릿의 가이드 이미지 목록 조회 (권한 필터링 적용)
    async fn get_template_guide_images(
        &self,
        template_id: i32,
        user_id: Option<i32>, // 권한 필터링용
    ) -> Result<Vec<GuideImage>, ServiceError>;

    // ========== 커스텀 템플릿-이미지 매핑 ==========

    /// 커스텀 템플릿의 이미지 매핑 업데이트 (기존 매핑 삭제 후 새로 생성)
    /// 권한 검증: image_ids의 모든 이미지가 접근 가능한지 확인
    async fn update_custom_template_image_mappings(
        &self,
        custom_template_id: i32,
        image_ids: Vec<i32>,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    /// 커스텀 템플릿의 가이드 이미지 목록 조회 (권한 필터링 적용)
    async fn get_custom_template_guide_images(
        &self,
        custom_template_id: i32,
        user_id: Option<i32>, // 권한 필터링용
    ) -> Result<Vec<GuideImage>, ServiceError>;

    // ========== 템플릿 이미지 (기존 구조 - 하위 호환성) ==========

    /// 템플릿 이미지 추가
    async fn add_template_image(
        &self,
        template_id: i32,
        new_image: NewReportGuideTemplateImage,
    ) -> Result<ReportGuideTemplateImage, ServiceError>;

    /// 템플릿 이미지 목록 조회 (이미지 소유권 필터링)
    async fn get_template_images(
        &self,
        template_id: i32,
        user_id: Option<i32>, // 이미지 접근 권한 검증용
    ) -> Result<Vec<ReportGuideTemplateImage>, ServiceError>;

    /// 사용자가 업로드한 모든 템플릿 이미지 조회
    async fn get_user_uploaded_images(
        &self,
        user_id: i32,
    ) -> Result<Vec<ReportGuideTemplateImage>, ServiceError>;

    /// 템플릿 이미지 공유 설정 변경
    async fn update_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
        user_id: i32, // 권한 검증용
    ) -> Result<ReportGuideTemplateImage, ServiceError>;

    /// 템플릿 이미지 삭제
    async fn delete_template_image(
        &self,
        image_id: i32,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    // ========== 사용자 커스텀 템플릿 ==========

    /// 원본 템플릿을 복사하여 커스텀 템플릿 생성
    async fn create_custom_template_from_base(
        &self,
        user_id: i32,
        base_template_id: i32,
    ) -> Result<UserCustomReportTemplate, ServiceError>;

    /// 커스텀 템플릿 생성 (원본 없이)
    async fn create_custom_template(
        &self,
        new_template: NewUserCustomReportTemplate,
        modalities: Vec<String>,
    ) -> Result<UserCustomReportTemplate, ServiceError>;

    /// 커스텀 템플릿 조회
    async fn get_custom_template(
        &self,
        id: i32,
        user_id: i32, // 권한 검증용
    ) -> Result<Option<UserCustomReportTemplate>, ServiceError>;

    /// 사용자의 커스텀 템플릿 목록 조회
    async fn get_custom_templates_by_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserCustomReportTemplate>, ServiceError>;

    /// 커스텀 템플릿 업데이트
    async fn update_custom_template(
        &self,
        id: i32,
        user_id: i32, // 권한 검증용
        update: UpdateUserCustomReportTemplate,
    ) -> Result<UserCustomReportTemplate, ServiceError>;

    /// 커스텀 템플릿 삭제
    async fn delete_custom_template(
        &self,
        id: i32,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    /// 커스텀 템플릿 모달리티 목록 조회
    async fn get_custom_modalities_by_template(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<String>, ServiceError>;

    /// 커스텀 템플릿 모달리티 업데이트 (기존 삭제 후 새로 생성)
    async fn update_custom_template_modalities(
        &self,
        custom_template_id: i32,
        modalities: Vec<String>,
    ) -> Result<(), ServiceError>;

    // ========== 커스텀 템플릿 이미지 ==========

    /// 커스텀 템플릿 이미지 추가 (본인 업로드 이미지만)
    async fn add_custom_template_image(
        &self,
        custom_template_id: i32,
        user_id: i32, // 권한 검증 및 이미지 소유권 확인용
        new_image: NewUserCustomTemplateImage,
    ) -> Result<UserCustomTemplateImage, ServiceError>;

    /// 커스텀 템플릿 이미지 목록 조회
    async fn get_custom_template_images(
        &self,
        custom_template_id: i32,
        user_id: i32, // 권한 검증용
    ) -> Result<Vec<UserCustomTemplateImage>, ServiceError>;

    /// 커스텀 템플릿 이미지 삭제
    async fn delete_custom_template_image(
        &self,
        image_id: i32,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    // ========== Report-템플릿 적용 ==========

    /// 템플릿을 Report에 적용 (description, conclusion, bodypart, 이미지 복사)
    async fn apply_template_to_report(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
        user_id: i32, // 권한 검증용
    ) -> Result<(), ServiceError>;

    // ========== Report 템플릿 + 이미지 스냅샷 (1:1) ==========

    /// Report의 이미지 스냅샷 목록 조회
    async fn get_report_guide_images(
        &self,
        report_id: i32,
    ) -> Result<Vec<(GuideImage, i32)>, ServiceError>;

    /// Report에 템플릿 적용 및 이미지 스냅샷 복사
    async fn set_report_template(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
        user_id: i32,
    ) -> Result<(), ServiceError>;

    /// Report에서 템플릿 및 이미지 제거
    async fn clear_report_template(&self, report_id: i32) -> Result<(), ServiceError>;
}

/// Report Guide Template Service 구현체
#[derive(Clone)]
pub struct ReportGuideTemplateServiceImpl<T, R>
where
    T: ReportGuideTemplateRepository + 'static,
    R: SeriesUserReportRepository + 'static,
{
    template_repository: Arc<T>,
    report_repository: Arc<R>,
}

impl<T, R> ReportGuideTemplateServiceImpl<T, R>
where
    T: ReportGuideTemplateRepository + 'static,
    R: SeriesUserReportRepository + 'static,
{
    pub fn new(template_repository: T, report_repository: R) -> Self {
        Self {
            template_repository: Arc::new(template_repository),
            report_repository: Arc::new(report_repository),
        }
    }
}

#[async_trait]
impl<T, R> ReportGuideTemplateService for ReportGuideTemplateServiceImpl<T, R>
where
    T: ReportGuideTemplateRepository + 'static,
    R: SeriesUserReportRepository + 'static,
{
    async fn create_template(
        &self,
        new_template: NewReportGuideTemplate,
        modalities: Vec<String>,
    ) -> Result<ReportGuideTemplate, ServiceError> {
        // 템플릿 생성
        let template = self
            .template_repository
            .as_ref()
            .create_template(&new_template)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 모달리티 추가
        for modality in modalities {
            self.template_repository
                .as_ref()
                .add_modality(template.id, &modality)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(template)
    }

    async fn get_template(&self, id: i32) -> Result<Option<ReportGuideTemplate>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_template_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_templates(
        &self,
        modality: Option<String>,
        bodypart: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<ReportGuideTemplate>, ServiceError> {
        // TODO: modality, bodypart 필터링 구현 필요
        self.template_repository
            .as_ref()
            .find_templates(None, None, is_active)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_template(
        &self,
        id: i32,
        update: UpdateReportGuideTemplate,
    ) -> Result<ReportGuideTemplate, ServiceError> {
        // 템플릿 존재 확인
        if self.get_template(id).await?.is_none() {
            return Err(ServiceError::NotFound("Template not found".into()));
        }

        self.template_repository
            .as_ref()
            .update_template(id, &update)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_template(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self
            .template_repository
            .as_ref()
            .delete_template(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Template not found".into()));
        }

        Ok(())
    }

    async fn get_modalities_by_template(&self, template_id: i32) -> Result<Vec<String>, ServiceError> {
        let modalities = self
            .template_repository
            .as_ref()
            .find_modalities_by_template(template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(modalities.into_iter().map(|m| m.modality).collect())
    }

    async fn update_template_modalities(
        &self,
        template_id: i32,
        modalities: Vec<String>,
    ) -> Result<(), ServiceError> {
        self.template_repository
            .as_ref()
            .delete_template_modalities_by_template(template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<_> = modalities
            .into_iter()
            .filter(|m| seen.insert(m.clone()))
            .collect();
        for modality in unique {
            let _ = self
                .template_repository
                .as_ref()
                .add_modality(template_id, &modality)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    // ========== 독립적인 가이드 이미지 ==========

    async fn create_guide_image(
        &self,
        new_image: NewGuideImage,
    ) -> Result<GuideImage, ServiceError> {
        self.template_repository
            .as_ref()
            .create_guide_image(&new_image)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_guide_image(&self, id: i32) -> Result<Option<GuideImage>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_guide_image_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_user_guide_images(
        &self,
        user_id: i32,
        is_shared: Option<bool>,
    ) -> Result<Vec<GuideImage>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_guide_images_by_user(user_id, is_shared)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_guide_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
        user_id: i32,
    ) -> Result<GuideImage, ServiceError> {
        // 이미지 소유권 확인
        let image = self
            .get_guide_image(image_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Image not found".into()))?;

        if image.uploaded_by != user_id {
            return Err(ServiceError::Forbidden(
                "You can only modify your own images".into(),
            ));
        }

        self.template_repository
            .as_ref()
            .update_guide_image_share_status(image_id, is_shared)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_guide_image(
        &self,
        image_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // guide_image 테이블만 대상. image_source="template" 이미지는 DELETE /api/report-guide-templates/{template_id}/images/{id} 사용
        let image = self
            .get_guide_image(image_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Image not found".into()))?;

        if image.uploaded_by != user_id {
            return Err(ServiceError::Forbidden(
                "You can only delete your own images".into(),
            ));
        }

        let deleted = self
            .template_repository
            .as_ref()
            .delete_guide_image(image_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Image not found".into()));
        }

        Ok(())
    }

    // ========== 템플릿-이미지 매핑 ==========

    async fn update_template_image_mappings(
        &self,
        template_id: i32,
        image_ids: Vec<i32>,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // 권한 검증: 모든 image_ids가 접근 가능한지 확인
        for image_id in &image_ids {
            let image = self
                .get_guide_image(*image_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Image {} not found", image_id)))?;

            // is_shared=true이거나 본인이 업로드한 이미지만 사용 가능
            if !image.is_shared && image.uploaded_by != user_id {
                return Err(ServiceError::Forbidden(format!(
                    "Cannot use private image {} from other users",
                    image_id
                )));
            }
        }

        // 기존 매핑 삭제
        self.template_repository
            .as_ref()
            .delete_template_image_mappings_by_template(template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 새 매핑 생성
        for (index, image_id) in image_ids.iter().enumerate() {
            let new_mapping = NewTemplateImageMapping {
                template_id,
                image_id: *image_id,
                display_order: index as i32,
            };

            self.template_repository
                .as_ref()
                .create_template_image_mapping(&new_mapping)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn get_template_guide_images(
        &self,
        template_id: i32,
        _user_id: Option<i32>,
    ) -> Result<Vec<GuideImage>, ServiceError> {
        // 템플릿에 지정된 이미지는 모두 접근 가능 (설계 요구사항 2)
        self.template_repository
            .as_ref()
            .find_guide_images_by_template(template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    // ========== 커스텀 템플릿-이미지 매핑 ==========

    async fn update_custom_template_image_mappings(
        &self,
        custom_template_id: i32,
        image_ids: Vec<i32>,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // 권한 검증: 모든 image_ids가 접근 가능한지 확인
        for image_id in &image_ids {
            let image = self
                .get_guide_image(*image_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Image {} not found", image_id)))?;

            // is_shared=true이거나 본인이 업로드한 이미지만 사용 가능
            if !image.is_shared && image.uploaded_by != user_id {
                return Err(ServiceError::Forbidden(format!(
                    "Cannot use private image {} from other users",
                    image_id
                )));
            }
        }

        // 기존 매핑 삭제
        self.template_repository
            .as_ref()
            .delete_custom_template_image_mappings_by_template(custom_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 새 매핑 생성
        for (index, image_id) in image_ids.iter().enumerate() {
            let new_mapping = NewCustomTemplateImageMapping {
                custom_template_id,
                image_id: *image_id,
                display_order: index as i32,
            };

            self.template_repository
                .as_ref()
                .create_custom_template_image_mapping(&new_mapping)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn get_custom_template_guide_images(
        &self,
        custom_template_id: i32,
        _user_id: Option<i32>,
    ) -> Result<Vec<GuideImage>, ServiceError> {
        // 템플릿에 지정된 이미지는 모두 접근 가능 (설계 요구사항 2)
        // 커스텀 템플릿은 get_custom_template에서 소유권 검증됨
        self.template_repository
            .as_ref()
            .find_guide_images_by_custom_template(custom_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    // ========== 템플릿 이미지 (기존 구조 - 하위 호환성) ==========

    async fn add_template_image(
        &self,
        template_id: i32,
        new_image: NewReportGuideTemplateImage,
    ) -> Result<ReportGuideTemplateImage, ServiceError> {
        // 템플릿 존재 확인
        if self.get_template(template_id).await?.is_none() {
            return Err(ServiceError::NotFound("Template not found".into()));
        }

        self.template_repository
            .as_ref()
            .add_template_image(&new_image)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_template_images(
        &self,
        template_id: i32,
        user_id: Option<i32>,
    ) -> Result<Vec<ReportGuideTemplateImage>, ServiceError> {
        let all_images = self
            .template_repository
            .as_ref()
            .find_template_images(template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 이미지 소유권 필터링: 공유 이미지 또는 본인이 업로드한 이미지만 반환
        if let Some(uid) = user_id {
            Ok(all_images
                .into_iter()
                .filter(|img| img.is_shared || img.uploaded_by == uid)
                .collect())
        } else {
            // user_id가 없으면 공유 이미지만 반환
            Ok(all_images.into_iter().filter(|img| img.is_shared).collect())
        }
    }

    async fn get_user_uploaded_images(
        &self,
        user_id: i32,
    ) -> Result<Vec<ReportGuideTemplateImage>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_template_images_by_user(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
        user_id: i32,
    ) -> Result<ReportGuideTemplateImage, ServiceError> {
        // 이미지 존재 및 소유권 확인
        let image = self
            .template_repository
            .as_ref()
            .find_template_image_by_id(image_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Image not found".into()))?;

        // 업로드한 사용자만 공유 설정 변경 가능
        if image.uploaded_by != user_id {
            return Err(ServiceError::Unauthorized(
                "Only the image uploader can change share status".into(),
            ));
        }

        self.template_repository
            .as_ref()
            .update_image_share_status(image_id, is_shared)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_template_image(
        &self,
        image_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // 이미지 존재 및 소유권 확인
        let image = self
            .template_repository
            .as_ref()
            .find_template_image_by_id(image_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Image not found".into()))?;

        // 업로드한 사용자만 삭제 가능
        if image.uploaded_by != user_id {
            return Err(ServiceError::Unauthorized(
                "Only the image uploader can delete the image".into(),
            ));
        }

        let deleted = self
            .template_repository
            .as_ref()
            .delete_template_image(image_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Image not found".into()));
        }

        Ok(())
    }

    async fn create_custom_template_from_base(
        &self,
        user_id: i32,
        base_template_id: i32,
    ) -> Result<UserCustomReportTemplate, ServiceError> {
        // 원본 템플릿 조회
        let base_template = self
            .get_template(base_template_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Base template not found".into()))?;

        // 커스텀 템플릿 생성 (원본 템플릿 값 복사)
        let new_custom = NewUserCustomReportTemplate {
            user_id,
            base_template_id: Some(base_template_id),
            description: base_template.description,
            conclusion: base_template.conclusion,
            bodypart: base_template.bodypart,
        };

        let custom_template = self
            .template_repository
            .as_ref()
            .create_custom_template(&new_custom)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 원본 템플릿의 모달리티 복사
        let modalities = self
            .template_repository
            .as_ref()
            .find_modalities_by_template(base_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        for modality in modalities {
            self.template_repository
                .as_ref()
                .add_custom_modality(custom_template.id, &modality.modality)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        // 원본 템플릿의 이미지를 매핑으로 참조 (새 구조: guide_image + mapping)
        let base_images = self
            .get_template_guide_images(base_template_id, None)
            .await?;
        for (idx, image) in base_images.iter().enumerate() {
            let new_mapping = NewCustomTemplateImageMapping {
                custom_template_id: custom_template.id,
                image_id: image.id,
                display_order: idx as i32,
            };

            self.template_repository
                .as_ref()
                .create_custom_template_image_mapping(&new_mapping)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(custom_template)
    }

    async fn create_custom_template(
        &self,
        new_template: NewUserCustomReportTemplate,
        modalities: Vec<String>,
    ) -> Result<UserCustomReportTemplate, ServiceError> {
        let template = self
            .template_repository
            .as_ref()
            .create_custom_template(&new_template)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 모달리티 추가
        for modality in modalities {
            self.template_repository
                .as_ref()
                .add_custom_modality(template.id, &modality)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(template)
    }

    async fn get_custom_template(
        &self,
        id: i32,
        user_id: i32,
    ) -> Result<Option<UserCustomReportTemplate>, ServiceError> {
        let template = self
            .template_repository
            .as_ref()
            .find_custom_template_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 소유권 확인
        if let Some(ref t) = template {
            if t.user_id != user_id {
                return Err(ServiceError::Unauthorized(
                    "You can only access your own custom templates".into(),
                ));
            }
        }

        Ok(template)
    }

    async fn get_custom_templates_by_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserCustomReportTemplate>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_custom_templates_by_user(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_custom_template(
        &self,
        id: i32,
        user_id: i32,
        update: UpdateUserCustomReportTemplate,
    ) -> Result<UserCustomReportTemplate, ServiceError> {
        // 소유권 확인
        let existing = self.get_custom_template(id, user_id).await?;
        if existing.is_none() {
            return Err(ServiceError::NotFound("Custom template not found".into()));
        }

        self.template_repository
            .as_ref()
            .update_custom_template(id, &update)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_custom_template(
        &self,
        id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // 소유권 확인
        let existing = self.get_custom_template(id, user_id).await?;
        if existing.is_none() {
            return Err(ServiceError::NotFound("Custom template not found".into()));
        }

        let deleted = self
            .template_repository
            .as_ref()
            .delete_custom_template(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Custom template not found".into()));
        }

        Ok(())
    }

    async fn get_custom_modalities_by_template(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<String>, ServiceError> {
        let modalities = self
            .template_repository
            .as_ref()
            .find_custom_modalities_by_template(custom_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(modalities.into_iter().map(|m| m.modality).collect())
    }

    async fn update_custom_template_modalities(
        &self,
        custom_template_id: i32,
        modalities: Vec<String>,
    ) -> Result<(), ServiceError> {
        self.template_repository
            .as_ref()
            .delete_custom_template_modalities_by_template(custom_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<_> = modalities
            .into_iter()
            .filter(|m| seen.insert(m.clone()))
            .collect();
        for modality in unique {
            self.template_repository
                .as_ref()
                .insert_custom_modality_ignore_conflict(custom_template_id, &modality)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    async fn add_custom_template_image(
        &self,
        custom_template_id: i32,
        user_id: i32,
        new_image: NewUserCustomTemplateImage,
    ) -> Result<UserCustomTemplateImage, ServiceError> {
        // 커스텀 템플릿 소유권 확인
        let template = self.get_custom_template(custom_template_id, user_id).await?;
        if template.is_none() {
            return Err(ServiceError::NotFound("Custom template not found".into()));
        }

        // 이미지 업로더가 본인인지 확인
        if new_image.uploaded_by != user_id {
            return Err(ServiceError::Unauthorized(
                "You can only add images you uploaded".into(),
            ));
        }

        self.template_repository
            .as_ref()
            .add_custom_template_image(&new_image)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_custom_template_images(
        &self,
        custom_template_id: i32,
        user_id: i32,
    ) -> Result<Vec<UserCustomTemplateImage>, ServiceError> {
        // 커스텀 템플릿 소유권 확인
        let template = self.get_custom_template(custom_template_id, user_id).await?;
        if template.is_none() {
            return Err(ServiceError::NotFound("Custom template not found".into()));
        }

        self.template_repository
            .as_ref()
            .find_custom_template_images(custom_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_custom_template_image(
        &self,
        image_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // 이미지 조회 및 소유권 확인
        let images = self
            .template_repository
            .as_ref()
            .find_custom_template_images(0) // TODO: image_id로 custom_template_id 찾기 필요
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 간단한 구현: 모든 커스텀 템플릿 이미지에서 찾기
        // TODO: 더 효율적인 방법으로 개선 필요
        let image = images
            .into_iter()
            .find(|img| img.id == image_id)
            .ok_or_else(|| ServiceError::NotFound("Image not found".into()))?;

        // 업로드한 사용자만 삭제 가능
        if image.uploaded_by != user_id {
            return Err(ServiceError::Unauthorized(
                "Only the image uploader can delete the image".into(),
            ));
        }

        let deleted = self
            .template_repository
            .as_ref()
            .delete_custom_template_image(image_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Image not found".into()));
        }

        Ok(())
    }

    async fn apply_template_to_report(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        // 템플릿 조회 및 권한 확인
        if let Some(tid) = template_id {
            // 원본 템플릿 조회
            let _template = self
                .get_template(tid)
                .await?
                .ok_or_else(|| ServiceError::NotFound("Template not found".into()))?;
        } else if let Some(ctid) = custom_template_id {
            // 커스텀 템플릿 소유권 확인
            let _template = self.get_custom_template(ctid, user_id).await?;
            if _template.is_none() {
                return Err(ServiceError::NotFound("Custom template not found".into()));
            }
        } else {
            return Err(ServiceError::ValidationError(
                "Either template_id or custom_template_id must be provided".into(),
            ));
        }

        self.set_report_template(report_id, template_id, custom_template_id, user_id)
            .await
    }

    async fn get_report_guide_images(
        &self,
        report_id: i32,
    ) -> Result<Vec<(GuideImage, i32)>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_guide_images_by_report(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn set_report_template(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        if template_id.is_some() && custom_template_id.is_some() {
            return Err(ServiceError::ValidationError(
                "Either template_id or custom_template_id must be provided, not both".into(),
            ));
        }
        if template_id.is_none() && custom_template_id.is_none() {
            return Err(ServiceError::ValidationError(
                "Either template_id or custom_template_id must be provided".into(),
            ));
        }

        // 1. Report에 template 설정
        self.report_repository
            .as_ref()
            .update_report_template(report_id, template_id, custom_template_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 2. 기존 report_image 삭제
        self.template_repository
            .as_ref()
            .delete_report_images_by_report(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 3. 템플릿 이미지를 report_image에 복사
        let image_entries: Vec<(i32, i32)> = if let Some(tid) = template_id {
            let mappings = self
                .template_repository
                .as_ref()
                .find_template_image_mappings(tid)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            mappings
                .into_iter()
                .map(|m| (m.image_id, m.display_order))
                .collect()
        } else if let Some(ctid) = custom_template_id {
            let mappings = self
                .template_repository
                .as_ref()
                .find_custom_template_image_mappings(ctid)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            mappings
                .into_iter()
                .map(|m| (m.image_id, m.display_order))
                .collect()
        } else {
            vec![]
        };

        if !image_entries.is_empty() {
            self.template_repository
                .as_ref()
                .insert_report_images(report_id, &image_entries)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn clear_report_template(&self, report_id: i32) -> Result<(), ServiceError> {
        self.report_repository
            .as_ref()
            .update_report_template(report_id, None, None)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        self.template_repository
            .as_ref()
            .delete_report_images_by_report(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
