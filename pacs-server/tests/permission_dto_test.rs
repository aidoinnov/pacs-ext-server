use pacs_server::application::dto::permission_dto::{
    RoleWithPermissionsResponse, RolesWithPermissionsListResponse, PaginationQuery, PermissionResponse
};
use serde_json;

#[test]
fn test_role_with_permissions_response_serialization() {
    // Given
    let role = RoleWithPermissionsResponse {
        id: 1,
        name: "시스템 관리자".to_string(),
        description: Some("전체 시스템 관리 권한".to_string()),
        scope: "GLOBAL".to_string(),
        permissions: vec![
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
        ],
    };

    // When
    let json = serde_json::to_string(&role).unwrap();

    // Then
    let expected = r#"{"id":1,"name":"시스템 관리자","description":"전체 시스템 관리 권한","scope":"GLOBAL","permissions":[{"id":1,"resource_type":"user","action":"create"},{"id":2,"resource_type":"user","action":"delete"}]}"#;
    assert_eq!(json, expected);
}

#[test]
fn test_role_with_permissions_response_deserialization() {
    // Given
    let json = r#"{
        "id": 1,
        "name": "시스템 관리자",
        "description": "전체 시스템 관리 권한",
        "scope": "GLOBAL",
        "permissions": [
            {
                "id": 1,
                "resource_type": "user",
                "action": "create"
            },
            {
                "id": 2,
                "resource_type": "user",
                "action": "delete"
            }
        ]
    }"#;

    // When
    let role: RoleWithPermissionsResponse = serde_json::from_str(json).unwrap();

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
fn test_roles_with_permissions_list_response_serialization() {
    // Given
    let response = RolesWithPermissionsListResponse {
        roles: vec![
            RoleWithPermissionsResponse {
                id: 1,
                name: "시스템 관리자".to_string(),
                description: Some("전체 시스템 관리 권한".to_string()),
                scope: "GLOBAL".to_string(),
                permissions: vec![],
            },
        ],
        total_count: 1,
        page: 1,
        page_size: 20,
        total_pages: 1,
    };

    // When
    let json = serde_json::to_string(&response).unwrap();

    // Then
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["total_count"], 1);
    assert_eq!(parsed["page"], 1);
    assert_eq!(parsed["page_size"], 20);
    assert_eq!(parsed["total_pages"], 1);
    assert!(parsed["roles"].is_array());
    assert_eq!(parsed["roles"].as_array().unwrap().len(), 1);
}

#[test]
fn test_pagination_query_deserialization() {
    // Given
    let json = r#"{"page": 2, "page_size": 50}"#;

    // When
    let query: PaginationQuery = serde_json::from_str(json).unwrap();

    // Then
    assert_eq!(query.page, Some(2));
    assert_eq!(query.page_size, Some(50));
}

#[test]
fn test_pagination_query_deserialization_empty() {
    // Given
    let json = r#"{}"#;

    // When
    let query: PaginationQuery = serde_json::from_str(json).unwrap();

    // Then
    assert_eq!(query.page, None);
    assert_eq!(query.page_size, None);
}

#[test]
fn test_permission_response_serialization() {
    // Given
    let permission = PermissionResponse {
        id: 1,
        resource_type: "user".to_string(),
        action: "create".to_string(),
    };

    // When
    let json = serde_json::to_string(&permission).unwrap();

    // Then
    let expected = r#"{"id":1,"resource_type":"user","action":"create"}"#;
    assert_eq!(json, expected);
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

    // When
    let json = serde_json::to_string(&role).unwrap();

    // Then
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["description"], serde_json::Value::Null);
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

    // When
    let json = serde_json::to_string(&response).unwrap();

    // Then
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["roles"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["total_count"], 0);
    assert_eq!(parsed["total_pages"], 0);
}

#[test]
fn test_pagination_query_with_invalid_values() {
    // Given - negative values should be handled gracefully
    let json = r#"{"page": -1, "page_size": -5}"#;

    // When
    let query: PaginationQuery = serde_json::from_str(json).unwrap();

    // Then
    assert_eq!(query.page, Some(-1)); // Raw values, validation happens in use case
    assert_eq!(query.page_size, Some(-5));
}