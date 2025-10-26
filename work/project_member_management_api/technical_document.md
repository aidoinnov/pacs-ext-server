# 프로젝트 멤버 관리 API 기술 문서

## 아키텍처 개요

프로젝트 멤버 관리 API는 Clean Architecture 패턴을 따라 구현되었으며, 기존 프로젝트-사용자 관리 시스템을 확장하여 더 명확한 멤버 관리 기능을 제공합니다.

## 데이터베이스 스키마

### 기존 테이블 활용
```sql
-- security_user_project 테이블 (기존)
CREATE TABLE security_user_project (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES security_user(id),
    project_id INTEGER NOT NULL REFERENCES security_project(id),
    role_id INTEGER NOT NULL REFERENCES security_role(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, project_id)
);
```

### 인덱스
```sql
-- 성능 최적화를 위한 인덱스
CREATE INDEX idx_security_user_project_user_id ON security_user_project(user_id);
CREATE INDEX idx_security_user_project_project_id ON security_user_project(project_id);
CREATE INDEX idx_security_user_project_role_id ON security_user_project(role_id);
```

## Domain Layer

### UserService 인터페이스 확장
```rust
#[async_trait]
pub trait UserService: Send + Sync {
    // 기존 메서드들...
    
    /// 사용자를 프로젝트에 멤버로 추가
    async fn add_user_to_project_with_role(
        &self,
        user_id: i32,
        project_id: i32,
        role_id: Option<i32>,
    ) -> Result<(), ServiceError>;
    
    /// 프로젝트 멤버십 정보 조회
    async fn get_project_membership(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<MembershipResponse>, ServiceError>;
}
```

### ServiceError 처리
```rust
pub enum ServiceError {
    NotFound(String),
    AlreadyExists(String),
    DatabaseError(String),
    ValidationError(String),
    // ... 기타 에러 타입들
}
```

## Application Layer

### DTO 정의
```rust
/// 멤버 추가 요청
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub user_id: i32,
    pub role_id: Option<i32>, // 선택사항, 기본 역할 자동 할당
}

/// 멤버십 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct MembershipResponse {
    pub is_member: bool,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub joined_at: Option<String>, // RFC3339 형식
}

/// 멤버 추가 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct AddMemberResponse {
    pub message: String,
    pub user_id: i32,
    pub project_id: i32,
    pub role_id: i32,
    pub role_name: String,
}

/// 멤버 제거 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct RemoveMemberResponse {
    pub message: String,
    pub user_id: i32,
    pub project_id: i32,
}
```

### Use Case 구현
```rust
impl<P, U> ProjectUserUseCase<P, U>
where
    P: ProjectService,
    U: UserService,
{
    /// 프로젝트에 멤버 추가
    pub async fn add_member_to_project(
        &self,
        project_id: i32,
        request: AddMemberRequest,
    ) -> Result<AddMemberResponse, ServiceError> {
        // 1. 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;
        
        // 2. 사용자를 프로젝트에 추가
        self.user_service
            .add_user_to_project_with_role(request.user_id, project_id, request.role_id)
            .await?;
        
        // 3. 추가된 멤버의 역할 정보 조회
        let membership = self.user_service
            .get_project_membership(request.user_id, project_id)
            .await?;
        
        // 4. 응답 데이터 구성
        let (role_name, role_id) = match membership {
            Some(m) => (
                m.role_name.unwrap_or_else(|| "Unknown".to_string()),
                m.role_id.unwrap_or(0)
            ),
            None => ("Unknown".to_string(), 0)
        };
        
        Ok(AddMemberResponse {
            message: "Member added to project successfully".to_string(),
            user_id: request.user_id,
            project_id,
            role_id,
            role_name,
        })
    }
}
```

## Infrastructure Layer

### SQL 쿼리 구현
```rust
impl UserServiceImpl {
    /// 사용자를 프로젝트에 추가
    async fn add_user_to_project_with_role(
        &self,
        user_id: i32,
        project_id: i32,
        role_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        // 1. 사용자 존재 확인
        let user = self.user_repository.find_by_id(user_id).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;
        
        // 2. 프로젝트 존재 확인
        let project = self.project_repository.find_by_id(project_id).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Project not found".to_string()))?;
        
        // 3. 역할 ID 결정 (기본값 또는 제공된 값)
        let final_role_id = match role_id {
            Some(id) => {
                // 제공된 역할 존재 확인
                self.role_repository.find_by_id(id).await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| ServiceError::NotFound("Role not found".to_string()))?
                    .id
            },
            None => {
                // 기본 역할 조회 (현재는 Viewer 역할이 없어서 에러)
                return Err(ServiceError::ValidationError(
                    "Default role not found. Please specify a role_id".to_string()
                ));
            }
        };
        
        // 4. 중복 멤버십 체크
        let existing_membership = sqlx::query!(
            "SELECT id FROM security_user_project WHERE user_id = $1 AND project_id = $2",
            user_id, project_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        if existing_membership.is_some() {
            return Err(ServiceError::AlreadyExists(
                "User is already a member of this project".to_string()
            ));
        }
        
        // 5. 멤버 추가
        sqlx::query!(
            "INSERT INTO security_user_project (user_id, project_id, role_id) VALUES ($1, $2, $3)",
            user_id, project_id, final_role_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// 프로젝트 멤버십 정보 조회
    async fn get_project_membership(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<MembershipResponse>, ServiceError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                sup.id,
                sup.role_id,
                sr.name as role_name,
                sup.created_at
            FROM security_user_project sup
            LEFT JOIN security_role sr ON sup.role_id = sr.id
            WHERE sup.user_id = $1 AND sup.project_id = $2
            "#,
            user_id, project_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        match result {
            Some(row) => Ok(Some(MembershipResponse {
                is_member: true,
                role_id: Some(row.role_id),
                role_name: row.role_name,
                joined_at: Some(row.created_at.to_rfc3339()),
            })),
            None => Ok(Some(MembershipResponse {
                is_member: false,
                role_id: None,
                role_name: None,
                joined_at: None,
            })),
        }
    }
}
```

## Presentation Layer

### Controller 구현
```rust
/// 프로젝트에 멤버 추가
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/members",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    request_body = AddMemberRequest,
    responses(
        (status = 200, description = "Member added successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Project or user not found"),
        (status = 409, description = "User is already a member"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-members"
)]
pub async fn add_project_member<P, U>(
    path: web::Path<i32>,
    request: web::Json<AddMemberRequest>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
{
    let project_id = path.into_inner();
    
    match use_case
        .add_member_to_project(project_id, request.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let status = match e.to_string().as_str() {
                s if s.contains("not found") => actix_web::http::StatusCode::NOT_FOUND,
                s if s.contains("already") => actix_web::http::StatusCode::CONFLICT,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            HttpResponse::build(status).json(json!({
                "error": format!("Failed to add member: {}", e)
            }))
        }
    }
}
```

### 라우팅 설정
```rust
pub fn configure_routes<P, U>(
    cfg: &mut web::ServiceConfig,
    project_user_use_case: Arc<ProjectUserUseCase<P, U>>,
) where
    P: ProjectService + 'static,
    U: UserService + 'static,
{
    cfg.app_data(web::Data::new(project_user_use_case))
        .service(
            web::scope("/projects")
                // 기존 라우트들...
                .route("/{project_id}/members", web::post().to(add_project_member::<P, U>))
                .route("/{project_id}/members/{user_id}", web::delete().to(remove_project_member::<P, U>))
                .route("/{project_id}/members/{user_id}/membership", web::get().to(check_project_membership::<P, U>))
        )
        .route("/users/{user_id}/projects", web::get().to(get_user_projects::<P, U>));
}
```

## API 명세

### 1. 멤버 추가 API
```
POST /api/projects/{project_id}/members
Content-Type: application/json

Request Body:
{
  "user_id": 123,
  "role_id": 1632  // 선택사항
}

Response (200 OK):
{
  "message": "Member added to project successfully",
  "user_id": 123,
  "project_id": 456,
  "role_id": 1632,
  "role_name": "PROJECT_ADMIN"
}

Error Responses:
- 404: Project or user not found
- 409: User is already a member
- 500: Internal server error
```

### 2. 멤버 제거 API
```
DELETE /api/projects/{project_id}/members/{user_id}

Response (200 OK):
{
  "message": "Member removed from project successfully",
  "user_id": 123,
  "project_id": 456
}

Error Responses:
- 404: Project or user not found
- 500: Internal server error
```

### 3. 멤버십 확인 API
```
GET /api/projects/{project_id}/members/{user_id}/membership

Response (200 OK):
{
  "is_member": true,
  "role_id": 1632,
  "role_name": "PROJECT_ADMIN",
  "joined_at": "2025-01-26T04:39:16Z"
}

Error Responses:
- 404: Project not found
- 500: Internal server error
```

## 성능 고려사항

### 1. 데이터베이스 최적화
- 적절한 인덱스 사용 (`user_id`, `project_id`, `role_id`)
- 효율적인 JOIN 쿼리 사용
- 필요한 컬럼만 SELECT

### 2. 에러 처리
- 적절한 HTTP 상태 코드 반환
- 명확한 에러 메시지 제공
- 로깅을 통한 디버깅 지원

### 3. 트랜잭션 처리
- 멤버 추가/제거 시 데이터 일관성 보장
- 롤백 가능한 트랜잭션 사용

## 보안 고려사항

### 1. 입력 검증
- 사용자 ID, 프로젝트 ID, 역할 ID 유효성 검사
- SQL 인젝션 방지를 위한 파라미터화된 쿼리 사용

### 2. 권한 체크
- 현재는 기본적인 존재 여부 검사만 수행
- 향후 멤버 추가/제거 권한 검증 추가 필요

### 3. 감사 로그
- 멤버 추가/제거 이벤트 로깅
- 사용자 행동 추적 가능

## 확장 가능성

### 1. 배치 작업
- 여러 사용자 동시 추가/제거 API
- CSV 파일을 통한 대량 멤버 관리

### 2. 알림 시스템
- 멤버 추가/제거 시 이메일 알림
- 실시간 웹소켓 알림

### 3. 권한 관리
- 역할별 멤버 관리 권한
- 세분화된 권한 체크

## 테스트 전략

### 1. 단위 테스트
- 각 서비스 메서드별 테스트
- Mock을 사용한 의존성 격리

### 2. 통합 테스트
- 실제 데이터베이스를 사용한 테스트
- API 엔드포인트 전체 테스트

### 3. 성능 테스트
- 대량 데이터 처리 성능 테스트
- 동시 요청 처리 테스트

## 모니터링 및 로깅

### 1. 메트릭 수집
- API 호출 횟수 및 응답 시간
- 에러 발생 빈도 및 유형

### 2. 로그 레벨
- INFO: 정상적인 멤버 추가/제거
- WARN: 중복 멤버십 시도
- ERROR: 데이터베이스 에러

### 3. 알림 설정
- 에러율 임계값 초과 시 알림
- 성능 저하 시 알림
