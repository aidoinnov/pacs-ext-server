use pacs_server::application::dto::project_user_dto::{
    UserWithRoleResponse, ProjectWithRoleResponse, ProjectMembersResponse, UserProjectsResponse,
    AssignRoleRequest, BatchAssignRolesRequest, UserRoleAssignment, RoleAssignmentResponse,
    BatchRoleAssignmentResponse, FailedAssignment
};
use serde_json::json;

#[test]
fn test_user_with_role_response_serialization() {
    let user = UserWithRoleResponse {
        user_id: 1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        role_id: Some(2),
        role_name: Some("Admin".to_string()),
        role_scope: Some("GLOBAL".to_string()),
    };
    
    let serialized = serde_json::to_string(&user).unwrap();
    assert!(serialized.contains("\"user_id\":1"));
    assert!(serialized.contains("\"username\":\"testuser\""));
    assert!(serialized.contains("\"role_name\":\"Admin\""));
}

#[test]
fn test_project_with_role_response_serialization() {
    let project = ProjectWithRoleResponse {
        project_id: 1,
        project_name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        is_active: true,
        start_date: None,
        end_date: None,
        role_id: Some(2),
        role_name: Some("Manager".to_string()),
        role_scope: Some("PROJECT".to_string()),
    };
    
    let serialized = serde_json::to_string(&project).unwrap();
    assert!(serialized.contains("\"project_id\":1"));
    assert!(serialized.contains("\"project_name\":\"Test Project\""));
    assert!(serialized.contains("\"role_name\":\"Manager\""));
}

#[test]
fn test_project_members_response_serialization() {
    let user = UserWithRoleResponse {
        user_id: 1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        role_id: Some(2),
        role_name: Some("Admin".to_string()),
        role_scope: Some("GLOBAL".to_string()),
    };
    
    let response = ProjectMembersResponse {
        members: vec![user],
        total_count: 1,
        page: 1,
        page_size: 20,
        total_pages: 1,
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"members\""));
    assert!(serialized.contains("\"total_count\":1"));
    assert!(serialized.contains("\"page\":1"));
}

#[test]
fn test_user_projects_response_serialization() {
    let project = ProjectWithRoleResponse {
        project_id: 1,
        project_name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        is_active: true,
        start_date: None,
        end_date: None,
        role_id: Some(2),
        role_name: Some("Manager".to_string()),
        role_scope: Some("PROJECT".to_string()),
    };
    
    let response = UserProjectsResponse {
        projects: vec![project],
        total_count: 1,
        page: 1,
        page_size: 20,
        total_pages: 1,
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"projects\""));
    assert!(serialized.contains("\"total_count\":1"));
    assert!(serialized.contains("\"page\":1"));
}

#[test]
fn test_assign_role_request_deserialization() {
    let json_str = r#"{"role_id": 2}"#;
    let request: AssignRoleRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(request.role_id, 2);
}

#[test]
fn test_batch_assign_roles_request_deserialization() {
    let json_str = r#"{
        "assignments": [
            {"user_id": 1, "role_id": 2},
            {"user_id": 3, "role_id": 4}
        ]
    }"#;
    let request: BatchAssignRolesRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(request.assignments.len(), 2);
    assert_eq!(request.assignments[0].user_id, 1);
    assert_eq!(request.assignments[0].role_id, 2);
    assert_eq!(request.assignments[1].user_id, 3);
    assert_eq!(request.assignments[1].role_id, 4);
}

#[test]
fn test_role_assignment_response_serialization() {
    let response = RoleAssignmentResponse {
        message: "Role assigned successfully".to_string(),
        user_id: 1,
        project_id: 2,
        role_id: 3,
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"message\":\"Role assigned successfully\""));
    assert!(serialized.contains("\"user_id\":1"));
    assert!(serialized.contains("\"project_id\":2"));
    assert!(serialized.contains("\"role_id\":3"));
}

#[test]
fn test_batch_role_assignment_response_serialization() {
    let failed_assignment = FailedAssignment {
        user_id: 1,
        role_id: 2,
        error: "User not found".to_string(),
    };
    
    let response = BatchRoleAssignmentResponse {
        message: "Batch assignment completed".to_string(),
        project_id: 1,
        assigned_count: 2,
        failed_assignments: vec![failed_assignment],
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"message\":\"Batch assignment completed\""));
    assert!(serialized.contains("\"assigned_count\":2"));
    assert!(serialized.contains("\"failed_assignments\""));
}

#[test]
fn test_failed_assignment_serialization() {
    let failed = FailedAssignment {
        user_id: 1,
        role_id: 2,
        error: "User not found".to_string(),
    };
    
    let serialized = serde_json::to_string(&failed).unwrap();
    assert!(serialized.contains("\"user_id\":1"));
    assert!(serialized.contains("\"role_id\":2"));
    assert!(serialized.contains("\"error\":\"User not found\""));
}
