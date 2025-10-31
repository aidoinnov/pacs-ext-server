# Capability UI 레이블 필드 추가 기술 문서

## 📋 개요

이 문서는 `security_capability` 테이블에 UI 레이블 필드를 추가하는 작업의 기술적 구현 내용을 설명합니다.

## 🏗️ 아키텍처 변경사항

### 데이터베이스 스키마 변경

```sql
-- 기존 테이블 구조
CREATE TABLE security_capability (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 변경된 테이블 구조
CREATE TABLE security_capability (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    display_label VARCHAR(50) NOT NULL DEFAULT '',     -- ✨ 추가
    description TEXT,
    category TEXT NOT NULL,
    category_label VARCHAR(50) NOT NULL DEFAULT '',    -- ✨ 추가
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 인덱스 추가

```sql
-- 카테고리 레이블 검색 성능 향상을 위한 인덱스
CREATE INDEX idx_capability_category_label ON security_capability(category_label);
```

## 🔧 코드 변경사항

### 1. Domain Entity 업데이트

**파일**: `pacs-server/src/domain/entities/capability.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Capability {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub display_label: String,        // ✨ 추가
    pub description: Option<String>,
    pub category: String,
    pub category_label: String,       // ✨ 추가
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCapability {
    pub name: String,
    pub display_name: String,
    pub display_label: String,        // ✨ 추가
    pub description: Option<String>,
    pub category: String,
    pub category_label: String,       // ✨ 추가
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCapability {
    pub display_name: Option<String>,
    pub display_label: Option<String>,    // ✨ 추가
    pub description: Option<String>,
    pub category: Option<String>,
    pub category_label: Option<String>,   // ✨ 추가
    pub is_active: Option<bool>,
}
```

### 2. DTO 업데이트

**파일**: `pacs-server/src/application/dto/role_capability_matrix_dto.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CapabilityInfo {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub display_label: String,        // ✨ 추가
    pub description: Option<String>,
    pub category: String,
    pub category_label: String,       // ✨ 추가
    pub permission_count: i32,
}
```

### 3. Repository 업데이트

**파일**: `pacs-server/src/infrastructure/repositories/capability_repository_impl.rs`

```rust
// 모든 SELECT 쿼리 업데이트
sqlx::query_as::<_, Capability>(
    "SELECT id, name, display_name, display_label, description, category, category_label, 
            is_active, created_at, updated_at
     FROM security_capability
     WHERE id = $1"
)

// INSERT 쿼리 업데이트
sqlx::query_as::<_, Capability>(
    "INSERT INTO security_capability (name, display_name, display_label, description, category, category_label)
     VALUES ($1, $2, $3, $4, $5, $6)
     RETURNING id, name, display_name, display_label, description, category, category_label, 
               is_active, created_at, updated_at"
)

// UPDATE 쿼리 업데이트
let mut query = String::from("UPDATE security_capability SET updated_at = NOW()");
if update.display_label.is_some() {
    query.push_str(&format!(", display_label = ${}", param_count));
    param_count += 1;
}
if update.category_label.is_some() {
    query.push_str(&format!(", category_label = ${}", param_count));
    param_count += 1;
}
```

### 4. Use Case 업데이트

**파일**: `pacs-server/src/application/use_cases/role_capability_matrix_use_case.rs`

```rust
let capability_info = CapabilityInfo {
    id: capability.id,
    name: capability.name,
    display_name: capability.display_name,
    display_label: capability.display_label,      // ✨ 추가
    description: capability.description,
    category: capability.category,
    category_label: capability.category_label,    // ✨ 추가
    permission_count: permissions.len() as i32,
};
```

## 📊 데이터 마이그레이션

### 마이그레이션 파일: `014_add_capability_ui_labels.sql`

```sql
-- 1. 필드 추가
ALTER TABLE security_capability 
ADD COLUMN display_label VARCHAR(50) NOT NULL DEFAULT '',
ADD COLUMN category_label VARCHAR(50) NOT NULL DEFAULT '';

-- 2. 기존 데이터 업데이트
-- MANAGE 카테고리
UPDATE security_capability SET display_label = 'Admin', category_label = 'MANAGE' WHERE name = 'SYSTEM_ADMIN';
UPDATE security_capability SET display_label = 'Users', category_label = 'MANAGE' WHERE name = 'USER_MANAGEMENT';
UPDATE security_capability SET display_label = 'Roles', category_label = 'MANAGE' WHERE name = 'ROLE_MANAGEMENT';
UPDATE security_capability SET display_label = 'Projects', category_label = 'MANAGE' WHERE name = 'PROJECT_MANAGEMENT';

-- PROJECT 카테고리
UPDATE security_capability SET display_label = 'CREATE', category_label = 'PROJECT' WHERE name = 'PROJECT_CREATE';
UPDATE security_capability SET display_label = 'ASSIGN', category_label = 'PROJECT' WHERE name = 'PROJECT_ASSIGN';
UPDATE security_capability SET display_label = 'EDIT', category_label = 'PROJECT' WHERE name = 'PROJECT_EDIT';

-- DICOM 카테고리
UPDATE security_capability SET display_label = 'READ', category_label = 'DICOM' WHERE name = 'DICOM_READ_ACCESS';
UPDATE security_capability SET display_label = 'WRITE', category_label = 'DICOM' WHERE name = 'DICOM_WRITE_ACCESS';
UPDATE security_capability SET display_label = 'DELETE', category_label = 'DICOM' WHERE name = 'DICOM_DELETE_ACCESS';
UPDATE security_capability SET display_label = 'SHARE', category_label = 'DICOM' WHERE name = 'DICOM_SHARE_ACCESS';

-- ANNOTATION 카테고리
UPDATE security_capability SET display_label = 'READ OWN', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_READ_OWN';
UPDATE security_capability SET display_label = 'READ ALL', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_READ_ALL';
UPDATE security_capability SET display_label = 'WRITE', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_WRITE';
UPDATE security_capability SET display_label = 'DELETE', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_DELETE';
UPDATE security_capability SET display_label = 'SHARE', category_label = 'ANNOTATION' WHERE name = 'ANNOTATION_SHARE';

-- MASK 카테고리
UPDATE security_capability SET display_label = 'READ', category_label = 'MASK' WHERE name = 'MASK_READ';
UPDATE security_capability SET display_label = 'WRITE', category_label = 'MASK' WHERE name = 'MASK_WRITE';
UPDATE security_capability SET display_label = 'DELETE', category_label = 'MASK' WHERE name = 'MASK_DELETE';

-- HANGING_PROTOCOL 카테고리
UPDATE security_capability SET display_label = 'MANAGE', category_label = 'HANGING_PROTOCOL' WHERE name = 'HANGING_PROTOCOL_MANAGEMENT';

-- 3. 인덱스 추가
CREATE INDEX idx_capability_category_label ON security_capability(category_label);
```

## 🔍 API 응답 변경사항

### 이전 API 응답

```json
{
  "capabilities_by_category": {
    "관리": [
      {
        "id": 36,
        "name": "USER_MANAGEMENT",
        "display_name": "사용자 관리",
        "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "permission_count": 4
      }
    ]
  }
}
```

### 개선된 API 응답

```json
{
  "capabilities_by_category": {
    "관리": [
      {
        "id": 36,
        "name": "USER_MANAGEMENT",
        "display_name": "사용자 관리",
        "display_label": "Users",        // ✨ 새로 추가
        "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "category_label": "MANAGE",      // ✨ 새로 추가
        "permission_count": 4
      }
    ]
  }
}
```

## 🎨 UI 활용 패턴

### 1. 표 헤더 구성

```javascript
// 카테고리별로 그룹화된 표 헤더
const categoryHeaders = capabilities.reduce((acc, cap) => {
  if (!acc[cap.category_label]) {
    acc[cap.category_label] = [];
  }
  acc[cap.category_label].push(cap.display_label);
  return acc;
}, {});

// 결과: {
//   "MANAGE": ["Admin", "Users", "Roles", "Projects"],
//   "PROJECT": ["CREATE", "ASSIGN", "EDIT"],
//   "DICOM": ["READ", "WRITE", "DELETE", "SHARE"]
// }
```

### 2. 표 셀 렌더링

```javascript
// 각 capability의 표시 레이블
const cellValue = capability.display_label;
const tooltip = `${capability.display_name}: ${capability.description}`;
```

### 3. 필터링 및 검색

```javascript
// 카테고리별 필터링
const filteredByCategory = capabilities.filter(cap => 
  cap.category_label === selectedCategory
);

// 레이블로 검색
const searchResults = capabilities.filter(cap => 
  cap.display_label.toLowerCase().includes(searchTerm.toLowerCase())
);
```

## ⚡ 성능 고려사항

### 1. 인덱스 최적화

```sql
-- 카테고리 레이블 검색을 위한 인덱스
CREATE INDEX idx_capability_category_label ON security_capability(category_label);

-- 복합 인덱스 (필요시)
CREATE INDEX idx_capability_category_active ON security_capability(category, is_active);
```

### 2. 쿼리 최적화

```sql
-- 카테고리별 capability 조회 (인덱스 활용)
SELECT * FROM security_capability 
WHERE category_label = 'MANAGE' 
  AND is_active = true 
ORDER BY display_name;
```

### 3. 캐싱 전략

```rust
// Use Case에서 카테고리별 그룹화 시 캐싱 고려
let capabilities_by_category: HashMap<String, Vec<CapabilityInfo>> = capabilities
    .into_iter()
    .map(|cap| (cap.category_label.clone(), cap))
    .fold(HashMap::new(), |mut acc, (category, cap)| {
        acc.entry(category).or_insert_with(Vec::new).push(cap);
        acc
    });
```

## 🔒 데이터 무결성

### 1. 제약 조건

```sql
-- NOT NULL 제약 조건
ALTER TABLE security_capability 
ALTER COLUMN display_label SET NOT NULL,
ALTER COLUMN category_label SET NOT NULL;

-- 길이 제한
ALTER TABLE security_capability 
ADD CONSTRAINT chk_display_label_length CHECK (LENGTH(display_label) <= 50),
ADD CONSTRAINT chk_category_label_length CHECK (LENGTH(category_label) <= 50);
```

### 2. 데이터 검증

```rust
// Rust 코드에서 데이터 검증
impl NewCapability {
    pub fn validate(&self) -> Result<(), String> {
        if self.display_label.is_empty() {
            return Err("display_label cannot be empty".to_string());
        }
        if self.category_label.is_empty() {
            return Err("category_label cannot be empty".to_string());
        }
        if self.display_label.len() > 50 {
            return Err("display_label too long".to_string());
        }
        if self.category_label.len() > 50 {
            return Err("category_label too long".to_string());
        }
        Ok(())
    }
}
```

## 🧪 테스트 전략

### 1. 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_info_creation() {
        let capability = Capability {
            id: 1,
            name: "USER_MANAGEMENT".to_string(),
            display_name: "사용자 관리".to_string(),
            display_label: "Users".to_string(),
            description: Some("사용자 계정 관리".to_string()),
            category: "관리".to_string(),
            category_label: "MANAGE".to_string(),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let capability_info = CapabilityInfo {
            id: capability.id,
            name: capability.name.clone(),
            display_name: capability.display_name.clone(),
            display_label: capability.display_label.clone(),
            description: capability.description.clone(),
            category: capability.category.clone(),
            category_label: capability.category_label.clone(),
            permission_count: 4,
        };

        assert_eq!(capability_info.display_label, "Users");
        assert_eq!(capability_info.category_label, "MANAGE");
    }
}
```

### 2. 통합 테스트

```rust
#[tokio::test]
async fn test_capability_api_with_labels() {
    let response = test_client
        .get("/api/roles/global/capabilities/matrix")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    
    let data: RoleCapabilityMatrixResponse = response.json().await.unwrap();
    let capability = &data.capabilities_by_category["관리"][0];
    
    assert!(!capability.display_label.is_empty());
    assert!(!capability.category_label.is_empty());
}
```

## 📈 모니터링 및 로깅

### 1. 성능 메트릭

```rust
// API 응답 시간 모니터링
let start = std::time::Instant::now();
let result = capability_service.get_global_role_capability_matrix_paginated(
    page, size, search.as_deref(), scope.as_deref()
).await?;
let duration = start.elapsed();

tracing::info!(
    "Capability matrix query completed in {:?}",
    duration
);
```

### 2. 에러 로깅

```rust
// 데이터베이스 에러 로깅
match sqlx::query_as::<_, Capability>(query)
    .bind(id)
    .fetch_optional(&self.pool)
    .await
{
    Ok(Some(capability)) => Ok(Some(capability)),
    Ok(None) => Ok(None),
    Err(e) => {
        tracing::error!("Database error in find_by_id: {}", e);
        Err(e.into())
    }
}
```

## 🚀 배포 전략

### 1. 마이그레이션 순서

1. **스키마 변경**: 필드 추가 (NOT NULL DEFAULT으로 안전)
2. **데이터 업데이트**: 기존 데이터에 레이블 값 설정
3. **인덱스 추가**: 성능 최적화
4. **애플리케이션 배포**: 새 코드 배포

### 2. 롤백 계획

```sql
-- 롤백 마이그레이션 (필요시)
ALTER TABLE security_capability 
DROP COLUMN display_label,
DROP COLUMN category_label;

DROP INDEX IF EXISTS idx_capability_category_label;
```

## 📚 참고 자료

- [PostgreSQL ALTER TABLE 문서](https://www.postgresql.org/docs/current/sql-altertable.html)
- [SQLx 마이그레이션 가이드](https://docs.rs/sqlx/latest/sqlx/migrate/index.html)
- [Rust Serde 직렬화](https://serde.rs/)
- [Actix-web API 개발](https://actix.rs/docs/)

## 🎯 결론

이 기술 문서는 Capability UI 레이블 필드 추가 작업의 모든 기술적 세부사항을 다룹니다. 구현된 솔루션은 확장 가능하고 유지보수가 용이하며, 향후 UI 개선과 다국어 지원을 위한 견고한 기반을 제공합니다.
