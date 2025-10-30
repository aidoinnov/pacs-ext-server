# Phase 3-2: `sqlx` for PostgreSQL 가이드

`sqlx`는 Rust의 비동기 SQL 라이브러리입니다. 가장 큰 특징은 컴파일 타임에 SQL 쿼리의 유효성을 검사하여, 런타임에 발생할 수 있는 많은 SQL 관련 에러를 미리 방지해준다는 점입니다. 이 프로젝트는 `sqlx`를 사용하여 PostgreSQL 데이터베이스와 통신합니다.

## 1. 설정

### `DATABASE_URL`

`sqlx`는 `.env` 파일에 있는 `DATABASE_URL` 환경 변수를 사용하여 데이터베이스에 연결합니다.

```
DATABASE_URL=postgres://user:password@host:port/database
```

### `sqlx-cli`

컴파일 타임 쿼리 검사를 위해서는 `sqlx-cli`라는 커맨드라인 도구가 필요합니다. `cargo install sqlx-cli`로 설치할 수 있습니다.

`sqlx database prepare` 명령을 실행하면, `sqlx`는 `DATABASE_URL`에 연결하여 쿼리를 분석하고, 오프라인에서도 컴파일 타임 검사가 가능하도록 `.sqlx` 디렉토리에 메타데이터를 저장합니다.

## 2. 커넥션 풀 (`PgPool`)

데이터베이스에 대한 모든 요청마다 새로 연결을 맺는 것은 매우 비효율적입니다. **커넥션 풀**은 미리 일정 개수의 데이터베이스 커넥션을 만들어두고, 필요할 때마다 빌려주고 반납받는 방식으로 성능을 크게 향상시킵니다.

`sqlx::PgPool`이 PostgreSQL을 위한 커넥션 풀 타입입니다. `main.rs`에서 단 한 번 생성하여 `Arc`로 감싸 애플리케이션 전체에서 공유합니다.

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    PgPoolOptions::new()
        .max_connections(10) // 풀의 최대 커넥션 수
        .connect(&db_url)
        .await
}
```

## 3. 쿼리 실행

`sqlx`는 쿼리 실행을 위한 여러 매크로와 함수를 제공합니다.

### `query()` - 기본 쿼리 실행

결과를 특정 구조체로 매핑할 필요가 없을 때 사용합니다. (예: `INSERT`, `UPDATE`, `DELETE`)

```rust
async fn create_user(pool: &PgPool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, name) VALUES ($1, $2)")
        .bind(user.id)      // $1에 바인딩
        .bind(&user.name)   // $2에 바인딩
        .execute(pool)      // 쿼리 실행
        .await?;
    Ok(())
}
```

### `query_as()` - 결과를 구조체로 매핑

`SELECT` 쿼리의 결과를 Rust 구조체로 직접 매핑할 때 사용합니다. 구조체는 `sqlx::FromRow` Trait을 구현해야 하며, 보통 `#[derive(sqlx::FromRow)]`를 붙여 자동으로 구현합니다.

```rust
#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
}

async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool) // 결과를 Option<User>로 가져옴
        .await?;
    Ok(user)
}
```

### `query_scalar()` - 단일 값 스칼라 결과

`SELECT COUNT(*)` 와 같이 단일 값을 결과로 받을 때 사용합니다.

```rust
async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool) // 단일 결과를 i64로 가져옴
        .await?;
    Ok(count)
}
```

## 4. 결과 가져오기 (Fetching)

쿼리를 실행한 후, `.await` 전에 어떤 메소드를 호출하느냐에 따라 결과를 가져오는 방식이 달라집니다.

-   `.execute()`: 쿼리를 실행하고 결과 행은 무시합니다. 영향 받은 행의 수만 반환합니다. (`INSERT`, `UPDATE`에 사용)
-   `.fetch_one()`: 정확히 하나의 결과 행을 가져옵니다. 결과가 없거나 여러 개이면 에러를 반환합니다.
-   `.fetch_optional()`: 최대 하나의 결과 행을 가져옵니다. 결과가 `Option<T>`으로 래핑됩니다.
-   `.fetch_all()`: 모든 결과 행을 `Vec<T>`으로 가져옵니다.

## 5. 트랜잭션 (Transactions)

여러 쿼리를 "All or Nothing"으로 처리해야 할 때 트랜잭션을 사용합니다. `pool.begin()`으로 트랜잭션을 시작하고, 모든 작업이 성공하면 `tx.commit()`을, 중간에 에러가 발생하면 `tx.rollback()`을 호출합니다. `sqlx`는 에러 발생 시 트랜잭션 객체가 `drop`될 때 자동으로 롤백을 수행하여 안전합니다.

```rust
async fn transfer_money(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?; // 트랜잭션 시작

    // 1. 출금
    sqlx::query("UPDATE accounts SET balance = balance - 100 WHERE id = 1")
        .execute(&mut *tx) // &mut *tx 또는 &mut tx 로 실행
        .await?;

    // 2. 입금
    sqlx::query("UPDATE accounts SET balance = balance + 100 WHERE id = 2")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?; // 모든 작업이 성공했으므로 커밋

    Ok(())
}
```

이 프로젝트의 `Infrastructure` 계층에 있는 모든 리포지토리 구현체들은 위에서 설명한 `sqlx`의 기능들을 사용하여 데이터베이스와 상호작용합니다.
