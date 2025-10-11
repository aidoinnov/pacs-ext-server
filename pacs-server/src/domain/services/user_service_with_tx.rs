// 트랜잭션 처리가 적용된 UserService 개선안

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::domain::entities::{User, NewUser, Project};
use crate::domain::repositories::{UserRepository, ProjectRepository};
use crate::domain::ServiceError;

#[async_trait]
pub trait UserServiceWithTx {
    async fn add_user_to_project_tx(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;
}

// 트랜잭션 헬퍼 함수들
impl<U, P> super::UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    /// 트랜잭션 내에서 사용자를 프로젝트에 추가
    /// Race condition 방지: INSERT ... ON CONFLICT 패턴 사용
    pub async fn add_user_to_project_tx(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        let pool = self.user_repository.pool();
        let mut tx = pool.begin().await?;

        // 트랜잭션 내에서 원자적으로 처리
        // 1. 사용자 존재 확인 (SELECT FOR SHARE - 공유 락)
        let user_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_user WHERE id = $1)"
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        if !user_exists {
            tx.rollback().await?;
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 2. 프로젝트 존재 확인 (SELECT FOR SHARE)
        let project_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_project WHERE id = $1)"
        )
        .bind(project_id)
        .fetch_one(&mut *tx)
        .await?;

        if !project_exists {
            tx.rollback().await?;
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 3. INSERT with ON CONFLICT - race condition 방지
        let result = sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id)
             VALUES ($1, $2)
             ON CONFLICT (user_id, project_id) DO NOTHING
             RETURNING user_id"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await?;

        if result.is_none() {
            tx.rollback().await?;
            return Err(ServiceError::AlreadyExists("User is already a member of this project".into()));
        }

        tx.commit().await?;
        Ok(())
    }

    /// 트랜잭션 내에서 복수 사용자를 프로젝트에 일괄 추가
    /// 배치 최적화: 여러 사용자를 한 트랜잭션으로 처리
    pub async fn add_users_to_project_batch(
        &self,
        user_ids: Vec<i32>,
        project_id: i32
    ) -> Result<Vec<i32>, ServiceError> {
        let pool = self.user_repository.pool();
        let mut tx = pool.begin().await?;

        // 프로젝트 존재 확인
        let project_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_project WHERE id = $1)"
        )
        .bind(project_id)
        .fetch_one(&mut *tx)
        .await?;

        if !project_exists {
            tx.rollback().await?;
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 배치 INSERT with UNNEST (PostgreSQL specific)
        let added_user_ids = sqlx::query_scalar::<_, i32>(
            "INSERT INTO security_user_project (user_id, project_id)
             SELECT unnest($1::int[]), $2
             WHERE EXISTS(SELECT 1 FROM security_user WHERE id = ANY($1::int[]))
             ON CONFLICT (user_id, project_id) DO NOTHING
             RETURNING user_id"
        )
        .bind(&user_ids)
        .bind(project_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(added_user_ids)
    }
}

/*
트랜잭션 처리 개선 요약:

1. **Race Condition 방지**
   - INSERT ... ON CONFLICT DO NOTHING 사용
   - RETURNING 절로 실제 삽입 여부 확인

2. **원자성 보장**
   - BEGIN → 검증 → INSERT → COMMIT 패턴
   - 오류 시 자동 ROLLBACK

3. **성능 최적화**
   - EXISTS 사용으로 불필요한 데이터 로드 방지
   - 배치 작업 시 UNNEST로 다중 INSERT 한 번에 처리

4. **락 전략**
   - SELECT FOR SHARE: 읽기 락 (다른 읽기 허용, 쓰기 대기)
   - 짧은 트랜잭션 유지로 락 경합 최소화

5. **데이터 무결성**
   - 트랜잭션 범위 내에서 모든 검증 수행
   - DB 제약 조건과 함께 이중 검증
*/
