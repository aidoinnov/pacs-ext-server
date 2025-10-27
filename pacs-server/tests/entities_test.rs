use chrono::{DateTime, Utc, TimeZone};
use pacs_server::domain::entities::*;
use pacs_server::domain::entities::access_condition::ResourceLevel;
use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod user_tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: 1,
            keycloak_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            full_name: None,
            organization: None,
            department: None,
            phone: None,
            created_at: Utc::now(),
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
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_new_user_creation() {
        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
            full_name: None,
            organization: None,
            department: None,
            phone: None,
        };

        assert_eq!(new_user.username, "newuser");
        assert_eq!(new_user.email, "new@example.com");
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: 1,
            keycloak_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            full_name: None,
            organization: None,
            department: None,
            phone: None,
            created_at: Utc::now(),
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
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("test@example.com"));
    }
}

#[cfg(test)]
mod project_tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project {
            id: 1,
            name: "Test Project".to_string(),
            description: Some("Test Description".to_string()),
            is_active: true,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(project.id, 1);
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("Test Description".to_string()));
        assert!(project.is_active);
    }

    #[test]
    fn test_new_project_without_description() {
        let new_project = NewProject {
            name: "New Project".to_string(),
            description: None,
            measurement_values: None,
        };

        assert_eq!(new_project.name, "New Project");
        assert!(new_project.description.is_none());
    }
}

#[cfg(test)]
mod role_tests {
    use super::*;

    #[test]
    fn test_role_scope_enum() {
        let global_scope = RoleScope::Global;
        let project_scope = RoleScope::Project;

        // Enum variants should be different
        assert_ne!(
            format!("{:?}", global_scope),
            format!("{:?}", project_scope)
        );
    }

    #[test]
    fn test_role_creation() {
        let role = Role {
            id: 1,
            name: "Admin".to_string(),
            description: Some("Administrator role".to_string()),
            scope: "GLOBAL".to_string(),
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(role.name, "Admin");
        assert_eq!(role.scope, "GLOBAL");
    }
}

#[cfg(test)]
mod access_condition_tests {
    use super::*;

    #[test]
    fn test_condition_type_enum() {
        let allow = ConditionType::Allow;
        let deny = ConditionType::Deny;
        let limit = ConditionType::Limit;

        assert_eq!(allow, ConditionType::Allow);
        assert_eq!(deny, ConditionType::Deny);
        assert_eq!(limit, ConditionType::Limit);
    }

    #[test]
    fn test_resource_level_enum() {
        let study = ResourceLevel::Study;
        let series = ResourceLevel::Series;
        let instance = ResourceLevel::Instance;

        assert_eq!(study, ResourceLevel::Study);
        assert_eq!(series, ResourceLevel::Series);
        assert_eq!(instance, ResourceLevel::Instance);
    }

    #[test]
    fn test_access_condition_creation() {
        let condition = AccessCondition {
            id: 1,
            resource_type: "DICOM".to_string(),
            resource_level: ResourceLevel::Study,
            dicom_tag: Some("(0010,0020)".to_string()),
            operator: "EQUALS".to_string(),
            value: Some("12345".to_string()),
            condition_type: ConditionType::Allow,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(condition.resource_type, "DICOM");
        assert_eq!(condition.resource_level, ResourceLevel::Study);
        assert_eq!(condition.condition_type, ConditionType::Allow);
    }
}

#[cfg(test)]
mod logs_tests {
    use super::*;

    #[test]
    fn test_grant_action_enum() {
        let grant = GrantAction::Grant;
        let revoke = GrantAction::Revoke;

        assert_eq!(grant, GrantAction::Grant);
        assert_eq!(revoke, GrantAction::Revoke);
    }

    #[test]
    fn test_access_log_creation() {
        let log = AccessLog {
            id: 1,
            user_id: 1,
            project_id: Some(1),
            resource_type: "STUDY".to_string(),
            study_uid: Some("1.2.3.4.5".to_string()),
            series_uid: None,
            instance_uid: None,
            action: "VIEW".to_string(),
            result: "SUCCESS".to_string(),
            dicom_tag_check: None,
            ae_title: Some("PACS_SERVER".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            session_id: Some("session123".to_string()),
            via_group_id: None,
            logged_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(log.user_id, 1);
        assert_eq!(log.action, "VIEW");
        assert_eq!(log.result, "SUCCESS");
    }

    #[test]
    fn test_new_access_log_creation() {
        let new_log = NewAccessLog {
            user_id: 1,
            project_id: Some(1),
            resource_type: "SERIES".to_string(),
            study_uid: Some("1.2.3.4.5".to_string()),
            series_uid: Some("1.2.3.4.5.6".to_string()),
            instance_uid: None,
            action: "DOWNLOAD".to_string(),
            result: "SUCCESS".to_string(),
            dicom_tag_check: None,
            ae_title: None,
            ip_address: Some("10.0.0.1".to_string()),
            session_id: None,
            via_group_id: None,
            measurement_values: None,
        };

        assert_eq!(new_log.resource_type, "SERIES");
        assert_eq!(new_log.action, "DOWNLOAD");
    }
}

#[cfg(test)]
mod annotation_tests {
    use super::*;

    #[test]
    fn test_annotation_with_jsonb() {
        let annotation = Annotation {
            id: 1,
            project_id: 1,
            user_id: 1,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.5.6".to_string()),
            instance_uid: Some("1.2.3.4.5.6.7".to_string()),
            tool_name: "Arrow".to_string(),
            tool_version: Some("1.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
            data: json!({
                "start": {"x": 100, "y": 200},
                "end": {"x": 300, "y": 400},
                "color": "red"
            }),
            description: Some("Test annotation".to_string()),
            is_shared: true,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            updated_at: Utc::timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(annotation.tool_name, "Arrow");
        assert!(annotation.is_shared);
        assert_eq!(annotation.data["color"], "red");
    }

    #[test]
    fn test_new_annotation_creation() {
        let new_annotation = NewAnnotation {
            project_id: 1,
            user_id: 1,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: None,
            instance_uid: None,
            tool_name: "ROI".to_string(),
            tool_version: Some("2.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
            data: json!({
                "points": [[10, 20], [30, 40], [50, 60]]
            }),
            description: Some("Test annotation".to_string()),
            is_shared: false,
            measurement_values: None,
        };

        assert_eq!(new_annotation.tool_name, "ROI");
        assert!(!new_annotation.is_shared);
    }
}

#[cfg(test)]
mod relations_tests {
    use super::*;

    #[test]
    fn test_user_project_relation() {
        let user_project = UserProject {
            id: 1,
            user_id: 10,
            project_id: 20,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(user_project.user_id, 10);
        assert_eq!(user_project.project_id, 20);
    }

    #[test]
    fn test_project_role_relation() {
        let project_role = ProjectRole {
            id: 1,
            project_id: 5,
            role_id: 3,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(project_role.project_id, 5);
        assert_eq!(project_role.role_id, 3);
    }

    #[test]
    fn test_role_permission_with_scope() {
        let role_permission = RolePermission {
            id: 1,
            role_id: 1,
            permission_id: 2,
            scope: Some("PROJECT".to_string()),
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(role_permission.scope, Some("PROJECT".to_string()));
    }

    #[test]
    fn test_project_permission_with_inheritance() {
        let project_permission = ProjectPermission {
            id: 1,
            project_id: 1,
            permission_id: 1,
            scope: None,
            inherits_from_role_permission: true,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert!(project_permission.inherits_from_role_permission);
    }
}

#[cfg(test)]
mod viewer_tests {
    use super::*;

    #[test]
    fn test_hanging_protocol() {
        let protocol = HangingProtocol {
            id: 1,
            project_id: 1,
            owner_user_id: 1,
            name: "Chest CT Protocol".to_string(),
            is_default: true,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(protocol.name, "Chest CT Protocol");
        assert!(protocol.is_default);
    }

    #[test]
    fn test_hp_layout() {
        let layout = HpLayout {
            id: 1,
            protocol_id: 1,
            rows: 2,
            cols: 3,
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(layout.rows, 2);
        assert_eq!(layout.cols, 3);
    }

    #[test]
    fn test_hp_viewport() {
        let viewport = HpViewport {
            id: 1,
            layout_id: 1,
            position_row: 0,
            position_col: 1,
            selection_rule: Some("FIRST_SERIES".to_string()),
            sort_order: Some("ASC".to_string()),
            created_at: Utc.timestamp_opt(1234567890, 0).unwrap(),
            measurement_values: None,
        };

        assert_eq!(viewport.position_row, 0);
        assert_eq!(viewport.position_col, 1);
        assert_eq!(viewport.selection_rule, Some("FIRST_SERIES".to_string()));
    }
}
