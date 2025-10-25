use async_trait::async_trait;
use sqlx::{PgPool, Row};
use crate::domain::entities::{Capability, NewCapability, UpdateCapability, Permission, Role};
use crate::domain::repositories::CapabilityRepository;

#[derive(Clone)]
pub struct CapabilityRepositoryImpl {
    pool: PgPool,
}

impl CapabilityRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CapabilityRepository for CapabilityRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Capability>, sqlx::Error> {
        sqlx::query_as::<_, Capability>(
            "SELECT id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at
             FROM security_capability
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Capability>, sqlx::Error> {
        sqlx::query_as::<_, Capability>(
            "SELECT id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at
             FROM security_capability
             WHERE is_active = true
             ORDER BY category, display_name"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_category(&self, category: &str) -> Result<Vec<Capability>, sqlx::Error> {
        sqlx::query_as::<_, Capability>(
            "SELECT id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at
             FROM security_capability
             WHERE category = $1 AND is_active = true
             ORDER BY display_name"
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_capability_permissions(&self, capability_id: i32) -> Result<Vec<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            "SELECT p.id, p.category, p.resource_type, p.action
             FROM security_permission p
             INNER JOIN security_capability_mapping cm ON p.id = cm.permission_id
             WHERE cm.capability_id = $1
             ORDER BY p.category, p.resource_type, p.action"
        )
        .bind(capability_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_role_capabilities(&self, role_id: i32) -> Result<Vec<Capability>, sqlx::Error> {
        sqlx::query_as::<_, Capability>(
            "SELECT c.id, c.name, c.display_name, c.description, c.category, c.is_active, c.created_at, c.updated_at
             FROM security_capability c
             INNER JOIN security_role_capability rc ON c.id = rc.capability_id
             WHERE rc.role_id = $1 AND c.is_active = true
             ORDER BY c.category, c.display_name"
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_capability: NewCapability) -> Result<Capability, sqlx::Error> {
        sqlx::query_as::<_, Capability>(
            "INSERT INTO security_capability (name, display_name, display_label, description, category, category_label)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at"
        )
        .bind(new_capability.name)
        .bind(new_capability.display_name)
        .bind(new_capability.display_label)
        .bind(new_capability.description)
        .bind(new_capability.category)
        .bind(new_capability.category_label)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, id: i32, update: UpdateCapability) -> Result<Capability, sqlx::Error> {
        let mut query = String::from("UPDATE security_capability SET updated_at = NOW()");
        let mut param_count = 1;

        if update.display_name.is_some() {
            query.push_str(&format!(", display_name = ${}", param_count));
            param_count += 1;
        }

        if update.display_label.is_some() {
            query.push_str(&format!(", display_label = ${}", param_count));
            param_count += 1;
        }

        if update.description.is_some() {
            query.push_str(&format!(", description = ${}", param_count));
            param_count += 1;
        }

        if update.category.is_some() {
            query.push_str(&format!(", category = ${}", param_count));
            param_count += 1;
        }

        if update.category_label.is_some() {
            query.push_str(&format!(", category_label = ${}", param_count));
            param_count += 1;
        }

        if update.is_active.is_some() {
            query.push_str(&format!(", is_active = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at", param_count));

        let mut query_builder = sqlx::query_as::<_, Capability>(&query);
        
        if let Some(display_name) = update.display_name {
            query_builder = query_builder.bind(display_name);
        }
        if let Some(display_label) = update.display_label {
            query_builder = query_builder.bind(display_label);
        }
        if let Some(description) = update.description {
            query_builder = query_builder.bind(description);
        }
        if let Some(category) = update.category {
            query_builder = query_builder.bind(category);
        }
        if let Some(category_label) = update.category_label {
            query_builder = query_builder.bind(category_label);
        }
        if let Some(is_active) = update.is_active {
            query_builder = query_builder.bind(is_active);
        }
        
        query_builder.bind(id).fetch_one(&self.pool).await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_capability WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn add_capability_permission(&self, capability_id: i32, permission_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO security_capability_mapping (capability_id, permission_id)
             VALUES ($1, $2)
             ON CONFLICT (capability_id, permission_id) DO NOTHING"
        )
        .bind(capability_id)
        .bind(permission_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_capability_permission(&self, capability_id: i32, permission_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM security_capability_mapping
             WHERE capability_id = $1 AND permission_id = $2"
        )
        .bind(capability_id)
        .bind(permission_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn assign_capability_to_role(&self, role_id: i32, capability_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO security_role_capability (role_id, capability_id)
             VALUES ($1, $2)
             ON CONFLICT (role_id, capability_id) DO NOTHING"
        )
        .bind(role_id)
        .bind(capability_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_capability_from_role(&self, role_id: i32, capability_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM security_role_capability
             WHERE role_id = $1 AND capability_id = $2"
        )
        .bind(role_id)
        .bind(capability_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_global_role_capability_matrix_paginated(
        &self,
        page: i32,
        size: i32,
        search: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>, i64), sqlx::Error> {
        let offset = (page - 1) * size;
        
        // 검색 조건 구성
        let mut where_conditions = vec!["scope = 'GLOBAL'".to_string()];
        let mut search_param = None;
        let mut scope_param = None;
        let mut param_count = 0;

        if let Some(search_term) = search {
            if !search_term.trim().is_empty() {
                param_count += 1;
                where_conditions.push(format!("(name ILIKE ${} OR description ILIKE ${})", param_count, param_count + 1));
                search_param = Some(format!("%{}%", search_term));
                param_count += 1; // 두 번째 검색 파라미터
            }
        }

        if let Some(scope_filter) = scope {
            if !scope_filter.trim().is_empty() {
                param_count += 1;
                where_conditions.push(format!("scope = ${}", param_count));
                scope_param = Some(scope_filter.to_string());
            }
        }

        let where_clause = where_conditions.join(" AND ");

        // 성능 최적화: 병렬 쿼리 실행
        let start_time = std::time::Instant::now();
        let (roles, capabilities, assignments, total_count) = tokio::try_join!(
            // 1. 페이지네이션된 역할들 조회
            async {
                let roles_query = format!(
                    "SELECT id, name, description, scope, created_at
                     FROM security_role
                     WHERE {}
                     ORDER BY name
                     LIMIT ${} OFFSET ${}",
                    where_clause,
                    param_count + 1,
                    param_count + 2
                );

                let mut roles_query = sqlx::query_as::<_, Role>(&roles_query);
                if let Some(ref search) = search_param {
                    roles_query = roles_query.bind(search).bind(search);
                }
                if let Some(ref scope) = scope_param {
                    roles_query = roles_query.bind(scope);
                }
                roles_query = roles_query.bind(size).bind(offset);
                roles_query.fetch_all(&self.pool).await
            },
            // 2. 모든 활성 Capability 조회
            async {
                sqlx::query_as::<_, Capability>(
                    "SELECT id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at
                     FROM security_capability
                     WHERE is_active = true
                     ORDER BY category, display_name"
                )
                .fetch_all(&self.pool)
                .await
            },
            // 3. 역할-Capability 할당 조회 (조회된 역할들에 대해서만)
            async {
                // 먼저 역할들을 조회해서 ID를 얻어야 함
                let roles_query = format!(
                    "SELECT id FROM security_role
                     WHERE {}
                     ORDER BY name
                     LIMIT ${} OFFSET ${}",
                    where_clause,
                    param_count + 1,
                    param_count + 2
                );

                let mut roles_query = sqlx::query_scalar::<_, i32>(&roles_query);
                if let Some(ref search) = search_param {
                    roles_query = roles_query.bind(search).bind(search);
                }
                if let Some(ref scope) = scope_param {
                    roles_query = roles_query.bind(scope);
                }
                roles_query = roles_query.bind(size).bind(offset);
                let role_ids: Vec<i32> = roles_query.fetch_all(&self.pool).await?;

                if role_ids.is_empty() {
                    return Ok(Vec::new());
                }

                // IN 절을 위한 동적 쿼리 생성
                let placeholders = role_ids.iter().enumerate()
                    .map(|(i, _)| format!("${}", i + 1))
                    .collect::<Vec<_>>()
                    .join(",");
                
                let query_string = format!(
                    "SELECT role_id, capability_id
                     FROM security_role_capability
                     WHERE role_id IN ({})",
                    placeholders
                );
                
                let mut query = sqlx::query_as::<_, (i32, i32)>(&query_string);
                
                // 각 role_id를 개별적으로 바인딩
                for role_id in &role_ids {
                    query = query.bind(role_id);
                }
                
                query.fetch_all(&self.pool).await
            },
            // 4. 총 개수 조회
            async {
                let count_query = format!(
                    "SELECT COUNT(*) FROM security_role WHERE {}",
                    where_clause
                );
                
                let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);
                if let Some(ref search) = search_param {
                    count_query = count_query.bind(search).bind(search);
                }
                if let Some(ref scope) = scope_param {
                    count_query = count_query.bind(scope);
                }
                count_query.fetch_one(&self.pool).await
            }
        )?;

        let query_time = start_time.elapsed();
        println!("🔍 Database query time: {:?}", query_time);

        Ok((roles, capabilities, assignments, total_count))
    }

    async fn get_global_role_capability_matrix(&self) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), sqlx::Error> {
        // 전역 역할들 조회
        let roles = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, scope, created_at
             FROM security_role
             WHERE scope = 'GLOBAL'
             ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        // 모든 활성 Capability 조회
        let capabilities = sqlx::query_as::<_, Capability>(
            "SELECT id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at
             FROM security_capability
             WHERE is_active = true
             ORDER BY category, display_name"
        )
        .fetch_all(&self.pool)
        .await?;

        // 역할-Capability 할당 조회
        let assignments = sqlx::query_as::<_, (i32, i32)>(
            "SELECT role_id, capability_id
             FROM security_role_capability
             INNER JOIN security_role r ON security_role_capability.role_id = r.id
             WHERE r.scope = 'GLOBAL'"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((roles, capabilities, assignments))
    }

    async fn get_project_role_capability_matrix(&self, project_id: i32) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), sqlx::Error> {
        // 프로젝트 역할들 조회
        let roles = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, scope, created_at
             FROM security_role
             WHERE scope = 'PROJECT'
             ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        // 모든 활성 Capability 조회
        let capabilities = sqlx::query_as::<_, Capability>(
            "SELECT id, name, display_name, display_label, description, category, category_label, is_active, created_at, updated_at
             FROM security_capability
             WHERE is_active = true
             ORDER BY category, display_name"
        )
        .fetch_all(&self.pool)
        .await?;

        // 프로젝트별 역할-Capability 할당 조회
        let assignments = sqlx::query_as::<_, (i32, i32)>(
            "SELECT rc.role_id, rc.capability_id
             FROM security_role_capability rc
             INNER JOIN security_role r ON rc.role_id = r.id
             WHERE r.scope = 'PROJECT'"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((roles, capabilities, assignments))
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
