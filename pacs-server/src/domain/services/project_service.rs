use crate::application::dto::project_dto::ProjectListQuery;
use crate::domain::entities::{NewProject, Project, ProjectStatus, Role, UpdateProject, User};
use crate::domain::repositories::{ProjectRepository, RoleRepository, UserRepository};
use crate::domain::ServiceError;
use async_trait::async_trait;

/// 프로젝트 관리 도메인 서비스
#[async_trait]
pub trait ProjectService: Send + Sync {
    /// 프로젝트 생성
    async fn create_project(&self, new_project: NewProject) -> Result<Project, ServiceError>;

    /// 프로젝트 조회
    async fn get_project(&self, id: i32) -> Result<Project, ServiceError>;

    /// 프로젝트 이름으로 조회
    async fn get_project_by_name(&self, name: &str) -> Result<Project, ServiceError>;

    /// 모든 프로젝트 조회
    async fn get_all_projects(&self) -> Result<Vec<Project>, ServiceError>;

    /// 페이지네이션된 프로젝트 조회
    async fn get_projects_paginated(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, ServiceError>;

    /// 필터링된 프로젝트 조회
    async fn get_projects_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<Vec<Project>, ServiceError>;

    /// 활성화된 프로젝트만 조회
    async fn get_active_projects(&self) -> Result<Vec<Project>, ServiceError>;

    /// 페이지네이션된 활성 프로젝트 조회
    async fn get_active_projects_paginated(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, ServiceError>;

    /// 전체 프로젝트 개수 조회
    async fn count_all_projects(&self) -> Result<i64, ServiceError>;

    /// 활성 프로젝트 개수 조회
    async fn count_active_projects(&self) -> Result<i64, ServiceError>;

    /// 필터링된 프로젝트 개수 조회
    async fn count_projects_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<i64, ServiceError>;

    /// 프로젝트 활성화
    async fn activate_project(&self, id: i32) -> Result<Project, ServiceError>;

    /// 프로젝트 비활성화
    async fn deactivate_project(&self, id: i32) -> Result<Project, ServiceError>;

    /// 프로젝트 수정
    async fn update_project(&self, id: i32, update: UpdateProject)
        -> Result<Project, ServiceError>;

    /// 프로젝트 삭제
    async fn delete_project(&self, id: i32) -> Result<(), ServiceError>;

    // === 멤버 관리 ===

    /// 프로젝트의 멤버 목록 조회
    async fn get_project_members(&self, project_id: i32) -> Result<Vec<User>, ServiceError>;

    /// 프로젝트 멤버 수 조회
    async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError>;

    // === 역할 관리 ===

    /// 프로젝트에 역할 할당
    async fn assign_role_to_project(
        &self,
        project_id: i32,
        role_id: i32,
    ) -> Result<(), ServiceError>;

    /// 프로젝트에서 역할 제거
    async fn remove_role_from_project(
        &self,
        project_id: i32,
        role_id: i32,
    ) -> Result<(), ServiceError>;

    /// 프로젝트에 할당된 역할 목록 조회
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>, ServiceError>;

    // === 사용자-프로젝트 역할 관리 ===

    /// 프로젝트 멤버 목록 조회 (역할 정보 포함, 페이지네이션)
    async fn get_project_members_with_roles(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<
        (
            Vec<crate::application::dto::project_user_dto::UserWithRoleResponse>,
            i64,
        ),
        ServiceError,
    >;

    /// 프로젝트 내 사용자에게 역할 할당
    async fn assign_user_role_in_project(
        &self,
        project_id: i32,
        user_id: i32,
        role_id: i32,
    ) -> Result<(), ServiceError>;

    // === 매트릭스 API 지원 ===

    /// 상태 필터로 프로젝트 조회 (페이지네이션)
    async fn get_projects_with_status_filter(
        &self,
        statuses: Option<Vec<ProjectStatus>>,
        project_ids: Option<Vec<i32>>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<Project>, i64), ServiceError>;

    /// 매트릭스용 사용자-프로젝트-역할 관계 조회
    async fn get_user_project_roles_matrix(
        &self,
        project_ids: Vec<i32>,
        user_ids: Vec<i32>,
    ) -> Result<Vec<UserProjectRoleInfo>, ServiceError>;
}

/// 매트릭스용 사용자-프로젝트-역할 정보
#[derive(Debug, Clone)]
pub struct UserProjectRoleInfo {
    pub project_id: i32,
    pub user_id: i32,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
}

#[derive(Clone)]
pub struct ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
    project_repository: P,
    user_repository: U,
    role_repository: R,
}

impl<P, U, R> ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
    pub fn new(project_repository: P, user_repository: U, role_repository: R) -> Self {
        Self {
            project_repository,
            user_repository,
            role_repository,
        }
    }
}

#[async_trait]
impl<P, U, R> ProjectService for ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
    async fn create_project(&self, new_project: NewProject) -> Result<Project, ServiceError> {
        // 프로젝트 이름 중복 체크
        if let Some(_) = self
            .project_repository
            .find_by_name(&new_project.name)
            .await?
        {
            return Err(ServiceError::AlreadyExists(
                "Project name already exists".into(),
            ));
        }

        // 프로젝트 이름 검증
        if new_project.name.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Project name cannot be empty".into(),
            ));
        }

        if new_project.name.len() > 255 {
            return Err(ServiceError::ValidationError(
                "Project name too long (max 255 characters)".into(),
            ));
        }

        Ok(self.project_repository.create(new_project).await?)
    }

    async fn get_project(&self, id: i32) -> Result<Project, ServiceError> {
        self.project_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Project not found".into()))
    }

    async fn get_project_by_name(&self, name: &str) -> Result<Project, ServiceError> {
        self.project_repository
            .find_by_name(name)
            .await?
            .ok_or(ServiceError::NotFound("Project not found".into()))
    }

    async fn get_all_projects(&self) -> Result<Vec<Project>, ServiceError> {
        Ok(self.project_repository.find_all().await?)
    }

    async fn get_projects_paginated(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, ServiceError> {
        Ok(self
            .project_repository
            .find_with_pagination(page, page_size, sort_by, sort_order)
            .await?)
    }

    async fn get_projects_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<Vec<Project>, ServiceError> {
        Ok(self.project_repository.find_with_filter(query).await?)
    }

    async fn get_active_projects(&self) -> Result<Vec<Project>, ServiceError> {
        Ok(self.project_repository.find_active().await?)
    }

    async fn get_active_projects_paginated(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, ServiceError> {
        Ok(self
            .project_repository
            .find_active_with_pagination(page, page_size, sort_by, sort_order)
            .await?)
    }

    async fn count_all_projects(&self) -> Result<i64, ServiceError> {
        Ok(self.project_repository.count_all().await?)
    }

    async fn count_active_projects(&self) -> Result<i64, ServiceError> {
        Ok(self.project_repository.count_active().await?)
    }

    async fn count_projects_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<i64, ServiceError> {
        Ok(self.project_repository.count_with_filter(query).await?)
    }

    async fn activate_project(&self, id: i32) -> Result<Project, ServiceError> {
        // RETURNING 절로 원자적 처리
        let project = sqlx::query_as::<_, Project>(
            "UPDATE security_project
             SET is_active = true
             WHERE id = $1
             RETURNING id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at",
        )
        .bind(id)
        .fetch_optional(self.project_repository.pool())
        .await?
        .ok_or(ServiceError::NotFound("Project not found".into()))?;

        Ok(project)
    }

    async fn deactivate_project(&self, id: i32) -> Result<Project, ServiceError> {
        // RETURNING 절로 원자적 처리
        let project = sqlx::query_as::<_, Project>(
            "UPDATE security_project
             SET is_active = false
             WHERE id = $1
             RETURNING id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at",
        )
        .bind(id)
        .fetch_optional(self.project_repository.pool())
        .await?
        .ok_or(ServiceError::NotFound("Project not found".into()))?;

        Ok(project)
    }

    async fn update_project(
        &self,
        id: i32,
        update: UpdateProject,
    ) -> Result<Project, ServiceError> {
        self.project_repository
            .update(id, &update)
            .await?
            .ok_or(ServiceError::NotFound("Project not found".into()))
    }

    async fn delete_project(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self.project_repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Project not found".into()))
        }
    }

    // === 멤버 관리 구현 ===

    async fn get_project_members(&self, project_id: i32) -> Result<Vec<User>, ServiceError> {
        // 프로젝트 존재 확인
        if self
            .project_repository
            .find_by_id(project_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let members = sqlx::query_as::<_, User>(
            "SELECT u.id, u.keycloak_id, u.username, u.email, u.created_at
             FROM security_user u
             INNER JOIN security_user_project up ON u.id = up.user_id
             WHERE up.project_id = $1
             ORDER BY u.username",
        )
        .bind(project_id)
        .fetch_all(self.project_repository.pool())
        .await?;

        Ok(members)
    }

    async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError> {
        // 프로젝트 존재 확인
        if self
            .project_repository
            .find_by_id(project_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(self.project_repository.pool())
        .await?;

        Ok(count)
    }

    // === 역할 관리 구현 ===

    async fn assign_role_to_project(
        &self,
        project_id: i32,
        role_id: i32,
    ) -> Result<(), ServiceError> {
        // INSERT with ON CONFLICT - Race condition 방지
        let result = sqlx::query(
            "INSERT INTO security_project_role (project_id, role_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_project WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_role WHERE id = $2)
             ON CONFLICT (project_id, role_id) DO NOTHING
             RETURNING project_id",
        )
        .bind(project_id)
        .bind(role_id)
        .fetch_optional(self.project_repository.pool())
        .await?;

        match result {
            Some(_) => Ok(()),
            None => {
                // 실패 원인 파악
                if self
                    .project_repository
                    .find_by_id(project_id)
                    .await?
                    .is_none()
                {
                    return Err(ServiceError::NotFound("Project not found".into()));
                }
                if self.role_repository.find_by_id(role_id).await?.is_none() {
                    return Err(ServiceError::NotFound("Role not found".into()));
                }
                Err(ServiceError::AlreadyExists(
                    "Role already assigned to this project".into(),
                ))
            }
        }
    }

    async fn remove_role_from_project(
        &self,
        project_id: i32,
        role_id: i32,
    ) -> Result<(), ServiceError> {
        let result =
            sqlx::query("DELETE FROM security_project_role WHERE project_id = $1 AND role_id = $2")
                .bind(project_id)
                .bind(role_id)
                .execute(self.project_repository.pool())
                .await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Role is not assigned to this project".into(),
            ))
        }
    }

    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>, ServiceError> {
        // 프로젝트 존재 확인
        if self
            .project_repository
            .find_by_id(project_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let roles = sqlx::query_as::<_, Role>(
            "SELECT r.id, r.name, r.description, r.scope, r.created_at
             FROM security_role r
             INNER JOIN security_project_role pr ON r.id = pr.role_id
             WHERE pr.project_id = $1
             ORDER BY r.name",
        )
        .bind(project_id)
        .fetch_all(self.project_repository.pool())
        .await?;

        Ok(roles)
    }

    // === 사용자-프로젝트 역할 관리 구현 ===

    async fn get_project_members_with_roles(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<
        (
            Vec<crate::application::dto::project_user_dto::UserWithRoleResponse>,
            i64,
        ),
        ServiceError,
    > {
        // 프로젝트 존재 확인
        if self
            .project_repository
            .find_by_id(project_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let offset = (page - 1) * page_size;

        // 프로젝트 멤버와 역할 정보를 함께 조회
        let users_with_roles = sqlx::query_as::<
            _,
            (
                i32,
                String,
                String,
                Option<String>,
                Option<i32>,
                Option<String>,
                Option<String>,
            ),
        >(
            "SELECT 
                u.id as user_id, u.username, u.email, u.full_name,
                r.id as role_id, r.name as role_name, r.scope as role_scope
             FROM security_user u
             INNER JOIN security_user_project up ON u.id = up.user_id
             LEFT JOIN security_role r ON up.role_id = r.id
             WHERE up.project_id = $1
             ORDER BY u.username
             LIMIT $2 OFFSET $3",
        )
        .bind(project_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.project_repository.pool())
        .await?;

        // 총 개수 조회
        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(self.project_repository.pool())
        .await?;

        // DTO로 변환
        let users: Vec<crate::application::dto::project_user_dto::UserWithRoleResponse> =
            users_with_roles
                .into_iter()
                .map(
                    |(user_id, username, email, full_name, role_id, role_name, role_scope)| {
                        crate::application::dto::project_user_dto::UserWithRoleResponse {
                            user_id,
                            username,
                            email,
                            full_name,
                            role_id,
                            role_name,
                            role_scope,
                        }
                    },
                )
                .collect();

        Ok((users, total_count))
    }

    async fn assign_user_role_in_project(
        &self,
        project_id: i32,
        user_id: i32,
        role_id: i32,
    ) -> Result<(), ServiceError> {
        println!("000");
        // 프로젝트 존재 확인
        if self
            .project_repository
            .find_by_id(project_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 역할 존재 확인
        if self.role_repository.find_by_id(role_id).await?.is_none() {
            return Err(ServiceError::NotFound("Role not found".into()));
        }
        println!("111");
        // 사용자가 프로젝트 멤버인지 확인
        let is_member = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(self.project_repository.pool())
        .await?;
        println!("222");
        if !is_member {
            return Err(ServiceError::NotFound(
                "User is not a member of this project".into(),
            ));
        }
        println!("333");
        // 역할 할당 (UPDATE)
        let result = sqlx::query(
            "UPDATE security_user_project 
             SET role_id = $1 
             WHERE user_id = $2 AND project_id = $3",
        )
        .bind(role_id)
        .bind(user_id)
        .bind(project_id)
        .execute(self.project_repository.pool())
        .await?;
        println!("444");
        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Failed to assign role to user".into(),
            ))
        }
    }

    // === 매트릭스 API 지원 구현 ===

    async fn get_projects_with_status_filter(
        &self,
        statuses: Option<Vec<ProjectStatus>>,
        project_ids: Option<Vec<i32>>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<Project>, i64), ServiceError> {
        let offset = (page - 1) * page_size;

        // Convert ProjectStatus enum to strings for SQL query
        let status_strings: Option<Vec<String>> = statuses.map(|statuses| {
            statuses
                .into_iter()
                .map(|status| match status {
                    ProjectStatus::Planning => "PLANNING".to_string(),
                    ProjectStatus::Active => "ACTIVE".to_string(),
                    ProjectStatus::Completed => "COMPLETED".to_string(),
                    ProjectStatus::Suspended => "SUSPENDED".to_string(),
                    ProjectStatus::Cancelled => "CANCELLED".to_string(),
                    ProjectStatus::PendingCompletion => "PENDING_COMPLETION".to_string(),
                    ProjectStatus::OverPlanning => "OVER_PLANNING".to_string(),
                })
                .collect()
        });

        // 프로젝트 조회 쿼리
        let projects = sqlx::query_as::<_, Project>(
            "SELECT id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at
             FROM security_project
             WHERE ($1::text[] IS NULL OR status::text = ANY($1))
               AND ($2::int[] IS NULL OR id = ANY($2))
             ORDER BY name
             LIMIT $3 OFFSET $4",
        )
        .bind(&status_strings)
        .bind(&project_ids)
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.project_repository.pool())
        .await?;

        // 총 개수 조회
        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM security_project
             WHERE ($1::text[] IS NULL OR status::text = ANY($1))
               AND ($2::int[] IS NULL OR id = ANY($2))",
        )
        .bind(&status_strings)
        .bind(&project_ids)
        .fetch_one(self.project_repository.pool())
        .await?;

        Ok((projects, total_count))
    }

    async fn get_user_project_roles_matrix(
        &self,
        project_ids: Vec<i32>,
        user_ids: Vec<i32>,
    ) -> Result<Vec<UserProjectRoleInfo>, ServiceError> {
        let relationships = sqlx::query_as::<_, (i32, i32, Option<i32>, Option<String>)>(
            "SELECT 
                p.id as project_id,
                u.id as user_id,
                up.role_id,
                r.name as role_name
             FROM security_project p
             CROSS JOIN security_user u
             LEFT JOIN security_user_project up ON p.id = up.project_id AND u.id = up.user_id
             LEFT JOIN security_role r ON up.role_id = r.id
             WHERE p.id = ANY($1)
               AND u.id = ANY($2)
             ORDER BY p.name, u.username",
        )
        .bind(&project_ids)
        .bind(&user_ids)
        .fetch_all(self.project_repository.pool())
        .await?;

        let result: Vec<UserProjectRoleInfo> = relationships
            .into_iter()
            .map(
                |(project_id, user_id, role_id, role_name)| UserProjectRoleInfo {
                    project_id,
                    user_id,
                    role_id,
                    role_name,
                },
            )
            .collect();

        Ok(result)
    }
}
