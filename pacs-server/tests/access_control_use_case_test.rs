use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use uuid::Uuid;

use pacs_server::application::dto::access_control_dto::{
    AccessLogListResponse, AccessLogResponse, CheckPermissionRequest, CheckPermissionResponse,
    LogDicomAccessRequest, ProjectAccessResponse, UserPermissionsResponse, PermissionInfo,
};
use pacs_server::application::use_cases::AccessControlUseCase;
use pacs_server::domain::entities::{AccessLog, Project, Role, User, Permission};
use pacs_server::domain::services::AccessControlService;
use pacs_server::domain::ServiceError;

// Mock AccessControlService for testing
#[derive(Clone)]
struct MockAccessControlService {
    users: HashMap<i32, User>,
    projects: HashMap<i32, Project>,
    roles: HashMap<i32, Role>,
    access_logs: Vec<AccessLog>,
    user_permissions: HashMap<i32, Vec<Permission>>, // user_id -> permissions
    project_members: HashMap<i32, Vec<i32>>, // project_id -> user_ids
}

impl MockAccessControlService {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            projects: HashMap::new(),
            roles: HashMap::new(),
            access_logs: Vec::new(),
            user_permissions: HashMap::new(),
            project_members: HashMap::new(),
        }
    }

    fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id, project);
    }

    fn add_role(&mut self, role: Role) {
        self.roles.insert(role.id, role);
    }

    fn add_access_log(&mut self, log: AccessLog) {
        self.access_logs.push(log);
    }

    fn add_user_permission(&mut self, user_id: i32, permission: Permission) {
        self.user_permissions.entry(user_id).or_default().push(permission);
    }

    fn add_project_member(&mut self, project_id: i32, user_id: i32) {
        self.project_members.entry(project_id).or_default().push(user_id);
    }
}

#[async_trait]
impl AccessControlService for MockAccessControlService {
    async fn log_dicom_access(
        &self,
        user_id: i32,
        project_id: Option<i32>,
        resource_type: String,
        study_uid: Option<String>,
        series_uid: Option<String>,
        instance_uid: Option<String>,
        action: String,
        result: String,
        ip_address: Option<String>,
        ae_title: Option<String>,
    ) -> Result<AccessLog, ServiceError> {
        let new_id = (self.access_logs.len() + 1) as i64;
        let log = AccessLog {
            id: new_id,
            user_id,
            project_id,
            resource_type,
            study_uid,
            series_uid,
            instance_uid,
            action,
            result,
            dicom_tag_check: None,
            ae_title,
            ip_address,
            session_id: None,
            via_group_id: None,
            logged_at: chrono::Utc::now(),
        };
        Ok(log)
    }

    async fn get_user_access_logs(&self, user_id: i32, limit: i64) -> Result<Vec<AccessLog>, ServiceError> {
        let logs: Vec<AccessLog> = self.access_logs
            .iter()
            .filter(|log| log.user_id == user_id)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(logs)
    }

    async fn get_project_access_logs(&self, project_id: i32, limit: i64) -> Result<Vec<AccessLog>, ServiceError> {
        let logs: Vec<AccessLog> = self.access_logs
            .iter()
            .filter(|log| log.project_id == Some(project_id))
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(logs)
    }

    async fn get_study_access_logs(&self, study_uid: &str, limit: i64) -> Result<Vec<AccessLog>, ServiceError> {
        let logs: Vec<AccessLog> = self.access_logs
            .iter()
            .filter(|log| log.study_uid.as_ref().map_or(false, |uid| uid == study_uid))
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(logs)
    }

    async fn count_user_access(&self, user_id: i32) -> Result<i64, ServiceError> {
        Ok(self.access_logs.iter().filter(|log| log.user_id == user_id).count() as i64)
    }

    async fn can_access_project(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        Ok(self.project_members.get(&project_id).map_or(false, |members| members.contains(&user_id)))
    }

    async fn check_permission(
        &self,
        user_id: i32,
        project_id: i32,
        resource_type: &str,
        action: &str,
    ) -> Result<bool, ServiceError> {
        if let Some(permissions) = self.user_permissions.get(&user_id) {
            Ok(permissions.iter().any(|p| p.resource_type == resource_type && p.action == action))
        } else {
            Ok(false)
        }
    }

    async fn get_user_permissions(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Vec<Permission>, ServiceError> {
        Ok(self.user_permissions.get(&user_id).cloned().unwrap_or_default())
    }

    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        Ok(self.project_members.get(&project_id).map_or(false, |members| members.contains(&user_id)))
    }
}

fn create_test_user() -> User {
    User {
        id: 1,
        keycloak_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: None,
        organization: None,
        department: None,
        phone: None,
        created_at: chrono::Utc::now(),
        updated_at: None,
        account_status: pacs_server::domain::entities::UserAccountStatus::Active,
        email_verified: true,
        email_verification_token: None,
        email_verification_expires_at: None,
        approved_by: None,
        approved_at: None,
        suspended_at: None,
        suspended_reason: None,
        deleted_at: None,
    }
}

fn create_test_project() -> Project {
    Project {
        id: 1,
        name: "Test Project".to_string(),
        description: Some("A test project".to_string()),
        is_active: true,
        sponsor: None,
        start_date: None,
        end_date: None,
        auto_complete: false,
        created_at: chrono::Utc::now(),
    }
}

fn create_test_role() -> Role {
    Role {
        id: 1,
        name: "Test Role".to_string(),
        description: Some("A test role".to_string()),
        scope: "GLOBAL".to_string(),
        created_at: chrono::Utc::now(),
    }
}

fn create_test_permission() -> Permission {
    Permission {
        id: 1,
        category: "Imaging".to_string(),
        resource_type: "STUDY".to_string(),
        action: "READ".to_string(),
    }
}

fn create_test_access_log() -> AccessLog {
    AccessLog {
        id: 1,
        user_id: 1,
        project_id: Some(1),
        resource_type: "STUDY".to_string(),
        study_uid: Some("1.2.3".to_string()),
        series_uid: None,
        instance_uid: None,
        action: "READ".to_string(),
        result: "SUCCESS".to_string(),
        dicom_tag_check: None,
        ae_title: None,
        ip_address: Some("127.0.0.1".to_string()),
        session_id: None,
        via_group_id: None,
        logged_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_access_control_use_case_log_dicom_access_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_user(create_test_user());
    mock_access_control_service.add_project(create_test_project());
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let request = LogDicomAccessRequest {
        user_id: 1,
        project_id: Some(1),
        resource_type: "STUDY".to_string(),
        study_uid: Some("1.2.3".to_string()),
        series_uid: None,
        instance_uid: None,
        action: "READ".to_string(),
        result: "SUCCESS".to_string(),
        ip_address: Some("127.0.0.1".to_string()),
        ae_title: None,
    };

    let result = access_control_use_case.log_dicom_access(request).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.user_id, 1);
    assert_eq!(response.resource_type, "STUDY");
}

#[tokio::test]
async fn test_access_control_use_case_get_user_access_logs_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_access_log(create_test_access_log());
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let result = access_control_use_case.get_user_access_logs(1, 10).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.total, 1);
    assert_eq!(response.logs[0].user_id, 1);
}

#[tokio::test]
async fn test_access_control_use_case_get_project_access_logs_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_access_log(create_test_access_log());
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let result = access_control_use_case.get_project_access_logs(1, 10).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.total, 1);
    assert_eq!(response.logs[0].project_id, Some(1));
}

#[tokio::test]
async fn test_access_control_use_case_get_study_access_logs_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_access_log(create_test_access_log());
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let result = access_control_use_case.get_study_access_logs("1.2.3", 10).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.logs.len(), 1);
    assert_eq!(response.total, 1);
    assert_eq!(response.logs[0].study_uid, Some("1.2.3".to_string()));
}


#[tokio::test]
async fn test_access_control_use_case_can_access_project_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_project_member(1, 1);
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let result = access_control_use_case.can_access_project(1, 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.can_access);
    assert!(response.is_member);
}

#[tokio::test]
async fn test_access_control_use_case_can_access_project_failure() {
    let mock_access_control_service = MockAccessControlService::new();
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let result = access_control_use_case.can_access_project(1, 999).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.can_access);
    assert!(!response.is_member);
}

#[tokio::test]
async fn test_access_control_use_case_check_permission_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_user_permission(1, create_test_permission());
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let request = CheckPermissionRequest {
        user_id: 1,
        project_id: 1,
        resource_type: "STUDY".to_string(),
        action: "READ".to_string(),
    };

    let result = access_control_use_case.check_permission(request).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.has_permission);
}

#[tokio::test]
async fn test_access_control_use_case_check_permission_failure() {
    let mock_access_control_service = MockAccessControlService::new();
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let request = CheckPermissionRequest {
        user_id: 1,
        project_id: 1,
        resource_type: "STUDY".to_string(),
        action: "WRITE".to_string(),
    };

    let result = access_control_use_case.check_permission(request).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.has_permission);
}

#[tokio::test]
async fn test_access_control_use_case_get_user_permissions_success() {
    let mut mock_access_control_service = MockAccessControlService::new();
    mock_access_control_service.add_user_permission(1, create_test_permission());
    
    let access_control_use_case = AccessControlUseCase::new(mock_access_control_service);

    let result = access_control_use_case.get_user_permissions(1, 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.permissions.len(), 1);
    assert_eq!(response.permissions[0].resource_type, "STUDY");
    assert_eq!(response.user_id, 1);
    assert_eq!(response.project_id, 1);
}
