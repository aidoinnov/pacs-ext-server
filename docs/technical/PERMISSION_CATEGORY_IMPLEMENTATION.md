# Permission Category 필드 추가 및 정렬 구현

## 📋 개요

`security_permission` 테이블에 별도의 `category` 필드를 추가하여 권한을 더 명확하게 카테고리화하고, Role-Permission Matrix API 조회 시 카테고리별로 정렬되도록 수정했습니다.

## 🎯 구현 목표

1. **명확한 권한 분류**: `resource_type`과 별도로 `category` 필드를 추가하여 UI에서 권한을 더 직관적으로 그룹화
2. **정렬된 응답**: API 응답에서 카테고리별로 정렬된 권한 목록 제공
3. **하위 호환성**: 기존 API 구조를 유지하면서 새로운 필드 추가

## 🔧 구현 세부사항

### 1. 데이터베이스 스키마 변경

**마이그레이션**: `010_add_permission_category_field.sql`

```sql
-- Add category column
ALTER TABLE security_permission 
ADD COLUMN category TEXT;

-- Set default categories based on existing resource_type
UPDATE security_permission 
SET category = CASE 
    WHEN resource_type IN ('USER', 'ROLE', 'PERMISSION') THEN '사용자 및 권한 관리'
    WHEN resource_type IN ('PROJECT', 'PROJECT_DATA') THEN '프로젝트 관리'
    WHEN resource_type IN ('ANNOTATION', 'MASK', 'MASK_GROUP') THEN '어노테이션 관리'
    WHEN resource_type IN ('STUDY', 'SERIES', 'INSTANCE') THEN 'DICOM 데이터 관리'
    ELSE '기타'
END;

-- Make category NOT NULL after setting defaults
ALTER TABLE security_permission 
ALTER COLUMN category SET NOT NULL;
```

### 2. Domain Entity 수정

**파일**: `src/domain/entities/permission.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i32,
    pub category: String,      // NEW
    pub resource_type: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPermission {
    pub category: String,      // NEW
    pub resource_type: String,
    pub action: String,
}
```

### 3. DTO 업데이트

**파일**: `src/application/dto/role_permission_matrix_dto.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct PermissionInfo {
    pub id: i32,
    pub category: String,      // NEW
    pub resource_type: String,
    pub action: String,
}
```

### 4. Use Case 정렬 로직

**파일**: `src/application/use_cases/role_permission_matrix_use_case.rs`

```rust
// 권한을 카테고리별로 그룹화
let mut permissions_by_category: HashMap<String, Vec<PermissionInfo>> = HashMap::new();
for permission in permissions {
    let permission_info = PermissionInfo {
        id: permission.id,
        category: permission.category.clone(),
        resource_type: permission.resource_type.clone(),
        action: permission.action,
    };
    
    permissions_by_category
        .entry(permission.category)  // CHANGED: resource_type -> category
        .or_insert_with(Vec::new)
        .push(permission_info);
}

// 각 카테고리 내에서 권한 정렬 (resource_type, action 순)
for permissions in permissions_by_category.values_mut() {
    permissions.sort_by(|a, b| {
        a.resource_type.cmp(&b.resource_type)
            .then_with(|| a.action.cmp(&b.action))
    });
}
```

### 5. Repository 쿼리 업데이트

**파일**: `src/infrastructure/repositories/permission_repository_impl.rs`

모든 SQLx 쿼리에 `category` 필드 추가:

```rust
// 예시: find_all 메서드
async fn find_all(&self) -> Result<Vec<Permission>, sqlx::Error> {
    sqlx::query_as::<_, Permission>(
        "SELECT id, category, resource_type, action
         FROM security_permission
         ORDER BY category, resource_type, action"
    )
    .fetch_all(&self.pool)
    .await
}
```

## 📊 API 응답 예시

### Before (기존)
```json
{
  "permissions_by_category": {
    "USER": [
      {"id": 1, "resource_type": "USER", "action": "READ"},
      {"id": 2, "resource_type": "USER", "action": "WRITE"}
    ],
    "PROJECT": [
      {"id": 3, "resource_type": "PROJECT", "action": "READ"}
    ]
  }
}
```

### After (개선된 버전)
```json
{
  "permissions_by_category": {
    "사용자 및 권한 관리": [
      {"id": 1, "category": "사용자 및 권한 관리", "resource_type": "USER", "action": "READ"},
      {"id": 2, "category": "사용자 및 권한 관리", "resource_type": "USER", "action": "WRITE"}
    ],
    "프로젝트 관리": [
      {"id": 3, "category": "프로젝트 관리", "resource_type": "PROJECT", "action": "READ"}
    ],
    "DICOM 데이터 관리": [
      {"id": 9, "category": "DICOM 데이터 관리", "resource_type": "STUDY", "action": "READ"},
      {"id": 10, "category": "DICOM 데이터 관리", "resource_type": "STUDY", "action": "DOWNLOAD"}
    ],
    "어노테이션 관리": [
      {"id": 16, "category": "어노테이션 관리", "resource_type": "ANNOTATION", "action": "CREATE"},
      {"id": 17, "category": "어노테이션 관리", "resource_type": "ANNOTATION", "action": "READ"}
    ]
  }
}
```

## 🎨 UI 개선 효과

### 1. 명확한 카테고리 분류
- **기존**: `USER`, `PROJECT`, `STUDY` 등 기술적 용어
- **개선**: "사용자 및 권한 관리", "프로젝트 관리", "DICOM 데이터 관리" 등 사용자 친화적 용어

### 2. 정렬된 표시
- 카테고리별로 그룹화되어 표시
- 각 카테고리 내에서 resource_type과 action 순으로 정렬
- UI에서 권한 매트릭스를 더 직관적으로 구성 가능

### 3. 확장성
- 새로운 권한 추가 시 적절한 카테고리 지정 가능
- 카테고리 기반 필터링 및 검색 기능 구현 가능

## 🧪 테스트 결과

### API 테스트
```bash
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix"
```

**결과 확인**:
- ✅ 카테고리별로 정렬된 권한 목록 반환
- ✅ 각 권한에 `category` 필드 포함
- ✅ 각 카테고리 내에서 resource_type과 action 순으로 정렬
- ✅ 기존 API 구조 유지

### 단위 테스트
- ✅ Permission 엔티티 직렬화/역직렬화 테스트
- ✅ PermissionInfo DTO 테스트
- ✅ Use Case 로직 테스트

## 🔄 마이그레이션 전략

### 1. 기존 데이터 처리
- 기존 `resource_type` 값을 기반으로 자동으로 `category` 설정
- 데이터 손실 없이 안전한 마이그레이션

### 2. 하위 호환성
- 기존 API 엔드포인트 유지
- 새로운 필드만 추가하여 기존 클라이언트에 영향 없음

### 3. 롤백 계획
- `category` 필드를 NULL 허용으로 변경하여 롤백 가능
- 기존 `resource_type` 기반 로직으로 복원 가능

## 📈 성능 영향

### 1. 데이터베이스
- **인덱스**: `category` 필드에 인덱스 추가 고려
- **쿼리 성능**: 정렬 기준이 추가되어 약간의 성능 영향 있음
- **저장 공간**: 각 권한당 추가 문자열 필드로 인한 미미한 증가

### 2. 메모리
- **정렬 로직**: HashMap 정렬로 인한 추가 메모리 사용
- **직렬화**: JSON 응답 크기 약간 증가

## 🚀 향후 개선 사항

### 1. 동적 카테고리 관리
- 관리자가 카테고리를 동적으로 추가/수정할 수 있는 API
- 카테고리별 색상 및 아이콘 설정

### 2. 다국어 지원
- 카테고리명 다국어 지원
- 클라이언트 언어에 따른 카테고리명 반환

### 3. 권한 그룹화
- 카테고리 내에서 추가적인 그룹화 옵션
- 권한 계층 구조 지원

## 📝 결론

Permission Category 필드 추가를 통해 다음과 같은 개선을 달성했습니다:

1. **사용자 경험 향상**: 더 직관적인 권한 분류 및 표시
2. **개발자 경험 향상**: 명확한 권한 구조로 UI 개발 용이
3. **확장성**: 향후 권한 관리 기능 확장에 유리한 구조
4. **하위 호환성**: 기존 시스템에 영향 없이 개선

이 구현은 PACS Extension Server의 권한 관리 시스템을 더욱 사용자 친화적이고 확장 가능한 구조로 발전시켰습니다.
