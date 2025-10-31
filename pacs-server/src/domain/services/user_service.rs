use crate::domain::entities::{NewUser, Project, UpdateUser, User};
use crate::domain::repositories::{ProjectRepository, UserRepository};
use async_trait::async_trait;
use uuid::Uuid;

/// 사용자 관리 도메인 서비스
#[async_trait]
pub trait UserService: Send + Sync {
    /// 사용자 생성
    async fn create_user(
        &self,
        username: String,
        email: String,
        keycloak_id: Uuid,
        full_name: Option<String>,
        organization: Option<String>,
        department: Option<String>,
        phone: Option<String>,
    ) -> Result<User, ServiceError>;

    /// 사용자 조회 (ID)
    async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError>;

    /// 사용자 조회 (Keycloak ID)
    async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<User, ServiceError>;

    /// 사용자 조회 (Username)
    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError>;

    /// 사용자 정보 업데이트
    async fn update_user(&self, update_user: UpdateUser) -> Result<User, ServiceError>;

    /// 사용자 삭제
    async fn delete_user(&self, id: i32) -> Result<(), ServiceError>;

    /// 사용자 존재 여부 확인
    async fn user_exists(&self, keycloak_id: Uuid) -> Result<bool, ServiceError>;

    // === 프로젝트 멤버십 관리 ===

    /// 사용자를 프로젝트에 추가
    async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;

    /// 프로젝트에서 사용자 제거
    async fn remove_user_from_project(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<(), ServiceError>;

    /// 사용자가 속한 프로젝트 목록 조회
    async fn get_user_projects(&self, user_id: i32) -> Result<Vec<Project>, ServiceError>;

    /// 사용자가 프로젝트 멤버인지 확인
    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError>;

    /// 사용자를 프로젝트에 역할과 함께 추가
    async fn add_user_to_project_with_role(
        &self,
        user_id: i32,
        project_id: i32,
        role_id: Option<i32>,
    ) -> Result<(), ServiceError>;

    /// 프로젝트 멤버십 정보 조회 (역할 정보 포함)
    async fn get_project_membership(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<crate::application::dto::project_user_dto::MembershipResponse>, ServiceError>;

    // === 사용자-프로젝트 역할 관리 ===

    /// 사용자의 프로젝트 목록 조회 (역할 정보 포함, 페이지네이션)
    async fn get_user_projects_with_roles(
        &self,
        user_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<
        (
            Vec<crate::application::dto::project_user_dto::ProjectWithRoleResponse>,
            i64,
        ),
        ServiceError,
    >;

    // === 매트릭스 API 지원 ===

    /// 필터로 사용자 조회 (페이지네이션)
    async fn get_users_with_filter(
        &self,
        user_ids: Option<Vec<i32>>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<User>, i64), ServiceError>;

    /// 정렬 및 필터링을 지원하는 사용자 목록 조회
    async fn get_users_with_sorting(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
        search: Option<&str>,
        user_ids: Option<&[i32]>,
    ) -> Result<(Vec<User>, i64), ServiceError>;

    /// 일괄 멤버십 조회 (배치 쿼리로 N+1 문제 해결)
    async fn get_memberships_batch(
        &self,
        user_ids: &[i32],
        project_ids: &[i32],
    ) -> Result<
        std::collections::HashMap<
            (i32, i32),
            crate::application::dto::user_project_matrix_dto::MembershipInfo,
        >,
        ServiceError,
    >;
}

#[derive(Clone)]
pub struct UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    user_repository: U,
    project_repository: P,
}

impl<U, P> UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    pub fn new(user_repository: U, project_repository: P) -> Self {
        Self {
            user_repository,
            project_repository,
        }
    }
}

#[async_trait]
impl<U, P> UserService for UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    async fn create_user(
        &self,
        username: String,
        email: String,
        keycloak_id: Uuid,
        full_name: Option<String>,
        organization: Option<String>,
        department: Option<String>,
        phone: Option<String>,
    ) -> Result<User, ServiceError> {
        // 중복 체크
        if let Some(_) = self
            .user_repository
            .find_by_keycloak_id(keycloak_id)
            .await?
        {
            return Err(ServiceError::AlreadyExists(
                "User with this keycloak_id already exists".into(),
            ));
        }

        if let Some(_) = self.user_repository.find_by_username(&username).await? {
            return Err(ServiceError::AlreadyExists("Username already taken".into()));
        }

        // 이메일 형식 검증
        if !email.contains('@') {
            return Err(ServiceError::ValidationError("Invalid email format".into()));
        }

        let new_user = NewUser {
            keycloak_id,
            username,
            email,
            full_name,
            organization,
            department,
            phone,
        };

        Ok(self.user_repository.create(new_user).await?)
    }

    async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_keycloak_id(keycloak_id)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_username(username)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn update_user(&self, update_user: UpdateUser) -> Result<User, ServiceError> {
        // 사용자 존재 여부 확인
        self.user_repository
            .find_by_id(update_user.id)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))?;

        // 이메일 중복 검사 (이메일이 변경되는 경우)
        if let Some(ref email) = update_user.email {
            if let Some(existing_user) = self.user_repository.find_by_email(email).await? {
                if existing_user.id != update_user.id {
                    return Err(ServiceError::AlreadyExists("Email already taken".into()));
                }
            }
        }

        // 사용자 정보 업데이트
        Ok(self.user_repository.update(&update_user).await?)
    }

    async fn delete_user(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self.user_repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("User not found".into()))
        }
    }

    async fn user_exists(&self, keycloak_id: Uuid) -> Result<bool, ServiceError> {
        Ok(self
            .user_repository
            .find_by_keycloak_id(keycloak_id)
            .await?
            .is_some())
    }

    // === 프로젝트 멤버십 관리 구현 ===

    async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        // INSERT with ON CONFLICT - Race condition 방지
        let result = sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_user WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_project WHERE id = $2)
             ON CONFLICT (user_id, project_id) DO NOTHING
             RETURNING user_id",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(self.user_repository.pool())
        .await?;

        match result {
            Some(_) => Ok(()),
            None => {
                // 실패 원인 파악
                if self.user_repository.find_by_id(user_id).await?.is_none() {
                    return Err(ServiceError::NotFound("User not found".into()));
                }
                if self
                    .project_repository
                    .find_by_id(project_id)
                    .await?
                    .is_none()
                {
                    return Err(ServiceError::NotFound("Project not found".into()));
                }
                Err(ServiceError::AlreadyExists(
                    "User is already a member of this project".into(),
                ))
            }
        }
    }

    async fn remove_user_from_project(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<(), ServiceError> {
        let result =
            sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
                .bind(user_id)
                .bind(project_id)
                .execute(self.user_repository.pool())
                .await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "User is not a member of this project".into(),
            ))
        }
    }

    async fn get_user_projects(&self, user_id: i32) -> Result<Vec<Project>, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        let projects = sqlx::query_as::<_, Project>(
            "SELECT p.id, p.name, p.description, p.is_active, p.created_at
             FROM security_project p
             INNER JOIN security_user_project up ON p.id = up.project_id
             WHERE up.user_id = $1
             ORDER BY p.name",
        )
        .bind(user_id)
        .fetch_all(self.user_repository.pool())
        .await?;

        Ok(projects)
    }

    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(self.user_repository.pool())
        .await?;

        Ok(result > 0)
    }

    async fn add_user_to_project_with_role(
        &self,
        user_id: i32,
        project_id: i32,
        role_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 프로젝트 존재 확인
        if self
            .project_repository
            .find_by_id(project_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 이미 멤버인지 확인
        if self.is_project_member(user_id, project_id).await? {
            return Err(ServiceError::AlreadyExists(
                "User is already a member of this project".into(),
            ));
        }

        // 기본 역할 설정 (role_id가 None인 경우 Viewer 역할 사용)
        let final_role_id = match role_id {
            Some(id) => {
                // 역할 존재 확인
                let role_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM security_role WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(self.user_repository.pool())
                .await?;

                if !role_exists {
                    return Err(ServiceError::NotFound("Role not found".into()));
                }
                id
            }
            None => {
                // 기본 Viewer 역할 ID 조회
                sqlx::query_scalar::<_, i32>(
                    "SELECT id FROM security_role WHERE name = 'Viewer' AND scope = 'project' LIMIT 1"
                )
                .fetch_one(self.user_repository.pool())
                .await
                .map_err(|_| ServiceError::NotFound("Default Viewer role not found".into()))?
            }
        };

        // 멤버 추가
        let result = sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id, role_id)
             VALUES ($1, $2, $3)
             RETURNING user_id",
        )
        .bind(user_id)
        .bind(project_id)
        .bind(final_role_id)
        .fetch_optional(self.user_repository.pool())
        .await?;

        match result {
            Some(_) => Ok(()),
            None => Err(ServiceError::DatabaseError(
                "Failed to add user to project".into(),
            )),
        }
    }

    async fn get_project_membership(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<crate::application::dto::project_user_dto::MembershipResponse>, ServiceError>
    {
        let result = sqlx::query_as::<
            _,
            (
                i32,
                Option<i32>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT up.user_id, up.role_id, r.name as role_name, up.created_at
             FROM security_user_project up
             LEFT JOIN security_role r ON up.role_id = r.id
             WHERE up.user_id = $1 AND up.project_id = $2",
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(self.user_repository.pool())
        .await?;

        match result {
            Some((_, role_id, role_name, joined_at)) => Ok(Some(
                crate::application::dto::project_user_dto::MembershipResponse {
                    is_member: true,
                    role_id,
                    role_name,
                    joined_at: Some(joined_at.to_rfc3339()),
                },
            )),
            None => Ok(Some(
                crate::application::dto::project_user_dto::MembershipResponse {
                    is_member: false,
                    role_id: None,
                    role_name: None,
                    joined_at: None,
                },
            )),
        }
    }

    // === 사용자-프로젝트 역할 관리 구현 ===

    async fn get_user_projects_with_roles(
        &self,
        user_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<
        (
            Vec<crate::application::dto::project_user_dto::ProjectWithRoleResponse>,
            i64,
        ),
        ServiceError,
    > {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        let offset = (page - 1) * page_size;

        // 사용자의 프로젝트와 역할 정보를 함께 조회 (기한 정보 포함)
        let projects_with_roles = sqlx::query_as::<
            _,
            (
                i32,
                String,
                Option<String>,
                bool,
                Option<String>,
                Option<String>,
                Option<i32>,
                Option<String>,
                Option<String>,
            ),
        >(
            "SELECT 
                p.id as project_id, 
                p.name as project_name, 
                p.description, 
                p.is_active,
                p.start_date,
                p.end_date,
                r.id as role_id, 
                r.name as role_name, 
                r.scope as role_scope
             FROM security_project p
             INNER JOIN security_user_project up ON p.id = up.project_id
             LEFT JOIN security_role r ON up.role_id = r.id
             WHERE up.user_id = $1
             ORDER BY p.name
             LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.user_repository.pool())
        .await?;

        // 총 개수 조회
        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(self.user_repository.pool())
        .await?;

        // DTO로 변환
        let projects: Vec<crate::application::dto::project_user_dto::ProjectWithRoleResponse> =
            projects_with_roles
                .into_iter()
                .map(
                    |(
                        project_id,
                        project_name,
                        description,
                        is_active,
                        start_date,
                        end_date,
                        role_id,
                        role_name,
                        role_scope,
                    )| {
                        crate::application::dto::project_user_dto::ProjectWithRoleResponse {
                            project_id,
                            project_name,
                            description,
                            is_active,
                            start_date,
                            end_date,
                            role_id,
                            role_name,
                            role_scope,
                        }
                    },
                )
                .collect();

        Ok((projects, total_count))
    }

    // === 매트릭스 API 지원 구현 ===

    async fn get_users_with_filter(
        &self,
        user_ids: Option<Vec<i32>>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<User>, i64), ServiceError> {
        let offset = (page - 1) * page_size;

        // 사용자 조회 쿼리
        let users = sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified, 
                    email_verification_token, email_verification_expires_at, 
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             WHERE ($1::int[] IS NULL OR id = ANY($1))
               AND account_status != 'DELETED'
             ORDER BY username
             LIMIT $2 OFFSET $3",
        )
        .bind(&user_ids)
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.user_repository.pool())
        .await?;

        // 총 개수 조회
        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM security_user
             WHERE ($1::int[] IS NULL OR id = ANY($1))
               AND account_status != 'DELETED'",
        )
        .bind(&user_ids)
        .fetch_one(self.user_repository.pool())
        .await?;

        Ok((users, total_count))
    }

    async fn get_users_with_sorting(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
        search: Option<&str>,
        user_ids: Option<&[i32]>,
    ) -> Result<(Vec<User>, i64), ServiceError> {
        let offset = (page - 1) * page_size;

        // 정렬 필드 검증 및 ORDER BY 절 구성
        let order_by = match sort_by {
            "username" => "username",
            "email" => "email",
            "created_at" => "created_at",
            _ => "username", // 기본값
        };

        let order_direction = match sort_order {
            "desc" => "DESC",
            _ => "ASC", // 기본값
        };

        // 검색 조건 구성
        let search_condition = if let Some(search_term) = search {
            format!(
                "AND (username ILIKE '%{}%' OR email ILIKE '%{}%')",
                search_term, search_term
            )
        } else {
            String::new()
        };

        // 사용자 조회 쿼리
        let query = format!(
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified, 
                    email_verification_token, email_verification_expires_at, 
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             WHERE ($1::int[] IS NULL OR id = ANY($1))
               AND account_status != 'DELETED'
               {}
             ORDER BY {} {}
             LIMIT $2 OFFSET $3",
            search_condition, order_by, order_direction
        );

        let users = sqlx::query_as::<_, User>(&query)
            .bind(&user_ids)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.user_repository.pool())
            .await?;

        // 총 개수 조회
        let count_query = format!(
            "SELECT COUNT(*)
             FROM security_user
             WHERE ($1::int[] IS NULL OR id = ANY($1))
               AND account_status != 'DELETED'
               {}",
            search_condition
        );

        let total_count = sqlx::query_scalar::<_, i64>(&count_query)
            .bind(&user_ids)
            .fetch_one(self.user_repository.pool())
            .await?;

        Ok((users, total_count))
    }

    async fn get_memberships_batch(
        &self,
        user_ids: &[i32],
        project_ids: &[i32],
    ) -> Result<
        std::collections::HashMap<
            (i32, i32),
            crate::application::dto::user_project_matrix_dto::MembershipInfo,
        >,
        ServiceError,
    > {
        use crate::application::dto::user_project_matrix_dto::MembershipInfo;

        // 단일 쿼리로 모든 멤버십 정보 조회 (joined_at 제거로 성능 최적화)
        let memberships = sqlx::query_as::<_, (i32, i32, Option<i32>, Option<String>)>(
            "SELECT up.user_id, up.project_id, up.role_id, r.name as role_name
             FROM security_user_project up
             LEFT JOIN security_role r ON up.role_id = r.id
             WHERE up.user_id = ANY($1) AND up.project_id = ANY($2)",
        )
        .bind(&user_ids)
        .bind(&project_ids)
        .fetch_all(self.user_repository.pool())
        .await?;

        // HashMap으로 변환하여 O(1) 조회 가능 (사전 용량 할당으로 재할당 방지)
        let estimated_capacity = user_ids.len().saturating_mul(project_ids.len());
        let mut membership_map = std::collections::HashMap::with_capacity(estimated_capacity);

        for (user_id, project_id, role_id, role_name) in memberships {
            membership_map.insert((user_id, project_id), MembershipInfo { role_id, role_name });
        }

        Ok(membership_map)
    }
}

// ServiceError는 이제 공통 모듈에서 가져옴
use crate::domain::ServiceError;

impl From<crate::application::services::SignedUrlError> for ServiceError {
    fn from(err: crate::application::services::SignedUrlError) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}
