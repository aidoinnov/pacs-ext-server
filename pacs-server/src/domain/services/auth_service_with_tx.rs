// 트랜잭션 처리가 적용된 AuthService 개선안

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::domain::entities::{User, NewUser};
use crate::domain::repositories::UserRepository;
use crate::domain::services::ServiceError;
use crate::infrastructure::auth::jwt_service::JwtService;
use crate::infrastructure::auth::claims::Claims;

pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

// 트랜잭션 헬퍼 함수들
impl<U> super::AuthServiceImpl<U>
where
    U: UserRepository,
{
    /// UPSERT 패턴으로 로그인 처리 (Race condition 방지)
    pub async fn login_upsert(&self, keycloak_id: Uuid, username: String, email: String) -> Result<AuthResponse, ServiceError> {
        let pool = self.user_repository.pool();
        let mut tx = pool.begin().await?;

        // PostgreSQL UPSERT (INSERT ... ON CONFLICT ... DO UPDATE)
        // keycloak_id 중복 시 기존 사용자 반환, 없으면 생성
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO security_user (keycloak_id, username, email)
             VALUES ($1, $2, $3)
             ON CONFLICT (keycloak_id) DO UPDATE
             SET username = EXCLUDED.username,
                 email = EXCLUDED.email
             RETURNING id, keycloak_id, username, email, created_at"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // JWT 토큰 생성
        let claims = Claims::new(
            user.id,
            user.keycloak_id,
            user.username.clone(),
            user.email.clone(),
            24, // 24시간 유효
        );

        let token = self.jwt_service
            .create_token(&claims)
            .map_err(|e| ServiceError::Unauthorized(format!("Failed to create token: {}", e)))?;

        Ok(AuthResponse { user, token })
    }

    /// 사용자 정보 업데이트와 함께 로그인 (이메일/이름 변경 시)
    pub async fn login_with_profile_update(
        &self,
        keycloak_id: Uuid,
        username: String,
        email: String,
        update_if_exists: bool
    ) -> Result<AuthResponse, ServiceError> {
        let pool = self.user_repository.pool();
        let mut tx = pool.begin().await?;

        let user = if update_if_exists {
            // 기존 사용자가 있으면 프로필 업데이트
            sqlx::query_as::<_, User>(
                "INSERT INTO security_user (keycloak_id, username, email)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (keycloak_id) DO UPDATE
                 SET username = EXCLUDED.username,
                     email = EXCLUDED.email
                 RETURNING id, keycloak_id, username, email, created_at"
            )
            .bind(keycloak_id)
            .bind(&username)
            .bind(&email)
            .fetch_one(&mut *tx)
            .await?
        } else {
            // 기존 사용자가 있으면 그대로 반환, 없으면 생성
            sqlx::query_as::<_, User>(
                "INSERT INTO security_user (keycloak_id, username, email)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (keycloak_id) DO NOTHING
                 RETURNING id, keycloak_id, username, email, created_at"
            )
            .bind(keycloak_id)
            .bind(&username)
            .bind(&email)
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or_else(|| {
                // ON CONFLICT로 INSERT 실패 시 기존 사용자 조회
                // 주의: 이 경우 별도 쿼리 필요
                sqlx::query_as::<_, User>(
                    "SELECT id, keycloak_id, username, email, created_at
                     FROM security_user WHERE keycloak_id = $1"
                )
                .bind(keycloak_id)
                .fetch_one(&mut *tx)
                .await
                .expect("User must exist after conflict")
            })
        };

        tx.commit().await?;

        // JWT 토큰 생성
        let claims = Claims::new(
            user.id,
            user.keycloak_id,
            user.username.clone(),
            user.email.clone(),
            24,
        );

        let token = self.jwt_service
            .create_token(&claims)
            .map_err(|e| ServiceError::Unauthorized(format!("Failed to create token: {}", e)))?;

        Ok(AuthResponse { user, token })
    }

    /// 최적화된 UPSERT 패턴 (CTE 사용)
    pub async fn login_optimized(&self, keycloak_id: Uuid, username: String, email: String) -> Result<AuthResponse, ServiceError> {
        let pool = self.user_repository.pool();

        // CTE를 사용한 원자적 UPSERT + 단일 쿼리
        let user = sqlx::query_as::<_, User>(
            r#"
            WITH upserted AS (
                INSERT INTO security_user (keycloak_id, username, email)
                VALUES ($1, $2, $3)
                ON CONFLICT (keycloak_id) DO UPDATE
                SET username = EXCLUDED.username,
                    email = EXCLUDED.email
                RETURNING id, keycloak_id, username, email, created_at
            )
            SELECT * FROM upserted
            "#
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(pool)
        .await?;

        // JWT 토큰 생성
        let claims = Claims::new(
            user.id,
            user.keycloak_id,
            user.username.clone(),
            user.email.clone(),
            24,
        );

        let token = self.jwt_service
            .create_token(&claims)
            .map_err(|e| ServiceError::Unauthorized(format!("Failed to create token: {}", e)))?;

        Ok(AuthResponse { user, token })
    }
}

/*
트랜잭션 처리 개선 요약:

1. **UPSERT 패턴 (INSERT ... ON CONFLICT)**
   - Race condition 완전 제거
   - 동시 로그인 시 한 사용자만 생성
   - 기존 사용자는 자동으로 반환

2. **프로필 업데이트 전략**
   - DO UPDATE: Keycloak에서 변경된 정보 반영
   - DO NOTHING: 최초 생성 정보 유지
   - 선택적 업데이트 지원

3. **성능 최적화**
   - 2번의 쿼리 → 1번의 쿼리 (UPSERT)
   - 트랜잭션 불필요 (단일 원자적 쿼리)
   - CTE 패턴으로 더욱 명확한 코드

4. **안정성**
   - 동시성 제어: DB 레벨에서 처리
   - 데이터 무결성: UNIQUE 제약 + ON CONFLICT
   - 멱등성 보장: 여러 번 호출해도 동일 결과

5. **주의사항**
   - username UNIQUE 제약 있으면 충돌 가능
   - 이 경우 keycloak_id만 CONFLICT 조건으로 사용
   - email 업데이트는 선택적으로 처리
*/
