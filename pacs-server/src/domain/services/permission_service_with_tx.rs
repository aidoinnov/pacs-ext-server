// 트랜잭션 처리가 적용된 PermissionService 개선안

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use crate::domain::entities::{Role, Permission, RoleScope};
use crate::domain::repositories::{PermissionRepository, RoleRepository};
use crate::domain::services::ServiceError;

// 트랜잭션 헬퍼 함수들
impl<PE, R> super::PermissionServiceImpl<PE, R>
where
    PE: PermissionRepository,
    R: RoleRepository,
{
    /// 트랜잭션 내에서 역할에 권한 할당 (Race condition 방지)
    pub async fn assign_permission_to_role_tx(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        let pool = self.role_repository.pool();
        let mut tx = pool.begin().await?;

        // INSERT with ON CONFLICT - 원자적 처리
        let result = sqlx::query(
            "INSERT INTO security_role_permission (role_id, permission_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_role WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_permission WHERE id = $2)
             ON CONFLICT (role_id, permission_id) DO NOTHING
             RETURNING role_id"
        )
        .bind(role_id)
        .bind(permission_id)
        .fetch_optional(&mut *tx)
        .await?;

        match result {
            Some(_) => {
                tx.commit().await?;
                Ok(())
            }
            None => {
                tx.rollback().await?;
                // 실패 원인 파악
                let role_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM security_role WHERE id = $1)"
                )
                .bind(role_id)
                .fetch_one(pool)
                .await?;

                if !role_exists {
                    return Err(ServiceError::NotFound("Role not found".into()));
                }

                let perm_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM security_permission WHERE id = $1)"
                )
                .bind(permission_id)
                .fetch_one(pool)
                .await?;

                if !perm_exists {
                    return Err(ServiceError::NotFound("Permission not found".into()));
                }

                Err(ServiceError::AlreadyExists("Permission already assigned to this role".into()))
            }
        }
    }

    /// 트랜잭션 내에서 프로젝트에 권한 할당
    pub async fn assign_permission_to_project_tx(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        let pool = self.role_repository.pool();
        let mut tx = pool.begin().await?;

        let result = sqlx::query(
            "INSERT INTO security_project_permission (project_id, permission_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_project WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_permission WHERE id = $2)
             ON CONFLICT (project_id, permission_id) DO NOTHING
             RETURNING project_id"
        )
        .bind(project_id)
        .bind(permission_id)
        .fetch_optional(&mut *tx)
        .await?;

        match result {
            Some(_) => {
                tx.commit().await?;
                Ok(())
            }
            None => {
                tx.rollback().await?;
                // 실패 원인 파악
                let project_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM security_project WHERE id = $1)"
                )
                .bind(project_id)
                .fetch_one(pool)
                .await?;

                if !project_exists {
                    return Err(ServiceError::NotFound("Project not found".into()));
                }

                let perm_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM security_permission WHERE id = $1)"
                )
                .bind(permission_id)
                .fetch_one(pool)
                .await?;

                if !perm_exists {
                    return Err(ServiceError::NotFound("Permission not found".into()));
                }

                Err(ServiceError::AlreadyExists("Permission already assigned to this project".into()))
            }
        }
    }

    /// 복수 권한을 역할에 일괄 할당 (배치 최적화)
    pub async fn assign_permissions_to_role_batch(
        &self,
        role_id: i32,
        permission_ids: Vec<i32>
    ) -> Result<Vec<i32>, ServiceError> {
        let pool = self.role_repository.pool();
        let mut tx = pool.begin().await?;

        // 역할 존재 확인
        let role_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_role WHERE id = $1)"
        )
        .bind(role_id)
        .fetch_one(&mut *tx)
        .await?;

        if !role_exists {
            tx.rollback().await?;
            return Err(ServiceError::NotFound("Role not found".into()));
        }

        // 배치 INSERT
        let assigned_permissions = sqlx::query_scalar::<_, i32>(
            "INSERT INTO security_role_permission (role_id, permission_id)
             SELECT $1, unnest($2::int[])
             WHERE EXISTS(SELECT 1 FROM security_permission WHERE id = ANY($2::int[]))
             ON CONFLICT (role_id, permission_id) DO NOTHING
             RETURNING permission_id"
        )
        .bind(role_id)
        .bind(&permission_ids)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(assigned_permissions)
    }

    /// 역할 생성과 권한 할당을 한 트랜잭션으로 처리
    pub async fn create_role_with_permissions(
        &self,
        name: String,
        scope: RoleScope,
        description: Option<String>,
        permission_ids: Vec<i32>
    ) -> Result<(Role, Vec<i32>), ServiceError> {
        let pool = self.role_repository.pool();
        let mut tx = pool.begin().await?;

        // 1. 역할 생성
        let role = sqlx::query_as::<_, Role>(
            "INSERT INTO security_role (name, description, scope)
             VALUES ($1, $2, $3)
             RETURNING id, name, description, scope, created_at"
        )
        .bind(&name)
        .bind(&description)
        .bind(scope.as_str())
        .fetch_one(&mut *tx)
        .await?;

        // 2. 권한 할당
        let assigned_permissions = sqlx::query_scalar::<_, i32>(
            "INSERT INTO security_role_permission (role_id, permission_id)
             SELECT $1, unnest($2::int[])
             WHERE EXISTS(SELECT 1 FROM security_permission WHERE id = ANY($2::int[]))
             RETURNING permission_id"
        )
        .bind(role.id)
        .bind(&permission_ids)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok((role, assigned_permissions))
    }
}

/*
트랜잭션 처리 개선 요약:

1. **원자적 할당 (Atomic Assignment)**
   - INSERT + EXISTS 서브쿼리를 한 문장으로 결합
   - ON CONFLICT로 중복 방지
   - RETURNING으로 성공 여부 확인

2. **오류 처리 개선**
   - INSERT 실패 시 원인 파악
   - 적절한 에러 타입 반환 (NotFound vs AlreadyExists)

3. **배치 작업 최적화**
   - UNNEST로 여러 권한 한 번에 할당
   - N번의 INSERT → 1번의 INSERT로 개선
   - 트랜잭션 오버헤드 감소

4. **복합 작업 지원**
   - create_role_with_permissions: 역할 생성 + 권한 할당을 원자적으로
   - All-or-nothing 보장

5. **성능 향상**
   - 불필요한 SELECT 제거
   - EXISTS 서브쿼리로 검증과 INSERT 통합
   - DB 왕복 횟수 최소화
*/
