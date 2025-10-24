use serde_json;
use pacs_server::application::dto::project_user_matrix_dto::*;
use pacs_server::domain::entities::ProjectStatus;

/// ProjectUserMatrixResponse 직렬화/역직렬화 테스트
#[test]
fn test_project_user_matrix_response_serialization() {
    // Given: 테스트용 매트릭스 응답 데이터
    let matrix_response = ProjectUserMatrixResponse {
        matrix: vec![
            ProjectUserMatrixRow {
                project_id: 1,
                project_name: "Test Project 1".to_string(),
                description: Some("Test Description".to_string()),
                status: "IN_PROGRESS".to_string(),
                user_roles: vec![
                    UserRoleCell {
                        user_id: 1,
                        username: "testuser1".to_string(),
                        email: "user1@example.com".to_string(),
                        role_id: Some(1),
                        role_name: Some("Admin".to_string()),
                    },
                    UserRoleCell {
                        user_id: 2,
                        username: "testuser2".to_string(),
                        email: "user2@example.com".to_string(),
                        role_id: None,
                        role_name: None,
                    },
                ],
            },
        ],
        users: vec![
            UserInfo {
                user_id: 1,
                username: "testuser1".to_string(),
                email: "user1@example.com".to_string(),
                full_name: Some("Test User 1".to_string()),
            },
            UserInfo {
                user_id: 2,
                username: "testuser2".to_string(),
                email: "user2@example.com".to_string(),
                full_name: Some("Test User 2".to_string()),
            },
        ],
        pagination: MatrixPagination {
            project_page: 1,
            project_page_size: 10,
            project_total_count: 1,
            project_total_pages: 1,
            user_page: 1,
            user_page_size: 10,
            user_total_count: 2,
            user_total_pages: 1,
        },
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&matrix_response).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    
    // When: JSON에서 역직렬화
    let deserialized: ProjectUserMatrixResponse = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.matrix.len(), matrix_response.matrix.len());
    assert_eq!(deserialized.users.len(), matrix_response.users.len());
    assert_eq!(deserialized.pagination.project_total_count, matrix_response.pagination.project_total_count);
}

/// MatrixQueryParams 직렬화/역직렬화 테스트
#[test]
fn test_matrix_query_params_serialization() {
    // Given: 테스트용 쿼리 파라미터
    let query_params = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_status: Some(vec!["IN_PROGRESS".to_string(), "COMPLETED".to_string()]),
        project_ids: Some(vec![1, 2, 3]),
        user_ids: Some(vec![1, 2]),
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&query_params).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    
    // When: JSON에서 역직렬화
    let deserialized: MatrixQueryParams = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.project_page, query_params.project_page);
    assert_eq!(deserialized.project_page_size, query_params.project_page_size);
    assert_eq!(deserialized.user_page, query_params.user_page);
    assert_eq!(deserialized.user_page_size, query_params.user_page_size);
    assert_eq!(deserialized.project_status, query_params.project_status);
    assert_eq!(deserialized.project_ids, query_params.project_ids);
    assert_eq!(deserialized.user_ids, query_params.user_ids);
}

/// UserRoleCell 직렬화/역직렬화 테스트
#[test]
fn test_user_role_cell_serialization() {
    // Given: 역할이 있는 사용자 셀
    let user_role_with_role = UserRoleCell {
        user_id: 1,
        username: "testuser1".to_string(),
        email: "user1@example.com".to_string(),
        role_id: Some(1),
        role_name: Some("Admin".to_string()),
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&user_role_with_role).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    assert!(json.contains("role_id"));
    assert!(json.contains("role_name"));
    
    // When: JSON에서 역직렬화
    let deserialized: UserRoleCell = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.user_id, user_role_with_role.user_id);
    assert_eq!(deserialized.username, user_role_with_role.username);
    assert_eq!(deserialized.email, user_role_with_role.email);
    assert_eq!(deserialized.role_id, user_role_with_role.role_id);
    assert_eq!(deserialized.role_name, user_role_with_role.role_name);
}

/// 역할이 없는 사용자 셀 테스트
#[test]
fn test_user_role_cell_without_role_serialization() {
    // Given: 역할이 없는 사용자 셀
    let user_role_without_role = UserRoleCell {
        user_id: 2,
        username: "testuser2".to_string(),
        email: "user2@example.com".to_string(),
        role_id: None,
        role_name: None,
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&user_role_without_role).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    assert!(json.contains("null")); // role_id와 role_name이 null
    
    // When: JSON에서 역직렬화
    let deserialized: UserRoleCell = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.user_id, user_role_without_role.user_id);
    assert_eq!(deserialized.username, user_role_without_role.username);
    assert_eq!(deserialized.email, user_role_without_role.email);
    assert_eq!(deserialized.role_id, user_role_without_role.role_id);
    assert_eq!(deserialized.role_name, user_role_without_role.role_name);
}

/// ProjectUserMatrixRow 직렬화/역직렬화 테스트
#[test]
fn test_project_user_matrix_row_serialization() {
    // Given: 테스트용 프로젝트 매트릭스 행
    let matrix_row = ProjectUserMatrixRow {
        project_id: 1,
        project_name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        status: "IN_PROGRESS".to_string(),
        user_roles: vec![
            UserRoleCell {
                user_id: 1,
                username: "testuser1".to_string(),
                email: "user1@example.com".to_string(),
                role_id: Some(1),
                role_name: Some("Admin".to_string()),
            },
        ],
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&matrix_row).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    assert!(json.contains("project_id"));
    assert!(json.contains("project_name"));
    assert!(json.contains("status"));
    assert!(json.contains("user_roles"));
    
    // When: JSON에서 역직렬화
    let deserialized: ProjectUserMatrixRow = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.project_id, matrix_row.project_id);
    assert_eq!(deserialized.project_name, matrix_row.project_name);
    assert_eq!(deserialized.description, matrix_row.description);
    assert_eq!(deserialized.status, matrix_row.status);
    assert_eq!(deserialized.user_roles.len(), matrix_row.user_roles.len());
}

/// UserInfo 직렬화/역직렬화 테스트
#[test]
fn test_user_info_serialization() {
    // Given: 테스트용 사용자 정보
    let user_info = UserInfo {
        user_id: 1,
        username: "testuser1".to_string(),
        email: "user1@example.com".to_string(),
        full_name: Some("Test User 1".to_string()),
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&user_info).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    assert!(json.contains("user_id"));
    assert!(json.contains("username"));
    assert!(json.contains("email"));
    assert!(json.contains("full_name"));
    
    // When: JSON에서 역직렬화
    let deserialized: UserInfo = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.user_id, user_info.user_id);
    assert_eq!(deserialized.username, user_info.username);
    assert_eq!(deserialized.email, user_info.email);
    assert_eq!(deserialized.full_name, user_info.full_name);
}

/// MatrixPagination 직렬화/역직렬화 테스트
#[test]
fn test_matrix_pagination_serialization() {
    // Given: 테스트용 페이지네이션 정보
    let pagination = MatrixPagination {
        project_page: 1,
        project_page_size: 10,
        project_total_count: 25,
        project_total_pages: 3,
        user_page: 1,
        user_page_size: 10,
        user_total_count: 15,
        user_total_pages: 2,
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&pagination).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    assert!(json.contains("project_page"));
    assert!(json.contains("project_page_size"));
    assert!(json.contains("project_total_count"));
    assert!(json.contains("project_total_pages"));
    assert!(json.contains("user_page"));
    assert!(json.contains("user_page_size"));
    assert!(json.contains("user_total_count"));
    assert!(json.contains("user_total_pages"));
    
    // When: JSON에서 역직렬화
    let deserialized: MatrixPagination = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.project_page, pagination.project_page);
    assert_eq!(deserialized.project_page_size, pagination.project_page_size);
    assert_eq!(deserialized.project_total_count, pagination.project_total_count);
    assert_eq!(deserialized.project_total_pages, pagination.project_total_pages);
    assert_eq!(deserialized.user_page, pagination.user_page);
    assert_eq!(deserialized.user_page_size, pagination.user_page_size);
    assert_eq!(deserialized.user_total_count, pagination.user_total_count);
    assert_eq!(deserialized.user_total_pages, pagination.user_total_pages);
}

/// 빈 데이터 직렬화 테스트
#[test]
fn test_empty_matrix_serialization() {
    // Given: 빈 매트릭스 응답
    let empty_matrix = ProjectUserMatrixResponse {
        matrix: vec![],
        users: vec![],
        pagination: MatrixPagination {
            project_page: 1,
            project_page_size: 10,
            project_total_count: 0,
            project_total_pages: 0,
            user_page: 1,
            user_page_size: 10,
            user_total_count: 0,
            user_total_pages: 0,
        },
    };
    
    // When: JSON으로 직렬화
    let json = serde_json::to_string(&empty_matrix).unwrap();
    
    // Then: 직렬화가 성공해야 함
    assert!(!json.is_empty());
    assert!(json.contains("matrix"));
    assert!(json.contains("users"));
    assert!(json.contains("pagination"));
    
    // When: JSON에서 역직렬화
    let deserialized: ProjectUserMatrixResponse = serde_json::from_str(&json).unwrap();
    
    // Then: 원본과 동일해야 함
    assert_eq!(deserialized.matrix.len(), 0);
    assert_eq!(deserialized.users.len(), 0);
    assert_eq!(deserialized.pagination.project_total_count, 0);
    assert_eq!(deserialized.pagination.user_total_count, 0);
}

/// JSON 파싱 에러 처리 테스트
#[test]
fn test_invalid_json_parsing() {
    // Given: 잘못된 JSON
    let invalid_json = r#"{"invalid": "json"}"#;
    
    // When: 역직렬화 시도
    let result: Result<ProjectUserMatrixResponse, _> = serde_json::from_str(invalid_json);
    
    // Then: 에러가 발생해야 함
    assert!(result.is_err());
}

/// 부분적 JSON 파싱 테스트
#[test]
fn test_partial_json_parsing() {
    // Given: 일부 필드만 있는 JSON
    let partial_json = r#"{
        "matrix": [],
        "users": [],
        "pagination": {
            "project_page": 1,
            "project_page_size": 10,
            "project_total_count": 0,
            "project_total_pages": 0,
            "user_page": 1,
            "user_page_size": 10,
            "user_total_count": 0,
            "user_total_pages": 0
        }
    }"#;
    
    // When: 역직렬화
    let result: Result<ProjectUserMatrixResponse, _> = serde_json::from_str(partial_json);
    
    // Then: 성공해야 함 (기본값이 적용됨)
    assert!(result.is_ok());
    
    let matrix_response = result.unwrap();
    assert_eq!(matrix_response.matrix.len(), 0);
    assert_eq!(matrix_response.users.len(), 0);
}
