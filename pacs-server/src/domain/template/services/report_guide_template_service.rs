//! # Report Guide Template Service 트레이트
//!
//! 이 모듈은 리포트 가이드 템플릿 비즈니스 로직을 위한 Service 트레이트를 정의합니다.

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

    // ========== 템플릿 이미지 ==========

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
        name: String,
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

    // ========== Report Guide Image 관리 ==========

    /// Report의 Guide Image 목록 조회
    async fn get_report_guides(
        &self,
        report_id: i32,
    ) -> Result<Vec<SeriesUserReportGuide>, ServiceError>;

    /// Report에 Guide Image 추가 (템플릿 또는 커스텀 템플릿에서)
    async fn add_report_guide(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
        display_order: i32,
    ) -> Result<SeriesUserReportGuide, ServiceError>;

    /// Report에서 Guide Image 삭제
    async fn delete_report_guide(
        &self,
        guide_id: i32,
    ) -> Result<(), ServiceError>;
}

/// Report Guide Template Service 구현체
#[derive(Clone)]
pub struct ReportGuideTemplateServiceImpl<T>
where
    T: ReportGuideTemplateRepository + 'static,
{
    template_repository: Arc<T>,
}

impl<T> ReportGuideTemplateServiceImpl<T>
where
    T: ReportGuideTemplateRepository + 'static,
{
    pub fn new(template_repository: T) -> Self {
        Self {
            template_repository: Arc::new(template_repository),
        }
    }
}

#[async_trait]
impl<T> ReportGuideTemplateService for ReportGuideTemplateServiceImpl<T>
where
    T: ReportGuideTemplateRepository + 'static,
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
        name: String,
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
            name,
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

        // 원본 템플릿의 공유 이미지만 복사 (사용자 전용 이미지는 복사하지 않음)
        let base_images = self.get_template_images(base_template_id, Some(user_id)).await?;
        for (idx, image) in base_images.iter().enumerate() {
            let new_custom_image = NewUserCustomTemplateImage {
                custom_template_id: custom_template.id,
                image_path: image.image_path.clone(),
                image_url: image.image_url.clone(),
                file_size: image.file_size,
                mime_type: image.mime_type.clone(),
                display_order: idx as i32,
                is_shared: false, // 커스텀 템플릿 이미지는 기본적으로 사용자 전용
                uploaded_by: user_id,
            };

            self.template_repository
                .as_ref()
                .add_custom_template_image(&new_custom_image)
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

        // Report-가이드 매핑 추가
        let new_guide = NewSeriesUserReportGuide {
            report_id,
            template_id,
            custom_template_id,
            display_order: 0,
        };

        self.template_repository
            .as_ref()
            .add_report_guide(&new_guide)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_report_guides(
        &self,
        report_id: i32,
    ) -> Result<Vec<SeriesUserReportGuide>, ServiceError> {
        self.template_repository
            .as_ref()
            .find_report_guides(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn add_report_guide(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
        display_order: i32,
    ) -> Result<SeriesUserReportGuide, ServiceError> {
        // template_id와 custom_template_id 중 하나만 있어야 함
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

        let new_guide = NewSeriesUserReportGuide {
            report_id,
            template_id,
            custom_template_id,
            display_order,
        };

        self.template_repository
            .as_ref()
            .add_report_guide(&new_guide)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_report_guide(
        &self,
        guide_id: i32,
    ) -> Result<(), ServiceError> {
        let deleted = self
            .template_repository
            .as_ref()
            .delete_report_guide(guide_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Guide not found".into()));
        }

        Ok(())
    }
}
