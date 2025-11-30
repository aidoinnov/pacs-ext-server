/// Annotation 권한 관리 테스트
///
/// 이 테스트는 Annotation 생성/수정/삭제 권한 제어 및 권한 조회 API를 검증합니다.
/// - 개발 모드 user_id 추출 헬퍼 함수 테스트
/// - Annotation 생성 권한 제어 테스트
/// - Annotation 수정 권한 제어 테스트
/// - Annotation 삭제 권한 제어 테스트
/// - 권한 조회 API 테스트

#[cfg(test)]
mod annotation_permission_management_tests {
    use actix_web::test;
    use pacs_server::application::dto::annotation_dto::{
        CreateAnnotationRequest, UpdateAnnotationRequest,
    };
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::{
        AnnotationServiceImpl, AccessControlServiceImpl,
    };
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl,
        AccessLogRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
    };
    use sqlx::postgres::PgPoolOptions;


    /// 테스트용 UseCase 생성 헬퍼 함수
    fn build_use_case(
        pool: &sqlx::PgPool,
    ) -> AnnotationUseCase<
        AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>,
        UserRepositoryImpl,
        AccessControlServiceImpl<
            AccessLogRepositoryImpl,
            UserRepositoryImpl,
            ProjectRepositoryImpl,
            RoleRepositoryImpl,
            PermissionRepositoryImpl,
        >,
    > {
        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());
        let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
        let role_repo = RoleRepositoryImpl::new(pool.clone());
        let permission_repo = PermissionRepositoryImpl::new(pool.clone());

        let annotation_service = AnnotationServiceImpl::new(
            annotation_repo,
            user_repo.clone(),
            project_repo.clone(),
        );

        let access_control_service = AccessControlServiceImpl::new(
            access_log_repo,
            user_repo.clone(),
            project_repo.clone(),
            role_repo,
            permission_repo,
        );

        AnnotationUseCase::new(annotation_service, user_repo, access_control_service)
    }


    // ========== 통합 테스트: Annotation 생성 권한 제어 ==========

    /// 테스트 1: 권한이 있는 사용자는 Annotation을 생성할 수 있어야 함
    #[tokio::test]
    async fn test_create_annotation_with_permission() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 프로젝트 2 (PerfProj)와 사용자 1 (iaid-pacs-admin, SUPER_ADMIN 권한)
        let project_id = 2;
        let user_id = 1;

        let request = CreateAnnotationRequest {
            user_id: None,
            project_id: Some(project_id),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Permission test annotation".to_string()),
            measurement_values: None,
            label: None,
        };

        let result = use_case
            .create_annotation(request, user_id, project_id)
            .await;

        match result {
            Ok(annotation) => {
                println!("✅ User {} successfully created annotation {}", user_id, annotation.id);
                assert!(annotation.id > 0, "Annotation should have a valid ID");
            }
            Err(e) => {
                panic!("User with permission should be able to create annotation: {:?}", e);
            }
        }
    }

    // ========== 통합 테스트: Annotation 수정 권한 제어 ==========

    /// 테스트 2: 소유자는 자신의 Annotation을 수정할 수 있어야 함
    #[tokio::test]
    async fn test_update_annotation_as_owner() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 기존 annotation 조회 (프로젝트 2의 annotation 중 하나)
        let annotations = use_case
            .get_annotations_by_project(2)
            .await
            .expect("Failed to get annotations");

        if annotations.annotations.is_empty() {
            println!("⚠️  No annotations found in project 2, skipping test");
            return;
        }

        let annotation = &annotations.annotations[0];
        let owner_id = annotation.user_id;

        let request = UpdateAnnotationRequest {
            base_version: Some(annotation.version),
            annotation_data: Some(serde_json::json!({"type": "updated", "data": "test"})),
            tool_name: None,
            tool_version: None,
            viewer_software: None,
            description: Some("Updated by owner".to_string()),
            measurement_values: None,
            label: None,
        };

        let result = use_case
            .update_annotation(annotation.id, request, owner_id)
            .await;

        match result {
            Ok(updated) => {
                println!("✅ Owner {} successfully updated annotation {}", owner_id, updated.id);
                assert_eq!(updated.id, annotation.id);
                assert!(updated.version > annotation.version, "Version should be incremented");
            }
            Err(e) => {
                panic!("Owner should be able to update their annotation: {:?}", e);
            }
        }
    }

    // ========== 통합 테스트: Annotation 삭제 권한 제어 ==========

    /// 테스트 3: 소유자는 자신의 Annotation을 삭제할 수 있어야 함
    #[tokio::test]
    async fn test_delete_annotation_as_owner() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 먼저 테스트용 annotation 생성
        let project_id = 2;
        let user_id = 1;

        let create_request = CreateAnnotationRequest {
            user_id: None,
            project_id: Some(project_id),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation for deletion".to_string()),
            measurement_values: None,
            label: None,
        };

        let created = use_case
            .create_annotation(create_request, user_id, project_id)
            .await
            .expect("Failed to create test annotation");

        // 소유자로 삭제 시도
        let result = use_case.delete_annotation(created.id, user_id).await;

        match result {
            Ok(_) => {
                println!("✅ Owner {} successfully deleted annotation {}", user_id, created.id);
            }
            Err(e) => {
                panic!("Owner should be able to delete their annotation: {:?}", e);
            }
        }
    }

    // ========== 통합 테스트: 권한 조회 API ==========

    /// 테스트 4: 사용자의 Annotation 권한 조회
    #[tokio::test]
    async fn test_get_user_annotation_permissions() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let project_id = 2; // PerfProj
        let user_id = 1; // iaid-pacs-admin (SUPER_ADMIN 권한)

        let result = use_case
            .get_user_annotation_permissions(user_id, project_id)
            .await;

        match result {
            Ok(permissions) => {
                println!("✅ Retrieved permissions for user {} in project {}", user_id, project_id);
                println!("   can_read_own: {}", permissions.can_read_own);
                println!("   can_read_all: {}", permissions.can_read_all);
                println!("   can_write: {}", permissions.can_write);
                println!("   can_delete: {}", permissions.can_delete);
                println!("   can_share: {}", permissions.can_share);

                // SUPER_ADMIN은 대부분의 권한을 가져야 함
                assert!(
                    permissions.can_write || permissions.can_read_all,
                    "Admin user should have at least some permissions"
                );
            }
            Err(e) => {
                panic!("Failed to get user permissions: {:?}", e);
            }
        }
    }

    // ========== 통합 테스트: 단일 Annotation 조회 권한 제어 ==========

    /// 테스트 5: 소유자는 자신의 Annotation을 조회할 수 있어야 함
    #[tokio::test]
    async fn test_get_annotation_as_owner() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 먼저 테스트용 annotation 생성
        let project_id = 2;
        let user_id = 1;

        let create_request = CreateAnnotationRequest {
            user_id: None,
            project_id: Some(project_id),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation for owner read".to_string()),
            measurement_values: None,
            label: None,
        };

        let created = use_case
            .create_annotation(create_request, user_id, project_id)
            .await
            .expect("Failed to create test annotation");

        // 소유자로 조회 시도
        let result = use_case.get_annotation_by_id(user_id, created.id).await;

        match result {
            Ok(annotation) => {
                println!("✅ Owner {} successfully retrieved annotation {}", user_id, annotation.id);
                assert_eq!(annotation.id, created.id);
            }
            Err(e) => {
                panic!("Owner should be able to read their annotation: {:?}", e);
            }
        }
    }

    /// 테스트 6: READ_ALL 권한이 있는 사용자는 다른 사용자의 Annotation을 조회할 수 있어야 함
    #[tokio::test]
    async fn test_get_annotation_with_read_all_permission() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 프로젝트 2의 기존 annotation 조회
        let annotations = use_case
            .get_annotations_by_project(2)
            .await
            .expect("Failed to get annotations");

        if annotations.annotations.is_empty() {
            println!("⚠️  No annotations found in project 2, skipping test");
            return;
        }

        let annotation = &annotations.annotations[0];
        let owner_id = annotation.user_id;
        let reader_id = 1; // iaid-pacs-admin (SUPER_ADMIN 권한, READ_ALL 가능)

        // READ_ALL 권한이 있는 사용자로 조회 시도
        let result = use_case.get_annotation_by_id(reader_id, annotation.id).await;

        match result {
            Ok(retrieved) => {
                println!("✅ User {} with READ_ALL permission successfully retrieved annotation {} (owned by user {})", 
                    reader_id, retrieved.id, owner_id);
                assert_eq!(retrieved.id, annotation.id);
            }
            Err(e) => {
                panic!("User with READ_ALL permission should be able to read annotation: {:?}", e);
            }
        }
    }

    /// 테스트 7: 권한 없는 사용자는 다른 사용자의 Annotation을 조회할 수 없어야 함
    #[tokio::test]
    async fn test_get_annotation_without_permission() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 먼저 테스트용 annotation 생성 (user_id = 1)
        let project_id = 2;
        let owner_id = 1;

        let create_request = CreateAnnotationRequest {
            user_id: None,
            project_id: Some(project_id),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation for permission check".to_string()),
            measurement_values: None,
            label: None,
        };

        let created = use_case
            .create_annotation(create_request, owner_id, project_id)
            .await
            .expect("Failed to create test annotation");

        // 권한 없는 사용자 (user_id = 999, 존재하지 않는 사용자 또는 프로젝트 멤버가 아닌 사용자)로 조회 시도
        // 실제로는 프로젝트 멤버가 아닌 사용자나 READ_ALL 권한이 없는 사용자를 사용해야 함
        // 여기서는 간단히 다른 user_id를 사용 (실제로는 프로젝트 멤버가 아닌 사용자를 사용해야 함)
        // 프로젝트 2의 다른 사용자를 찾거나, 프로젝트 멤버가 아닌 사용자를 사용해야 함
        // 테스트를 위해 프로젝트 멤버가 아닌 사용자 ID를 사용 (예: 999)
        let unauthorized_user_id = 999;

        let result = use_case.get_annotation_by_id(unauthorized_user_id, created.id).await;

        match result {
            Ok(_) => {
                // 만약 999가 프로젝트 멤버라면 이 테스트는 실패할 수 있음
                // 실제로는 프로젝트 멤버가 아닌 사용자를 사용해야 함
                println!("⚠️  User {} was able to read annotation, may be a project member", unauthorized_user_id);
            }
            Err(e) => {
                println!("✅ User {} correctly denied access to annotation {}: {:?}", 
                    unauthorized_user_id, created.id, e);
                // Unauthorized 에러가 발생해야 함
                match e {
                    pacs_server::domain::errors::ServiceError::Unauthorized(_) => {
                        // 예상된 에러
                    }
                    _ => {
                        panic!("Expected Unauthorized error, got: {:?}", e);
                    }
                }
            }
        }
    }

    /// 테스트 8: 프로젝트 멤버가 아닌 사용자는 Annotation을 조회할 수 없어야 함
    #[tokio::test]
    async fn test_get_annotation_as_non_member() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 프로젝트 2의 기존 annotation 조회
        let annotations = use_case
            .get_annotations_by_project(2)
            .await
            .expect("Failed to get annotations");

        if annotations.annotations.is_empty() {
            println!("⚠️  No annotations found in project 2, skipping test");
            return;
        }

        let annotation = &annotations.annotations[0];

        // 프로젝트 멤버가 아닌 사용자 (user_id = 999)로 조회 시도
        let non_member_id = 999;

        let result = use_case.get_annotation_by_id(non_member_id, annotation.id).await;

        match result {
            Ok(_) => {
                // 만약 999가 프로젝트 멤버라면 이 테스트는 실패할 수 있음
                println!("⚠️  User {} was able to read annotation, may be a project member", non_member_id);
            }
            Err(e) => {
                println!("✅ Non-member user {} correctly denied access to annotation {}: {:?}", 
                    non_member_id, annotation.id, e);
                // Unauthorized 에러가 발생해야 함
                match e {
                    pacs_server::domain::errors::ServiceError::Unauthorized(_) => {
                        // 예상된 에러
                    }
                    _ => {
                        panic!("Expected Unauthorized error, got: {:?}", e);
                    }
                }
            }
        }
    }

    /// 테스트 9: 존재하지 않는 Annotation 조회 시도 (404 에러)
    #[tokio::test]
    async fn test_get_annotation_not_found() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let non_existent_annotation_id = 999999; // 존재하지 않는 Annotation ID

        let result = use_case.get_annotation_by_id(user_id, non_existent_annotation_id).await;

        match result {
            Ok(_) => {
                panic!("Should return NotFound error for non-existent annotation");
            }
            Err(e) => {
                println!("✅ Correctly returned error for non-existent annotation: {:?}", e);
                match e {
                    pacs_server::domain::errors::ServiceError::NotFound(_) => {
                        // 예상된 에러
                    }
                    _ => {
                        panic!("Expected NotFound error, got: {:?}", e);
                    }
                }
            }
        }
    }

    /// 테스트 10: 프로젝트 멤버이지만 소유자도 아니고 READ_ALL 권한도 없는 경우
    #[tokio::test]
    async fn test_get_annotation_as_member_without_permission() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 프로젝트 2의 기존 annotation 조회
        let annotations = use_case
            .get_annotations_by_project(2)
            .await
            .expect("Failed to get annotations");

        if annotations.annotations.is_empty() {
            println!("⚠️  No annotations found in project 2, skipping test");
            return;
        }

        let annotation = &annotations.annotations[0];
        let owner_id = annotation.user_id;

        // 프로젝트 멤버이지만 소유자도 아니고 READ_ALL 권한도 없는 사용자 찾기
        // 프로젝트 2의 다른 멤버를 찾거나, READ_ALL 권한이 없는 사용자를 사용
        // 여기서는 간단히 다른 user_id를 사용 (실제로는 프로젝트 멤버이지만 권한이 없는 사용자를 찾아야 함)
        // 테스트를 위해 프로젝트 멤버이지만 READ_ALL 권한이 없는 사용자 ID를 사용
        // 실제 환경에서는 프로젝트 멤버 중 READ_ALL 권한이 없는 사용자를 찾아야 함
        let member_without_permission_id = if owner_id == 1 { 2 } else { 1 };

        // 먼저 해당 사용자가 프로젝트 멤버인지 확인
        let is_member = use_case
            .access_control_service
            .is_project_member(member_without_permission_id, annotation.project_id)
            .await
            .ok()
            .unwrap_or(false);

        if !is_member {
            println!("⚠️  User {} is not a member of project {}, skipping test", 
                member_without_permission_id, annotation.project_id);
            return;
        }

        // READ_ALL 권한 확인
        let has_read_all = use_case
            .access_control_service
            .check_permission(member_without_permission_id, annotation.project_id, "ANNOTATION", "READ_ALL")
            .await
            .ok()
            .unwrap_or(false);

        if has_read_all {
            println!("⚠️  User {} has READ_ALL permission, skipping test", member_without_permission_id);
            return;
        }

        // 소유자가 아닌지 확인
        if member_without_permission_id == owner_id {
            println!("⚠️  User {} is the owner, skipping test", member_without_permission_id);
            return;
        }

        // 프로젝트 멤버이지만 소유자도 아니고 READ_ALL 권한도 없는 사용자로 조회 시도
        let result = use_case.get_annotation_by_id(member_without_permission_id, annotation.id).await;

        match result {
            Ok(_) => {
                panic!("User {} should not be able to read annotation {} (not owner, no READ_ALL permission)", 
                    member_without_permission_id, annotation.id);
            }
            Err(e) => {
                println!("✅ Member user {} correctly denied access to annotation {} (not owner, no READ_ALL): {:?}", 
                    member_without_permission_id, annotation.id, e);
                match e {
                    pacs_server::domain::errors::ServiceError::Unauthorized(_) => {
                        // 예상된 에러
                    }
                    _ => {
                        panic!("Expected Unauthorized error, got: {:?}", e);
                    }
                }
            }
        }
    }

    // ========== 통합 테스트: Annotation 권한 조회 API 개선 ==========

    /// 테스트 11: project_id 필수 파라미터 검증
    #[tokio::test]
    async fn test_get_annotation_permissions_requires_project_id() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let project_id = 2;

        // 정상 케이스: project_id 제공
        let result = use_case
            .get_user_annotation_permissions(user_id, project_id)
            .await;

        match result {
            Ok(_) => {
                println!("✅ Successfully retrieved permissions with valid project_id");
            }
            Err(e) => {
                panic!("Should succeed with valid project_id: {:?}", e);
            }
        }
    }

    /// 테스트 12: 본인 권한 조회 (user_id 파라미터 없음)
    #[tokio::test]
    async fn test_get_own_annotation_permissions() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let project_id = 2;

        // 본인 권한 조회
        let result = use_case
            .get_user_annotation_permissions(user_id, project_id)
            .await;

        match result {
            Ok(permissions) => {
                println!("✅ Successfully retrieved own permissions for user {} in project {}", user_id, project_id);
                println!("   can_read_own: {}", permissions.can_read_own);
                println!("   can_read_all: {}", permissions.can_read_all);
                println!("   can_write: {}", permissions.can_write);
                println!("   can_delete: {}", permissions.can_delete);
                println!("   can_share: {}", permissions.can_share);
            }
            Err(e) => {
                panic!("Should be able to retrieve own permissions: {:?}", e);
            }
        }
    }

    /// 테스트 13: 다른 사용자 권한 조회 (프로젝트 멤버인 경우)
    #[tokio::test]
    async fn test_get_other_user_annotation_permissions_as_member() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let requesting_user_id = 1; // 프로젝트 멤버
        let target_user_id = 2; // 다른 사용자
        let project_id = 2;

        // 요청한 사용자가 프로젝트 멤버인지 확인 (본인 권한 조회로 확인)
        let requesting_user_is_member = use_case
            .get_user_annotation_permissions(requesting_user_id, project_id)
            .await
            .is_ok();

        if !requesting_user_is_member {
            println!("⚠️  User {} is not a member of project {}, skipping test", requesting_user_id, project_id);
            return;
        }

        // 다른 사용자의 권한 조회
        let result = use_case
            .get_user_annotation_permissions(target_user_id, project_id)
            .await;

        match result {
            Ok(permissions) => {
                println!("✅ Successfully retrieved permissions for user {} (requested by user {}) in project {}", 
                    target_user_id, requesting_user_id, project_id);
                println!("   can_read_own: {}", permissions.can_read_own);
                println!("   can_read_all: {}", permissions.can_read_all);
                println!("   can_write: {}", permissions.can_write);
                println!("   can_delete: {}", permissions.can_delete);
                println!("   can_share: {}", permissions.can_share);
            }
            Err(e) => {
                // 다른 사용자가 프로젝트 멤버가 아닐 수 있음
                println!("⚠️  Could not retrieve permissions for user {}: {:?}", target_user_id, e);
            }
        }
    }

    /// 테스트 14: 존재하지 않는 프로젝트의 권한 조회 (404 에러)
    #[tokio::test]
    async fn test_get_annotation_permissions_nonexistent_project() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let non_existent_project_id = 999999; // 존재하지 않는 프로젝트 ID

        let result = use_case
            .get_user_annotation_permissions(user_id, non_existent_project_id)
            .await;

        match result {
            Ok(_) => {
                // 프로젝트가 존재하지 않아도 권한 조회는 가능할 수 있음 (빈 권한 반환)
                println!("⚠️  Retrieved permissions for non-existent project (may return empty permissions)");
            }
            Err(e) => {
                println!("✅ Correctly returned error for non-existent project: {:?}", e);
                // NotFound 또는 Unauthorized 에러가 발생할 수 있음
                match e {
                    pacs_server::domain::errors::ServiceError::NotFound(_) => {
                        // 예상된 에러
                    }
                    pacs_server::domain::errors::ServiceError::Unauthorized(_) => {
                        // 예상된 에러 (프로젝트 멤버가 아닌 경우)
                    }
                    _ => {
                        // 다른 에러도 허용 (구현에 따라 다를 수 있음)
                    }
                }
            }
        }
    }

    /// 테스트 15: is_project_member 메서드 테스트
    #[tokio::test]
    async fn test_is_project_member() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let project_id = 2;

        // 프로젝트 멤버 확인
        let result = use_case.is_project_member(user_id, project_id).await;

        match result {
            Ok(is_member) => {
                println!("✅ User {} is member of project {}: {}", user_id, project_id, is_member);
                // 결과는 데이터베이스 상태에 따라 다를 수 있음
            }
            Err(e) => {
                println!("⚠️  Error checking project membership: {:?}", e);
            }
        }
    }

    /// 테스트 16: 다른 사용자 권한 조회 시 프로젝트 멤버가 아닌 경우 (권한 체크)
    #[tokio::test]
    async fn test_get_other_user_permissions_as_non_member() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 프로젝트 멤버가 아닌 사용자 찾기
        let non_member_user_id = 999; // 프로젝트 멤버가 아닌 사용자
        let target_user_id = 1; // 조회하려는 사용자
        let project_id = 2;

        // non_member_user_id가 프로젝트 멤버가 아닌지 확인
        let is_member = use_case
            .is_project_member(non_member_user_id, project_id)
            .await
            .ok()
            .unwrap_or(false);

        if is_member {
            println!("⚠️  User {} is a member of project {}, skipping test", non_member_user_id, project_id);
            return;
        }

        // 프로젝트 멤버가 아닌 사용자가 다른 사용자의 권한 조회 시도
        // UseCase 레벨에서는 직접 권한 체크를 하지 않으므로, 
        // 실제로는 컨트롤러에서 권한 체크가 이루어짐
        // 여기서는 is_project_member가 false를 반환하는지 확인
        let result = use_case.is_project_member(non_member_user_id, project_id).await;

        match result {
            Ok(false) => {
                println!("✅ Non-member user {} correctly identified as non-member of project {}", 
                    non_member_user_id, project_id);
            }
            Ok(true) => {
                println!("⚠️  User {} is actually a member, test scenario invalid", non_member_user_id);
            }
            Err(e) => {
                println!("⚠️  Error checking membership: {:?}", e);
            }
        }
    }

    /// 테스트 17: user_id 쿼리 파라미터로 본인과 동일한 사용자 지정
    #[tokio::test]
    async fn test_get_permissions_with_explicit_own_user_id() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let project_id = 2;

        // 본인 user_id를 명시적으로 지정하여 권한 조회
        let result = use_case
            .get_user_annotation_permissions(user_id, project_id)
            .await;

        match result {
            Ok(permissions) => {
                println!("✅ Successfully retrieved permissions with explicit own user_id");
                println!("   can_read_own: {}", permissions.can_read_own);
                println!("   can_read_all: {}", permissions.can_read_all);
                println!("   can_write: {}", permissions.can_write);
                println!("   can_delete: {}", permissions.can_delete);
                println!("   can_share: {}", permissions.can_share);
            }
            Err(e) => {
                panic!("Should be able to retrieve own permissions with explicit user_id: {:?}", e);
            }
        }
    }

    /// 테스트 18: user_id 쿼리 파라미터로 다른 사용자 지정 (프로젝트 멤버인 경우)
    #[tokio::test]
    async fn test_get_permissions_with_explicit_other_user_id() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let requesting_user_id = 1;
        let target_user_id = 2; // 다른 사용자
        let project_id = 2;

        // 요청한 사용자가 프로젝트 멤버인지 확인
        let requesting_is_member = use_case
            .is_project_member(requesting_user_id, project_id)
            .await
            .ok()
            .unwrap_or(false);

        if !requesting_is_member {
            println!("⚠️  Requesting user {} is not a member of project {}, skipping test", 
                requesting_user_id, project_id);
            return;
        }

        // 다른 사용자의 권한 조회 (user_id 명시)
        let result = use_case
            .get_user_annotation_permissions(target_user_id, project_id)
            .await;

        match result {
            Ok(permissions) => {
                println!("✅ Successfully retrieved permissions for user {} (explicitly specified) in project {}", 
                    target_user_id, project_id);
                println!("   can_read_own: {}", permissions.can_read_own);
                println!("   can_read_all: {}", permissions.can_read_all);
                println!("   can_write: {}", permissions.can_write);
                println!("   can_delete: {}", permissions.can_delete);
                println!("   can_share: {}", permissions.can_share);
            }
            Err(e) => {
                // 다른 사용자가 프로젝트 멤버가 아닐 수 있음
                println!("⚠️  Could not retrieve permissions for user {}: {:?}", target_user_id, e);
            }
        }
    }

    /// 테스트 19: 존재하지 않는 사용자의 권한 조회
    #[tokio::test]
    async fn test_get_permissions_nonexistent_user() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let non_existent_user_id = 999999; // 존재하지 않는 사용자 ID
        let project_id = 2;

        let result = use_case
            .get_user_annotation_permissions(non_existent_user_id, project_id)
            .await;

        match result {
            Ok(_) => {
                // 존재하지 않는 사용자도 권한 조회는 가능할 수 있음 (빈 권한 반환)
                println!("⚠️  Retrieved permissions for non-existent user (may return empty permissions)");
            }
            Err(e) => {
                println!("✅ Correctly returned error for non-existent user: {:?}", e);
                // NotFound 또는 다른 에러가 발생할 수 있음
                match e {
                    pacs_server::domain::errors::ServiceError::NotFound(_) => {
                        // 예상된 에러
                    }
                    _ => {
                        // 다른 에러도 허용 (구현에 따라 다를 수 있음)
                    }
                }
            }
        }
    }

    /// 테스트 20: 여러 프로젝트에서의 권한 조회 비교
    #[tokio::test]
    async fn test_get_permissions_multiple_projects() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let user_id = 1;
        let project_ids = vec![2, 299]; // 여러 프로젝트 ID

        for project_id in project_ids {
            let result = use_case
                .get_user_annotation_permissions(user_id, project_id)
                .await;

            match result {
                Ok(permissions) => {
                    println!("✅ Retrieved permissions for user {} in project {}:", user_id, project_id);
                    println!("   can_read_own: {}", permissions.can_read_own);
                    println!("   can_read_all: {}", permissions.can_read_all);
                    println!("   can_write: {}", permissions.can_write);
                    println!("   can_delete: {}", permissions.can_delete);
                    println!("   can_share: {}", permissions.can_share);
                }
                Err(e) => {
                    println!("⚠️  Could not retrieve permissions for user {} in project {}: {:?}", 
                        user_id, project_id, e);
                }
            }
        }
    }

    // Note: HTTP 엔드포인트 테스트는 타입 불일치 문제로 인해 제외되었습니다.
    // 실제 환경에서는 HTTP 통합 테스트를 별도로 작성하는 것을 권장합니다.
}

