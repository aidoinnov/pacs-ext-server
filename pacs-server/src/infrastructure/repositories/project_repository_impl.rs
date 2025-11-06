use crate::application::dto::project_dto::ProjectListQuery;
use crate::domain::entities::{NewProject, Project, UpdateProject};
use crate::domain::repositories::ProjectRepository;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ProjectRepositoryImpl {
    pool: PgPool,
}

impl ProjectRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for ProjectRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at
             FROM security_project
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at
             FROM security_project
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at
             FROM security_project
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at
             FROM security_project
             WHERE is_active = true
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_project: NewProject) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO security_project (name, description, sponsor, start_date, end_date, auto_complete, status)
             VALUES ($1, $2, $3, $4, $5, $6, 'PLANNING'::project_status)
             RETURNING id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at"
        )
        .bind(new_project.name)
        .bind(new_project.description)
        .bind(new_project.sponsor)
        .bind(new_project.start_date)
        .bind(new_project.end_date)
        .bind(new_project.auto_complete)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(
        &self,
        id: i32,
        update: &UpdateProject,
    ) -> Result<Option<Project>, sqlx::Error> {
        let mut query = String::from("UPDATE security_project SET ");
        let mut param_count = 1;

        if let Some(name) = &update.name {
            query.push_str(&format!("name = ${}, ", param_count));
            param_count += 1;
        }
        if let Some(description) = &update.description {
            query.push_str(&format!("description = ${}, ", param_count));
            param_count += 1;
        }
        if let Some(sponsor) = &update.sponsor {
            query.push_str(&format!("sponsor = ${}, ", param_count));
            param_count += 1;
        }
        if let Some(start_date) = &update.start_date {
            query.push_str(&format!("start_date = ${}, ", param_count));
            param_count += 1;
        }
        if let Some(end_date) = &update.end_date {
            query.push_str(&format!("end_date = ${}, ", param_count));
            param_count += 1;
        }
        if let Some(status) = &update.status {
            query.push_str(&format!("status = ${}::project_status, ", param_count));
            param_count += 1;
        }
        if let Some(auto_complete) = &update.auto_complete {
            query.push_str(&format!("auto_complete = ${}, ", param_count));
            param_count += 1;
        }
        if let Some(is_active) = &update.is_active {
            query.push_str(&format!("is_active = ${}, ", param_count));
            param_count += 1;
        }

        if param_count == 1 {
            return self.find_by_id(id).await;
        }

        query.pop(); // Remove trailing comma
        query.pop(); // Remove trailing space

        query.push_str(&format!(
            " WHERE id = ${} RETURNING id, name, description, sponsor, start_date, end_date, auto_complete, is_active, status, created_at",
            param_count
        ));

        let mut bind_query = sqlx::query_as::<_, Project>(&query);
        param_count = 1;

        if let Some(name) = &update.name {
            bind_query = bind_query.bind(name);
            param_count += 1;
        }
        if let Some(description) = &update.description {
            bind_query = bind_query.bind(description);
            param_count += 1;
        }
        if let Some(sponsor) = &update.sponsor {
            bind_query = bind_query.bind(sponsor);
            param_count += 1;
        }
        if let Some(start_date) = &update.start_date {
            bind_query = bind_query.bind(start_date);
            param_count += 1;
        }
        if let Some(end_date) = &update.end_date {
            bind_query = bind_query.bind(end_date);
            param_count += 1;
        }
        if let Some(status) = &update.status {
            bind_query = bind_query.bind(status);
            param_count += 1;
        }
        if let Some(auto_complete) = &update.auto_complete {
            bind_query = bind_query.bind(auto_complete);
            param_count += 1;
        }
        if let Some(is_active) = &update.is_active {
            bind_query = bind_query.bind(is_active);
            param_count += 1;
        }

        bind_query = bind_query.bind(id);

        let result = bind_query.fetch_optional(&self.pool).await?;
        Ok(result)
    }

    async fn set_active(&self, id: i32, is_active: bool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE security_project SET is_active = $2 WHERE id = $1")
            .bind(id)
            .bind(is_active)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_with_pagination(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let order_column = match sort_by {
            "name" => "name",
            "start_date" => "start_date",
            _ => "created_at",
        };
        let order_direction = match sort_order {
            "asc" => "ASC",
            _ => "DESC",
        };

        let query = format!(
            "SELECT id, name, description, sponsor, start_date, end_date, 
                    auto_complete, is_active, status, created_at
             FROM security_project
             ORDER BY {} {}
             LIMIT $1 OFFSET $2",
            order_column, order_direction
        );

        sqlx::query_as::<_, Project>(&query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
    }

    async fn find_active_with_pagination(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let order_column = match sort_by {
            "name" => "name",
            "start_date" => "start_date",
            _ => "created_at",
        };
        let order_direction = match sort_order {
            "asc" => "ASC",
            _ => "DESC",
        };

        let query = format!(
            "SELECT id, name, description, sponsor, start_date, end_date, 
                    auto_complete, is_active, status, created_at
             FROM security_project
             WHERE is_active = true
             ORDER BY {} {}
             LIMIT $1 OFFSET $2",
            order_column, order_direction
        );

        sqlx::query_as::<_, Project>(&query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
    }

    async fn find_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<Vec<Project>, sqlx::Error> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("desc");
        let order_column = match sort_by {
            "name" => "name",
            "start_date" => "start_date",
            _ => "created_at",
        };
        let order_direction = match sort_order {
            "asc" => "ASC",
            _ => "DESC",
        };

        let mut where_clauses = Vec::new();
        let mut param_count = 1;

        if let Some(status) = &query.status {
            where_clauses.push(format!("status = ${}::project_status", param_count));
            param_count += 1;
        }
        if let Some(sponsor) = &query.sponsor {
            where_clauses.push(format!("sponsor = ${}", param_count));
            param_count += 1;
        }
        if let Some(start_date_from) = &query.start_date_from {
            where_clauses.push(format!("start_date >= ${}", param_count));
            param_count += 1;
        }
        if let Some(start_date_to) = &query.start_date_to {
            where_clauses.push(format!("start_date <= ${}", param_count));
            param_count += 1;
        }
        if let Some(end_date_from) = &query.end_date_from {
            where_clauses.push(format!("end_date >= ${}", param_count));
            param_count += 1;
        }
        if let Some(end_date_to) = &query.end_date_to {
            where_clauses.push(format!("end_date <= ${}", param_count));
            param_count += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT id, name, description, sponsor, start_date, end_date, 
                    auto_complete, is_active, status, created_at
             FROM security_project
             {}
             ORDER BY {} {}
             LIMIT ${} OFFSET ${}",
            where_clause,
            order_column,
            order_direction,
            param_count,
            param_count + 1
        );

        let mut query_builder = sqlx::query_as::<_, Project>(&sql);

        param_count = 1;
        if let Some(status) = &query.status {
            query_builder = query_builder.bind(status);
            param_count += 1;
        }
        if let Some(sponsor) = &query.sponsor {
            query_builder = query_builder.bind(sponsor);
            param_count += 1;
        }
        if let Some(start_date_from) = &query.start_date_from {
            query_builder = query_builder.bind(start_date_from);
            param_count += 1;
        }
        if let Some(start_date_to) = &query.start_date_to {
            query_builder = query_builder.bind(start_date_to);
            param_count += 1;
        }
        if let Some(end_date_from) = &query.end_date_from {
            query_builder = query_builder.bind(end_date_from);
            param_count += 1;
        }
        if let Some(end_date_to) = &query.end_date_to {
            query_builder = query_builder.bind(end_date_to);
            param_count += 1;
        }

        query_builder
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
    }

    async fn count_all(&self) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM security_project")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.0)
    }

    async fn count_active(&self) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM security_project WHERE is_active = true")
                .fetch_one(&self.pool)
                .await?;
        Ok(result.0)
    }

    async fn count_with_filter(&self, query: &ProjectListQuery) -> Result<i64, sqlx::Error> {
        let mut where_clauses = Vec::new();
        let mut param_count = 1;

        if let Some(status) = &query.status {
            where_clauses.push(format!("status = ${}::project_status", param_count));
            param_count += 1;
        }
        if let Some(sponsor) = &query.sponsor {
            where_clauses.push(format!("sponsor = ${}", param_count));
            param_count += 1;
        }
        if let Some(start_date_from) = &query.start_date_from {
            where_clauses.push(format!("start_date >= ${}", param_count));
            param_count += 1;
        }
        if let Some(start_date_to) = &query.start_date_to {
            where_clauses.push(format!("start_date <= ${}", param_count));
            param_count += 1;
        }
        if let Some(end_date_from) = &query.end_date_from {
            where_clauses.push(format!("end_date >= ${}", param_count));
            param_count += 1;
        }
        if let Some(end_date_to) = &query.end_date_to {
            where_clauses.push(format!("end_date <= ${}", param_count));
            param_count += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::from("SELECT COUNT(*) FROM security_project")
        } else {
            format!(
                "SELECT COUNT(*) FROM security_project WHERE {}",
                where_clauses.join(" AND ")
            )
        };

        let mut query_builder = sqlx::query_as::<_, (i64,)>(&where_clause);

        param_count = 1;
        if let Some(status) = &query.status {
            query_builder = query_builder.bind(status);
            param_count += 1;
        }
        if let Some(sponsor) = &query.sponsor {
            query_builder = query_builder.bind(sponsor);
            param_count += 1;
        }
        if let Some(start_date_from) = &query.start_date_from {
            query_builder = query_builder.bind(start_date_from);
            param_count += 1;
        }
        if let Some(start_date_to) = &query.start_date_to {
            query_builder = query_builder.bind(start_date_to);
            param_count += 1;
        }
        if let Some(end_date_from) = &query.end_date_from {
            query_builder = query_builder.bind(end_date_from);
            param_count += 1;
        }
        if let Some(end_date_to) = &query.end_date_to {
            query_builder = query_builder.bind(end_date_to);
            param_count += 1;
        }

        let result = query_builder.fetch_one(&self.pool).await?;
        Ok(result.0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
