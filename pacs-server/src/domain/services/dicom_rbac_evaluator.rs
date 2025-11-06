use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RbacEvaluationResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

#[async_trait]
pub trait DicomRbacEvaluator: Send + Sync {
    async fn evaluate_study_access(
        &self,
        user_id: i32,
        project_id: i32,
        study_id: i32,
    ) -> RbacEvaluationResult;
    async fn evaluate_series_access(
        &self,
        user_id: i32,
        project_id: i32,
        series_id: i32,
    ) -> RbacEvaluationResult;
    async fn evaluate_instance_access(
        &self,
        user_id: i32,
        project_id: i32,
        instance_id: i32,
    ) -> RbacEvaluationResult;
    async fn evaluate_study_uid(
        &self,
        user_id: i32,
        project_id: i32,
        study_uid: &str,
    ) -> RbacEvaluationResult;
    async fn evaluate_series_uid(
        &self,
        user_id: i32,
        project_id: i32,
        series_uid: &str,
    ) -> RbacEvaluationResult;
    async fn evaluate_instance_uid(
        &self,
        user_id: i32,
        project_id: i32,
        instance_uid: &str,
    ) -> RbacEvaluationResult;
}
