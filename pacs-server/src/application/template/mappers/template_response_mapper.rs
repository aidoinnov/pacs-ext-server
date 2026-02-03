//! Report Guide Template 엔티티 → DTO 변환

use crate::application::template::dto::report_guide_template_dto::*;
use crate::domain::template::entities::report_guide_template::*;

// ========== 원본 템플릿 ==========

pub fn to_template_response(
    template: ReportGuideTemplate,
    images: Vec<ReportGuideTemplateImage>,
) -> ReportGuideTemplateResponse {
    ReportGuideTemplateResponse {
        id: template.id,
        description: template.description,
        conclusion: template.conclusion,
        bodypart: template.bodypart,
        is_shared: template.is_shared,
        is_active: template.is_active,
        created_by: template.created_by,
        modalities: vec![],
        images: images
            .into_iter()
            .map(|img| to_image_response(img))
            .collect(),
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

pub fn to_template_response_with_guide_images(
    template: ReportGuideTemplate,
    images: Vec<GuideImage>,
) -> ReportGuideTemplateResponse {
    ReportGuideTemplateResponse {
        id: template.id,
        description: template.description,
        conclusion: template.conclusion,
        bodypart: template.bodypart,
        is_shared: template.is_shared,
        is_active: template.is_active,
        created_by: template.created_by,
        modalities: vec![],
        images: images
            .into_iter()
            .map(|img| guide_image_to_template_image_response(img, 0))
            .collect(),
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

pub fn to_template_response_with_merged_images(
    template: ReportGuideTemplate,
    images: Vec<TemplateImageResponse>,
) -> ReportGuideTemplateResponse {
    ReportGuideTemplateResponse {
        id: template.id,
        description: template.description,
        conclusion: template.conclusion,
        bodypart: template.bodypart,
        is_shared: template.is_shared,
        is_active: template.is_active,
        created_by: template.created_by,
        modalities: vec![],
        images,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

/// 기존 구조(deprecated) + 신규 구조 이미지 병합
pub fn merge_template_images(
    old_images: Vec<ReportGuideTemplateImage>,
    guide_images: Vec<GuideImage>,
) -> Vec<TemplateImageResponse> {
    let mut result: Vec<TemplateImageResponse> = old_images
        .into_iter()
        .map(|img| to_image_response(img))
        .collect();
    let offset = result.len() as i32;
    for (idx, img) in guide_images.into_iter().enumerate() {
        result.push(guide_image_to_template_image_response(
            img,
            offset + idx as i32,
        ));
    }
    result
}

// ========== 유효 템플릿 ==========

pub fn to_effective_from_original(
    template: &ReportGuideTemplate,
    images: Vec<TemplateImageResponse>,
) -> EffectiveReportTemplateResponse {
    EffectiveReportTemplateResponse {
        source: "original".to_string(),
        template_id: Some(template.id),
        custom_template_id: None,
        base_template_id: None,
        description: template.description.clone(),
        conclusion: template.conclusion.clone(),
        bodypart: template.bodypart.clone(),
        modalities: vec![],
        images,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

pub fn to_effective_from_custom(
    template: &UserCustomReportTemplate,
    images: Vec<GuideImage>,
) -> EffectiveReportTemplateResponse {
    let image_responses: Vec<TemplateImageResponse> = images
        .into_iter()
        .map(|img| guide_image_to_template_image_response(img, 0))
        .collect();
    EffectiveReportTemplateResponse {
        source: "custom".to_string(),
        template_id: None,
        custom_template_id: Some(template.id),
        base_template_id: template.base_template_id,
        description: template.description.clone(),
        conclusion: template.conclusion.clone(),
        bodypart: template.bodypart.clone(),
        modalities: vec![],
        images: image_responses,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

// ========== 커스텀 템플릿 ==========

pub fn to_custom_template_response(
    template: UserCustomReportTemplate,
    images: Vec<UserCustomTemplateImage>,
) -> UserCustomReportTemplateResponse {
    UserCustomReportTemplateResponse {
        id: template.id,
        user_id: template.user_id,
        base_template_id: template.base_template_id,
        description: template.description,
        conclusion: template.conclusion,
        bodypart: template.bodypart,
        is_active: template.is_active,
        modalities: vec![],
        images: images
            .into_iter()
            .map(|img| to_custom_image_response(img))
            .collect(),
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

pub fn to_custom_template_response_with_guide_images(
    template: UserCustomReportTemplate,
    images: Vec<GuideImage>,
) -> UserCustomReportTemplateResponse {
    UserCustomReportTemplateResponse {
        id: template.id,
        user_id: template.user_id,
        base_template_id: template.base_template_id,
        description: template.description,
        conclusion: template.conclusion,
        bodypart: template.bodypart,
        is_active: template.is_active,
        modalities: vec![],
        images: images
            .into_iter()
            .map(|img| guide_image_to_custom_template_image_response(img, 0))
            .collect(),
        created_at: template.created_at,
        updated_at: template.updated_at,
    }
}

// ========== 이미지 변환 ==========

pub fn to_image_response(image: ReportGuideTemplateImage) -> TemplateImageResponse {
    TemplateImageResponse {
        id: image.id,
        image_source: "template".to_string(),
        template_id: Some(image.template_id),
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

pub fn to_custom_image_response(image: UserCustomTemplateImage) -> CustomTemplateImageResponse {
    CustomTemplateImageResponse {
        id: image.id,
        image_source: "custom_template".to_string(),
        custom_template_id: Some(image.custom_template_id),
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

pub fn to_guide_image_response(image: GuideImage) -> GuideImageResponse {
    GuideImageResponse {
        id: image.id,
        image_source: "guide".to_string(),
        image_path: image.image_path,
        image_url: image.image_url,
        file_size: image.file_size,
        mime_type: image.mime_type,
        is_shared: image.is_shared,
        uploaded_by: image.uploaded_by,
        created_at: image.created_at,
    }
}

/// GuideImage → TemplateImageResponse (display_order 지정)
pub fn guide_image_to_template_image_response(
    image: GuideImage,
    display_order: i32,
) -> TemplateImageResponse {
    TemplateImageResponse {
        id: image.id,
        image_source: "guide".to_string(),
        template_id: None,
        image_path: image.image_path,
        image_url: image.image_url,
        file_size: image.file_size,
        mime_type: image.mime_type,
        display_order,
        is_shared: image.is_shared,
        uploaded_by: image.uploaded_by,
        created_at: image.created_at,
    }
}

/// GuideImage → CustomTemplateImageResponse (display_order 지정)
pub fn guide_image_to_custom_template_image_response(
    image: GuideImage,
    display_order: i32,
) -> CustomTemplateImageResponse {
    CustomTemplateImageResponse {
        id: image.id,
        image_source: "guide".to_string(),
        custom_template_id: None,
        image_path: image.image_path,
        image_url: image.image_url,
        file_size: image.file_size,
        mime_type: image.mime_type,
        display_order,
        is_shared: image.is_shared,
        uploaded_by: image.uploaded_by,
        created_at: image.created_at,
    }
}
