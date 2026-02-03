use crate::application::services::SignedUrlService;
use crate::application::template::dto::report_guide_template_dto::*;
use crate::application::template::mappers::template_response_mapper as mapper;
use crate::domain::reporting::repositories::SeriesUserReportRepository;
use crate::domain::template::entities::report_guide_template::*;
use crate::domain::template::services::ReportGuideTemplateService;
use crate::domain::ServiceError;
use std::collections::HashMap;
use std::sync::Arc;

const IMAGE_SIGNED_URL_TTL: u64 = 3600;

pub struct ReportGuideTemplateUseCase<T, R, SUS>
where
    T: ReportGuideTemplateService,
    R: SeriesUserReportRepository,
    SUS: SignedUrlService,
{
    template_service: Arc<T>,
    report_repository: Arc<R>,
    signed_url_service: Arc<SUS>,
}

impl<T, R, SUS> ReportGuideTemplateUseCase<T, R, SUS>
where
    T: ReportGuideTemplateService,
    R: SeriesUserReportRepository,
    SUS: SignedUrlService,
{
    pub fn new(
        template_service: Arc<T>,
        report_repository: Arc<R>,
        signed_url_service: Arc<SUS>,
    ) -> Self {
        Self {
            template_service,
            report_repository,
            signed_url_service,
        }
    }

    /// 이미지 목록의 image_url을 signed URL로 치환.
    /// Object Storage 미설정 등으로 실패 시 경고 로그만 남기고 기존 image_url 유지 (API는 성공 반환).
    async fn populate_signed_urls_for_images(
        &self,
        images: &mut [TemplateImageResponse],
    ) {
        if images.is_empty() {
            return;
        }
        let paths: Vec<String> = images.iter().map(|i| i.image_path.clone()).collect();
        match self
            .signed_url_service
            .generate_download_urls_bulk(paths, Some(IMAGE_SIGNED_URL_TTL))
            .await
        {
            Ok(urls) => {
                for (img, (_, url_opt)) in images.iter_mut().zip(urls.into_iter()) {
                    if let Some(url) = url_opt {
                        img.image_url = url;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Signed URL 생성 실패, 기존 image_url 유지: {:?}",
                    e
                );
            }
        }
    }

    async fn populate_signed_urls_for_guide_images(
        &self,
        images: &mut [GuideImageResponse],
    ) {
        if images.is_empty() {
            return;
        }
        let paths: Vec<String> = images.iter().map(|i| i.image_path.clone()).collect();
        match self
            .signed_url_service
            .generate_download_urls_bulk(paths, Some(IMAGE_SIGNED_URL_TTL))
            .await
        {
            Ok(urls) => {
                for (img, (_, url_opt)) in images.iter_mut().zip(urls.into_iter()) {
                    if let Some(url) = url_opt {
                        img.image_url = url;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Signed URL 생성 실패, 기존 image_url 유지: {:?}",
                    e
                );
            }
        }
    }

    async fn populate_signed_urls_for_custom_images(
        &self,
        images: &mut [CustomTemplateImageResponse],
    ) {
        if images.is_empty() {
            return;
        }
        let paths: Vec<String> = images.iter().map(|i| i.image_path.clone()).collect();
        match self
            .signed_url_service
            .generate_download_urls_bulk(paths, Some(IMAGE_SIGNED_URL_TTL))
            .await
        {
            Ok(urls) => {
                for (img, (_, url_opt)) in images.iter_mut().zip(urls.into_iter()) {
                    if let Some(url) = url_opt {
                        img.image_url = url;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Signed URL 생성 실패, 기존 image_url 유지: {:?}",
                    e
                );
            }
        }
    }

    /// Report가 요청한 user_id 소유인지 검증. 아님 Err(ServiceError::Unauthorized/NotFound)
    async fn verify_report_ownership(&self, report_id: i32, user_id: i32) -> Result<(), ServiceError> {
        let report = self
            .report_repository
            .find_by_id(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        match report {
            Some(r) if r.user_id == user_id => Ok(()),
            Some(_) => Err(ServiceError::Unauthorized(
                "Report belongs to another user".into(),
            )),
            None => Err(ServiceError::NotFound("Report not found".into())),
        }
    }

    // ========== 원본 템플릿 ==========

    pub async fn create_template(
        &self,
        request: CreateReportGuideTemplateRequest,
        created_by: i32,
    ) -> Result<ReportGuideTemplateResponse, ServiceError> {
        let new_template = NewReportGuideTemplate {
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
            is_shared: request.is_shared.unwrap_or(true),
            created_by,
        };

        let modalities = request.modalities.clone();
        let template = self
            .template_service
            .as_ref()
            .create_template(new_template, request.modalities)
            .await?;

        // 이미지 매핑 생성 (image_ids가 있는 경우)
        if !request.image_ids.is_empty() {
            self.template_service
                .as_ref()
                .update_template_image_mappings(template.id, request.image_ids, created_by)
                .await?;
        }

        // 가이드 이미지 조회 (새로운 독립적인 이미지 구조 사용)
        let images = self
            .template_service
            .as_ref()
            .get_template_guide_images(template.id, Some(created_by))
            .await?;

        let mut response = mapper::to_template_response_with_guide_images(template, images);
        response.modalities = modalities;
        self.populate_signed_urls_for_images(&mut response.images).await;
        Ok(response)
    }

    pub async fn get_template(
        &self,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<Option<ReportGuideTemplateResponse>, ServiceError> {
        let template = self.template_service.as_ref().get_template(id).await?;

        if let Some(t) = template {
            let images = self
                .fetch_merged_template_images(id, user_id)
                .await?;
            let mut response = mapper::to_template_response_with_merged_images(t.clone(), images);
            response.modalities = self
                .template_service
                .as_ref()
                .get_modalities_by_template(id)
                .await?;
            self.populate_signed_urls_for_images(&mut response.images).await;
            Ok(Some(response))
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
                .fetch_merged_template_images(template.id, user_id)
                .await?;
            let mut resp = mapper::to_template_response_with_merged_images(template.clone(), images);
            resp.modalities = self
                .template_service
                .as_ref()
                .get_modalities_by_template(template.id)
                .await?;
            self.populate_signed_urls_for_images(&mut resp.images).await;
            responses.push(resp);
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
        user_id: i32,
    ) -> Result<ReportGuideTemplateResponse, ServiceError> {
        let update = UpdateReportGuideTemplate {
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
            is_shared: request.is_shared,
            is_active: request.is_active,
        };

        let template = self.template_service.as_ref().update_template(id, update).await?;

        // 모달리티 업데이트 (modalities가 지정된 경우)
        if let Some(modalities) = request.modalities {
            self.template_service
                .as_ref()
                .update_template_modalities(id, modalities)
                .await?;
        }

        // 이미지 매핑 업데이트 (image_ids가 지정된 경우)
        if let Some(image_ids) = request.image_ids {
            self.template_service
                .as_ref()
                .update_template_image_mappings(id, image_ids, user_id)
                .await?;
        }

        let images = self
            .template_service
            .as_ref()
            .get_template_guide_images(id, Some(user_id))
            .await?;

        let modalities = self
            .template_service
            .as_ref()
            .get_modalities_by_template(id)
            .await?;

        let mut response = mapper::to_template_response_with_guide_images(template, images);
        response.modalities = modalities;
        self.populate_signed_urls_for_images(&mut response.images).await;
        Ok(response)
    }

    pub async fn delete_template(&self, id: i32) -> Result<(), ServiceError> {
        self.template_service.as_ref().delete_template(id).await
    }

    // ========== 독립적인 가이드 이미지 ==========

    pub async fn upload_guide_image(
        &self,
        request: GuideImageUploadCompleteRequest,
        uploaded_by: i32,
    ) -> Result<GuideImageUploadCompleteResponse, ServiceError> {
        let image_path = request.file_path.clone();
        let new_image = NewGuideImage {
            image_path: image_path.clone(),
            image_url: image_path.clone(),
            file_size: Some(request.file_size),
            mime_type: request.mime_type,
            is_shared: request.is_shared.unwrap_or(true),
            uploaded_by,
        };

        let image = self
            .template_service
            .as_ref()
            .create_guide_image(new_image)
            .await?;

        let mut img_response = mapper::to_guide_image_response(image);
        self.populate_signed_urls_for_guide_images(std::slice::from_mut(&mut img_response))
            .await;
        Ok(GuideImageUploadCompleteResponse {
            success: true,
            message: "Image uploaded successfully".to_string(),
            image: img_response,
        })
    }

    pub async fn get_user_guide_images(
        &self,
        user_id: i32,
        is_shared: Option<bool>,
    ) -> Result<GuideImageListResponse, ServiceError> {
        let images = self
            .template_service
            .as_ref()
            .get_user_guide_images(user_id, is_shared)
            .await?;

        let total_count = images.len() as i64;
        let mut image_responses: Vec<GuideImageResponse> = images
            .into_iter()
            .map(|img| mapper::to_guide_image_response(img))
            .collect();
        self.populate_signed_urls_for_guide_images(&mut image_responses)
            .await;

        Ok(GuideImageListResponse {
            success: true,
            images: image_responses,
            total_count,
        })
    }

    pub async fn update_guide_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
        user_id: i32,
    ) -> Result<GuideImageResponse, ServiceError> {
        let image = self
            .template_service
            .as_ref()
            .update_guide_image_share_status(image_id, is_shared, user_id)
            .await?;

        let mut resp = mapper::to_guide_image_response(image);
        self.populate_signed_urls_for_guide_images(std::slice::from_mut(&mut resp))
            .await;
        Ok(resp)
    }

    pub async fn delete_guide_image(
        &self,
        image_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.template_service
            .as_ref()
            .delete_guide_image(image_id, user_id)
            .await
    }

    // ========== 템플릿 이미지 (기존 구조 - 하위 호환성) ==========

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

        let mut resp = mapper::to_image_response(image);
        self.populate_signed_urls_for_images(std::slice::from_mut(&mut resp))
            .await;
        Ok(resp)
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

        let mut resp = mapper::to_image_response(image);
        self.populate_signed_urls_for_images(std::slice::from_mut(&mut resp))
            .await;
        Ok(resp)
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
            .create_custom_template_from_base(user_id, request.base_template_id)
            .await?;

        let images = self
            .template_service
            .as_ref()
            .get_custom_template_guide_images(template.id, Some(user_id))
            .await?;

        let mut response = mapper::to_custom_template_response_with_guide_images(template, images);
        self.populate_signed_urls_for_custom_images(&mut response.images)
            .await;
        Ok(response)
    }

    pub async fn create_custom_template(
        &self,
        user_id: i32,
        request: CreateCustomTemplateRequest,
    ) -> Result<UserCustomReportTemplateResponse, ServiceError> {
        let new_template = NewUserCustomReportTemplate {
            user_id,
            base_template_id: None,
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
        };

        let template = self
            .template_service
            .as_ref()
            .create_custom_template(new_template, request.modalities)
            .await?;

        // 이미지 매핑 생성 (image_ids가 있는 경우)
        if !request.image_ids.is_empty() {
            self.template_service
                .as_ref()
                .update_custom_template_image_mappings(template.id, request.image_ids, user_id)
                .await?;
        }

        let images = self
            .template_service
            .as_ref()
            .get_custom_template_guide_images(template.id, Some(user_id))
            .await?;

        let mut response = mapper::to_custom_template_response_with_guide_images(template, images);
        self.populate_signed_urls_for_custom_images(&mut response.images)
            .await;
        Ok(response)
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
            let template_id = t.id;
            let images = self
                .template_service
                .as_ref()
                .get_custom_template_guide_images(template_id, Some(user_id))
                .await?;

            let mut response = mapper::to_custom_template_response_with_guide_images(t, images);
            response.modalities = self
                .template_service
                .as_ref()
                .get_custom_modalities_by_template(template_id)
                .await?;
            self.populate_signed_urls_for_custom_images(&mut response.images)
                .await;
            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    /// 사용자 기준 유효 템플릿 목록: 원본 중 커스텀이 있으면 커스텀, 없으면 원본 + 처음부터 만든 커스텀
    pub async fn get_effective_report_templates(
        &self,
        user_id: i32,
        modality: Option<String>,
        bodypart: Option<String>,
    ) -> Result<EffectiveReportTemplateListResponse, ServiceError> {
        let originals = self
            .template_service
            .as_ref()
            .get_templates(modality.clone(), bodypart.clone(), Some(true))
            .await?;

        let customs = self
            .template_service
            .as_ref()
            .get_custom_templates_by_user(user_id)
            .await?;

        // base_template_id -> custom (원본 기반 커스텀)
        let custom_by_base: HashMap<i32, _> = customs
            .iter()
            .filter_map(|c| c.base_template_id.map(|bid| (bid, c)))
            .collect();

        let from_scratch: Vec<_> = customs
            .iter()
            .filter(|c| c.base_template_id.is_none())
            .collect();

        let mut result = Vec::new();

        for original in originals {
            if let Some(custom) = custom_by_base.get(&original.id) {
                let images = self
                    .template_service
                    .as_ref()
                    .get_custom_template_guide_images(custom.id, Some(user_id))
                    .await?;
                let mut eff = mapper::to_effective_from_custom(custom, images);
                eff.modalities = self
                    .template_service
                    .as_ref()
                    .get_custom_modalities_by_template(custom.id)
                    .await?;
                self.populate_signed_urls_for_images(&mut eff.images).await;
                result.push(eff);
            } else {
                let images = self
                    .fetch_merged_template_images(original.id, Some(user_id))
                    .await?;
                let mut eff = mapper::to_effective_from_original(&original, images);
                eff.modalities = self
                    .template_service
                    .as_ref()
                    .get_modalities_by_template(original.id)
                    .await?;
                self.populate_signed_urls_for_images(&mut eff.images).await;
                result.push(eff);
            }
        }

        for custom in from_scratch {
            let images = self
                .template_service
                .as_ref()
                .get_custom_template_guide_images(custom.id, Some(user_id))
                .await?;
            let mut eff = mapper::to_effective_from_custom(custom, images);
            eff.modalities = self
                .template_service
                .as_ref()
                .get_custom_modalities_by_template(custom.id)
                .await?;
            self.populate_signed_urls_for_images(&mut eff.images).await;
            result.push(eff);
        }

        Ok(EffectiveReportTemplateListResponse {
            success: true,
            templates: result,
        })
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
            let template_id = template.id;
            let images = self
                .template_service
                .as_ref()
                .get_custom_template_guide_images(template_id, Some(user_id))
                .await?;
            let mut resp = mapper::to_custom_template_response_with_guide_images(template, images);
            resp.modalities = self
                .template_service
                .as_ref()
                .get_custom_modalities_by_template(template_id)
                .await?;
            self.populate_signed_urls_for_custom_images(&mut resp.images)
                .await;
            responses.push(resp);
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

        // 모달리티 업데이트 (modalities가 지정된 경우)
        if let Some(modalities) = request.modalities {
            self.template_service
                .as_ref()
                .update_custom_template_modalities(template.id, modalities)
                .await?;
        }

        // 이미지 매핑 업데이트 (image_ids가 지정된 경우)
        if let Some(image_ids) = request.image_ids {
            self.template_service
                .as_ref()
                .update_custom_template_image_mappings(template.id, image_ids, user_id)
                .await?;
        }

        let images = self
            .template_service
            .as_ref()
            .get_custom_template_guide_images(template.id, Some(user_id))
            .await?;

        let mut response = mapper::to_custom_template_response_with_guide_images(template, images);
        response.modalities = self
            .template_service
            .as_ref()
            .get_custom_modalities_by_template(response.id)
            .await?;
        self.populate_signed_urls_for_custom_images(&mut response.images)
            .await;
        Ok(response)
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

        let mut resp = mapper::to_custom_image_response(image);
        self.populate_signed_urls_for_custom_images(std::slice::from_mut(&mut resp))
            .await;
        Ok(resp)
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
        self.verify_report_ownership(report_id, user_id).await?;
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

    // ========== Report 템플릿 + 이미지 (1:1) ==========

    pub async fn get_report_guides(
        &self,
        report_id: i32,
        user_id: i32,
    ) -> Result<Vec<ReportGuideResponse>, ServiceError> {
        self.verify_report_ownership(report_id, user_id).await?;

        let report = self
            .report_repository
            .find_by_id(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Report not found".into()))?;

        let (template_id, custom_template_id) = (report.template_id, report.custom_template_id);
        if template_id.is_none() && custom_template_id.is_none() {
            return Ok(vec![]);
        }

        let images_with_order = self
            .template_service
            .as_ref()
            .get_report_guide_images(report_id)
            .await?;

        let mut images: Vec<TemplateImageResponse> = images_with_order
            .into_iter()
            .map(|(img, order)| mapper::guide_image_to_template_image_response(img, order))
            .collect();
        self.populate_signed_urls_for_images(&mut images).await;

        Ok(vec![ReportGuideResponse {
            id: report_id,
            report_id,
            template_id,
            custom_template_id,
            display_order: 0,
            images: Some(images),
            created_at: report.updated_at,
        }])
    }

    pub async fn add_report_guide(
        &self,
        report_id: i32,
        user_id: i32,
        request: AddReportGuideRequest,
    ) -> Result<ReportGuideResponse, ServiceError> {
        self.verify_report_ownership(report_id, user_id).await?;

        self.template_service
            .as_ref()
            .set_report_template(
                report_id,
                request.template_id,
                request.custom_template_id,
                user_id,
            )
            .await?;

        let report = self
            .report_repository
            .find_by_id(report_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Report not found".into()))?;

        let images_with_order = self
            .template_service
            .as_ref()
            .get_report_guide_images(report_id)
            .await?;
        let mut images: Vec<TemplateImageResponse> = images_with_order
            .into_iter()
            .map(|(img, order)| mapper::guide_image_to_template_image_response(img, order))
            .collect();
        self.populate_signed_urls_for_images(&mut images).await;

        Ok(ReportGuideResponse {
            id: report_id,
            report_id,
            template_id: report.template_id,
            custom_template_id: report.custom_template_id,
            display_order: 0,
            images: Some(images),
            created_at: report.updated_at,
        })
    }

    pub async fn delete_report_guide(
        &self,
        report_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.verify_report_ownership(report_id, user_id).await?;
        self.template_service
            .as_ref()
            .clear_report_template(report_id)
            .await
    }

    // ========== Helper Methods ==========

    /// 하위 호환: 기존 구조(deprecated) + 신규 구조 이미지 병합
    async fn fetch_merged_template_images(
        &self,
        template_id: i32,
        user_id: Option<i32>,
    ) -> Result<Vec<TemplateImageResponse>, ServiceError> {
        let guide_images = self
            .template_service
            .as_ref()
            .get_template_guide_images(template_id, user_id)
            .await?;
        let old_images = self
            .template_service
            .as_ref()
            .get_template_images(template_id, user_id)
            .await?;
        Ok(mapper::merge_template_images(old_images, guide_images))
    }
}

