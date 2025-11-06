use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

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

/// Capability 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CapabilityInfo {
    /// Capability ID
    pub id: i32,
    /// 내부 이름 (예: MANAGE_USERS)
    pub name: String,
    /// UI 표시 이름 (예: "사용자 관리")
    pub display_name: String,
    /// UI 표시용 짧은 레이블 (예: "Admin", "User")
    pub display_label: String,
    /// 설명
    pub description: Option<String>,
    /// 카테고리
    pub category: String,
    /// UI 카테고리 짧은 레이블 (예: "MANAGE", "PROJECT")
    pub category_label: String,
    /// 매핑된 Permission 개수
    pub permission_count: i32,
}

/// 역할-Capability 할당 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RoleCapabilityAssignment {
    /// 역할 ID
    pub role_id: i32,
    /// Capability ID
    pub capability_id: i32,
    /// 할당 여부
    pub assigned: bool,
}

/// Capability 할당 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CapabilityAssignmentRequest {
    /// 할당 여부
    pub assign: bool,
}

/// 역할-Capability 매트릭스 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RoleCapabilityMatrixResponse {
    /// 역할 목록
    pub roles: Vec<RoleInfo>,
    /// 카테고리별 Capability 목록
    pub capabilities_by_category: HashMap<String, Vec<CapabilityInfo>>,
    /// 할당 정보
    pub assignments: Vec<RoleCapabilityAssignment>,
    /// 페이지네이션 정보
    pub pagination: PaginationInfo,
}

/// 페이지네이션 정보
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct PaginationInfo {
    /// 현재 페이지 (1부터 시작)
    pub current_page: i32,
    /// 페이지 크기
    pub page_size: i32,
    /// 총 페이지 수
    pub total_pages: i32,
    /// 총 항목 수
    pub total_items: i64,
    /// 다음 페이지 존재 여부
    pub has_next: bool,
    /// 이전 페이지 존재 여부
    pub has_previous: bool,
}

/// 역할-Capability 매트릭스 쿼리 파라미터
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RoleCapabilityMatrixQuery {
    /// 페이지 번호 (기본값: 1)
    pub page: Option<i32>,
    /// 페이지 크기 (기본값: 10, 최대: 100)
    pub size: Option<i32>,
    /// 역할 이름 검색
    pub search: Option<String>,
    /// 역할 범위 필터 (GLOBAL, PROJECT)
    pub scope: Option<String>,
}

/// Capability 상세 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CapabilityDetailResponse {
    /// Capability 정보
    pub capability: CapabilityInfo,
    /// 매핑된 Permission 목록
    pub permissions: Vec<PermissionInfo>,
}

/// 역할-Capability 할당 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateRoleCapabilityRequest {
    /// 할당 여부
    pub assign: bool,
}

/// Capability 생성 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateCapabilityRequest {
    /// 내부 이름
    pub name: String,
    /// UI 표시 이름
    pub display_name: String,
    /// 설명
    pub description: Option<String>,
    /// 카테고리
    pub category: String,
}

/// Capability 수정 요청
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateCapabilityRequest {
    /// UI 표시 이름
    pub display_name: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 카테고리
    pub category: Option<String>,
    /// 활성화 여부
    pub is_active: Option<bool>,
}

/// Capability 목록 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CapabilityListResponse {
    /// Capability 목록
    pub capabilities: Vec<CapabilityInfo>,
    /// 총 개수
    pub total: i64,
}

/// Permission 정보 (기존 role_permission_matrix_dto에서 재사용)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_info_serialization() {
        let capability_info = CapabilityInfo {
            id: 1,
            name: "MANAGE_USERS".to_string(),
            display_name: "사용자 관리".to_string(),
            display_label: "Users".to_string(),
            description: Some("사용자 생성, 수정, 삭제".to_string()),
            category: "관리".to_string(),
            category_label: "MANAGE".to_string(),
            permission_count: 5,
        };

        let json = serde_json::to_string(&capability_info).unwrap();
        let deserialized: CapabilityInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(capability_info, deserialized);
    }

    #[test]
    fn test_role_capability_assignment_serialization() {
        let assignment = RoleCapabilityAssignment {
            role_id: 1,
            capability_id: 2,
            assigned: true,
        };

        let json = serde_json::to_string(&assignment).unwrap();
        let deserialized: RoleCapabilityAssignment = serde_json::from_str(&json).unwrap();

        assert_eq!(assignment, deserialized);
    }

    #[test]
    fn test_role_capability_matrix_response_serialization() {
        let roles = vec![RoleInfo {
            id: 1,
            name: "SUPER_ADMIN".to_string(),
            description: Some("시스템 전체 관리자".to_string()),
            scope: "GLOBAL".to_string(),
        }];

        let mut capabilities_by_category = HashMap::new();
        capabilities_by_category.insert(
            "관리".to_string(),
            vec![CapabilityInfo {
                id: 1,
                name: "MANAGE_USERS".to_string(),
                display_name: "사용자 관리".to_string(),
                display_label: "Users".to_string(),
                description: Some("사용자 생성, 수정, 삭제".to_string()),
                category: "관리".to_string(),
                category_label: "MANAGE".to_string(),
                permission_count: 5,
            }],
        );

        let assignments = vec![RoleCapabilityAssignment {
            role_id: 1,
            capability_id: 1,
            assigned: true,
        }];

        let pagination = PaginationInfo {
            current_page: 1,
            page_size: 1,
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_previous: false,
        };

        let response = RoleCapabilityMatrixResponse {
            roles,
            capabilities_by_category,
            assignments,
            pagination,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RoleCapabilityMatrixResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }
}
