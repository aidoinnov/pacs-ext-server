# 프로젝트 Repository Status 컬럼 에러 수정 - 기술 문서

## 📋 문서 개요
- **문서명**: 프로젝트 Repository Status 컬럼 에러 수정 기술 문서
- **작성일**: 2025-01-26
- **작성자**: AI Assistant
- **버전**: 1.0

## 🔍 기술적 배경

### 문제 상황
`PUT /api/projects/{project_id}/users/{user_id}/role` API 호출 시 다음과 같은 에러가 발생했습니다:

```json
{
  "error": "Failed to assign role: Database error: no column found for name: status"
}
```

### 기술적 원인 분석

#### 1. 엔티티 구조
`Project` 엔티티는 다음과 같이 정의되어 있습니다:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub status: ProjectStatus,  // ← 이 필드가 문제의 원인
    pub created_at: DateTime<Utc>,
}
```

#### 2. 데이터베이스 스키마
`security_project` 테이블에는 `status` 컬럼이 존재합니다:

```sql
CREATE TABLE security_project (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    status project_status_enum NOT NULL DEFAULT 'ACTIVE',  -- ← 이 컬럼이 존재
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### 3. Repository 구현 문제
`project_repository_impl.rs`의 SQL 쿼리들이 `status` 컬럼을 SELECT하지 않아서 SQLx가 매핑할 때 에러가 발생했습니다.

## 🔧 해결 방법

### 수정 전후 비교

#### 수정 전 (문제가 있던 코드)
```rust
async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT id, name, description, is_active, created_at  -- ← status 누락
         FROM security_project
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}
```

#### 수정 후 (해결된 코드)
```rust
async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT id, name, description, is_active, status, created_at  -- ← status 추가
         FROM security_project
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}
```

### 수정된 모든 함수

#### 1. `find_by_id`
```sql
-- 수정 전
SELECT id, name, description, is_active, created_at

-- 수정 후  
SELECT id, name, description, is_active, status, created_at
```

#### 2. `find_by_name`
```sql
-- 수정 전
SELECT id, name, description, is_active, created_at

-- 수정 후
SELECT id, name, description, is_active, status, created_at
```

#### 3. `find_all`
```sql
-- 수정 전
SELECT id, name, description, is_active, created_at

-- 수정 후
SELECT id, name, description, is_active, status, created_at
```

#### 4. `find_active`
```sql
-- 수정 전
SELECT id, name, description, is_active, created_at

-- 수정 후
SELECT id, name, description, is_active, status, created_at
```

#### 5. `create`
```sql
-- 수정 전
RETURNING id, name, description, is_active, created_at

-- 수정 후
RETURNING id, name, description, is_active, status, created_at
```

#### 6. `update`
```sql
-- 수정 전
RETURNING id, name, description, is_active, created_at

-- 수정 후
RETURNING id, name, description, is_active, status, created_at
```

## 🧪 테스트 및 검증

### 테스트 시나리오
1. **컴파일 테스트**: `cargo check` 실행하여 컴파일 에러 없음 확인
2. **서버 시작 테스트**: `cargo run &` 백그라운드 실행 성공 확인
3. **API 테스트**: 실제 API 엔드포인트 호출하여 에러 해결 확인

### 테스트 결과

#### API 테스트 명령어
```bash
curl -X PUT "http://localhost:8080/api/projects/2/users/1/role" \
     -H "Content-Type: application/json" \
     -d '{"role_id": 1632}' -v
```

#### 테스트 결과
- **상태 코드**: `HTTP 200 OK`
- **응답 메시지**: `{"message":"Role assigned successfully","user_id":1,"project_id":2,"role_id":1632}`
- **에러**: ❌ "no column found for name: status" 에러 **완전 해결**

## 🔍 기술적 세부사항

### SQLx 매핑 메커니즘
SQLx는 `query_as::<_, Project>`를 사용할 때 다음과 같은 과정을 거칩니다:

1. **컬럼 순서 확인**: SELECT 절의 컬럼 순서와 엔티티 필드 순서 매칭
2. **타입 변환**: PostgreSQL 타입을 Rust 타입으로 변환
3. **매핑 검증**: 모든 필드가 올바르게 매핑되는지 확인

### 에러 발생 과정
1. `Project` 엔티티는 6개 필드를 가지고 있음 (`id`, `name`, `description`, `is_active`, `status`, `created_at`)
2. SQL 쿼리는 5개 컬럼만 SELECT (`id`, `name`, `description`, `is_active`, `created_at`)
3. SQLx가 `status` 필드를 매핑하려고 시도하지만 해당 컬럼이 없어서 에러 발생

### 해결 과정
1. 모든 SQL 쿼리에 `status` 컬럼 추가
2. 컬럼 순서를 엔티티 필드 순서와 일치시킴
3. SQLx가 모든 필드를 올바르게 매핑할 수 있도록 수정

## 🚨 주의사항 및 모범 사례

### 주의사항
1. **컬럼 순서**: SELECT 절의 컬럼 순서가 엔티티 필드 순서와 일치해야 함
2. **타입 일치**: 데이터베이스 컬럼 타입과 Rust 필드 타입이 호환되어야 함
3. **NULL 처리**: NULL 허용 컬럼은 `Option<T>` 타입으로 매핑해야 함

### 모범 사례
1. **일관성 유지**: 모든 Repository 함수에서 동일한 컬럼 세트 사용
2. **명시적 매핑**: `FromRow` 트레이트 사용 시 모든 필드 명시
3. **테스트 강화**: Repository 함수에 대한 단위 테스트 작성

## 🔄 향후 개선 방안

### 1. 자동화된 검증
```rust
// 매크로를 사용한 자동 검증 예시
#[derive(FromRow)]
pub struct Project {
    #[sqlx(rename = "id")]
    pub id: i32,
    #[sqlx(rename = "name")]
    pub name: String,
    // ... 다른 필드들
}
```

### 2. 테스트 자동화
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_find_by_id_includes_all_fields() {
        // 모든 필드가 올바르게 매핑되는지 테스트
    }
}
```

### 3. 문서화 강화
- SQL 쿼리 작성 가이드라인 수립
- 엔티티-테이블 매핑 규칙 문서화
- Repository 패턴 모범 사례 가이드 작성

## 📚 관련 자료

### 참고 문서
- [SQLx 공식 문서](https://docs.rs/sqlx/latest/sqlx/)
- [Rust FromRow 트레이트](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html)
- [PostgreSQL 데이터 타입](https://www.postgresql.org/docs/current/datatype.html)

### 관련 파일
- `pacs-server/src/domain/entities/project.rs`
- `pacs-server/src/infrastructure/repositories/project_repository_impl.rs`
- `pacs-server/migrations/` (데이터베이스 스키마)

## 🎯 결론

이번 수정을 통해 프로젝트 Repository의 SQL 쿼리와 엔티티 간의 매핑 문제가 완전히 해결되었습니다. 앞으로는 엔티티 필드와 SQL 쿼리 컬럼 간의 일치성을 더욱 엄격하게 관리하여 유사한 문제가 발생하지 않도록 해야 합니다.
