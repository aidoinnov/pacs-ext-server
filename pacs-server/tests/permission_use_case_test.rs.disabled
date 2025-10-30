use pacs_server::application::dto::permission_dto::{
    RoleWithPermissionsResponse, RolesWithPermissionsListResponse, PaginationQuery, PermissionResponse
};

#[test]
fn test_role_with_permissions_response_creation() {
    // Given
    let permissions = vec![
        PermissionResponse {
            id: 1,
            resource_type: "user".to_string(),
            action: "create".to_string(),
        },
        PermissionResponse {
            id: 2,
            resource_type: "user".to_string(),
            action: "delete".to_string(),
        },
    ];

    let role = RoleWithPermissionsResponse {
        id: 1,
        name: "시스템 관리자".to_string(),
        description: Some("전체 시스템 관리 권한".to_string()),
        scope: "GLOBAL".to_string(),
        permissions,
    };

    // Then
    assert_eq!(role.id, 1);
    assert_eq!(role.name, "시스템 관리자");
    assert_eq!(role.description, Some("전체 시스템 관리 권한".to_string()));
    assert_eq!(role.scope, "GLOBAL");
    assert_eq!(role.permissions.len(), 2);
    assert_eq!(role.permissions[0].resource_type, "user");
    assert_eq!(role.permissions[0].action, "create");
}

#[test]
fn test_roles_with_permissions_list_response_creation() {
    // Given
    let roles = vec![
        RoleWithPermissionsResponse {
            id: 1,
            name: "시스템 관리자".to_string(),
            description: Some("전체 시스템 관리 권한".to_string()),
            scope: "GLOBAL".to_string(),
            permissions: vec![],
        },
    ];

    let response = RolesWithPermissionsListResponse {
        roles,
        total_count: 1,
        page: 1,
        page_size: 20,
        total_pages: 1,
    };

    // Then
    assert_eq!(response.roles.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 20);
    assert_eq!(response.total_pages, 1);
}

#[test]
fn test_pagination_query_creation() {
    // Given
    let query = PaginationQuery {
        page: Some(2),
        page_size: Some(50),
    };

    // Then
    assert_eq!(query.page, Some(2));
    assert_eq!(query.page_size, Some(50));
}

#[test]
fn test_pagination_query_default_values() {
    // Given
    let query = PaginationQuery {
        page: None,
        page_size: None,
    };

    // Then
    assert_eq!(query.page, None);
    assert_eq!(query.page_size, None);
}

#[test]
fn test_permission_response_creation() {
    // Given
    let permission = PermissionResponse {
        id: 1,
        resource_type: "user".to_string(),
        action: "create".to_string(),
    };

    // Then
    assert_eq!(permission.id, 1);
    assert_eq!(permission.resource_type, "user");
    assert_eq!(permission.action, "create");
}

#[test]
fn test_role_with_permissions_response_without_description() {
    // Given
    let role = RoleWithPermissionsResponse {
        id: 1,
        name: "시스템 관리자".to_string(),
        description: None, // No description
        scope: "GLOBAL".to_string(),
        permissions: vec![],
    };

    // Then
    assert_eq!(role.id, 1);
    assert_eq!(role.name, "시스템 관리자");
    assert_eq!(role.description, None);
    assert_eq!(role.scope, "GLOBAL");
    assert_eq!(role.permissions.len(), 0);
}

#[test]
fn test_roles_with_permissions_list_response_empty() {
    // Given
    let response = RolesWithPermissionsListResponse {
        roles: vec![],
        total_count: 0,
        page: 1,
        page_size: 20,
        total_pages: 0,
    };

    // Then
    assert_eq!(response.roles.len(), 0);
    assert_eq!(response.total_count, 0);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 20);
    assert_eq!(response.total_pages, 0);
}