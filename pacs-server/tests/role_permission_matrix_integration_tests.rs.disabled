use pacs_server::application::dto::role_permission_matrix_dto::*;
use std::collections::HashMap;

#[test]
fn test_role_permission_matrix_dto_creation() {
    let role_info = RoleInfo {
        id: 1,
        name: "Admin".to_string(),
        description: Some("Administrator role".to_string()),
        scope: "GLOBAL".to_string(),
    };
    
    assert_eq!(role_info.id, 1);
    assert_eq!(role_info.name, "Admin");
    assert_eq!(role_info.scope, "GLOBAL");
}

#[test]
fn test_permission_info_creation() {
    let permission_info = PermissionInfo {
        id: 1,
        resource_type: "USER".to_string(),
        action: "READ".to_string(),
    };
    
    assert_eq!(permission_info.id, 1);
    assert_eq!(permission_info.resource_type, "USER");
    assert_eq!(permission_info.action, "READ");
}

#[test]
fn test_role_permission_assignment_creation() {
    let assignment = RolePermissionAssignment {
        role_id: 1,
        permission_id: 2,
        assigned: true,
    };
    
    assert_eq!(assignment.role_id, 1);
    assert_eq!(assignment.permission_id, 2);
    assert!(assignment.assigned);
}

#[test]
fn test_assign_permission_request_creation() {
    let request = AssignPermissionRequest {
        assign: true,
    };
    
    assert!(request.assign);
}

#[test]
fn test_assign_permission_response_creation() {
    let response = AssignPermissionResponse {
        success: true,
        message: "Permission assigned successfully".to_string(),
    };
    
    assert!(response.success);
    assert_eq!(response.message, "Permission assigned successfully");
}

#[test]
fn test_matrix_response_creation() {
    let roles = vec![
        RoleInfo {
            id: 1,
            name: "Admin".to_string(),
            description: Some("Administrator role".to_string()),
            scope: "GLOBAL".to_string(),
        },
    ];
    
    let mut permissions_by_category = HashMap::new();
    permissions_by_category.insert("USER".to_string(), vec![
        PermissionInfo {
            id: 1,
            resource_type: "USER".to_string(),
            action: "READ".to_string(),
        },
    ]);
    
    let assignments = vec![
        RolePermissionAssignment {
            role_id: 1,
            permission_id: 1,
            assigned: true,
        },
    ];
    
    let matrix = RolePermissionMatrixResponse {
        roles,
        permissions_by_category,
        assignments,
    };
    
    assert_eq!(matrix.roles.len(), 1);
    assert_eq!(matrix.permissions_by_category.len(), 1);
    assert_eq!(matrix.assignments.len(), 1);
}

