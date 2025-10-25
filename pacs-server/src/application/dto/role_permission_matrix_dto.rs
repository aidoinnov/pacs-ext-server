use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_role_info_serialization() {
        let role_info = RoleInfo {
            id: 1,
            name: "Admin".to_string(),
            description: Some("Administrator role".to_string()),
            scope: "GLOBAL".to_string(),
        };

        let json = serde_json::to_string(&role_info).unwrap();
        let deserialized: RoleInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(role_info, deserialized);
    }

    #[test]
    fn test_role_info_deserialization() {
        let json = r#"{
            "id": 1,
            "name": "Admin",
            "description": "Administrator role",
            "scope": "GLOBAL"
        }"#;

        let role_info: RoleInfo = serde_json::from_str(json).unwrap();
        
        assert_eq!(role_info.id, 1);
        assert_eq!(role_info.name, "Admin");
        assert_eq!(role_info.description, Some("Administrator role".to_string()));
        assert_eq!(role_info.scope, "GLOBAL");
    }

    #[test]
    fn test_role_info_without_description() {
        let role_info = RoleInfo {
            id: 2,
            name: "User".to_string(),
            description: None,
            scope: "PROJECT".to_string(),
        };

        let json = serde_json::to_string(&role_info).unwrap();
        let deserialized: RoleInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(role_info, deserialized);
        assert_eq!(deserialized.description, None);
    }

    #[test]
    fn test_permission_info_serialization() {
        let permission_info = PermissionInfo {
            id: 1,
            category: "사용자 및 권한 관리".to_string(),
            resource_type: "USER".to_string(),
            action: "READ".to_string(),
        };

        let json = serde_json::to_string(&permission_info).unwrap();
        let deserialized: PermissionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(permission_info, deserialized);
    }

    #[test]
    fn test_permission_info_deserialization() {
        let json = r#"{
            "id": 1,
            "category": "사용자 및 권한 관리",
            "resource_type": "USER",
            "action": "READ"
        }"#;

        let permission_info: PermissionInfo = serde_json::from_str(json).unwrap();
        
        assert_eq!(permission_info.id, 1);
        assert_eq!(permission_info.category, "사용자 및 권한 관리");
        assert_eq!(permission_info.resource_type, "USER");
        assert_eq!(permission_info.action, "READ");
    }

    #[test]
    fn test_role_permission_assignment_serialization() {
        let assignment = RolePermissionAssignment {
            role_id: 1,
            permission_id: 2,
            assigned: true,
        };

        let json = serde_json::to_string(&assignment).unwrap();
        let deserialized: RolePermissionAssignment = serde_json::from_str(&json).unwrap();

        assert_eq!(assignment, deserialized);
    }

    #[test]
    fn test_role_permission_assignment_deserialization() {
        let json = r#"{
            "role_id": 1,
            "permission_id": 2,
            "assigned": true
        }"#;

        let assignment: RolePermissionAssignment = serde_json::from_str(json).unwrap();
        
        assert_eq!(assignment.role_id, 1);
        assert_eq!(assignment.permission_id, 2);
        assert_eq!(assignment.assigned, true);
    }

    #[test]
    fn test_assign_permission_request_serialization() {
        let request = AssignPermissionRequest {
            assign: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AssignPermissionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_assign_permission_request_deserialization() {
        let json = r#"{
            "assign": false
        }"#;

        let request: AssignPermissionRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.assign, false);
    }

    #[test]
    fn test_assign_permission_response_serialization() {
        let response = AssignPermissionResponse {
            success: true,
            message: "Permission assigned successfully".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AssignPermissionResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_assign_permission_response_deserialization() {
        let json = r#"{
            "success": false,
            "message": "Permission not found"
        }"#;

        let response: AssignPermissionResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.success, false);
        assert_eq!(response.message, "Permission not found");
    }

    #[test]
    fn test_role_permission_matrix_response_serialization() {
        let roles = vec![
            RoleInfo {
                id: 1,
                name: "Admin".to_string(),
                description: Some("Administrator role".to_string()),
                scope: "GLOBAL".to_string(),
            },
            RoleInfo {
                id: 2,
                name: "User".to_string(),
                description: None,
                scope: "GLOBAL".to_string(),
            },
        ];

        let mut permissions_by_category = HashMap::new();
        permissions_by_category.insert("사용자 및 권한 관리".to_string(), vec![
            PermissionInfo {
                id: 1,
                category: "사용자 및 권한 관리".to_string(),
                resource_type: "USER".to_string(),
                action: "READ".to_string(),
            },
            PermissionInfo {
                id: 2,
                category: "사용자 및 권한 관리".to_string(),
                resource_type: "USER".to_string(),
                action: "WRITE".to_string(),
            },
        ]);
        permissions_by_category.insert("프로젝트 관리".to_string(), vec![
            PermissionInfo {
                id: 3,
                category: "프로젝트 관리".to_string(),
                resource_type: "PROJECT".to_string(),
                action: "READ".to_string(),
            },
        ]);

        let assignments = vec![
            RolePermissionAssignment {
                role_id: 1,
                permission_id: 1,
                assigned: true,
            },
            RolePermissionAssignment {
                role_id: 1,
                permission_id: 2,
                assigned: true,
            },
            RolePermissionAssignment {
                role_id: 1,
                permission_id: 3,
                assigned: true,
            },
            RolePermissionAssignment {
                role_id: 2,
                permission_id: 1,
                assigned: true,
            },
            RolePermissionAssignment {
                role_id: 2,
                permission_id: 2,
                assigned: false,
            },
            RolePermissionAssignment {
                role_id: 2,
                permission_id: 3,
                assigned: false,
            },
        ];

        let matrix = RolePermissionMatrixResponse {
            roles,
            permissions_by_category,
            assignments,
        };

        let json = serde_json::to_string(&matrix).unwrap();
        let deserialized: RolePermissionMatrixResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(matrix.roles.len(), deserialized.roles.len());
        assert_eq!(matrix.permissions_by_category.len(), deserialized.permissions_by_category.len());
        assert_eq!(matrix.assignments.len(), deserialized.assignments.len());
    }

    #[test]
    fn test_empty_matrix_response() {
        let matrix = RolePermissionMatrixResponse {
            roles: vec![],
            permissions_by_category: HashMap::new(),
            assignments: vec![],
        };

        let json = serde_json::to_string(&matrix).unwrap();
        let deserialized: RolePermissionMatrixResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(matrix.roles.len(), 0);
        assert_eq!(matrix.permissions_by_category.len(), 0);
        assert_eq!(matrix.assignments.len(), 0);
        assert_eq!(deserialized.roles.len(), 0);
        assert_eq!(deserialized.permissions_by_category.len(), 0);
        assert_eq!(deserialized.assignments.len(), 0);
    }
}

/// 역할 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RoleInfo {
    /// 역할 ID
    pub id: i32,
    /// 역할 이름
    pub name: String,
    /// 역할 설명
    pub description: Option<String>,
    /// 역할 범위 (GLOBAL, PROJECT)
    pub scope: String,
}

/// 권한 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct PermissionInfo {
    /// 권한 ID
    pub id: i32,
    /// 권한 카테고리
    pub category: String,
    /// 리소스 타입
    pub resource_type: String,
    /// 액션
    pub action: String,
}

/// 역할-권한 할당 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RolePermissionAssignment {
    /// 역할 ID
    pub role_id: i32,
    /// 권한 ID
    pub permission_id: i32,
    /// 할당 여부
    pub assigned: bool,
}

/// 역할-권한 매트릭스 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RolePermissionMatrixResponse {
    /// 역할 목록
    pub roles: Vec<RoleInfo>,
    /// 카테고리별 권한 목록
    pub permissions_by_category: HashMap<String, Vec<PermissionInfo>>,
    /// 역할-권한 할당 정보
    pub assignments: Vec<RolePermissionAssignment>,
}

/// 권한 할당/제거 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AssignPermissionRequest {
    /// 할당 여부 (true: 할당, false: 제거)
    pub assign: bool,
}

/// 권한 할당/제거 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AssignPermissionResponse {
    /// 성공 여부
    pub success: bool,
    /// 메시지
    pub message: String,
}
