# 05. 공통 코드 패턴

이 프로젝트 전반에서 반복적으로 사용되는 표준 코드 패턴들입니다. 이 패턴들을 숙지하면 코드를 더 빠르고 일관성 있게 작성할 수 있습니다.

## 1. 컨트롤러 함수 패턴

-   **역할**: HTTP 요청을 받아 유스케이스를 호출하고, 결과를 HTTP 응답으로 변환합니다.
-   **구조**:
    1.  `async fn`으로 선언됩니다.
    2.  인자로 요청 데이터(DTO, 경로 파라미터 등)와 `web::Data`로 래핑된 서비스/유스케이스를 주입받습니다.
    3.  반환 타입은 항상 `Result<HttpResponse, Error>` 입니다.
    4.  유스케이스를 호출하고 `?`로 에러를 전파합니다.
    5.  성공 시 `HttpResponse`를 생성하여 `Ok()`로 감싸 반환합니다.

```rust
// in presentation/controllers/project_controller.rs

pub async fn create_project(
    project_dto: web::Json<CreateProjectDto>,
    use_case: web::Data<ProjectUseCase>,
) -> Result<HttpResponse, Error> {
    let new_project = use_case.create_project(project_dto.into_inner()).await?;
    Ok(HttpResponse::Created().json(new_project))
}
```

## 2. 유스케이스/서비스 함수 패턴

-   **역할**: 비즈니스 로직을 수행하며, 하나 이상의 리포지토리를 조율합니다.
-   **구조**:
    1.  구조체는 구체적인 구현이 아닌 리포지토리 `Trait`에 대한 `Arc<dyn ...>`를 멤버로 갖습니다.
    2.  함수는 입력으로 DTO나 원시 타입을 받습니다.
    3.  리포지토리 메소드를 호출하여 데이터를 조회하거나 변경합니다.
    4.  결과를 `Result`로 반환합니다.

```rust
// in application/use_cases/project_use_case.rs

pub struct ProjectUseCase {
    pub project_repo: Arc<dyn IProjectRepository>,
    pub user_repo: Arc<dyn IUserRepository>,
}

impl ProjectUseCase {
    pub async fn create_project_for_user(
        &self,
        dto: CreateProjectDto,
        user_id: Uuid,
    ) -> Result<Project, Error> {
        // 여러 리포지토리를 사용하여 로직 처리
        let user = self.user_repo.find_by_id(user_id).await?;
        if user.is_none() { return Err(Error::NotFound); }

        let project = Project::new(dto.name, user_id);
        self.project_repo.save(&project).await?;

        Ok(project)
    }
}
```

## 3. 리포지토리 Trait과 구현체 패턴

-   **역할**: 데이터 영속성 로직을 추상화하고 구현합니다.
-   **구조**:
    1.  **Trait (`domain`)**: `async_trait` 매크로와 함께 필요한 함수들을 정의합니다.
    2.  **구현체 (`infrastructure`)**: `PgPool` 같은 DB 커넥션 풀을 멤버로 갖고, `Trait`을 `impl`하여 실제 SQL 쿼리를 실행합니다.

```rust
// Trait in domain/repositories/project_repository.rs
#[async_trait::async_trait]
pub trait IProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>, Error>;
}

// Impl in infrastructure/repositories/project_repository_impl.rs
pub struct ProjectRepositoryImpl {
    pub db_pool: PgPool,
}

#[async_trait::async_trait]
impl IProjectRepository for ProjectRepositoryImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>, Error> {
        sqlx::query_as("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(Into::into)
    }
}
```

## 4. 의존성 주입 패턴 (`main.rs`)

-   **역할**: 애플리케이션 시작 시 모든 구성 요소(DB 풀, 리포지토리, 유스케이스)를 생성하고 서로 연결합니다.
-   **구조**:
    1.  설정 파일을 로드하고 데이터베이스 커넥션 풀(`PgPool`)을 생성합니다.
    2.  DB 풀을 `Arc`로 감싸 리포지토리 구현체(`...Impl`)들을 생성합니다.
    3.  리포지토리 구현체들을 `Arc`로 감싸 유스케이스/서비스들을 생성합니다.
    4.  유스케이스들을 `web::Data`로 감싸 웹 서버의 `app_data`로 등록합니다.

```rust
// in main.rs (간략화된 버전)

async fn main() -> std::io::Result<()> {
    // 1. DB 풀 생성
    let db_pool = Arc::new(create_db_pool().await);

    // 2. 리포지토리 구현체 생성
    let user_repo = Arc::new(UserRepositoryImpl { db_pool: db_pool.clone() });
    let project_repo = Arc::new(ProjectRepositoryImpl { db_pool: db_pool.clone() });

    // 3. 유스케이스 생성
    let user_use_case = web::Data::new(UserUseCase { user_repo });
    let project_use_case = web::Data::new(ProjectUseCase { project_repo });

    // 4. 웹 서버에 유스케이스 등록 및 실행
    HttpServer::new(move || {
        App::new()
            .app_data(user_use_case.clone()) // 서버 상태에 등록
            .app_data(project_use_case.clone())
            .configure(routes::configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```
