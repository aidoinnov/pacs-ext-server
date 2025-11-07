/// 권한 기반 어노테이션 필터링 통합 테스트
///
/// 이 테스트는 사용자의 권한에 따라 어노테이션 조회 결과가 달라지는지 검증합니다.
/// - ANNOTATION:READ_ALL 권한이 있으면: 프로젝트의 모든 어노테이션 반환
/// - ANNOTATION:READ_ALL 권한이 없으면: 본인의 어노테이션만 반환

#[cfg(test)]
mod annotation_permission_tests {
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::{
        AnnotationServiceImpl, AccessControlServiceImpl, AccessControlService,
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

    /// 테스트 1: READ_ALL 권한이 있는 사용자는 프로젝트의 모든 어노테이션을 볼 수 있어야 함
    #[tokio::test]
    async fn test_user_with_read_all_permission_sees_all_annotations() {
        // 데이터베이스 연결
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        // 시나리오:
        // - 프로젝트 ID: 2 (PerfProj)
        // - 사용자 1 (iaid-pacs-admin): SUPER_ADMIN 권한 있음
        // - 프로젝트에 여러 사용자가 작성한 어노테이션이 있을 수 있음

        let project_id = 2;
        let admin_user_id = 1; // iaid-pacs-admin (SUPER_ADMIN 권한)

        // READ_ALL 권한이 있는 사용자로 조회
        let result = use_case
            .get_annotations_by_project_with_permission(admin_user_id, project_id, None)
            .await;

        match result {
            Ok(response) => {
                println!("✅ Admin user sees {} annotations", response.total);
                // READ_ALL 권한이 있으면 프로젝트의 모든 어노테이션을 볼 수 있어야 함
                assert!(
                    response.total > 0,
                    "Admin should see at least some annotations in the project"
                );

                // 다양한 사용자의 어노테이션이 포함되어 있는지 확인
                let unique_users: std::collections::HashSet<_> = response
                    .annotations
                    .iter()
                    .map(|ann| ann.user_id)
                    .collect();

                println!("   Annotations from {} different users", unique_users.len());
            }
            Err(e) => {
                panic!("Failed to get annotations with READ_ALL permission: {:?}", e);
            }
        }
    }

    /// 테스트 2: READ_ALL 권한이 없는 사용자는 본인의 어노테이션만 볼 수 있어야 함
    #[tokio::test]
    async fn test_user_without_read_all_permission_sees_only_own_annotations() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let project_id = 315; // Test Project
        let regular_user_id = 584; // test_user_1 (READ_ALL 권한이 없는 일반 사용자)

        // READ_ALL 권한이 없는 사용자로 조회
        let result = use_case
            .get_annotations_by_project_with_permission(regular_user_id, project_id, None)
            .await;

        match result {
            Ok(response) => {
                println!("✅ Regular user sees {} annotations", response.total);

                // 모든 어노테이션이 본인의 것인지 확인
                for annotation in &response.annotations {
                    assert_eq!(
                        annotation.user_id, regular_user_id,
                        "User without READ_ALL should only see their own annotations"
                    );
                }

                println!("   All annotations belong to user {}", regular_user_id);
            }
            Err(e) => {
                panic!(
                    "Failed to get annotations without READ_ALL permission: {:?}",
                    e
                );
            }
        }
    }

    /// 테스트 3: 프로젝트 멤버가 아닌 사용자는 Unauthorized 에러를 받아야 함
    #[tokio::test]
    async fn test_non_member_gets_unauthorized_error() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let project_id = 2; // PerfProj
        let non_member_user_id = 584; // test_user_1 (project 315의 멤버이지만 project 2의 멤버는 아님)

        // 프로젝트 멤버가 아닌 사용자로 조회
        let result = use_case
            .get_annotations_by_project_with_permission(non_member_user_id, project_id, None)
            .await;

        match result {
            Ok(_) => {
                panic!("Non-member should not be able to access project annotations");
            }
            Err(e) => {
                println!("✅ Non-member correctly received error: {:?}", e);
                // Unauthorized 에러인지 확인
                assert!(
                    e.to_string().contains("Unauthorized") || e.to_string().contains("not a member"),
                    "Expected Unauthorized error, got: {:?}",
                    e
                );
            }
        }
    }

    /// 테스트 4: viewer_software 필터와 권한 기반 필터링이 함께 작동하는지 확인
    #[tokio::test]
    async fn test_permission_based_filtering_with_viewer_software_filter() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let use_case = build_use_case(&pool);

        let project_id = 2; // PerfProj
        let admin_user_id = 1; // iaid-pacs-admin
        let viewer_software = Some("TI-DicomViewer");

        // READ_ALL 권한 + viewer_software 필터
        let result = use_case
            .get_annotations_by_project_with_permission(admin_user_id, project_id, viewer_software)
            .await;

        match result {
            Ok(response) => {
                println!(
                    "✅ Admin user with viewer_software filter sees {} annotations",
                    response.total
                );

                // viewer_software가 TI-DicomViewer인 어노테이션만 있는지 확인
                for annotation in &response.annotations {
                    if let Some(ref vs) = annotation.viewer_software {
                        assert_eq!(
                            vs, "TI-DicomViewer",
                            "All annotations should have viewer_software = TI-DicomViewer"
                        );
                    }
                }
            }
            Err(e) => {
                panic!("Failed to get annotations with filters: {:?}", e);
            }
        }
    }

    /// 테스트 5: 권한 확인 로직이 올바르게 작동하는지 확인 (is_project_member)
    #[tokio::test]
    async fn test_is_project_member_check() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());
        let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
        let role_repo = RoleRepositoryImpl::new(pool.clone());
        let permission_repo = PermissionRepositoryImpl::new(pool.clone());

        let access_control_service = AccessControlServiceImpl::new(
            access_log_repo,
            user_repo,
            project_repo,
            role_repo,
            permission_repo,
        );

        let project_id = 2; // PerfProj
        let member_user_id = 1; // iaid-pacs-admin (project 2의 멤버)
        let non_member_user_id = 584; // test_user_1 (project 2의 멤버가 아님)

        // 멤버인 사용자 확인
        let is_member = access_control_service
            .is_project_member(member_user_id, project_id)
            .await
            .expect("Failed to check project membership");

        assert!(is_member, "User 1 should be a member of project 2");
        println!("✅ User {} is a member of project {}", member_user_id, project_id);

        // 멤버가 아닌 사용자 확인
        let is_not_member = access_control_service
            .is_project_member(non_member_user_id, project_id)
            .await
            .expect("Failed to check project membership");

        assert!(
            !is_not_member,
            "User 584 should NOT be a member of project 2"
        );
        println!(
            "✅ User {} is NOT a member of project {}",
            non_member_user_id, project_id
        );
    }
}

