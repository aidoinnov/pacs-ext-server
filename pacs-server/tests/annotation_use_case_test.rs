#[cfg(test)]
mod annotation_use_case_tests {
    use actix_web::{test, App};
    use pacs_server::application::dto::annotation_dto::{
        CreateAnnotationRequest, UpdateAnnotationRequest
    };
    use pacs_server::domain::entities::Annotation;
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::AnnotationServiceImpl;
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl,
    };
    use pacs_server::presentation::controllers::annotation_controller;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;
    use serde_json::json;

    async fn setup_test_app() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<sqlx::Pool<sqlx::Postgres>>,
    ) {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));

        let app = test::init_service(
            App::new().configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone())),
        )
        .await;

        (app, pool)
    }

async fn cleanup_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>) {
    // Foreign key constraint를 완전히 비활성화
    sqlx::query("SET session_replication_role = replica").execute(pool.as_ref()).await.unwrap();
    
    // 모든 테이블을 순서대로 삭제 (foreign key constraint 비활성화 상태에서)
    sqlx::query("DELETE FROM annotation_annotation_history").execute(pool.as_ref()).await.unwrap();
    sqlx::query("DELETE FROM annotation_annotation").execute(pool.as_ref()).await.unwrap();
    sqlx::query("DELETE FROM security_access_log").execute(pool.as_ref()).await.unwrap();
    sqlx::query("DELETE FROM security_user_project").execute(pool.as_ref()).await.unwrap();
    sqlx::query("DELETE FROM security_project").execute(pool.as_ref()).await.unwrap();
    sqlx::query("DELETE FROM security_user").execute(pool.as_ref()).await.unwrap();
    
    // Foreign key constraint를 다시 활성화
    sqlx::query("SET session_replication_role = DEFAULT").execute(pool.as_ref()).await.unwrap();
    
    // 시퀀스 리셋 (auto-increment ID 초기화)
    sqlx::query("ALTER SEQUENCE security_user_id_seq RESTART WITH 1").execute(pool.as_ref()).await.unwrap();
    sqlx::query("ALTER SEQUENCE security_project_id_seq RESTART WITH 1").execute(pool.as_ref()).await.unwrap();
    sqlx::query("ALTER SEQUENCE annotation_annotation_id_seq RESTART WITH 1").execute(pool.as_ref()).await.unwrap();
}

    #[actix_web::test]
    async fn test_create_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        let create_req = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
            annotation_data: json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            description: Some("Test annotation".to_string()),
        };

        // Test the use case directly instead of HTTP request
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        // 사용자가 프로젝트 멤버인지 확인
        let is_member = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to check project membership");
        
        println!("User {} is member of project {}: {}", user_id, project_id, is_member > 0);

        let result = annotation_use_case.create_annotation(create_req, user_id, project_id).await;
        match result {
            Ok(annotation) => {
                println!("Annotation created successfully with ID: {}", annotation.id);
                // 생성된 annotation을 다시 조회해서 확인
                let retrieved = annotation_use_case.get_annotation_by_id(annotation.id).await;
                match retrieved {
                    Ok(retrieved_annotation) => {
                        println!("Retrieved annotation: {:?}", retrieved_annotation);
                    }
                    Err(e) => {
                        println!("Failed to retrieve created annotation: {:?}", e);
                        // 데이터베이스에서 직접 확인
                        let direct_check = sqlx::query_as::<_, Annotation>(
                            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                                    viewer_software, description
                             FROM annotation_annotation
                             WHERE id = $1"
                        )
                        .bind(annotation.id)
                        .fetch_optional(pool.as_ref())
                        .await;
                        match direct_check {
                            Ok(Some(ann)) => println!("Direct DB query found annotation: {:?}", ann),
                            Ok(None) => println!("Direct DB query found no annotation with ID: {}", annotation.id),
                            Err(e) => println!("Direct DB query failed: {:?}", e),
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to create annotation: {:?}", e);
                panic!("Annotation creation failed: {:?}", e);
            }
        }

        // Verify annotation was created
        let annotation_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM annotation_annotation WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to count annotations");

        assert_eq!(annotation_count, 1);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_get_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        println!("test_get_annotation_use_case - Created user_id: {}, project_id: {}", user_id, project_id);

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        // Verify user is member of project
        let is_member = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to check project membership");

        println!("test_get_annotation_use_case - User {} is member of project {}: {}", user_id, project_id, is_member > 0);

        // Create annotation
        let annotation_result = sqlx::query(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, data, is_shared) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(project_id)
        .bind(user_id)
        .bind("1.2.3.4.5")
        .bind("1.2.3.4.6")
        .bind("1.2.3.4.7")
        .bind("test_tool")
        .bind(json!({"type": "test"}))
        .bind(false)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        // Test the use case directly instead of HTTP request
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        let result = annotation_use_case.get_annotation_by_id(annotation_id).await;
        assert!(result.is_ok());

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_update_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project with unique identifiers
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_update_{}", keycloak_id))
        .bind(&format!("test_update_{}@example.com", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project Update {}", keycloak_id))
        .bind(&format!("Test Description Update {}", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        // Create annotation using use case
        let create_req = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: json!({"type": "test"}),
            description: Some("Test annotation for update".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
        };

        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        let annotation = annotation_use_case.create_annotation(create_req, user_id, project_id).await
            .expect("Failed to create test annotation");
        let annotation_id = annotation.id;

        let update_req = UpdateAnnotationRequest {
            tool_name: Some("updated_tool".to_string()),
            tool_version: Some("2.0.0".to_string()),
            viewer_software: Some("updated_viewer".to_string()),
            annotation_data: Some(json!({"type": "updated", "x": 200, "y": 300})),
            description: Some("Updated annotation".to_string()),
        };

        let result = annotation_use_case.update_annotation(annotation_id, update_req).await;
        match result {
            Ok(_) => println!("Annotation updated successfully"),
            Err(e) => {
                println!("Failed to update annotation: {:?}", e);
                panic!("Annotation update failed: {:?}", e);
            }
        }

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_delete_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project with unique identifiers
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_delete_{}", keycloak_id))
        .bind(&format!("test_delete_{}@example.com", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project Delete {}", keycloak_id))
        .bind(&format!("Test Description Delete {}", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        // Create annotation using use case
        let create_req = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: json!({"type": "test"}),
            description: Some("Test annotation for deletion".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
        };

        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        let annotation = annotation_use_case.create_annotation(create_req, user_id, project_id).await
            .expect("Failed to create test annotation");
        let annotation_id = annotation.id;

        let result = annotation_use_case.delete_annotation(annotation_id).await;
        assert!(result.is_ok());

        // Verify annotation was deleted
        let annotation_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM annotation_annotation WHERE id = $1)"
        )
        .bind(annotation_id)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to check annotation existence");

        assert!(!annotation_exists);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_list_annotations_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project with unique identifiers
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_list_{}", keycloak_id))
        .bind(&format!("test_list_{}@example.com", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project List {}", keycloak_id))
        .bind(&format!("Test Description List {}", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        // Create multiple annotations using use case
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        for i in 0..3 {
            let create_req = CreateAnnotationRequest {
                user_id: Some(336),
                project_id: Some(299),
            user_id: Some(336),
            project_id: Some(299),
                study_instance_uid: format!("1.2.3.4.{}", i),
                series_instance_uid: format!("1.2.3.5.{}", i),
                sop_instance_uid: format!("1.2.3.6.{}", i),
                annotation_data: json!({"type": "test", "index": i}),
                description: Some(format!("Test annotation {}", i)),
                tool_name: Some("test_tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                viewer_software: Some("test_viewer".to_string()),
            };

            annotation_use_case.create_annotation(create_req, user_id, project_id).await
                .expect("Failed to create test annotation");
        }

        let result = annotation_use_case.get_annotations_by_project(project_id).await;
        match result {
            Ok(annotation_list) => {
                println!("Found {} annotations", annotation_list.annotations.len());
                assert_eq!(annotation_list.annotations.len(), 3);
            }
            Err(e) => {
                println!("Failed to get annotations: {:?}", e);
                panic!("Failed to get annotations: {:?}", e);
            }
        }

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_annotation_not_found_use_case() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/annotations/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_annotation_validation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        // Test with invalid data using use case directly
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        // Test with empty study_uid (should fail validation)
        let invalid_req = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "".to_string(),
            series_instance_uid: "1.2.3.4.5".to_string(),
            sop_instance_uid: "1.2.3.4.6".to_string(),
            annotation_data: serde_json::json!({"type": "test"}),
            description: Some("Test".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
        };

        let result = annotation_use_case.create_annotation(invalid_req, user_id, project_id).await;
        assert!(result.is_err());

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_annotations_by_user_with_viewer_filter() {
        let (app, pool) = setup_test_app().await;
        let annotation_use_case = test_get_annotation_use_case(&pool).await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 테스트 데이터 생성
        let annotation1 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test1"}),
            description: Some("Test annotation 1".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let annotation2 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.8".to_string(),
            series_instance_uid: "1.2.3.4.9".to_string(),
            sop_instance_uid: "1.2.3.4.10".to_string(),
            annotation_data: serde_json::json!({"type": "test2"}),
            description: Some("Test annotation 2".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
        };

        let annotation3 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.11".to_string(),
            series_instance_uid: "1.2.3.4.12".to_string(),
            sop_instance_uid: "1.2.3.4.13".to_string(),
            annotation_data: serde_json::json!({"type": "test3"}),
            description: Some("Test annotation 3".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        // 어노테이션들 생성
        annotation_use_case.create_annotation(annotation1, user_id, project_id).await.unwrap();
        annotation_use_case.create_annotation(annotation2, user_id, project_id).await.unwrap();
        annotation_use_case.create_annotation(annotation3, user_id, project_id).await.unwrap();

        // OHIF Viewer로 필터링
        let ohif_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.total, 2);
        assert!(ohif_annotations.annotations.iter().all(|a| a.viewer_software == Some("OHIF Viewer".to_string())));

        // DICOM Viewer로 필터링
        let dicom_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("DICOM Viewer")).await.unwrap();
        assert_eq!(dicom_annotations.total, 1);
        assert!(dicom_annotations.annotations.iter().all(|a| a.viewer_software == Some("DICOM Viewer".to_string())));

        // 필터 없이 모든 어노테이션 조회
        let all_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, None).await.unwrap();
        assert_eq!(all_annotations.total, 3);

        // 존재하지 않는 viewer_software로 필터링
        let no_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("NonExistent Viewer")).await.unwrap();
        assert_eq!(no_annotations.total, 0);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_annotations_by_project_with_viewer_filter() {
        let (app, pool) = setup_test_app().await;
        let annotation_use_case = test_get_annotation_use_case(&pool).await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 테스트 데이터 생성
        let annotation1 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test1"}),
            description: Some("Test annotation 1".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let annotation2 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.8".to_string(),
            series_instance_uid: "1.2.3.4.9".to_string(),
            sop_instance_uid: "1.2.3.4.10".to_string(),
            annotation_data: serde_json::json!({"type": "test2"}),
            description: Some("Test annotation 2".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
        };

        // 어노테이션들 생성
        annotation_use_case.create_annotation(annotation1, user_id, project_id).await.unwrap();
        annotation_use_case.create_annotation(annotation2, user_id, project_id).await.unwrap();

        // OHIF Viewer로 필터링
        let ohif_annotations = annotation_use_case.get_annotations_by_project_with_viewer(project_id, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.total, 1);
        assert!(ohif_annotations.annotations.iter().all(|a| a.viewer_software == Some("OHIF Viewer".to_string())));

        // DICOM Viewer로 필터링
        let dicom_annotations = annotation_use_case.get_annotations_by_project_with_viewer(project_id, Some("DICOM Viewer")).await.unwrap();
        assert_eq!(dicom_annotations.total, 1);
        assert!(dicom_annotations.annotations.iter().all(|a| a.viewer_software == Some("DICOM Viewer".to_string())));

        // 필터 없이 모든 어노테이션 조회
        let all_annotations = annotation_use_case.get_annotations_by_project_with_viewer(project_id, None).await.unwrap();
        assert_eq!(all_annotations.total, 2);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_annotations_by_study_with_viewer_filter() {
        let (app, pool) = setup_test_app().await;
        let annotation_use_case = test_get_annotation_use_case(&pool).await;
        
        let user_id = 336;
        let project_id = 299;
        let study_uid = "1.2.3.4.5";
        
        // 테스트 데이터 생성
        let annotation1 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: study_uid.to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test1"}),
            description: Some("Test annotation 1".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let annotation2 = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: study_uid.to_string(),
            series_instance_uid: "1.2.3.4.8".to_string(),
            sop_instance_uid: "1.2.3.4.9".to_string(),
            annotation_data: serde_json::json!({"type": "test2"}),
            description: Some("Test annotation 2".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
        };

        // 어노테이션들 생성
        annotation_use_case.create_annotation(annotation1, user_id, project_id).await.unwrap();
        annotation_use_case.create_annotation(annotation2, user_id, project_id).await.unwrap();

        // OHIF Viewer로 필터링
        let ohif_annotations = annotation_use_case.get_annotations_by_study_with_viewer(study_uid, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.total, 1);
        assert!(ohif_annotations.annotations.iter().all(|a| a.viewer_software == Some("OHIF Viewer".to_string())));

        // DICOM Viewer로 필터링
        let dicom_annotations = annotation_use_case.get_annotations_by_study_with_viewer(study_uid, Some("DICOM Viewer")).await.unwrap();
        assert_eq!(dicom_annotations.total, 1);
        assert!(dicom_annotations.annotations.iter().all(|a| a.viewer_software == Some("DICOM Viewer".to_string())));

        // 필터 없이 모든 어노테이션 조회
        let all_annotations = annotation_use_case.get_annotations_by_study_with_viewer(study_uid, None).await.unwrap();
        assert_eq!(all_annotations.total, 2);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_create_annotation_with_measurement_values() {
        let (app, pool) = setup_test_app().await;
        let (user_id, project_id) = create_test_data(&pool).await;

        let annotation_request = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: json!({"type": "measurement", "points": [[0, 0], [100, 100]]}),
            description: Some("폐 결절 크기 측정".to_string()),
            tool_name: Some("Measurement Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
            measurement_values: Some(json!([
                {"id": "m1", "type": "raw", "values": [42.3, 18.7], "unit": "mm"},
                {"id": "m2", "type": "mean", "values": [30.5], "unit": "mm"}
            ])),
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation_request)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["measurement_values"].is_array());
        assert_eq!(body["measurement_values"][0]["id"], "m1");
        assert_eq!(body["measurement_values"][0]["type"], "raw");
        assert_eq!(body["measurement_values"][0]["unit"], "mm");
        assert_eq!(body["measurement_values"][1]["id"], "m2");
        assert_eq!(body["measurement_values"][1]["type"], "mean");

        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_create_annotation_without_measurement_values() {
        let (app, pool) = setup_test_app().await;
        let (user_id, project_id) = create_test_data(&pool).await;

        let annotation_request = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: json!({"type": "point", "coordinates": [50, 50]}),
            description: Some("단순 포인트 어노테이션".to_string()),
            tool_name: Some("Point Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
            measurement_values: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation_request)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["measurement_values"].is_null());

        cleanup_test_data(&pool, user_id, project_id).await;
    }
}
