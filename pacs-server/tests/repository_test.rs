use pacs_server::domain::entities::*;
use pacs_server::domain::repositories::*;
use pacs_server::infrastructure::repositories::*;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use serde_json::json;

// Helper function to get test database pool
async fn get_test_pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[cfg(test)]
mod user_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_user() {
        let pool = get_test_pool().await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };

        // Create user
        let created = repo.create(new_user.clone()).await.unwrap();
        assert_eq!(created.username, new_user.username);
        assert_eq!(created.email, new_user.email);

        // Find by ID
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, new_user.username);

        // Find by username
        let found = repo.find_by_username(&new_user.username).await.unwrap();
        assert!(found.is_some());

        // Find by email
        let found = repo.find_by_email(&new_user.email).await.unwrap();
        assert!(found.is_some());

        // Find by keycloak_id
        let found = repo.find_by_keycloak_id(new_user.keycloak_id).await.unwrap();
        assert!(found.is_some());

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_all_users() {
        let pool = get_test_pool().await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let initial_count = repo.find_all().await.unwrap().len();

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };

        let created = repo.create(new_user).await.unwrap();

        let all_users = repo.find_all().await.unwrap();
        assert!(all_users.len() >= initial_count + 1);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = get_test_pool().await;
        let repo = UserRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };

        let created = repo.create(new_user).await.unwrap();
        let deleted = repo.delete(created.id).await.unwrap();
        assert!(deleted);

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());
    }
}

#[cfg(test)]
mod project_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_project() {
        let pool = get_test_pool().await;
        let repo = ProjectRepositoryImpl::new(pool.clone());

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: Some("Test Description".to_string()),
        };

        // Create project
        let created = repo.create(new_project.clone()).await.unwrap();
        assert_eq!(created.name, new_project.name);
        assert_eq!(created.description, new_project.description);
        assert!(created.is_active);

        // Find by ID
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());

        // Find by name
        let found = repo.find_by_name(&new_project.name).await.unwrap();
        assert!(found.is_some());

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_project() {
        let pool = get_test_pool().await;
        let repo = ProjectRepositoryImpl::new(pool.clone());

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: Some("Original Description".to_string()),
        };

        let created = repo.create(new_project).await.unwrap();

        let updated_project = NewProject {
            name: format!("Updated Project {}", Uuid::new_v4()),
            description: Some("Updated Description".to_string()),
        };

        let updated = repo.update(created.id, updated_project.clone()).await.unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.name, updated_project.name);
        assert_eq!(updated.description, updated_project.description);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_set_active_project() {
        let pool = get_test_pool().await;
        let repo = ProjectRepositoryImpl::new(pool.clone());

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: None,
        };

        let created = repo.create(new_project).await.unwrap();
        assert!(created.is_active);

        // Set inactive
        let result = repo.set_active(created.id, false).await.unwrap();
        assert!(result);

        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert!(!found.is_active);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_active_projects() {
        let pool = get_test_pool().await;
        let repo = ProjectRepositoryImpl::new(pool.clone());

        let new_project = NewProject {
            name: format!("Active Project {}", Uuid::new_v4()),
            description: None,
        };

        let created = repo.create(new_project).await.unwrap();
        let active_projects = repo.find_active().await.unwrap();
        assert!(active_projects.iter().any(|p| p.id == created.id));

        // Set inactive and check
        repo.set_active(created.id, false).await.unwrap();
        let active_projects = repo.find_active().await.unwrap();
        assert!(!active_projects.iter().any(|p| p.id == created.id));

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }
}

#[cfg(test)]
mod role_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_role() {
        let pool = get_test_pool().await;
        let repo = RoleRepositoryImpl::new(pool.clone());

        let new_role = NewRole {
            name: format!("Test Role {}", Uuid::new_v4()),
            description: Some("Test Role Description".to_string()),
            scope: RoleScope::Global,
        };

        // Create role
        let created = repo.create(new_role.clone()).await.unwrap();
        assert_eq!(created.name, new_role.name);
        assert_eq!(created.scope, "GLOBAL");

        // Find by ID
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());

        // Find by name
        let found = repo.find_by_name(&new_role.name).await.unwrap();
        assert!(found.is_some());

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_scope() {
        let pool = get_test_pool().await;
        let repo = RoleRepositoryImpl::new(pool.clone());

        let global_role = NewRole {
            name: format!("Global Role {}", Uuid::new_v4()),
            description: None,
            scope: RoleScope::Global,
        };

        let project_role = NewRole {
            name: format!("Project Role {}", Uuid::new_v4()),
            description: None,
            scope: RoleScope::Project,
        };

        let created_global = repo.create(global_role).await.unwrap();
        let created_project = repo.create(project_role).await.unwrap();

        let global_roles = repo.find_by_scope("GLOBAL").await.unwrap();
        assert!(global_roles.iter().any(|r| r.id == created_global.id));

        let project_roles = repo.find_by_scope("PROJECT").await.unwrap();
        assert!(project_roles.iter().any(|r| r.id == created_project.id));

        // Cleanup
        repo.delete(created_global.id).await.unwrap();
        repo.delete(created_project.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_role() {
        let pool = get_test_pool().await;
        let repo = RoleRepositoryImpl::new(pool.clone());

        let new_role = NewRole {
            name: format!("Original Role {}", Uuid::new_v4()),
            description: Some("Original Description".to_string()),
            scope: RoleScope::Global,
        };

        let created = repo.create(new_role).await.unwrap();

        let updated_role = NewRole {
            name: format!("Updated Role {}", Uuid::new_v4()),
            description: Some("Updated Description".to_string()),
            scope: RoleScope::Project,
        };

        let updated = repo.update(created.id, updated_role.clone()).await.unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.name, updated_role.name);
        assert_eq!(updated.scope, "PROJECT");

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }
}

#[cfg(test)]
mod permission_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_permission() {
        let pool = get_test_pool().await;
        let repo = PermissionRepositoryImpl::new(pool.clone());

        let new_permission = NewPermission {
            resource_type: format!("TEST_RESOURCE_{}", Uuid::new_v4()),
            action: "READ".to_string(),
        };

        // Create permission
        let created = repo.create(new_permission.clone()).await.unwrap();
        assert_eq!(created.resource_type, new_permission.resource_type);
        assert_eq!(created.action, new_permission.action);

        // Find by ID
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());

        // Find by resource and action
        let found = repo
            .find_by_resource_and_action(&new_permission.resource_type, &new_permission.action)
            .await
            .unwrap();
        assert!(found.is_some());

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_resource_type() {
        let pool = get_test_pool().await;
        let repo = PermissionRepositoryImpl::new(pool.clone());

        let resource_type = format!("STUDY_{}", Uuid::new_v4());

        let perm1 = NewPermission {
            resource_type: resource_type.clone(),
            action: "READ".to_string(),
        };
        let perm2 = NewPermission {
            resource_type: resource_type.clone(),
            action: "WRITE".to_string(),
        };

        let created1 = repo.create(perm1).await.unwrap();
        let created2 = repo.create(perm2).await.unwrap();

        let permissions = repo.find_by_resource_type(&resource_type).await.unwrap();
        assert_eq!(permissions.len(), 2);

        // Cleanup
        repo.delete(created1.id).await.unwrap();
        repo.delete(created2.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_all_permissions() {
        let pool = get_test_pool().await;
        let repo = PermissionRepositoryImpl::new(pool.clone());

        let initial_count = repo.find_all().await.unwrap().len();

        let new_permission = NewPermission {
            resource_type: format!("TEST_{}", Uuid::new_v4()),
            action: "TEST".to_string(),
        };

        let created = repo.create(new_permission).await.unwrap();

        let all_permissions = repo.find_all().await.unwrap();
        assert_eq!(all_permissions.len(), initial_count + 1);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }
}

#[cfg(test)]
mod access_log_repository_tests {
    use super::*;
    use chrono::{NaiveDateTime, Utc};

    #[tokio::test]
    async fn test_create_access_log() {
        let pool = get_test_pool().await;
        let repo = AccessLogRepositoryImpl::new(pool.clone());

        // First create a user for the log
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("loguser_{}", Uuid::new_v4()),
            email: format!("log_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let new_log = NewAccessLog {
            user_id: user.id,
            project_id: None,
            resource_type: "STUDY".to_string(),
            study_uid: Some("1.2.3.4.5".to_string()),
            series_uid: None,
            instance_uid: None,
            action: "VIEW".to_string(),
            result: "SUCCESS".to_string(),
            dicom_tag_check: None,
            ae_title: Some("TEST_AE".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            session_id: Some("session123".to_string()),
            via_group_id: None,
        };

        let created = repo.create(new_log.clone()).await.unwrap();
        assert_eq!(created.user_id, user.id);
        assert_eq!(created.action, "VIEW");
        assert_eq!(created.result, "SUCCESS");

        // Cleanup - delete logs first, then user
        sqlx::query("DELETE FROM security_access_log WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let pool = get_test_pool().await;
        let repo = AccessLogRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("loguser_{}", Uuid::new_v4()),
            email: format!("log_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let new_log = NewAccessLog {
            user_id: user.id,
            project_id: None,
            resource_type: "STUDY".to_string(),
            study_uid: Some("1.2.3.4.5".to_string()),
            series_uid: None,
            instance_uid: None,
            action: "VIEW".to_string(),
            result: "SUCCESS".to_string(),
            dicom_tag_check: None,
            ae_title: None,
            ip_address: None,
            session_id: None,
            via_group_id: None,
        };

        repo.create(new_log).await.unwrap();

        let logs = repo.find_by_user_id(user.id, 10).await.unwrap();
        assert!(logs.len() > 0);
        assert!(logs.iter().all(|l| l.user_id == user.id));

        // Cleanup - delete logs first, then user
        sqlx::query("DELETE FROM security_access_log WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_count_by_user_id() {
        let pool = get_test_pool().await;
        let repo = AccessLogRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("loguser_{}", Uuid::new_v4()),
            email: format!("log_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let initial_count = repo.count_by_user_id(user.id).await.unwrap();

        let new_log = NewAccessLog {
            user_id: user.id,
            project_id: None,
            resource_type: "STUDY".to_string(),
            study_uid: Some("1.2.3.4.5".to_string()),
            series_uid: None,
            instance_uid: None,
            action: "VIEW".to_string(),
            result: "SUCCESS".to_string(),
            dicom_tag_check: None,
            ae_title: None,
            ip_address: None,
            session_id: None,
            via_group_id: None,
        };

        repo.create(new_log).await.unwrap();

        let count = repo.count_by_user_id(user.id).await.unwrap();
        assert_eq!(count, initial_count + 1);

        // Cleanup - delete logs first, then user
        sqlx::query("DELETE FROM security_access_log WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
    }
}

#[cfg(test)]
mod annotation_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_annotation() {
        let pool = get_test_pool().await;
        let repo = AnnotationRepositoryImpl::new(pool.clone());

        // First create a user and project
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: Some("Test Description".to_string()),
        };
        let project = project_repo.create(new_project).await.unwrap();

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user.id)
        .bind(project.id)
        .execute(&pool)
        .await
        .unwrap();

        let new_annotation = NewAnnotation {
            project_id: project.id,
            user_id: user.id,
            study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_uid: Some("1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string()),
            instance_uid: Some("1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            is_shared: false,
        };

        // Create annotation
        let created = repo.create(new_annotation.clone()).await.unwrap();
        assert_eq!(created.project_id, new_annotation.project_id);
        assert_eq!(created.user_id, new_annotation.user_id);
        assert_eq!(created.study_uid, new_annotation.study_uid);
        assert_eq!(created.tool_name, new_annotation.tool_name);
        assert_eq!(created.is_shared, new_annotation.is_shared);

        // Find by ID
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);

        // Find by user ID
        let found = repo.find_by_user_id(user.id).await.unwrap();
        assert!(found.len() > 0);
        assert!(found.iter().any(|a| a.id == created.id));

        // Find by project ID
        let found = repo.find_by_project_id(project.id).await.unwrap();
        assert!(found.len() > 0);
        assert!(found.iter().any(|a| a.id == created.id));

        // Find by study UID
        let found = repo.find_by_study_uid(&new_annotation.study_uid).await.unwrap();
        assert!(found.len() > 0);
        assert!(found.iter().any(|a| a.id == created.id));

        // Cleanup
        repo.delete(created.id).await.unwrap();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
        project_repo.delete(project.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_annotation() {
        let pool = get_test_pool().await;
        let repo = AnnotationRepositoryImpl::new(pool.clone());

        // Create user and project
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: Some("Test Description".to_string()),
        };
        let project = project_repo.create(new_project).await.unwrap();

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user.id)
        .bind(project.id)
        .execute(&pool)
        .await
        .unwrap();

        let new_annotation = NewAnnotation {
            project_id: project.id,
            user_id: user.id,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            is_shared: false,
        };

        let created = repo.create(new_annotation).await.unwrap();

        // Update annotation
        let updated_data = json!({"type": "rectangle", "x": 200, "y": 300, "width": 100, "height": 80});
        let updated = repo.update(created.id, updated_data.clone(), false).await.unwrap();
        assert_eq!(updated.unwrap().data, updated_data);

        // Cleanup
        repo.delete(created.id).await.unwrap();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
        project_repo.delete(project.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_annotation() {
        let pool = get_test_pool().await;
        let repo = AnnotationRepositoryImpl::new(pool.clone());

        // Create user and project
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: Some("Test Description".to_string()),
        };
        let project = project_repo.create(new_project).await.unwrap();

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user.id)
        .bind(project.id)
        .execute(&pool)
        .await
        .unwrap();

        let new_annotation = NewAnnotation {
            project_id: project.id,
            user_id: user.id,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            is_shared: false,
        };

        let created = repo.create(new_annotation).await.unwrap();
        let deleted = repo.delete(created.id).await.unwrap();
        assert!(deleted);

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        // Cleanup
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
        project_repo.delete(project.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_annotation_history() {
        let pool = get_test_pool().await;
        let repo = AnnotationRepositoryImpl::new(pool.clone());

        // Create user and project
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let new_user = NewUser {
            keycloak_id: Uuid::new_v4(),
            username: format!("testuser_{}", Uuid::new_v4()),
            email: format!("test_{}@example.com", Uuid::new_v4()),
        };
        let user = user_repo.create(new_user).await.unwrap();

        let new_project = NewProject {
            name: format!("Test Project {}", Uuid::new_v4()),
            description: Some("Test Description".to_string()),
        };
        let project = project_repo.create(new_project).await.unwrap();

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user.id)
        .bind(project.id)
        .execute(&pool)
        .await
        .unwrap();

        let new_annotation = NewAnnotation {
            project_id: project.id,
            user_id: user.id,
            study_uid: "1.2.3.4.5".to_string(),
            series_uid: Some("1.2.3.4.6".to_string()),
            instance_uid: Some("1.2.3.4.7".to_string()),
            tool_name: "test_tool".to_string(),
            tool_version: Some("1.0.0".to_string()),
            data: json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            is_shared: false,
        };

        let created = repo.create(new_annotation).await.unwrap();

        // Create history entry
        let created_history = repo.create_history(
            created.id,
            user.id,
            "CREATE",
            None,
            Some(created.data.clone())
        ).await.unwrap();
        assert_eq!(created_history.annotation_id, created.id);
        assert_eq!(created_history.action, "CREATE");

        // Find history by annotation ID
        let history_entries = repo.get_history(created.id).await.unwrap();
        assert!(history_entries.len() > 0);
        assert!(history_entries.iter().any(|h| h.id == created_history.id));

        // Cleanup
        repo.delete(created.id).await.unwrap();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();
        user_repo.delete(user.id).await.unwrap();
        project_repo.delete(project.id).await.unwrap();
    }
}
