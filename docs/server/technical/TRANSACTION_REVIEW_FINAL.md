# DB 트랜잭션 처리 최종 검토 보고서

## 🔍 2차 검토 결과

1차 검토에서 발견한 5가지 주요 Race Condition 외에 추가로 발견된 문제점과 권장사항을 정리합니다.

---

## 📊 전체 문제 요약

### 🔴 심각 (즉시 수정 필요) - 5건
1. `add_user_to_project` - Race Condition
2. `assign_permission_to_role` - Race Condition
3. `assign_permission_to_project` - Race Condition
4. `assign_role_to_project` - Race Condition
5. `login` - 동시 로그인 중복 생성

### 🟠 중요 (조속히 개선) - 3건
6. `activate_project` / `deactivate_project` - UPDATE 후 SELECT 비원자성
7. `delete_user` / `delete_project` - CASCADE 의존성 검증 부족
8. `check_permission` - 2번의 별도 쿼리로 일관성 문제 가능

### 🟡 권장 (성능 개선) - 4건
9. 배치 작업 미지원
10. Repository 레이어 트랜잭션 미지원
11. 중복 쿼리 최적화 가능
12. 접근 로그 검증 쿼리 최적화

---

## 🆕 신규 발견 문제

### 6. activate_project / deactivate_project (project_service.rs:131-147)

**문제점**:
```rust
async fn activate_project(&self, id: i32) -> Result<Project, ServiceError> {
    let updated = self.project_repository.set_active(id, true).await?;
    if updated {
        self.get_project(id).await  // ⚠️ 별도 SELECT - 비원자적
    } else {
        Err(ServiceError::NotFound("Project not found".into()))
    }
}
```

**위험**:
- UPDATE와 SELECT 사이에 다른 트랜잭션이 데이터 변경 가능
- 프로젝트가 UPDATE 직후 삭제되면 NotFound 에러 발생
- 반환된 프로젝트 상태가 UPDATE 시점과 다를 수 있음

**해결책**:
```rust
// ✅ RETURNING 절로 원자적 처리
async fn activate_project(&self, id: i32) -> Result<Project, ServiceError> {
    let project = sqlx::query_as::<_, Project>(
        "UPDATE security_project
         SET is_active = true
         WHERE id = $1
         RETURNING id, name, description, is_active, created_at"
    )
    .bind(id)
    .fetch_optional(self.project_repository.pool())
    .await?
    .ok_or(ServiceError::NotFound("Project not found".into()))?;

    Ok(project)
}
```

---

### 7. delete_user / delete_project - CASCADE 검증 부족

**현재 구조**:
- DB 스키마에 `ON DELETE CASCADE` 설정됨
- 서비스 레이어에서 CASCADE 영향 확인 없음
- 사용자/프로젝트 삭제 시 연관 데이터 자동 삭제

**잠재적 문제**:
```sql
-- 사용자 삭제 시 CASCADE로 함께 삭제되는 데이터:
DELETE FROM security_user WHERE id = 1;
-- → security_user_project (프로젝트 멤버십)
-- → security_access_log (접근 로그)
-- → security_user_group (그룹 멤버십)

-- 의도하지 않은 데이터 손실 가능!
```

**개선 권장사항**:

1. **소프트 삭제 (Soft Delete) 패턴**
```rust
// ✅ 삭제 대신 비활성화
pub async fn soft_delete_user(&self, id: i32) -> Result<User, ServiceError> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE security_user
         SET deleted_at = NOW(), is_active = false
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING *"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ServiceError::NotFound("User not found".into()))?;

    Ok(user)
}
```

2. **삭제 전 검증 + 트랜잭션**
```rust
// ✅ 연관 데이터 확인 후 삭제
pub async fn delete_user_safe(&self, id: i32) -> Result<DeleteResult, ServiceError> {
    let mut tx = pool.begin().await?;

    // 1. 연관 데이터 수 확인
    let (project_count, access_log_count): (i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM security_user_project WHERE user_id = $1),
            (SELECT COUNT(*) FROM security_access_log WHERE user_id = $1)"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    // 2. 연관 데이터가 있으면 경고 반환 (선택적으로 강제 삭제 허용)
    if project_count > 0 || access_log_count > 0 {
        tx.rollback().await?;
        return Ok(DeleteResult::HasDependencies {
            user_id: id,
            project_memberships: project_count,
            access_logs: access_log_count,
        });
    }

    // 3. 안전하게 삭제
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(DeleteResult::Deleted)
}

pub enum DeleteResult {
    Deleted,
    HasDependencies {
        user_id: i32,
        project_memberships: i64,
        access_logs: i64,
    },
}
```

---

### 8. check_permission - 일관성 문제 (access_control_service.rs:209-272)

**문제점**:
```rust
// 1번째 쿼리: 역할 기반 권한 확인
let has_permission = sqlx::query_scalar::<_, i64>(...)
    .fetch_one(pool).await?;

if has_permission > 0 {
    return Ok(true);
}

// 2번째 쿼리: 프로젝트 직접 권한 확인
let project_permission = sqlx::query_scalar::<_, i64>(...)
    .fetch_one(pool).await?;
```

**위험**:
- 1번 쿼리와 2번 쿼리 사이에 권한이 변경될 수 있음
- 트랜잭션 격리 없이 2번의 별도 읽기 → Phantom Read 가능

**해결책**:
```rust
// ✅ 단일 쿼리로 통합 + EXISTS 최적화
async fn check_permission(
    &self,
    user_id: i32,
    project_id: i32,
    resource_type: &str,
    action: &str,
) -> Result<bool, ServiceError> {
    let has_permission = sqlx::query_scalar::<_, bool>(
        "WITH permission_id AS (
            SELECT id FROM security_permission
            WHERE resource_type = $3 AND action = $4
            LIMIT 1
        )
        SELECT EXISTS(
            -- 역할 기반 권한
            SELECT 1
            FROM security_role_permission rp
            INNER JOIN security_project_role pr ON rp.role_id = pr.role_id
            INNER JOIN security_user_project up ON pr.project_id = up.project_id
            WHERE up.user_id = $1
              AND up.project_id = $2
              AND rp.permission_id = (SELECT id FROM permission_id)

            UNION ALL

            -- 프로젝트 직접 권한
            SELECT 1
            FROM security_project_permission pp
            INNER JOIN security_user_project up ON pp.project_id = up.project_id
            WHERE up.user_id = $1
              AND pp.project_id = $2
              AND pp.permission_id = (SELECT id FROM permission_id)

            LIMIT 1  -- 하나만 찾으면 충분
        )"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(resource_type)
    .bind(action)
    .fetch_one(pool)
    .await?;

    Ok(has_permission)
}

// 성능 향상: 2번 DB 왕복 → 1번 DB 왕복
// 일관성 보장: 단일 쿼리 내에서 스냅샷 격리
```

---

## 🔧 Repository 레이어 개선

### 현재 상태
- Repository는 단순 CRUD만 제공
- 트랜잭션 지원 없음
- 복잡한 쿼리는 Service에서 직접 작성

### 권장 개선

**1. Repository에 트랜잭션 메서드 추가**
```rust
// ✅ Repository에 트랜잭션 헬퍼 제공
#[async_trait]
pub trait UserRepository: Send + Sync {
    // 기존 메서드들...

    // 트랜잭션 실행 헬퍼
    async fn with_transaction<F, T>(&self, f: F) -> Result<T, sqlx::Error>
    where
        F: FnOnce(&mut Transaction<Postgres>) -> BoxFuture<'_, Result<T, sqlx::Error>> + Send,
        T: Send;
}

impl UserRepository for UserRepositoryImpl {
    async fn with_transaction<F, T>(&self, f: F) -> Result<T, sqlx::Error>
    where
        F: FnOnce(&mut Transaction<Postgres>) -> BoxFuture<'_, Result<T, sqlx::Error>> + Send,
        T: Send,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}

// 사용 예시
async fn complex_operation(&self) -> Result<(), ServiceError> {
    self.user_repository.with_transaction(|tx| {
        Box::pin(async move {
            // 트랜잭션 내 작업
            sqlx::query("...").execute(&mut *tx).await?;
            sqlx::query("...").execute(&mut *tx).await?;
            Ok(())
        })
    }).await?;

    Ok(())
}
```

**2. 복잡한 쿼리를 Repository로 이동**
```rust
// ❌ 현재: Service에 SQL 노출
async fn get_user_permissions(&self, user_id: i32, project_id: i32) -> Result<Vec<Permission>, ServiceError> {
    let permissions = sqlx::query_as::<_, Permission>(
        "SELECT DISTINCT p.id, p.resource_type, p.action ..."
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_all(self.user_repository.pool())
    .await?;
}

// ✅ 개선: Repository에 캡슐화
#[async_trait]
pub trait PermissionRepository {
    async fn find_user_permissions_in_project(
        &self,
        user_id: i32,
        project_id: i32
    ) -> Result<Vec<Permission>, sqlx::Error>;
}

// Service는 깔끔하게
async fn get_user_permissions(&self, user_id: i32, project_id: i32) -> Result<Vec<Permission>, ServiceError> {
    Ok(self.permission_repository
        .find_user_permissions_in_project(user_id, project_id)
        .await?)
}
```

---

## 🚀 성능 최적화 추가 권장사항

### 1. 인덱스 최적화 확인

```sql
-- 복합 인덱스 추가 검토
CREATE INDEX CONCURRENTLY idx_user_project_permissions ON security_role_permission(role_id, permission_id);
CREATE INDEX CONCURRENTLY idx_project_direct_permissions ON security_project_permission(project_id, permission_id);

-- 부분 인덱스 (활성 프로젝트만)
CREATE INDEX CONCURRENTLY idx_active_projects ON security_project(id) WHERE is_active = true;

-- 커버링 인덱스 (INDEX ONLY SCAN 유도)
CREATE INDEX CONCURRENTLY idx_user_project_covering
ON security_user_project(user_id, project_id)
INCLUDE (created_at);
```

### 2. 쿼리 플랜 분석

```rust
// 성능 측정 매크로 추가
#[cfg(debug_assertions)]
macro_rules! explain_analyze {
    ($query:expr) => {{
        let plan = sqlx::query(&format!("EXPLAIN ANALYZE {}", $query))
            .fetch_all(pool)
            .await?;
        tracing::debug!("Query plan: {:?}", plan);
        sqlx::query($query)
    }};
}
```

### 3. 연결 풀 최적화

```rust
// sqlx::PgPoolOptions 설정 검토
let pool = PgPoolOptions::new()
    .max_connections(20)           // 최대 연결 수
    .min_connections(5)             // 최소 유지 연결
    .acquire_timeout(Duration::from_secs(3))  // 획득 타임아웃
    .idle_timeout(Duration::from_secs(300))   // 유휴 연결 유지 시간
    .max_lifetime(Duration::from_secs(1800))  // 연결 최대 수명
    .connect(&database_url)
    .await?;
```

---

## 📋 우선순위별 적용 계획

### Phase 1: 즉시 적용 (1-2일)
- [x] Race Condition 5건 수정
  - INSERT ... ON CONFLICT 패턴 적용
  - UPSERT로 login 개선

### Phase 2: 중요 개선 (3-5일)
- [ ] activate/deactivate - RETURNING 절 사용
- [ ] 삭제 작업 - 소프트 삭제 또는 검증 추가
- [ ] check_permission - 단일 쿼리 통합

### Phase 3: Repository 리팩토링 (1주)
- [ ] 트랜잭션 헬퍼 메서드 추가
- [ ] 복잡한 쿼리 Repository로 이동
- [ ] 배치 작업 메서드 추가

### Phase 4: 성능 최적화 (1-2주)
- [ ] 인덱스 최적화
- [ ] 쿼리 플랜 분석 및 튜닝
- [ ] 연결 풀 설정 최적화
- [ ] 부하 테스트 및 모니터링

---

## 🧪 테스트 전략

### 1. 동시성 테스트
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_user_project_assignment() {
    let pool = get_test_pool().await;
    let service = create_service(&pool);

    // 100개 동시 요청
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let svc = service.clone();
            tokio::spawn(async move {
                svc.add_user_to_project(1, 1).await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // 정확히 1개만 성공, 99개는 AlreadyExists
    let success = results.iter().filter(|r| matches!(
        r.as_ref().unwrap().as_ref(),
        Ok(_)
    )).count();

    assert_eq!(success, 1, "Only one insert should succeed");
}
```

### 2. 트랜잭션 격리 레벨 테스트
```rust
#[tokio::test]
async fn test_repeatable_read_isolation() {
    let pool = get_test_pool().await;

    // SET TRANSACTION ISOLATION LEVEL REPEATABLE READ
    let mut tx1 = pool.begin().await.unwrap();
    let mut tx2 = pool.begin().await.unwrap();

    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx1).await.unwrap();

    // tx1이 읽고, tx2가 수정, tx1이 다시 읽기
    // → Phantom Read 발생하지 않아야 함
}
```

### 3. CASCADE 영향 테스트
```rust
#[tokio::test]
async fn test_cascade_delete_impact() {
    let pool = get_test_pool().await;

    // 사용자와 연관 데이터 생성
    let user = create_test_user(&pool).await;
    let project = create_test_project(&pool).await;
    add_user_to_project(&pool, user.id, project.id).await;
    log_access(&pool, user.id, project.id).await;

    // 사용자 삭제
    delete_user(&pool, user.id).await;

    // CASCADE로 삭제된 데이터 확인
    let membership_count = count_memberships(&pool, user.id).await;
    let log_count = count_logs(&pool, user.id).await;

    assert_eq!(membership_count, 0, "Memberships should be cascaded");
    assert_eq!(log_count, 0, "Logs should be cascaded");
}
```

---

## 📊 성능 벤치마크 목표

| 작업 | 현재 | 목표 | 개선률 |
|------|------|------|--------|
| add_user_to_project | 3 쿼리 | 1 쿼리 | 66% |
| assign_permissions (10개) | 30 쿼리 | 2 쿼리 | 93% |
| check_permission | 2-3 쿼리 | 1 쿼리 | 66% |
| login (중복) | 2 쿼리 + 경합 | 1 쿼리 | 50% + 안정성 |
| activate_project | 2 쿼리 | 1 쿼리 | 50% |

---

## ⚠️ 주의사항 및 마이그레이션 가이드

### 1. 기존 코드와의 호환성
- 새로운 `*_tx` 메서드는 기존 메서드와 병행 운영
- 점진적 마이그레이션: 하나씩 교체하며 테스트
- Feature flag로 새 구현 제어 가능

### 2. 배포 전 체크리스트
- [ ] 모든 테스트 통과 확인
- [ ] 동시성 테스트 10회 이상 반복 성공
- [ ] 프로덕션 트래픽 1/10 규모 부하 테스트
- [ ] 롤백 계획 수립
- [ ] 모니터링 대시보드 준비

### 3. 모니터링 지표
```rust
// 트랜잭션 메트릭 수집
#[tracing::instrument(skip(self))]
async fn add_user_to_project_tx(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
    let start = Instant::now();

    let result = /* 트랜잭션 실행 */;

    metrics::histogram!("db.transaction.duration", start.elapsed());
    metrics::counter!("db.transaction.total").increment(1);

    if result.is_err() {
        metrics::counter!("db.transaction.errors").increment(1);
    }

    result
}
```

---

## 📚 추가 학습 자료

1. **PostgreSQL 트랜잭션 격리 레벨**
   - [공식 문서](https://www.postgresql.org/docs/current/transaction-iso.html)
   - READ COMMITTED vs REPEATABLE READ vs SERIALIZABLE

2. **SQLx 트랜잭션 Best Practices**
   - [Transaction 가이드](https://docs.rs/sqlx/latest/sqlx/trait.Executor.html)
   - Connection Pooling 전략

3. **동시성 제어 패턴**
   - Optimistic Locking (Version 필드)
   - Pessimistic Locking (SELECT FOR UPDATE)
   - MVCC (Multi-Version Concurrency Control)

4. **성능 튜닝**
   - [Use The Index, Luke](https://use-the-index-luke.com/)
   - PostgreSQL EXPLAIN 분석 방법

---

## ✅ 결론

### 발견된 문제 총 12건
- 🔴 심각: 5건 (Race Condition, 중복 생성)
- 🟠 중요: 3건 (비원자성 UPDATE, CASCADE 검증, 일관성)
- 🟡 권장: 4건 (배치 미지원, Repository 개선, 최적화)

### 핵심 개선 방향
1. **INSERT ... ON CONFLICT** 패턴으로 Race Condition 제거
2. **RETURNING** 절로 UPDATE-SELECT 원자화
3. **단일 쿼리 통합**으로 성능 및 일관성 향상
4. **소프트 삭제** 또는 **검증 후 삭제**로 데이터 안정성 확보
5. **Repository 트랜잭션 지원**으로 아키텍처 개선

모든 개선안은 `*_with_tx.rs` 파일과 이 문서에 상세히 기술되어 있습니다.
