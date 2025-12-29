use crate::application::template::dto::report_guide_template_dto::*;
use crate::domain::template::entities::report_guide_template::*;
use crate::domain::template::services::ReportGuideTemplateService;
use crate::domain::ServiceError;
use std::sync::Arc;

pub struct ReportGuideTemplateUseCase<T>
where
    T: ReportGuideTemplateService,
{
    template_service: Arc<T>,
}

impl<T> ReportGuideTemplateUseCase<T>
where
    T: ReportGuideTemplateService,
{
    pub fn new(template_service: Arc<T>) -> Self {
        Self { template_service }
    }

    // ========== 원본 템플릿 ==========

    pub async fn create_template(
        &self,
        request: CreateReportGuideTemplateRequest,
        created_by: i32,
    ) -> Result<ReportGuideTemplateResponse, ServiceError> {
        let new_template = NewReportGuideTemplate {
            name: request.name,
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
            is_shared: request.is_shared.unwrap_or(true),
            created_by,
        };

        let template = self
            .template_service
            .as_ref()
            .create_template(new_template, request.modalities)
            .await?;

        // 모달리티 및 이미지 조회
        let modalities = self
            .template_service
            .as_ref()
            .get_template_images(template.id, Some(created_by))
            .await?;

        Ok(self.to_template_response(template, modalities))
    }

    pub async fn get_template(
        &self,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<Option<ReportGuideTemplateResponse>, ServiceError> {
        let template = self.template_service.as_ref().get_template(id).await?;

        if let Some(t) = template {
            let modalities = self
                .template_service
                .as_ref()
                .get_template_images(id, user_id)
                .await?;

            Ok(Some(self.to_template_response(t, modalities)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_templates(
        &self,
        modality: Option<String>,
        bodypart: Option<String>,
        is_active: Option<bool>,
        user_id: Option<i32>,
    ) -> Result<ReportGuideTemplateListResponse, ServiceError> {
        let templates = self
            .template_service
            .as_ref()
            .get_templates(modality, bodypart, is_active)
            .await?;

        let mut responses = Vec::new();
        for template in templates {
            let images = self
                .template_service
                .as_ref()
                .get_template_images(template.id, user_id)
                .await?;
            responses.push(self.to_template_response(template, images));
        }

        Ok(ReportGuideTemplateListResponse {
            success: true,
            templates: responses,
        })
    }

    pub async fn update_template(
        &self,
        id: i32,
        request: UpdateReportGuideTemplateRequest,
    ) -> Result<ReportGuideTemplateResponse, ServiceError> {
        let update = UpdateReportGuideTemplate {
            name: request.name,
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
            is_shared: request.is_shared,
            is_active: request.is_active,
        };

        let template = self.template_service.as_ref().update_template(id, update).await?;

        let images = self
            .template_service
            .as_ref()
            .get_template_images(id, None)
            .await?;

        Ok(self.to_template_response(template, images))
    }

    pub async fn delete_template(&self, id: i32) -> Result<(), ServiceError> {
        self.template_service.as_ref().delete_template(id).await
    }

    pub async fn add_template_image(
        &self,
        template_id: i32,
        request: AddTemplateImageRequest,
        uploaded_by: i32,
    ) -> Result<TemplateImageResponse, ServiceError> {
        let new_image = NewReportGuideTemplateImage {
            template_id,
            image_path: request.image_path,
            image_url: request.image_url,
            file_size: request.file_size,
            mime_type: request.mime_type,
            display_order: request.display_order.unwrap_or(0),
            is_shared: request.is_shared.unwrap_or(true),
            uploaded_by,
        };

        let image = self
            .template_service
            .as_ref()
            .add_template_image(template_id, new_image)
            .await?;

        Ok(self.to_image_response(image))
    }

    pub async fn update_image_share_status(
        &self,
        image_id: i32,
        request: UpdateImageShareStatusRequest,
        user_id: i32,
    ) -> Result<TemplateImageResponse, ServiceError> {
        let image = self
            .template_service
            .as_ref()
            .update_image_share_status(image_id, request.is_shared, user_id)
            .await?;

        Ok(self.to_image_response(image))
    }

    pub async fn delete_template_image(
        &self,
        image_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.template_service
            .as_ref()
            .delete_template_image(image_id, user_id)
            .await
    }

    // ========== 사용자 커스텀 템플릿 ==========

    pub async fn create_custom_template_from_base(
        &self,
        user_id: i32,
        request: CreateCustomTemplateFromBaseRequest,
    ) -> Result<UserCustomReportTemplateResponse, ServiceError> {
        let template = self
            .template_service
            .as_ref()
            .create_custom_template_from_base(user_id, request.base_template_id, request.name)
            .await?;

        let modalities = self
            .template_service
            .as_ref()
            .get_custom_template_images(template.id, user_id)
            .await?;

        Ok(self.to_custom_template_response(template, modalities))
    }

    pub async fn create_custom_template(
        &self,
        user_id: i32,
        request: CreateCustomTemplateRequest,
    ) -> Result<UserCustomReportTemplateResponse, ServiceError> {
        let new_template = NewUserCustomReportTemplate {
            user_id,
            base_template_id: None,
            name: request.name,
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
        };

        let template = self
            .template_service
            .as_ref()
            .create_custom_template(new_template, request.modalities)
            .await?;

        let images = self
            .template_service
            .as_ref()
            .get_custom_template_images(template.id, user_id)
            .await?;

        Ok(self.to_custom_template_response(template, images))
    }

    pub async fn get_custom_template(
        &self,
        id: i32,
        user_id: i32,
    ) -> Result<Option<UserCustomReportTemplateResponse>, ServiceError> {
        let template = self
            .template_service
            .as_ref()
            .get_custom_template(id, user_id)
            .await?;

        if let Some(t) = template {
            let images = self
                .template_service
                .as_ref()
                .get_custom_template_images(t.id, user_id)
                .await?;

            Ok(Some(self.to_custom_template_response(t, images)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_custom_templates_by_user(
        &self,
        user_id: i32,
    ) -> Result<UserCustomTemplateListResponse, ServiceError> {
        let templates = self
            .template_service
            .as_ref()
            .get_custom_templates_by_user(user_id)
            .await?;

        let mut responses = Vec::new();
        for template in templates {
            let images = self
                .template_service
                .as_ref()
                .get_custom_template_images(template.id, user_id)
                .await?;
            responses.push(self.to_custom_template_response(template, images));
        }

        Ok(UserCustomTemplateListResponse {
            success: true,
            templates: responses,
        })
    }

    pub async fn update_custom_template(
        &self,
        id: i32,
        user_id: i32,
        request: UpdateCustomTemplateRequest,
    ) -> Result<UserCustomReportTemplateResponse, ServiceError> {
        let update = UpdateUserCustomReportTemplate {
            name: request.name,
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
            is_active: request.is_active,
        };

        let template = self
            .template_service
            .as_ref()
            .update_custom_template(id, user_id, update)
            .await?;

        let images = self
            .template_service
            .as_ref()
            .get_custom_template_images(template.id, user_id)
            .await?;

        Ok(self.to_custom_template_response(template, images))
    }

    pub async fn delete_custom_template(
        &self,
        id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.template_service
            .as_ref()
            .delete_custom_template(id, user_id)
            .await
    }

    pub async fn add_custom_template_image(
        &self,
        custom_template_id: i32,
        user_id: i32,
        request: AddCustomTemplateImageRequest,
    ) -> Result<CustomTemplateImageResponse, ServiceError> {
        let new_image = NewUserCustomTemplateImage {
            custom_template_id,
            image_path: request.image_path,
            image_url: request.image_url,
            file_size: request.file_size,
            mime_type: request.mime_type,
            display_order: request.display_order.unwrap_or(0),
            is_shared: false, // 커스텀 템플릿 이미지는 기본적으로 사용자 전용
            uploaded_by: user_id,
        };

        let image = self
            .template_service
            .as_ref()
            .add_custom_template_image(custom_template_id, user_id, new_image)
            .await?;

        Ok(self.to_custom_image_response(image))
    }

    pub async fn delete_custom_template_image(
        &self,
        image_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.template_service
            .as_ref()
            .delete_custom_template_image(image_id, user_id)
            .await
    }

    // ========== Report-템플릿 적용 ==========

    pub async fn apply_template_to_report(
        &self,
        report_id: i32,
        request: ApplyTemplateToReportRequest,
        user_id: i32,
    ) -> Result<ApplyTemplateToReportResponse, ServiceError> {
        self.template_service
            .as_ref()
            .apply_template_to_report(
                report_id,
                request.template_id,
                request.custom_template_id,
                user_id,
            )
            .await?;

        Ok(ApplyTemplateToReportResponse {
            success: true,
            message: "Template applied successfully".to_string(),
        })
    }

    // ========== Report Guide Image 관리 ==========

    pub async fn get_report_guides(
        &self,
        report_id: i32,
    ) -> Result<Vec<SeriesUserReportGuide>, ServiceError> {
        self.template_service
            .as_ref()
            .get_report_guides(report_id)
            .await
    }

    pub async fn add_report_guide(
        &self,
        report_id: i32,
        request: AddReportGuideRequest,
    ) -> Result<ReportGuideResponse, ServiceError> {
        let guide = self
            .template_service
            .as_ref()
            .add_report_guide(
                report_id,
                request.template_id,
                request.custom_template_id,
                request.display_order.unwrap_or(0),
            )
            .await?;

        Ok(ReportGuideResponse {
            id: guide.id,
            report_id: guide.report_id,
            template_id: guide.template_id,
            custom_template_id: guide.custom_template_id,
            display_order: guide.display_order,
            created_at: guide.created_at,
        })
    }

    pub async fn delete_report_guide(
        &self,
        guide_id: i32,
    ) -> Result<(), ServiceError> {
        self.template_service
            .as_ref()
            .delete_report_guide(guide_id)
            .await
    }

    // ========== Helper Methods ==========

    fn to_template_response(
        &self,
        template: ReportGuideTemplate,
        images: Vec<ReportGuideTemplateImage>,
    ) -> ReportGuideTemplateResponse {
        // TODO: 모달리티 조회 추가 필요
        ReportGuideTemplateResponse {
            id: template.id,
            name: template.name,
            description: template.description,
            conclusion: template.conclusion,
            bodypart: template.bodypart,
            is_shared: template.is_shared,
            is_active: template.is_active,
            created_by: template.created_by,
            modalities: vec![], // TODO: 모달리티 조회 추가
            images: images.into_iter().map(|img| self.to_image_response(img)).collect(),
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }

    fn to_custom_template_response(
        &self,
        template: UserCustomReportTemplate,
        images: Vec<UserCustomTemplateImage>,
    ) -> UserCustomReportTemplateResponse {
        // TODO: 모달리티 조회 추가 필요
        UserCustomReportTemplateResponse {
            id: template.id,
            user_id: template.user_id,
            base_template_id: template.base_template_id,
            name: template.name,
            description: template.description,
            conclusion: template.conclusion,
            bodypart: template.bodypart,
            is_active: template.is_active,
            modalities: vec![], // TODO: 모달리티 조회 추가
            images: images
                .into_iter()
                .map(|img| self.to_custom_image_response(img))
                .collect(),
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }

    fn to_image_response(&self, image: ReportGuideTemplateImage) -> TemplateImageResponse {
        TemplateImageResponse {
            id: image.id,
            image_path: image.image_path,
            image_url: image.image_url,
            file_size: image.file_size,
            mime_type: image.mime_type,
            display_order: image.display_order,
            is_shared: image.is_shared,
            uploaded_by: image.uploaded_by,
            created_at: image.created_at,
        }
    }

    fn to_custom_image_response(
        &self,
        image: UserCustomTemplateImage,
    ) -> CustomTemplateImageResponse {
        CustomTemplateImageResponse {
            id: image.id,
            image_path: image.image_path,
            image_url: image.image_url,
            file_size: image.file_size,
            mime_type: image.mime_type,
            display_order: image.display_order,
            is_shared: image.is_shared,
            uploaded_by: image.uploaded_by,
            created_at: image.created_at,
        }
    }
}

