#[cfg(test)]
mod annotation_repository_viewer_filter_tests {
use pacs_server::domain::entities::{Annotation, NewAnnotation};
use pacs_server::domain::repositories::AnnotationRepository;
use pacs_server::infrastructure::repositories::AnnotationRepositoryImpl;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use chrono::{DateTime, Utc};

    async fn setup_test_repository() -> (AnnotationRepositoryImpl, Arc<sqlx::Pool<sqlx::Postgres>>) {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let pool = Arc::new(pool);
        let repository = AnnotationRepositoryImpl::new(pool.clone());

        (repository, pool)
    }

    async fn cleanup_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32, project_id: i32) {
        // 테스트 데이터 정리
        sqlx::query("DELETE FROM annotation_annotation WHERE user_id = $1 AND project_id = $2")
            .bind(user_id)
            .bind(project_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_find_by_user_id_with_viewer_filter() {
        let (repository, pool) = setup_test_repository().await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 테스트 데이터 생성
        let annotation1 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test1"}),
            is_shared: false,
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation 1".to_string()),
        };

        let annotation2 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.8".to_string(),
            series_uid: Some("1.2.3.4.9".to_string()),
            instance_uid: Some("1.2.3.4.10".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test2"}),
            is_shared: false,
            viewer_software: Some("DICOM Viewer".to_string()),
            description: Some("Test annotation 2".to_string()),
        };

        let annotation3 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.11".to_string(),
            series_uid: Some("1.2.3.4.12".to_string()),
            instance_uid: Some("1.2.3.4.13".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test3"}),
            is_shared: false,
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation 3".to_string()),
        };

        // 어노테이션들 생성
        repository.create(annotation1).await.unwrap();
        repository.create(annotation2).await.unwrap();
        repository.create(annotation3).await.unwrap();

        // OHIF Viewer로 필터링
        let ohif_annotations = repository.find_by_user_id_with_viewer(user_id, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.len(), 2);
        assert!(ohif_annotations.iter().all(|a| a.viewer_software == Some("OHIF Viewer".to_string())));

        // DICOM Viewer로 필터링
        let dicom_annotations = repository.find_by_user_id_with_viewer(user_id, Some("DICOM Viewer")).await.unwrap();
        assert_eq!(dicom_annotations.len(), 1);
        assert!(dicom_annotations.iter().all(|a| a.viewer_software == Some("DICOM Viewer".to_string())));

        // 필터 없이 모든 어노테이션 조회
        let all_annotations = repository.find_by_user_id_with_viewer(user_id, None).await.unwrap();
        assert_eq!(all_annotations.len(), 3);

        // 존재하지 않는 viewer_software로 필터링
        let no_annotations = repository.find_by_user_id_with_viewer(user_id, Some("NonExistent Viewer")).await.unwrap();
        assert_eq!(no_annotations.len(), 0);

        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_find_by_project_id_with_viewer_filter() {
        let (repository, pool) = setup_test_repository().await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 테스트 데이터 생성
        let annotation1 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test1"}),
            is_shared: false,
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation 1".to_string()),
        };

        let annotation2 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.8".to_string(),
            series_uid: Some("1.2.3.4.9".to_string()),
            instance_uid: Some("1.2.3.4.10".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test2"}),
            is_shared: false,
            viewer_software: Some("DICOM Viewer".to_string()),
            description: Some("Test annotation 2".to_string()),
        };

        // 어노테이션들 생성
        repository.create(annotation1).await.unwrap();
        repository.create(annotation2).await.unwrap();

        // OHIF Viewer로 필터링
        let ohif_annotations = repository.find_by_project_id_with_viewer(project_id, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.len(), 1);
        assert!(ohif_annotations.iter().all(|a| a.viewer_software == Some("OHIF Viewer".to_string())));

        // DICOM Viewer로 필터링
        let dicom_annotations = repository.find_by_project_id_with_viewer(project_id, Some("DICOM Viewer")).await.unwrap();
        assert_eq!(dicom_annotations.len(), 1);
        assert!(dicom_annotations.iter().all(|a| a.viewer_software == Some("DICOM Viewer".to_string())));

        // 필터 없이 모든 어노테이션 조회
        let all_annotations = repository.find_by_project_id_with_viewer(project_id, None).await.unwrap();
        assert_eq!(all_annotations.len(), 2);

        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_find_by_study_uid_with_viewer_filter() {
        let (repository, pool) = setup_test_repository().await;
        
        let user_id = 336;
        let project_id = 299;
        let study_uid = "1.2.3.4.5";
        
        // 테스트 데이터 생성
        let annotation1 = NewAnnotation {
            project_id,
            user_id,
            study_uid: study_uid.to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test1"}),
            is_shared: false,
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation 1".to_string()),
        };

        let annotation2 = NewAnnotation {
            project_id,
            user_id,
            study_uid: study_uid.to_string(),
            series_uid: Some("1.2.3.4.8".to_string()),
            instance_uid: Some("1.2.3.4.9".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test2"}),
            is_shared: false,
            viewer_software: Some("DICOM Viewer".to_string()),
            description: Some("Test annotation 2".to_string()),
        };

        // 어노테이션들 생성
        repository.create(annotation1).await.unwrap();
        repository.create(annotation2).await.unwrap();

        // OHIF Viewer로 필터링
        let ohif_annotations = repository.find_by_study_uid_with_viewer(study_uid, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.len(), 1);
        assert!(ohif_annotations.iter().all(|a| a.viewer_software == Some("OHIF Viewer".to_string())));

        // DICOM Viewer로 필터링
        let dicom_annotations = repository.find_by_study_uid_with_viewer(study_uid, Some("DICOM Viewer")).await.unwrap();
        assert_eq!(dicom_annotations.len(), 1);
        assert!(dicom_annotations.iter().all(|a| a.viewer_software == Some("DICOM Viewer".to_string())));

        // 필터 없이 모든 어노테이션 조회
        let all_annotations = repository.find_by_study_uid_with_viewer(study_uid, None).await.unwrap();
        assert_eq!(all_annotations.len(), 2);

        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_viewer_software_case_sensitivity() {
        let (repository, pool) = setup_test_repository().await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 테스트 데이터 생성 (대소문자 구분)
        let annotation1 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test1"}),
            is_shared: false,
            viewer_software: Some("OHIF Viewer".to_string()),
            description: Some("Test annotation 1".to_string()),
        };

        let annotation2 = NewAnnotation {
            project_id,
            user_id,
            study_uid: "1.2.3.4.8".to_string(),
            series_uid: Some("1.2.3.4.9".to_string()),
            instance_uid: Some("1.2.3.4.10".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: serde_json::json!({"type": "test2"}),
            is_shared: false,
            viewer_software: Some("ohif viewer".to_string()), // 소문자
            description: Some("Test annotation 2".to_string()),
        };

        // 어노테이션들 생성
        repository.create(annotation1).await.unwrap();
        repository.create(annotation2).await.unwrap();

        // 대소문자 구분하여 필터링
        let ohif_annotations = repository.find_by_user_id_with_viewer(user_id, Some("OHIF Viewer")).await.unwrap();
        assert_eq!(ohif_annotations.len(), 1);

        let ohif_lower_annotations = repository.find_by_user_id_with_viewer(user_id, Some("ohif viewer")).await.unwrap();
        assert_eq!(ohif_lower_annotations.len(), 1);

        // 대소문자가 다른 경우 결과가 다름
        assert_ne!(ohif_annotations[0].id, ohif_lower_annotations[0].id);

        cleanup_test_data(&pool, user_id, project_id).await;
    }
}
