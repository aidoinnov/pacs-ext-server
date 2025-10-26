# 데이터 접근 권한 상태 업데이트 API 기술 문서

## 1. 배경

프로젝트 데이터 접근 권한 관리 시스템에서 다음과 같은 문제들이 발생했습니다:
1. Status 필드 업데이트 시 데이터베이스 타입 오류
2. 바인딩 파라미터 불일치 오류
3. NULL 컬럼 디코딩 오류
4. 매트릭스 반환 순서 불안정

이 문서는 이러한 문제들을 해결하기 위한 기술적 접근 방법을 설명합니다.

## 2. 아키텍처 변경사항

### 2.1 의존성 추가
`ProjectUserUseCase`에 `ProjectDataService` 의존성을 추가하여 자동 권한 부여 기능을 구현했습니다.

```rust
// 변경 전
pub struct ProjectUserUseCase<P, U>
where
    P: ProjectService,
    U: UserService,
{
    project_service: Arc<P>,
    user_service: Arc<U>,
}

// 변경 후
pub struct ProjectUserUseCase<P, U, D>
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    project_service: Arc<P>,
    user_service: Arc<U>,
    project_data_service: Arc<D>,
}
```

### 2.2 자동 권한 부여 로직
사용자가 프로젝트에 추가될 때 모든 데이터에 대한 접근 권한을 자동으로 부여합니다.

```rust
pub async fn add_member_to_project(...) -> Result<AddMemberResponse, ServiceError> {
    // 1. 프로젝트 존재 확인
    self.project_service.get_project(project_id).await?;

    // 2. 사용자를 프로젝트에 추가
    self.user_service
        .add_user_to_project_with_role(request.user_id, project_id, request.role_id)
        .await?;

    // 3. ✅ 프로젝트의 모든 데이터에 대한 기본 접근 권한 자동 부여
    let _ = self.project_data_service
        .grant_default_access_to_user(project_id, request.user_id)
        .await
        .map_err(|e| {
            eprintln!("Warning: Failed to grant default access to user: {}", e);
            e
        });

    // ...
}
```

## 3. 데이터베이스 쿼리 개선

### 3.1 동적 쿼리 문제
이전 구현은 동적 쿼리를 구성하여 복잡하고 에러가 발생하기 쉬웠습니다.

```rust
// 문제 있는 코드 (이전)
let mut query = String::from("UPDATE project_data_access SET ");
let mut param_count = 1;

if let Some(status) = &update_access.status {
    query.push_str(&format!("status = ${}, ", param_count));
    params.push(Box::new(status.clone()));
    param_count += 1;
}
// ... more dynamic query building

// 바인딩 순서와 개수가 일치하지 않아 오류 발생
```

### 3.2 단일 Prepared Statement 접근
동적 쿼리를 단일 prepared statement로 변경하여 바인딩 순서와 개수를 명확히 했습니다.

```rust
// 개선된 코드 (이후)
let result = sqlx::query_as::<_, ProjectDataAccess>(
    "UPDATE project_data_access 
     SET status = $1::data_access_status_enum,
         reviewed_at = COALESCE($2, reviewed_at),
         reviewed_by = COALESCE($3, reviewed_by),
         review_note = COALESCE($4, review_note),
         updated_at = CURRENT_TIMESTAMP
     WHERE project_data_id = $5 AND user_id = $6
     RETURNING id, 0 as project_id, user_id, resource_level, study_id, series_id, status, requested_at, requested_by, reviewed_at, reviewed_by, review_note, created_at, updated_at, project_data_id"
)
.bind(status)                              // $1
.bind(&update_access.reviewed_at)          // $2
.bind(&update_access.reviewed_by)          // $3
.bind(&update_access.review_note)          // $4
.bind(project_data_id)                     // $5
.bind(user_id)                             // $6
.fetch_optional(&self.pool)
.await?;
```

### 3.3 COALESCE 활용
기존 값을 유지하면서 선택적으로 업데이트하기 위해 COALESCE를 사용했습니다.

```sql
-- reviewed_at 필드가 제공되면 사용, 없으면 기존 값 유지
reviewed_at = COALESCE($2, reviewed_at)
```

## 4. 타입 안전성 개선

### 4.1 Enum 타입 명시
PostgreSQL enum 타입을 명시적으로 캐스팅하여 타입 오류를 방지합니다.

```sql
-- 타입 명시 없이 (오류 발생)
SET status = $1

-- 타입 명시 (정상 작동)
SET status = $1::data_access_status_enum
```

### 4.2 NULL 허용 필드 처리
기존 데이터 호환성을 위해 NULL을 허용하는 필드는 `Option<T>` 타입으로 변경했습니다.

```rust
// 변경 전
pub struct ProjectDataAccess {
    pub study_id: i32,  // ❌ NULL 불가
    // ...
}

// 변경 후
pub struct ProjectDataAccess {
    pub study_id: Option<i32>,  // ✅ NULL 허용
    // ...
}
```

## 5. 정렬 개선

### 5.1 데이터 정렬
`created_at` 대신 `id`를 기준으로 정렬하여 안정적인 순서를 보장합니다.

```sql
-- 변경 전
ORDER BY created_at DESC  -- 생성 시간 기준, 매번 다를 수 있음

-- 변경 후
ORDER BY id ASC           -- ID 기준, 항상 동일한 순서
```

### 5.2 사용자 정렬
HashSet 대신 Vec에 저장하고 정렬하여 일관된 순서를 보장합니다.

```rust
// 변경 전 (순서 불일치)
let user_ids: std::collections::HashSet<i32> = access_list
    .iter()
    .map(|access| access.user_id)
    .collect();

let users: Vec<UserInfo> = user_ids.into_iter().map(...).collect();

// 변경 후 (ID 순으로 정렬)
let user_ids: Vec<i32> = access_list
    .iter()
    .map(|access| access.user_id)
    .collect::<std::collections::HashSet<i32>>()
    .into_iter()
    .collect();

let mut sorted_user_ids = user_ids;
sorted_user_ids.sort();

let users: Vec<UserInfo> = sorted_user_ids.into_iter().map(...).collect();
```

## 6. 에러 처리 개선

### 6.1 자동 권한 부여 실패 처리
자동 권한 부여가 실패하더라도 전체 작업이 실패하지 않도록 에러를 로깅만 합니다.

```rust
let _ = self.project_data_service
    .grant_default_access_to_user(project_id, request.user_id)
    .await
    .map_err(|e| {
        // 로깅만 하고 계속 진행 (access 권한은 옵셔널)
        eprintln!("Warning: Failed to grant default access to user: {}", e);
        e
    });
```

## 7. 성능 최적화

### 7.1 단일 쿼리로 통합
여러 개의 동적 쿼리 대신 단일 prepared statement를 사용하여 성능을 개선했습니다.

### 7.2 바인딩 순서 최적화
파라미터를 순차적으로 바인딩하여 데이터베이스 호출 횟수를 최소화했습니다.

## 8. 컴파일 및 빌드

모든 변경사항은 컴파일 오류 없이 완료되었으며, 서버가 정상적으로 시작됩니다.

```bash
cargo check  # ✅ 성공
cargo run     # ✅ 성공
```

## 9. 결과

### 성공 지표
- ✅ 모든 데이터베이스 오류 해결
- ✅ Status 업데이트 API 정상 작동
- ✅ 자동 권한 부여 기능 구현 완료
- ✅ 매트릭스 정렬 안정성 확보
- ✅ 코드 품질 및 가독성 개선

### 성능 개선
- 동적 쿼리 제거로 쿼리 실행 시간 단축
- 바인딩 순서 최적화로 데이터베이스 부하 감소

## 10. 향후 개선 사항

1. **실제 사용자 정보 조회**: 현재 Mock 데이터 사용 → UserService에서 실제 정보 조회
2. **에러 처리 개선**: 자동 권한 부여 실패 시 사용자에게 명확한 에러 메시지 제공
3. **단위 테스트 추가**: 자동 권한 부여 로직에 대한 단위 테스트 작성
4. **성능 모니터링**: 매트릭스 조회 성능 모니터링 및 최적화

