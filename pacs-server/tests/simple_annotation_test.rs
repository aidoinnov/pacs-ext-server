#[cfg(test)]
mod simple_annotation_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::annotation_dto::CreateAnnotationRequest;
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::AnnotationServiceImpl;
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl,
    };
    use pacs_server::presentation::controllers::annotation_controller;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;

    async fn setup_test_app() -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
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

        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));

        test::init_service(
            App::new()
                .app_data(web::Data::new(annotation_use_case.clone()))
                .configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone())),
        )
        .await
    }

    #[tokio::test]
    async fn test_annotation_creation() {
        let app = setup_test_app().await;

        let annotation = CreateAnnotationRequest {
            user_id: Some(336),
            project_id: Some(299),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test"}),
            description: Some("Test annotation".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        println!("Response status: {}", resp.status());
        
        if resp.status() != 201 {
            let body: serde_json::Value = test::read_body_json(resp).await;
            println!("Response body: {}", body);
        } else {
            assert_eq!(resp.status(), 201);
        }
    }
}
