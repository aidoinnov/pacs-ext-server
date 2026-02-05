//! Access Control DTO 단위 테스트 (add-job: MePermissionsResponse, MeCapabilitiesResponse)

use pacs_server::application::dto::access_control_dto::{MeCapabilitiesResponse, MePermissionsResponse};
use serde_json::json;

#[test]
fn test_me_permissions_response_serialization() {
    let resp = MePermissionsResponse {
        permissions: vec![
            "project_data.assign".to_string(),
            "ROLE.READ".to_string(),
            "PROJECT.READ".to_string(),
        ],
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("project_data.assign"));
    assert!(json.contains("ROLE.READ"));
    assert!(json.contains("permissions"));
}

#[test]
fn test_me_permissions_response_deserialization() {
    let json = json!({
        "permissions": ["project_data.assign", "PROJECT.READ"]
    });
    let resp: MePermissionsResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.permissions.len(), 2);
    assert_eq!(resp.permissions[0], "project_data.assign");
    assert_eq!(resp.permissions[1], "PROJECT.READ");
}

#[test]
fn test_me_permissions_response_empty() {
    let resp = MePermissionsResponse {
        permissions: vec![],
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert_eq!(json, r#"{"permissions":[]}"#);
}

#[test]
fn test_me_capabilities_response_serialization() {
    let resp = MeCapabilitiesResponse {
        capability_codes: vec![
            "PROJECT_DATA_ASSIGN".to_string(),
            "ROLE_MANAGEMENT".to_string(),
        ],
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("PROJECT_DATA_ASSIGN"));
    assert!(json.contains("ROLE_MANAGEMENT"));
    assert!(json.contains("capability_codes"));
}

#[test]
fn test_me_capabilities_response_deserialization() {
    let json = json!({
        "capability_codes": ["PROJECT_DATA_ASSIGN", "PROJECT_VIEW"]
    });
    let resp: MeCapabilitiesResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.capability_codes.len(), 2);
    assert_eq!(resp.capability_codes[0], "PROJECT_DATA_ASSIGN");
    assert_eq!(resp.capability_codes[1], "PROJECT_VIEW");
}

#[test]
fn test_me_capabilities_response_empty() {
    let resp = MeCapabilitiesResponse {
        capability_codes: vec![],
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert_eq!(json, r#"{"capability_codes":[]}"#);
}
