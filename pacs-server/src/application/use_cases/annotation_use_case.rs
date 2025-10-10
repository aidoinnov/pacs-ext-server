use crate::application::dto::{
    CreateAnnotationRequest, UpdateAnnotationRequest, AnnotationResponse, AnnotationListResponse,
};
use crate::domain::services::{AnnotationService};
use crate::domain::services::annotation_service::ServiceError;
use crate::domain::entities::{NewAnnotation};

/// Annotation 관리 유스케이스
pub struct AnnotationUseCase<A: AnnotationService> {
    annotation_service: A,
}

impl<A: AnnotationService> AnnotationUseCase<A> {
    pub fn new(annotation_service: A) -> Self {
        Self { annotation_service }
    }

    /// Annotation 생성
    pub async fn create_annotation(&self, request: CreateAnnotationRequest, user_id: i32, project_id: i32) -> Result<AnnotationResponse, ServiceError> {
        let new_annotation = NewAnnotation {
            project_id,
            user_id,
            study_uid: request.study_instance_uid,
            series_uid: Some(request.series_instance_uid),
            instance_uid: Some(request.sop_instance_uid),
            tool_name: request.tool_name.unwrap_or_else(|| "manual".to_string()),
            tool_version: request.tool_version,
            viewer_software: request.viewer_software,
            description: request.description,
            data: request.annotation_data,
            is_shared: false, // 기본값은 비공유
        };

        let annotation = self.annotation_service.create_annotation(new_annotation).await?;

        Ok(AnnotationResponse {
            id: annotation.id,
            user_id: annotation.user_id,
            study_instance_uid: annotation.study_uid,
            series_instance_uid: annotation.series_uid.unwrap_or_default(),
            sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
            annotation_data: annotation.data,
            tool_name: Some(annotation.tool_name),
            tool_version: annotation.tool_version,
            viewer_software: annotation.viewer_software,
            description: annotation.description,
            created_at: annotation.created_at,
            updated_at: annotation.updated_at,
        })
    }

    /// Annotation 조회 (ID)
    pub async fn get_annotation_by_id(&self, annotation_id: i32) -> Result<AnnotationResponse, ServiceError> {
        let annotation = self.annotation_service.get_annotation_by_id(annotation_id).await?;

        Ok(AnnotationResponse {
            id: annotation.id,
            user_id: annotation.user_id,
            study_instance_uid: annotation.study_uid,
            series_instance_uid: annotation.series_uid.unwrap_or_default(),
            sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
            annotation_data: annotation.data,
            tool_name: Some(annotation.tool_name),
            tool_version: annotation.tool_version,
            viewer_software: annotation.viewer_software,
            description: annotation.description,
            created_at: annotation.created_at,
            updated_at: annotation.updated_at,
        })
    }

    /// 프로젝트의 Annotation 목록 조회
    pub async fn get_annotations_by_project(&self, project_id: i32) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_annotations_by_project(project_id).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// 사용자의 Annotation 목록 조회
    pub async fn get_annotations_by_user(&self, user_id: i32) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_annotations_by_user(user_id).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// Study UID로 Annotation 목록 조회
    pub async fn get_annotations_by_study(&self, study_uid: &str) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_annotations_by_study(study_uid).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// Series UID로 Annotation 목록 조회
    pub async fn get_annotations_by_series(&self, series_uid: &str) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_annotations_by_series(series_uid).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// Instance UID로 Annotation 목록 조회
    pub async fn get_annotations_by_instance(&self, instance_uid: &str) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_annotations_by_instance(instance_uid).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// 프로젝트와 Study UID로 Annotation 목록 조회
    pub async fn get_annotations_by_project_and_study(&self, project_id: i32, study_uid: &str) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_annotations_by_project_and_study(project_id, study_uid).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// 공유 Annotation 목록 조회
    pub async fn get_shared_annotations(&self, project_id: i32) -> Result<AnnotationListResponse, ServiceError> {
        let annotations = self.annotation_service.get_shared_annotations(project_id).await?;

        let total = annotations.len();
        let annotation_responses = annotations
            .into_iter()
            .map(|annotation| AnnotationResponse {
                id: annotation.id,
                user_id: annotation.user_id,
                study_instance_uid: annotation.study_uid,
                series_instance_uid: annotation.series_uid.unwrap_or_default(),
                sop_instance_uid: annotation.instance_uid.unwrap_or_default(),
                annotation_data: annotation.data,
                tool_name: Some(annotation.tool_name),
                tool_version: annotation.tool_version,
                viewer_software: annotation.viewer_software,
                description: annotation.description,
                created_at: annotation.created_at,
                updated_at: annotation.updated_at,
            })
            .collect();

        Ok(AnnotationListResponse {
            annotations: annotation_responses,
            total,
        })
    }

    /// Annotation 업데이트
    pub async fn update_annotation(&self, annotation_id: i32, request: UpdateAnnotationRequest) -> Result<AnnotationResponse, ServiceError> {
        // 현재 annotation 조회
        let current_annotation = self.annotation_service.get_annotation_by_id(annotation_id).await?;
        
        // 업데이트할 데이터 결정
        let new_data = request.annotation_data.unwrap_or(current_annotation.data);
        let is_shared = false; // 현재는 is_shared 업데이트 기능 없음

        let updated_annotation = self.annotation_service.update_annotation(annotation_id, new_data, is_shared).await?;

        Ok(AnnotationResponse {
            id: updated_annotation.id,
            user_id: updated_annotation.user_id,
            study_instance_uid: updated_annotation.study_uid,
            series_instance_uid: updated_annotation.series_uid.unwrap_or_default(),
            sop_instance_uid: updated_annotation.instance_uid.unwrap_or_default(),
            annotation_data: updated_annotation.data,
            tool_name: Some(updated_annotation.tool_name),
            tool_version: updated_annotation.tool_version,
            viewer_software: updated_annotation.viewer_software,
            description: updated_annotation.description,
            created_at: updated_annotation.created_at,
            updated_at: updated_annotation.updated_at,
        })
    }

    /// Annotation 삭제
    pub async fn delete_annotation(&self, annotation_id: i32) -> Result<(), ServiceError> {
        self.annotation_service.delete_annotation(annotation_id).await
    }

    /// Annotation 접근 권한 확인
    pub async fn can_access_annotation(&self, user_id: i32, annotation_id: i32) -> Result<bool, ServiceError> {
        self.annotation_service.can_access_annotation(user_id, annotation_id).await
    }
}
