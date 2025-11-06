# 🗃️ PACS 마스크 업로드 Repository 구현

## 📋 개요
PACS 마스크 업로드 시스템을 위한 Repository 패턴 구현 문서입니다. Clean Architecture 원칙에 따라 데이터 접근 계층을 추상화했습니다.

## 🏗️ 아키텍처 설계

### 1. Repository 계층 구조
```
Domain Layer (Trait 정의)
├── MaskGroupRepository
├── MaskRepository
└── ServiceError

Infrastructure Layer (구현체)
├── MaskGroupRepositoryImpl
├── MaskRepositoryImpl
└── PostgreSQL 연동
```

### 2. 의존성 구조
```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json", "bigdecimal"] }
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
num-traits = "0.2"
```

## 🔧 핵심 컴포넌트

### 1. MaskGroupRepository Trait
```rust
#[async_trait]
pub trait MaskGroupRepository: Send + Sync {
    async fn create(&self, new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError>;
    async fn get_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError>;
    async fn update(&self, id: i32, update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError>;
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;
    async fn list(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;
    async fn get_masks_in_group(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError>;
    async fn get_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError>;
    async fn count(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
    ) -> Result<i64, ServiceError>;
}
```

### 2. MaskRepository Trait
```rust
#[async_trait]
pub trait MaskRepository: Send + Sync {
    async fn create(&self, new_mask: &NewMask) -> Result<Mask, ServiceError>;
    async fn get_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError>;
    async fn update(&self, id: i32, update_mask: &UpdateMask) -> Result<Mask, ServiceError>;
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;
    async fn list(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    async fn get_stats(&self, mask_group_id: Option<i32>) -> Result<MaskStats, ServiceError>;
    async fn count(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
    ) -> Result<i64, ServiceError>;
}
```

## 🚀 구현체 상세

### 1. MaskGroupRepositoryImpl
PostgreSQL을 사용한 MaskGroupRepository 구현체입니다.

#### 주요 기능
- CRUD 작업 구현
- 동적 쿼리 바인딩
- BigDecimal → i64 변환
- 에러 처리

#### 핵심 메서드 구현

##### Create 메서드
```rust
async fn create(&self, new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError> {
    let result = sqlx::query_as!(
        MaskGroup,
        r#"
        INSERT INTO annotation_mask_group (
            annotation_id, group_name, model_name, version, modality,
            slice_count, mask_type, description, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, annotation_id, group_name, model_name, version, modality,
                  slice_count, mask_type, description, created_by, created_at, updated_at
        "#,
        new_mask_group.annotation_id,
        new_mask_group.group_name,
        new_mask_group.model_name,
        new_mask_group.version,
        new_mask_group.modality,
        new_mask_group.slice_count,
        new_mask_group.mask_type,
        new_mask_group.description,
        new_mask_group.created_by,
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| ServiceError::DatabaseError(format!("Failed to create mask group: {}", e)))?;

    Ok(result)
}
```

##### List 메서드 (동적 쿼리)
```rust
async fn list(
    &self,
    annotation_id: Option<i32>,
    created_by: Option<i32>,
    modality: Option<String>,
    mask_type: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<MaskGroup>, ServiceError> {
    let mut query = r#"
        SELECT id, annotation_id, group_name, model_name, version, modality,
               slice_count, mask_type, description, created_by, created_at, updated_at
        FROM annotation_mask_group
        WHERE 1 = 1
    "#.to_string();

    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 1;

    // 동적 WHERE 절 구성
    if let Some(ann_id) = annotation_id {
        query.push_str(&format!(" AND annotation_id = ${}", param_count));
        params.push(Box::new(ann_id));
        param_count += 1;
    }
    // ... 다른 필터들

    query.push_str(&format!(" ORDER BY created_at DESC OFFSET ${} LIMIT ${}", param_count, param_count + 1));
    params.push(Box::new(offset.unwrap_or(0)));
    params.push(Box::new(limit.unwrap_or(50)));

    // 쿼리 실행
    let query_result = sqlx::query(&query)
        .bind(params[0].as_ref())
        .bind(params[1].as_ref())
        // ... 다른 파라미터들
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to list mask groups: {}", e)))?;

    // Row를 MaskGroup으로 변환
    let mask_groups: Vec<MaskGroup> = query_result
        .into_iter()
        .map(|row| MaskGroup {
            id: row.get("id"),
            annotation_id: row.get("annotation_id"),
            group_name: row.get("group_name"),
            model_name: row.get("model_name"),
            version: row.get("version"),
            modality: row.get("modality"),
            slice_count: row.get("slice_count"),
            mask_type: row.get("mask_type"),
            description: row.get("description"),
            created_by: row.get("created_by"),
            created_at: DateTime::from_naive_utc_and_offset(
                row.get::<NaiveDateTime, _>("created_at").unwrap_or_default(), 
                Utc
            ),
            updated_at: DateTime::from_naive_utc_and_offset(
                row.get::<NaiveDateTime, _>("updated_at").unwrap_or_default(), 
                Utc
            ),
        })
        .collect();

    Ok(mask_groups)
}
```

### 2. MaskRepositoryImpl
PostgreSQL을 사용한 MaskRepository 구현체입니다.

#### 주요 기능
- CRUD 작업 구현
- 동적 쿼리 바인딩
- BigDecimal → i64 변환
- 에러 처리

## 🔧 기술적 해결사항

### 1. BigDecimal → i64 변환
```rust
use num_traits::cast::ToPrimitive;

// 방법 1: ToPrimitive trait 사용
let total_size_bytes = result.total_size_bytes
    .unwrap_or_default()
    .to_i64()
    .unwrap_or(0);

// 방법 2: String 변환 후 파싱 (fallback)
let total_size_bytes = result.total_size_bytes
    .unwrap_or_default()
    .to_string()
    .parse::<i64>()
    .unwrap_or(0);
```

### 2. 동적 쿼리 바인딩
```rust
// 파라미터 타입을 Box<dyn sqlx::Encode>로 통일
let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();

// 파라미터 추가
if let Some(ann_id) = annotation_id {
    query.push_str(&format!(" AND annotation_id = ${}", param_count));
    params.push(Box::new(ann_id));
    param_count += 1;
}

// 쿼리 실행 시 바인딩
let query_result = sqlx::query(&query)
    .bind(params[0].as_ref())
    .bind(params[1].as_ref())
    // ... 다른 파라미터들
    .fetch_all(&self.pool)
    .await?;
```

### 3. Option<T> 타입 처리
```rust
// 데이터베이스에서 Option<NaiveDateTime>을 DateTime<Utc>로 변환
created_at: DateTime::from_naive_utc_and_offset(
    row.get::<NaiveDateTime, _>("created_at").unwrap_or_default(), 
    Utc
),
```

### 4. 에러 처리
```rust
.map_err(|e| ServiceError::DatabaseError(format!("Failed to create mask group: {}", e)))?;
```

## 📊 성능 최적화

### 1. 인덱스 활용
- `annotation_id` 인덱스로 어노테이션별 조회 최적화
- `created_by` 인덱스로 사용자별 조회 최적화
- `modality`, `mask_type` 인덱스로 필터링 최적화

### 2. 페이징 처리
```rust
query.push_str(&format!(" ORDER BY created_at DESC OFFSET ${} LIMIT ${}", param_count, param_count + 1));
params.push(Box::new(offset.unwrap_or(0)));
params.push(Box::new(limit.unwrap_or(50)));
```

### 3. 배치 처리
- 여러 레코드를 한 번에 처리
- 트랜잭션 사용으로 일관성 보장

## 🧪 테스트 전략

### 1. 단위 테스트
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    
    #[tokio::test]
    async fn test_create_mask_group() {
        let pool = create_test_pool().await;
        let repo = MaskGroupRepositoryImpl::new(pool);
        
        let new_mask_group = NewMaskGroup::new(
            1, // annotation_id
            "Test Group".to_string(),
            "Test Model".to_string(),
            "1.0".to_string(),
            "CT".to_string(),
            10, // slice_count
            "segmentation".to_string(),
            "Test description".to_string(),
            Some(1), // created_by
        );
        
        let result = repo.create(&new_mask_group).await;
        assert!(result.is_ok());
        
        let created = result.unwrap();
        assert_eq!(created.group_name, Some("Test Group".to_string()));
        assert_eq!(created.annotation_id, 1);
    }
}
```

### 2. 통합 테스트
- 실제 데이터베이스와 연동 테스트
- 트랜잭션 롤백으로 데이터 정리
- 에러 시나리오 테스트

## 🔄 의존성 주입

### 1. main.rs에서 설정
```rust
// Repository 초기화
let mask_group_repo = MaskGroupRepositoryImpl::new(pool.clone());
let mask_repo = MaskRepositoryImpl::new(pool.clone());

// Service에 주입 (향후 구현)
// let mask_group_service = MaskGroupService::new(mask_group_repo);
// let mask_service = MaskService::new(mask_repo);
```

### 2. 테스트에서 Mock 사용
```rust
// Mock Repository 구현
struct MockMaskGroupRepository {
    // Mock 데이터 저장
}

#[async_trait]
impl MaskGroupRepository for MockMaskGroupRepository {
    // Mock 구현
}
```

## 📈 모니터링 및 로깅

### 1. 쿼리 성능 모니터링
```rust
let start = std::time::Instant::now();
let result = sqlx::query_as!(MaskGroup, query)
    .fetch_all(&self.pool)
    .await?;
let duration = start.elapsed();

if duration.as_millis() > 1000 {
    warn!("Slow query detected: {}ms", duration.as_millis());
}
```

### 2. 에러 로깅
```rust
error!("Database error in create_mask_group: {}", error);
```

## 🔒 보안 고려사항

### 1. SQL 인젝션 방지
- sqlx의 매개변수화된 쿼리 사용
- 동적 쿼리 구성 시 주의

### 2. 권한 검증
- 사용자별 데이터 접근 제어
- 어노테이션 소유권 확인

### 3. 데이터 검증
- 입력 데이터 유효성 검사
- 비즈니스 규칙 적용

## 📚 참고 자료
- [SQLx 문서](https://docs.rs/sqlx/latest/sqlx/)
- [PostgreSQL 문서](https://www.postgresql.org/docs/)
- [Rust async-trait](https://docs.rs/async-trait/latest/async_trait/)

---
**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
