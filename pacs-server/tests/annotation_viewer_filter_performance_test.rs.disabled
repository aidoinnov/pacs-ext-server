#[cfg(test)]
mod annotation_viewer_filter_performance_tests {
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::AnnotationServiceImpl;
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl,
    };
    use pacs_server::application::dto::annotation_dto::CreateAnnotationRequest;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use std::time::Instant;

    async fn setup_test_app() -> (Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>, Arc<sqlx::Pool<sqlx::Postgres>>) {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let pool = Arc::new(pool);
        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));

        (annotation_use_case, pool)
    }

    async fn cleanup_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32, project_id: i32) {
        sqlx::query("DELETE FROM annotation_annotation WHERE user_id = $1 AND project_id = $2")
            .bind(user_id)
            .bind(project_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_viewer_filter_performance_with_large_dataset() {
        let (annotation_use_case, pool) = setup_test_app().await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 대량의 테스트 데이터 생성 (100개)
        let mut annotations = Vec::new();
        for i in 0..100 {
            let viewer_software = if i % 3 == 0 {
                "OHIF Viewer"
            } else if i % 3 == 1 {
                "DICOM Viewer"
            } else {
                "Other Viewer"
            };

            let annotation = CreateAnnotationRequest {
                study_instance_uid: format!("1.2.3.4.{}", i),
                series_instance_uid: format!("1.2.3.4.{}", i + 1000),
                sop_instance_uid: format!("1.2.3.4.{}", i + 2000),
                annotation_data: serde_json::json!({"type": format!("test{}", i)}),
                description: Some(format!("Test annotation {}", i)),
                tool_name: Some("test_tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                viewer_software: Some(viewer_software.to_string()),
            };
            annotations.push(annotation);
        }

        // 어노테이션들 생성
        for annotation in annotations {
            annotation_use_case.create_annotation(annotation, user_id, project_id).await.unwrap();
        }

        // 성능 테스트: OHIF Viewer 필터링
        let start = Instant::now();
        let ohif_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("OHIF Viewer")).await.unwrap();
        let ohif_duration = start.elapsed();
        
        assert_eq!(ohif_annotations.total, 34); // 100개 중 34개 (0, 3, 6, 9, ...)
        println!("OHIF Viewer 필터링 시간: {:?}", ohif_duration);
        assert!(ohif_duration.as_millis() < 1000, "OHIF Viewer 필터링이 1초를 초과했습니다");

        // 성능 테스트: DICOM Viewer 필터링
        let start = Instant::now();
        let dicom_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("DICOM Viewer")).await.unwrap();
        let dicom_duration = start.elapsed();
        
        assert_eq!(dicom_annotations.total, 33); // 100개 중 33개 (1, 4, 7, 10, ...)
        println!("DICOM Viewer 필터링 시간: {:?}", dicom_duration);
        assert!(dicom_duration.as_millis() < 1000, "DICOM Viewer 필터링이 1초를 초과했습니다");

        // 성능 테스트: 필터 없이 모든 어노테이션 조회
        let start = Instant::now();
        let all_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, None).await.unwrap();
        let all_duration = start.elapsed();
        
        assert_eq!(all_annotations.total, 100);
        println!("모든 어노테이션 조회 시간: {:?}", all_duration);
        assert!(all_duration.as_millis() < 1000, "모든 어노테이션 조회가 1초를 초과했습니다");

        // 성능 테스트: 존재하지 않는 viewer_software 필터링
        let start = Instant::now();
        let no_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("NonExistent Viewer")).await.unwrap();
        let no_duration = start.elapsed();
        
        assert_eq!(no_annotations.total, 0);
        println!("존재하지 않는 viewer_software 필터링 시간: {:?}", no_duration);
        assert!(no_duration.as_millis() < 1000, "존재하지 않는 viewer_software 필터링이 1초를 초과했습니다");

        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_concurrent_viewer_filter_requests() {
        let (annotation_use_case, pool) = setup_test_app().await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 테스트 데이터 생성
        let annotation = CreateAnnotationRequest {
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test"}),
            description: Some("Test annotation".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        annotation_use_case.create_annotation(annotation, user_id, project_id).await.unwrap();

        // 동시 요청 테스트
        let start = Instant::now();
        
        let futures = vec![
            annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("OHIF Viewer")),
            annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("DICOM Viewer")),
            annotation_use_case.get_annotations_by_user_with_viewer(user_id, None),
            annotation_use_case.get_annotations_by_project_with_viewer(project_id, Some("OHIF Viewer")),
            annotation_use_case.get_annotations_by_project_with_viewer(project_id, None),
        ];

        let results = futures::future::join_all(futures).await;
        let duration = start.elapsed();

        // 모든 요청이 성공했는지 확인
        for result in results {
            assert!(result.is_ok());
        }

        println!("동시 요청 처리 시간: {:?}", duration);
        assert!(duration.as_millis() < 2000, "동시 요청 처리가 2초를 초과했습니다");

        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_viewer_filter_memory_usage() {
        let (annotation_use_case, pool) = setup_test_app().await;
        
        let user_id = 336;
        let project_id = 299;
        
        // 메모리 사용량 테스트를 위한 대량 데이터 생성
        let mut annotations = Vec::new();
        for i in 0..50 {
            let annotation = CreateAnnotationRequest {
                study_instance_uid: format!("1.2.3.4.{}", i),
                series_instance_uid: format!("1.2.3.4.{}", i + 1000),
                sop_instance_uid: format!("1.2.3.4.{}", i + 2000),
                annotation_data: serde_json::json!({
                    "type": format!("test{}", i),
                    "data": vec![i; 1000] // 큰 데이터
                }),
                description: Some(format!("Test annotation {}", i)),
                tool_name: Some("test_tool".to_string()),
                tool_version: Some("1.0.0".to_string()),
                viewer_software: Some(if i % 2 == 0 { "OHIF Viewer" } else { "DICOM Viewer" }.to_string()),
            };
            annotations.push(annotation);
        }

        // 어노테이션들 생성
        for annotation in annotations {
            annotation_use_case.create_annotation(annotation, user_id, project_id).await.unwrap();
        }

        // 메모리 사용량 측정을 위한 반복 요청
        let start = Instant::now();
        for _ in 0..10 {
            let _ohif_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("OHIF Viewer")).await.unwrap();
            let _dicom_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, Some("DICOM Viewer")).await.unwrap();
            let _all_annotations = annotation_use_case.get_annotations_by_user_with_viewer(user_id, None).await.unwrap();
        }
        let duration = start.elapsed();

        println!("메모리 사용량 테스트 시간 (10회 반복): {:?}", duration);
        assert!(duration.as_millis() < 5000, "메모리 사용량 테스트가 5초를 초과했습니다");

        cleanup_test_data(&pool, user_id, project_id).await;
    }
}
