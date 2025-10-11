use async_trait::async_trait;
use crate::domain::entities::{Annotation, AnnotationHistory, NewAnnotation};
use crate::domain::repositories::{AnnotationRepository, UserRepository, ProjectRepository};
use crate::domain::ServiceError;

/// Annotation 관리 도메인 서비스
#[async_trait]
pub trait AnnotationService: Send + Sync {
    /// Annotation 생성
    async fn create_annotation(&self, new_annotation: NewAnnotation) -> Result<Annotation, ServiceError>;

    /// Annotation 조회 (ID)
    async fn get_annotation_by_id(&self, id: i32) -> Result<Annotation, ServiceError>;

    /// 프로젝트의 Annotation 목록 조회
    async fn get_annotations_by_project(&self, project_id: i32) -> Result<Vec<Annotation>, ServiceError>;

    /// 사용자의 Annotation 목록 조회
    async fn get_annotations_by_user(&self, user_id: i32) -> Result<Vec<Annotation>, ServiceError>;

    /// Study UID로 Annotation 목록 조회
    async fn get_annotations_by_study(&self, study_uid: &str) -> Result<Vec<Annotation>, ServiceError>;

    /// Series UID로 Annotation 목록 조회
    async fn get_annotations_by_series(&self, series_uid: &str) -> Result<Vec<Annotation>, ServiceError>;

    /// Instance UID로 Annotation 목록 조회
    async fn get_annotations_by_instance(&self, instance_uid: &str) -> Result<Vec<Annotation>, ServiceError>;

    /// 프로젝트와 Study UID로 Annotation 목록 조회
    async fn get_annotations_by_project_and_study(&self, project_id: i32, study_uid: &str) -> Result<Vec<Annotation>, ServiceError>;

    /// 공유 Annotation 목록 조회
    async fn get_shared_annotations(&self, project_id: i32) -> Result<Vec<Annotation>, ServiceError>;

    /// Annotation 업데이트
    async fn update_annotation(&self, id: i32, data: serde_json::Value, is_shared: bool) -> Result<Annotation, ServiceError>;

    /// Annotation 삭제
    async fn delete_annotation(&self, id: i32) -> Result<(), ServiceError>;

    /// Annotation 히스토리 생성
    async fn create_history(&self, annotation_id: i32, user_id: i32, action: &str, data_before: Option<serde_json::Value>, data_after: Option<serde_json::Value>) -> Result<AnnotationHistory, ServiceError>;

    /// Annotation 히스토리 조회
    async fn get_annotation_history(&self, annotation_id: i32) -> Result<Vec<AnnotationHistory>, ServiceError>;

    /// 사용자가 Annotation에 접근할 수 있는지 확인
    async fn can_access_annotation(&self, user_id: i32, annotation_id: i32) -> Result<bool, ServiceError>;
}

pub struct AnnotationServiceImpl<A, U, P>
where
    A: AnnotationRepository,
    U: UserRepository,
    P: ProjectRepository,
{
    annotation_repository: A,
    user_repository: U,
    project_repository: P,
}

impl<A, U, P> AnnotationServiceImpl<A, U, P>
where
    A: AnnotationRepository,
    U: UserRepository,
    P: ProjectRepository,
{
    pub fn new(annotation_repository: A, user_repository: U, project_repository: P) -> Self {
        Self {
            annotation_repository,
            user_repository,
            project_repository,
        }
    }
}

#[async_trait]
impl<A, U, P> AnnotationService for AnnotationServiceImpl<A, U, P>
where
    A: AnnotationRepository,
    U: UserRepository,
    P: ProjectRepository,
{
    async fn create_annotation(&self, new_annotation: NewAnnotation) -> Result<Annotation, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(new_annotation.user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(new_annotation.project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 사용자가 프로젝트 멤버인지 확인
        let is_member = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(new_annotation.user_id)
        .bind(new_annotation.project_id)
        .fetch_one(self.annotation_repository.pool())
        .await?;

        if is_member == 0 {
            return Err(ServiceError::Unauthorized("User is not a member of this project".into()));
        }

        Ok(self.annotation_repository.create(new_annotation).await?)
    }

    async fn get_annotation_by_id(&self, id: i32) -> Result<Annotation, ServiceError> {
        self.annotation_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Annotation not found".into()))
    }

    async fn get_annotations_by_project(&self, project_id: i32) -> Result<Vec<Annotation>, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        Ok(self.annotation_repository.find_by_project_id(project_id).await?)
    }

    async fn get_annotations_by_user(&self, user_id: i32) -> Result<Vec<Annotation>, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        Ok(self.annotation_repository.find_by_user_id(user_id).await?)
    }

    async fn get_annotations_by_study(&self, study_uid: &str) -> Result<Vec<Annotation>, ServiceError> {
        Ok(self.annotation_repository.find_by_study_uid(study_uid).await?)
    }

    async fn get_annotations_by_series(&self, series_uid: &str) -> Result<Vec<Annotation>, ServiceError> {
        Ok(self.annotation_repository.find_by_series_uid(series_uid).await?)
    }

    async fn get_annotations_by_instance(&self, instance_uid: &str) -> Result<Vec<Annotation>, ServiceError> {
        Ok(self.annotation_repository.find_by_instance_uid(instance_uid).await?)
    }

    async fn get_annotations_by_project_and_study(&self, project_id: i32, study_uid: &str) -> Result<Vec<Annotation>, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        Ok(self.annotation_repository.find_by_project_and_study(project_id, study_uid).await?)
    }

    async fn get_shared_annotations(&self, project_id: i32) -> Result<Vec<Annotation>, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        Ok(self.annotation_repository.find_shared_annotations(project_id).await?)
    }

    async fn update_annotation(&self, id: i32, data: serde_json::Value, is_shared: bool) -> Result<Annotation, ServiceError> {
        // Annotation 존재 확인
        let annotation = self.get_annotation_by_id(id).await?;

        // 업데이트 실행
        match self.annotation_repository.update(id, data, is_shared).await? {
            Some(updated_annotation) => {
                // 히스토리 생성
                self.create_history(
                    id,
                    annotation.user_id,
                    "UPDATE",
                    Some(annotation.data),
                    Some(updated_annotation.data.clone())
                ).await?;
                Ok(updated_annotation)
            }
            None => Err(ServiceError::NotFound("Annotation not found".into()))
        }
    }

    async fn delete_annotation(&self, id: i32) -> Result<(), ServiceError> {
        // Annotation 존재 확인
        self.get_annotation_by_id(id).await?;

        let deleted = self.annotation_repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Annotation not found".into()))
        }
    }

    async fn create_history(&self, annotation_id: i32, user_id: i32, action: &str, data_before: Option<serde_json::Value>, data_after: Option<serde_json::Value>) -> Result<AnnotationHistory, ServiceError> {
        Ok(self.annotation_repository.create_history(annotation_id, user_id, action, data_before, data_after).await?)
    }

    async fn get_annotation_history(&self, annotation_id: i32) -> Result<Vec<AnnotationHistory>, ServiceError> {
        // Annotation 존재 확인
        self.get_annotation_by_id(annotation_id).await?;

        Ok(self.annotation_repository.get_history(annotation_id).await?)
    }

    async fn can_access_annotation(&self, user_id: i32, annotation_id: i32) -> Result<bool, ServiceError> {
        // Annotation 조회
        let annotation = self.get_annotation_by_id(annotation_id).await?;

        // 사용자가 프로젝트 멤버인지 확인
        let is_member = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(annotation.project_id)
        .fetch_one(self.annotation_repository.pool())
        .await?;

        Ok(is_member > 0)
    }
}


