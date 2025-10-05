# DB 트랜잭션 처리 검토 및 최적화 가이드

## 📋 검토 요약

현재 서비스 레이어의 DB 트랜잭션 처리를 검토하여 **원자성 문제**, **Race Condition 위험**, **성능 최적화** 가능성을 식별했습니다.

---

## 🔴 심각한 문제 (즉시 수정 필요)

### 1. Race Condition 위험이 있는 작업들

모든 "확인 → INSERT" 패턴은 동시 요청 시 중복 생성 위험이 있습니다.

| 메서드 | 파일 | 문제 | 해결방안 |
|--------|------|------|----------|
| `add_user_to_project` | user_service.rs:130 | 존재확인 → 중복확인 → INSERT | INSERT ... ON CONFLICT |
| `assign_permission_to_role` | permission_service.rs:123 | 존재확인 → 중복확인 → INSERT | INSERT ... ON CONFLICT |
| `assign_permission_to_project` | permission_service.rs:189 | 존재확인 → 중복확인 → INSERT | INSERT ... ON CONFLICT |
| `assign_role_to_project` | project_service.rs:198 | 존재확인 → 중복확인 → INSERT | INSERT ... ON CONFLICT |
| `login` | auth_service.rs:40 | find → create | UPSERT (ON CONFLICT) |

### 문제 시나리오 예시

```rust
// ❌ 현재 코드 (Race Condition 위험)
// 요청 A: 사용자 존재 확인 ✓
// 요청 B: 사용자 존재 확인 ✓
// 요청 A: 중복 확인 (없음) ✓
// 요청 B: 중복 확인 (없음) ✓
// 요청 A: INSERT 성공
// 요청 B: INSERT 시도 → DB 제약 위반 에러!

// ✅ 개선안 (원자적 처리)
INSERT INTO security_user_project (user_id, project_id)
SELECT $1, $2
WHERE EXISTS(SELECT 1 FROM security_user WHERE id = $1)
  AND EXISTS(SELECT 1 FROM security_project WHERE id = $2)
ON CONFLICT (user_id, project_id) DO NOTHING
RETURNING user_id;
```

---

## 🟡 중간 우선순위 (권장 개선)

### 2. 트랜잭션 경계 부족

| 메서드 | 파일 | 문제 | 개선효과 |
|--------|------|------|----------|
| `check_permission` | access_control_service.rs:209 | 2번의 별도 쿼리 | 성능 향상 + 일관성 |
| `log_dicom_access` | access_control_service.rs:111 | 검증 쿼리 여러 번 | DB 왕복 감소 |

### 개선안: 단일 쿼리로 통합

```rust
// ❌ 현재: 2번의 쿼리
// 1. 역할 기반 권한 확인
// 2. 프로젝트 직접 권한 확인

// ✅ 개선: UNION으로 1번의 쿼리
SELECT EXISTS(
    SELECT 1 FROM security_role_permission rp
    INNER JOIN security_project_role pr ON rp.role_id = pr.role_id
    INNER JOIN security_user_project up ON pr.project_id = up.project_id
    WHERE up.user_id = $1 AND up.project_id = $2
      AND rp.permission_id = (
          SELECT id FROM security_permission
          WHERE resource_type = $3 AND action = $4
      )

    UNION ALL

    SELECT 1 FROM security_project_permission pp
    INNER JOIN security_user_project up ON pp.project_id = up.project_id
    WHERE up.user_id = $1 AND pp.project_id = $2
      AND pp.permission_id = (
          SELECT id FROM security_permission
          WHERE resource_type = $3 AND action = $4
      )
) AS has_permission;
```

---

## 🟢 성능 최적화 가능

### 3. 배치 작업 지원

현재는 단건 처리만 가능하지만, 배치 작업으로 성능을 크게 개선할 수 있습니다.

```rust
// ✅ 배치 권한 할당 (UNNEST 사용)
pub async fn assign_permissions_to_role_batch(
    &self,
    role_id: i32,
    permission_ids: Vec<i32>
) -> Result<Vec<i32>, ServiceError> {
    let pool = self.role_repository.pool();
    let mut tx = pool.begin().await?;

    let assigned = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_role_permission (role_id, permission_id)
         SELECT $1, unnest($2::int[])
         WHERE EXISTS(SELECT 1 FROM security_permission WHERE id = ANY($2::int[]))
         ON CONFLICT (role_id, permission_id) DO NOTHING
         RETURNING permission_id"
    )
    .bind(role_id)
    .bind(&permission_ids)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(assigned)
}

// 성능 비교:
// - 10개 권한 단건 처리: 10개 트랜잭션 + 20번 DB 왕복
// - 10개 권한 배치 처리: 1개 트랜잭션 + 2번 DB 왕복
// → 약 10배 성능 향상
```

### 4. 복합 작업 원자성

여러 관련 작업을 하나의 트랜잭션으로 묶어 처리:

```rust
// ✅ 역할 생성 + 권한 할당을 한 트랜잭션으로
pub async fn create_role_with_permissions(
    &self,
    name: String,
    scope: RoleScope,
    description: Option<String>,
    permission_ids: Vec<i32>
) -> Result<(Role, Vec<i32>), ServiceError> {
    let mut tx = pool.begin().await?;

    // 1. 역할 생성
    let role = sqlx::query_as::<_, Role>(
        "INSERT INTO security_role (name, description, scope)
         VALUES ($1, $2, $3)
         RETURNING *"
    )
    .bind(&name)
    .bind(&description)
    .bind(scope.as_str())
    .fetch_one(&mut *tx)
    .await?;

    // 2. 권한 할당
    let assigned = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_role_permission (role_id, permission_id)
         SELECT $1, unnest($2::int[])
         RETURNING permission_id"
    )
    .bind(role.id)
    .bind(&permission_ids)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((role, assigned))
}
```

---

## 📊 트랜잭션 전략 가이드

### 트랜잭션이 필요한 경우

✅ **반드시 트랜잭션 사용**
- 여러 테이블에 걸친 작업
- All-or-Nothing이 필요한 작업
- Race Condition 위험이 있는 작업

❌ **트랜잭션 불필요**
- 단일 INSERT/UPDATE/DELETE
- 읽기 전용 작업
- 멱등성이 보장되는 작업

### 락 전략

```rust
// READ 락 (다른 읽기 허용, 쓰기 대기)
SELECT * FROM users WHERE id = $1 FOR SHARE;

// WRITE 락 (읽기/쓰기 모두 대기)
SELECT * FROM users WHERE id = $1 FOR UPDATE;

// NO WAIT (락 대기 안함, 즉시 에러)
SELECT * FROM users WHERE id = $1 FOR UPDATE NOWAIT;

// SKIP LOCKED (락 걸린 행 건너뛰기)
SELECT * FROM queue WHERE processed = false
FOR UPDATE SKIP LOCKED LIMIT 10;
```

---

## 🚀 적용 우선순위

### Phase 1: 즉시 적용 (Race Condition 제거)
1. ✅ `add_user_to_project` → INSERT ... ON CONFLICT
2. ✅ `assign_permission_to_role` → INSERT ... ON CONFLICT
3. ✅ `assign_permission_to_project` → INSERT ... ON CONFLICT
4. ✅ `assign_role_to_project` → INSERT ... ON CONFLICT
5. ✅ `login` → UPSERT

### Phase 2: 성능 최적화
1. ✅ 배치 작업 메서드 추가
   - `add_users_to_project_batch`
   - `assign_permissions_to_role_batch`
2. ✅ 복합 작업 원자화
   - `create_role_with_permissions`
   - `create_project_with_members`

### Phase 3: 쿼리 최적화
1. ✅ `check_permission` UNION 쿼리로 통합
2. ✅ `get_user_permissions` DISTINCT 최적화
3. ✅ 불필요한 SELECT 제거

---

## 💡 구현 예시

### 파일별 개선안

개선된 트랜잭션 처리 구현은 다음 파일에서 확인:

1. **user_service_with_tx.rs**
   - `add_user_to_project_tx()` - ON CONFLICT 패턴
   - `add_users_to_project_batch()` - 배치 처리

2. **permission_service_with_tx.rs**
   - `assign_permission_to_role_tx()` - 원자적 할당
   - `assign_permissions_to_role_batch()` - 배치 할당
   - `create_role_with_permissions()` - 복합 작업

3. **auth_service_with_tx.rs**
   - `login_upsert()` - UPSERT 패턴
   - `login_optimized()` - CTE 최적화

---

## 🧪 테스트 가이드

### 동시성 테스트

```rust
#[tokio::test]
async fn test_concurrent_add_user_to_project() {
    let pool = get_test_pool().await;
    let service = create_service(&pool);

    // 동시에 같은 작업 100번 실행
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let service = service.clone();
            tokio::spawn(async move {
                service.add_user_to_project(1, 1).await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // 1번만 성공, 99번은 AlreadyExists 에러
    let success_count = results.iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();
    assert_eq!(success_count, 1);
}
```

### 트랜잭션 롤백 테스트

```rust
#[tokio::test]
async fn test_transaction_rollback() {
    let pool = get_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    // 작업 수행
    let result = perform_work(&mut tx).await;

    if result.is_err() {
        tx.rollback().await.unwrap();
        // DB에 변경사항 없음을 확인
        assert_no_changes(&pool).await;
    }
}
```

---

## 📚 참고 자료

### PostgreSQL 트랜잭션 문서
- [Transaction Isolation](https://www.postgresql.org/docs/current/transaction-iso.html)
- [Row Locking](https://www.postgresql.org/docs/current/explicit-locking.html)
- [INSERT ON CONFLICT](https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT)

### SQLx 트랜잭션 가이드
- [SQLx Transactions](https://docs.rs/sqlx/latest/sqlx/trait.Executor.html#method.begin)
- [Connection Pooling](https://docs.rs/sqlx/latest/sqlx/pool/index.html)

### 성능 최적화
- [N+1 Query Problem](https://stackoverflow.com/questions/97197/what-is-the-n1-selects-problem)
- [Batch Operations](https://www.postgresql.org/docs/current/populate.html)

---

## ⚠️ 주의사항

1. **트랜잭션 타임아웃**
   - 긴 트랜잭션은 락 경합 유발
   - 5초 이상 트랜잭션은 경고

2. **데드락 방지**
   - 항상 같은 순서로 테이블 접근
   - 불필요한 락 최소화

3. **성능 모니터링**
   - 슬로우 쿼리 로그 활성화
   - 트랜잭션 시간 측정

4. **롤백 비용**
   - 큰 트랜잭션 롤백은 비용이 큼
   - 작은 단위로 커밋 권장
