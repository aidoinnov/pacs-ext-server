use crate::domain::entities::project_data::{
    DataAccessStatus, NewProjectDataAccess, ProjectDataAccess, UpdateProjectDataAccess,
};
use crate::domain::repositories::ProjectDataAccessRepository;
use sqlx::PgPool;

pub struct ProjectDataAccessRepositoryImpl {
    pool: PgPool,
}

impl ProjectDataAccessRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ProjectDataAccessRepository for ProjectDataAccessRepositoryImpl {
    async fn create(
        &self,
        new_access: &NewProjectDataAccess,
    ) -> Result<ProjectDataAccess, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataAccess>(
            "INSERT INTO project_data_access (project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at"
        )
        .bind(new_access.project_data_id)
        .bind(new_access.user_id)
        .bind(&new_access.status)
        .bind(new_access.requested_at)
        .bind(new_access.requested_by)
        .bind(new_access.reviewed_at)
        .bind(new_access.reviewed_by)
        .bind(&new_access.review_note)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectDataAccess>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at
             FROM project_data_access WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_project_data_id(
        &self,
        project_data_id: i32,
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let results = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at
             FROM project_data_access 
             WHERE project_data_id = $1
             ORDER BY created_at DESC"
        )
        .bind(project_data_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let results = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at
             FROM project_data_access 
             WHERE user_id = $1
             ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn find_by_project_data_and_user(
        &self,
        project_data_id: i32,
        user_id: i32,
    ) -> Result<Option<ProjectDataAccess>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at
             FROM project_data_access 
             WHERE project_data_id = $1 AND user_id = $2"
        )
        .bind(project_data_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_matrix_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        let results = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT pda.id, pda.project_data_id, pda.user_id, pda.status, pda.requested_at, pda.requested_by, pda.reviewed_at, pda.reviewed_by, pda.review_note, pda.created_at, pda.updated_at
             FROM project_data_access pda
             INNER JOIN project_data pd ON pda.project_data_id = pd.id
             WHERE pd.project_id = $1
             ORDER BY pda.created_at DESC
             LIMIT $2 OFFSET $3"
        )
        .bind(project_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn find_by_status(
        &self,
        status: DataAccessStatus,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        let results = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at
             FROM project_data_access 
             WHERE status = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3"
        )
        .bind(&status)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn find_by_user_and_status(
        &self,
        user_id: i32,
        status: DataAccessStatus,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        let results = sqlx::query_as::<_, ProjectDataAccess>(
            "SELECT id, project_data_id, user_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at
             FROM project_data_access 
             WHERE user_id = $1 AND status = $2
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4"
        )
        .bind(user_id)
        .bind(&status)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn count_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_data_access pda
             INNER JOIN project_data pd ON pda.project_data_id = pd.id
             WHERE pd.project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn count_by_status(&self, status: DataAccessStatus) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_data_access WHERE status = $1",
        )
        .bind(&status)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn update(
        &self,
        id: i32,
        update_access: &UpdateProjectDataAccess,
    ) -> Result<Option<ProjectDataAccess>, sqlx::Error> {
        let mut query = String::from("UPDATE project_data_access SET ");
        let mut param_count = 1;

        if let Some(status) = &update_access.status {
            query.push_str(&format!(
                "status = ${}::data_access_status_enum, ",
                param_count
            ));
            param_count += 1;
        }

        if let Some(requested_at) = &update_access.requested_at {
            query.push_str(&format!("requested_at = ${}, ", param_count));
            param_count += 1;
        }

        if let Some(requested_by) = &update_access.requested_by {
            query.push_str(&format!("requested_by = ${}, ", param_count));
            param_count += 1;
        }

        if let Some(reviewed_at) = &update_access.reviewed_at {
            query.push_str(&format!("reviewed_at = ${}, ", param_count));
            param_count += 1;
        }

        if let Some(reviewed_by) = &update_access.reviewed_by {
            query.push_str(&format!("reviewed_by = ${}, ", param_count));
            param_count += 1;
        }

        if let Some(review_note) = &update_access.review_note {
            query.push_str(&format!("review_note = ${}, ", param_count));
            param_count += 1;
        }

        if param_count == 1 {
            // No fields to update
            return self.find_by_id(id).await;
        }

        // Remove trailing comma and space
        query.pop();
        query.pop();

        query.push_str(" WHERE id = $");
        query.push_str(&(param_count as i32 + 1).to_string());
        query.push_str(" RETURNING id, project_id, user_id, resource_level, study_id, series_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at, project_data_id");

        // Build the bind parameters in the correct order
        let mut bind_query = sqlx::query_as::<_, ProjectDataAccess>(&query);

        // Bind status if present
        if let Some(status) = &update_access.status {
            bind_query = bind_query.bind(status);
        }

        // Bind other fields if present
        if let Some(requested_at) = &update_access.requested_at {
            bind_query = bind_query.bind(requested_at);
        }

        if let Some(requested_by) = &update_access.requested_by {
            bind_query = bind_query.bind(requested_by);
        }

        if let Some(reviewed_at) = &update_access.reviewed_at {
            bind_query = bind_query.bind(reviewed_at);
        }

        if let Some(reviewed_by) = &update_access.reviewed_by {
            bind_query = bind_query.bind(reviewed_by);
        }

        if let Some(review_note) = &update_access.review_note {
            bind_query = bind_query.bind(review_note);
        }

        // Bind WHERE clause parameter (id)
        bind_query = bind_query.bind(id);

        // Execute the query
        let result = bind_query.fetch_optional(&self.pool).await?;

        Ok(result)
    }

    async fn update_by_project_data_and_user(
        &self,
        project_data_id: i32,
        user_id: i32,
        update_access: &UpdateProjectDataAccess,
    ) -> Result<Option<ProjectDataAccess>, sqlx::Error> {
        // Simple approach: update only status field
        if let Some(status) = &update_access.status {
            let result = sqlx::query_as::<_, ProjectDataAccess>(
                "UPDATE project_data_access 
                 SET status = $1::data_access_status_enum,
                     reviewed_at = COALESCE($2, reviewed_at),
                     reviewed_by = COALESCE($3, reviewed_by),
                     review_note = COALESCE($4, review_note),
                     updated_at = CURRENT_TIMESTAMP
                 WHERE project_data_id = $5 AND user_id = $6
                 RETURNING id, 0 as project_id, user_id, resource_level, study_id, series_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at, project_data_id"
            )
            .bind(status)
            .bind(&update_access.reviewed_at)
            .bind(&update_access.reviewed_by)
            .bind(&update_access.review_note)
            .bind(project_data_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(result)
        } else {
            // No status update, just return current record
            self.find_by_project_data_and_user(project_data_id, user_id)
                .await
        }
    }

    async fn create_batch(
        &self,
        access_list: &[NewProjectDataAccess],
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let mut results = Vec::new();

        for access in access_list {
            let result = self.create(access).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn update_batch(
        &self,
        project_data_id: i32,
        user_ids: &[i32],
        update_access: &UpdateProjectDataAccess,
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error> {
        let mut results = Vec::new();

        for user_id in user_ids {
            if let Some(result) = self
                .update_by_project_data_and_user(project_data_id, *user_id, update_access)
                .await?
            {
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM project_data_access WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_by_project_data_and_user(
        &self,
        project_data_id: i32,
        user_id: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM project_data_access WHERE project_data_id = $1 AND user_id = $2",
        )
        .bind(project_data_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
