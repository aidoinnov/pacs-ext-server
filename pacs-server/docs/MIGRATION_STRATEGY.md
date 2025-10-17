# 데이터베이스 마이그레이션 전략

## 개요

PACS Extension Server의 데이터베이스 마이그레이션 전략에 대한 상세한 기술 문서입니다. Kubernetes 환경에서 RDS를 사용하는 프로덕션 환경과 로컬 개발 환경을 모두 고려한 마이그레이션 전략을 제시합니다.

---

## 1. 마이그레이션 전략 개요

### 1.1 환경별 마이그레이션 방식

| 환경 | 마이그레이션 방식 | 제어 방법 | 실행 시점 |
|------|------------------|-----------|-----------|
| **개발** | 서버 사이드 자동 | `RUN_MIGRATIONS=true` | 애플리케이션 시작 시 |
| **프로덕션** | 수동 실행 | `RUN_MIGRATIONS=false` | 배포 전 별도 실행 |
| **테스트** | 서버 사이드 자동 | `RUN_MIGRATIONS=true` | CI/CD 파이프라인에서 |

### 1.2 마이그레이션 파일 구조
```
migrations/
├── 001_initial_schema.sql      # 초기 스키마 생성
├── 002_initial_seed_data.sql   # 시드 데이터 삽입
├── 003_add_mask_tables.sql     # 마스크 테이블 추가
└── README.md                   # 마이그레이션 가이드
```

---

## 2. 서버 사이드 자동 마이그레이션

### 2.1 구현 방식

#### 2.1.1 main.rs에서 마이그레이션 실행
```rust
// src/main.rs
use sqlx::migrate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 데이터베이스 연결 풀 생성
    let pool = create_connection_pool().await?;
    
    // 마이그레이션 실행 (환경 변수로 제어)
    let run_migrations = std::env::var("RUN_MIGRATIONS")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    if run_migrations {
        print!("🔄 Running database migrations... ");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run database migrations");
        println!("✅ Done");
    } else {
        println!("⏭️  Skipping migrations (set RUN_MIGRATIONS=true to enable)");
    }
    
    // 애플리케이션 시작
    start_server().await?;
    
    Ok(())
}
```

#### 2.1.2 환경별 설정
```bash
# env.development
RUN_MIGRATIONS=true

# env.production
RUN_MIGRATIONS=false
```

### 2.2 장점과 단점

#### 2.2.1 장점
- **간단한 구현**: 코드 변경 최소화
- **환경별 제어**: 환경 변수로 쉽게 제어
- **롤백 용이**: 애플리케이션 재시작으로 롤백
- **일관성**: 모든 환경에서 동일한 방식

#### 2.2.2 단점
- **프로덕션 위험**: 실패 시 애플리케이션 시작 불가
- **권한 문제**: 애플리케이션 계정에 DDL 권한 필요
- **롤백 복잡성**: 마이그레이션 실패 시 수동 개입 필요

---

## 3. 대안적 마이그레이션 방식

### 3.1 ArgoCD ConfigMap 방식

#### 3.1.1 ConfigMap 생성
```yaml
# k8s/migration-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: database-migrations
  namespace: pacs-extension
data:
  001_initial_schema.sql: |
    -- 초기 스키마 생성
    CREATE SCHEMA IF NOT EXISTS security;
    -- ... 스키마 정의
  002_initial_seed_data.sql: |
    -- 시드 데이터 삽입
    INSERT INTO security.roles (name, description) VALUES
    ('SUPER_ADMIN', 'Super Administrator');
    -- ... 시드 데이터
```

#### 3.1.2 Init Container 사용
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pacs-server
spec:
  template:
    spec:
      initContainers:
      - name: migration
        image: postgres:15
        command: ['sh', '-c']
        args:
        - |
          psql "$DATABASE_URL" -f /migrations/001_initial_schema.sql
          psql "$DATABASE_URL" -f /migrations/002_initial_seed_data.sql
        volumeMounts:
        - name: migration-files
          mountPath: /migrations
      containers:
      - name: pacs-server
        image: pacs-server:latest
        env:
        - name: RUN_MIGRATIONS
          value: "false"
      volumes:
      - name: migration-files
        configMap:
          name: database-migrations
```

### 3.2 Kubernetes Job 방식

#### 3.2.1 마이그레이션 Job 생성
```yaml
# k8s/migration-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: database-migration
  namespace: pacs-extension
spec:
  template:
    spec:
      containers:
      - name: migration
        image: postgres:15
        command: ['sh', '-c']
        args:
        - |
          for file in /migrations/*.sql; do
            echo "Executing: $file"
            psql "$DATABASE_URL" -f "$file" || exit 1
          done
        volumeMounts:
        - name: migration-files
          mountPath: /migrations
      volumes:
      - name: migration-files
        configMap:
          name: database-migrations
      restartPolicy: Never
```

#### 3.2.2 ArgoCD Workflow
```yaml
# argocd/migration-workflow.yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: database-migration
spec:
  entrypoint: migrate
  templates:
  - name: migrate
    steps:
    - - name: run-migration
        template: migration-job
  - name: migration-job
    resource:
      action: create
      manifest: |
        apiVersion: batch/v1
        kind: Job
        # ... Job 정의
```

---

## 4. 마이그레이션 실행 전략

### 4.1 개발 환경

#### 4.1.1 자동 마이그레이션 활성화
```bash
# 환경 변수 설정
export RUN_MIGRATIONS=true

# 애플리케이션 실행
cargo run
```

#### 4.1.2 Docker Compose 사용
```yaml
# docker-compose.yaml
version: '3.8'
services:
  pacs-server:
    build: .
    environment:
      - RUN_MIGRATIONS=true
      - DATABASE_URL=postgres://admin:admin123@postgres:5432/pacs_db
    depends_on:
      - postgres
```

### 4.2 프로덕션 환경

#### 4.2.1 수동 마이그레이션 실행
```bash
# 1. 마이그레이션 실행
sqlx migrate run --database-url "$DATABASE_URL"

# 2. 애플리케이션 배포
kubectl apply -f k8s/deployment.yaml
```

#### 4.2.2 CI/CD 파이프라인 통합
```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Run Database Migration
      run: |
        sqlx migrate run --database-url "${{ secrets.DATABASE_URL }}"
    
    - name: Deploy Application
      run: |
        kubectl apply -f k8s/deployment.yaml
```

---

## 5. 마이그레이션 파일 관리

### 5.1 파일 명명 규칙

#### 5.1.1 버전 번호 형식
```
001_initial_schema.sql
002_initial_seed_data.sql
003_add_mask_tables.sql
004_add_indexes.sql
005_update_constraints.sql
```

#### 5.1.2 설명적 이름 사용
```
001_create_security_schema.sql
002_create_viewer_schema.sql
003_create_annotation_schema.sql
004_add_user_management_tables.sql
005_add_study_management_tables.sql
```

### 5.2 마이그레이션 파일 구조

#### 5.2.1 스키마 마이그레이션
```sql
-- 001_initial_schema.sql
-- 목적: 초기 데이터베이스 스키마 생성
-- 의존성: 없음
-- 롤백: 001_initial_schema_rollback.sql

BEGIN;

-- 스키마 생성
CREATE SCHEMA IF NOT EXISTS security;
CREATE SCHEMA IF NOT EXISTS viewer;
CREATE SCHEMA IF NOT EXISTS annotation;

-- 테이블 생성
CREATE TABLE security.roles (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 인덱스 생성
CREATE INDEX idx_roles_name ON security.roles(name);

COMMIT;
```

#### 5.2.2 데이터 마이그레이션
```sql
-- 002_initial_seed_data.sql
-- 목적: 초기 시드 데이터 삽입
-- 의존성: 001_initial_schema.sql
-- 롤백: 002_initial_seed_data_rollback.sql

BEGIN;

-- 역할 데이터 삽입
INSERT INTO security.roles (name, description) VALUES
('SUPER_ADMIN', 'Super Administrator with full access'),
('PROJECT_ADMIN', 'Project Administrator with project-level access'),
('RESEARCHER', 'Researcher with study access'),
('VIEWER', 'Viewer with read-only access'),
('ANNOTATOR', 'Annotator with annotation access');

-- 권한 데이터 삽입
INSERT INTO security.permissions (name, description) VALUES
('USER', 'User management permission'),
('PROJECT', 'Project management permission'),
('STUDY', 'Study management permission'),
('SERIES', 'Series management permission'),
('INSTANCE', 'Instance management permission'),
('ANNOTATION', 'Annotation management permission'),
('MASK', 'Mask management permission'),
('HANGING_PROTOCOL', 'Hanging protocol management permission');

COMMIT;
```

---

## 6. 롤백 전략

### 6.1 롤백 파일 생성

#### 6.1.1 자동 롤백 생성
```bash
#!/bin/bash
# generate-rollback.sh

MIGRATION_FILE=$1
ROLLBACK_FILE="${MIGRATION_FILE%.sql}_rollback.sql"

echo "-- 롤백 파일: $ROLLBACK_FILE" > "$ROLLBACK_FILE"
echo "-- 원본 파일: $MIGRATION_FILE" >> "$ROLLBACK_FILE"
echo "-- 생성일: $(date)" >> "$ROLLBACK_FILE"
echo "" >> "$ROLLBACK_FILE"
echo "BEGIN;" >> "$ROLLBACK_FILE"

# DROP 문 생성
grep -i "CREATE TABLE" "$MIGRATION_FILE" | sed 's/CREATE TABLE/DROP TABLE IF EXISTS/' | sed 's/ (/ CASCADE;/' >> "$ROLLBACK_FILE"
grep -i "CREATE SCHEMA" "$MIGRATION_FILE" | sed 's/CREATE SCHEMA/DROP SCHEMA IF EXISTS/' | sed 's/ IF NOT EXISTS//' >> "$ROLLBACK_FILE"

echo "COMMIT;" >> "$ROLLBACK_FILE"

echo "롤백 파일 생성 완료: $ROLLBACK_FILE"
```

#### 6.1.2 수동 롤백 작성
```sql
-- 001_initial_schema_rollback.sql
-- 목적: 초기 스키마 롤백
-- 주의: 모든 데이터가 삭제됩니다!

BEGIN;

-- 테이블 삭제 (의존성 순서 고려)
DROP TABLE IF EXISTS security.role_permission CASCADE;
DROP TABLE IF EXISTS security.user_project CASCADE;
DROP TABLE IF EXISTS security.project_role CASCADE;
-- ... 다른 테이블들

-- 스키마 삭제
DROP SCHEMA IF EXISTS annotation CASCADE;
DROP SCHEMA IF EXISTS viewer CASCADE;
DROP SCHEMA IF EXISTS security CASCADE;

COMMIT;
```

### 6.2 롤백 실행

#### 6.2.1 자동 롤백
```bash
# 특정 마이그레이션 롤백
sqlx migrate revert --database-url "$DATABASE_URL"

# 모든 마이그레이션 롤백
sqlx migrate revert --database-url "$DATABASE_URL" --all
```

#### 6.2.2 수동 롤백
```bash
# 롤백 파일 실행
psql "$DATABASE_URL" -f 001_initial_schema_rollback.sql
```

---

## 7. 마이그레이션 검증

### 7.1 스키마 검증

#### 7.1.1 스키마 일치성 확인
```sql
-- 스키마 검증 쿼리
SELECT 
    schemaname,
    tablename,
    columnname,
    datatype,
    isnullable
FROM information_schema.columns 
WHERE schemaname IN ('security', 'viewer', 'annotation')
ORDER BY schemaname, tablename, ordinalposition;
```

#### 7.1.2 제약 조건 확인
```sql
-- 제약 조건 검증
SELECT 
    tc.table_schema,
    tc.table_name,
    tc.constraint_name,
    tc.constraint_type,
    kcu.column_name
FROM information_schema.table_constraints tc
JOIN information_schema.key_column_usage kcu 
    ON tc.constraint_name = kcu.constraint_name
WHERE tc.table_schema IN ('security', 'viewer', 'annotation')
ORDER BY tc.table_schema, tc.table_name, tc.constraint_name;
```

### 7.2 데이터 검증

#### 7.2.1 데이터 무결성 확인
```sql
-- 외래 키 제약 조건 검증
SELECT 
    tc.table_schema,
    tc.table_name,
    tc.constraint_name,
    kcu.column_name,
    ccu.table_schema AS foreign_table_schema,
    ccu.table_name AS foreign_table_name,
    ccu.column_name AS foreign_column_name
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name
JOIN information_schema.constraint_column_usage AS ccu
    ON ccu.constraint_name = tc.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema IN ('security', 'viewer', 'annotation');
```

#### 7.2.2 데이터 개수 확인
```sql
-- 테이블별 레코드 수 확인
SELECT 
    schemaname,
    tablename,
    n_tup_ins AS inserted_rows,
    n_tup_upd AS updated_rows,
    n_tup_del AS deleted_rows,
    n_live_tup AS live_rows
FROM pg_stat_user_tables 
WHERE schemaname IN ('security', 'viewer', 'annotation')
ORDER BY schemaname, tablename;
```

---

## 8. 모니터링 및 로깅

### 8.1 마이그레이션 로깅

#### 8.1.1 마이그레이션 실행 로그
```rust
// src/infrastructure/migration/logger.rs
use tracing::{info, warn, error};
use sqlx::migrate::MigrateDatabase;

pub struct MigrationLogger;

impl MigrationLogger {
    pub async fn log_migration_start(version: &str, description: &str) {
        info!(
            version = version,
            description = description,
            "Starting database migration"
        );
    }
    
    pub async fn log_migration_success(version: &str, duration: std::time::Duration) {
        info!(
            version = version,
            duration_ms = duration.as_millis(),
            "Database migration completed successfully"
        );
    }
    
    pub async fn log_migration_failure(version: &str, error: &str) {
        error!(
            version = version,
            error = error,
            "Database migration failed"
        );
    }
}
```

#### 8.1.2 마이그레이션 히스토리 테이블
```sql
-- 마이그레이션 실행 히스토리 테이블
CREATE TABLE IF NOT EXISTS migration_history (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER,
    success BOOLEAN NOT NULL,
    error_message TEXT
);

CREATE INDEX idx_migration_history_version ON migration_history(version);
CREATE INDEX idx_migration_history_executed_at ON migration_history(executed_at);
```

### 8.2 알림 및 모니터링

#### 8.2.1 실패 알림
```rust
// src/infrastructure/migration/notifier.rs
use serde_json::json;

pub struct MigrationNotifier;

impl MigrationNotifier {
    pub async fn notify_failure(version: &str, error: &str) {
        let payload = json!({
            "text": format!("Database migration failed: {} - {}", version, error),
            "channel": "#alerts",
            "username": "migration-bot"
        });
        
        // Slack 알림 전송
        if let Ok(webhook_url) = std::env::var("SLACK_WEBHOOK_URL") {
            reqwest::Client::new()
                .post(&webhook_url)
                .json(&payload)
                .send()
                .await
                .ok();
        }
    }
}
```

#### 8.2.2 메트릭 수집
```rust
// src/infrastructure/migration/metrics.rs
use prometheus::{Counter, Histogram, Registry};

pub struct MigrationMetrics {
    pub migrations_total: Counter,
    pub migrations_duration: Histogram,
    pub migrations_failed: Counter,
}

impl MigrationMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            migrations_total: Counter::new(
                "database_migrations_total",
                "Total number of database migrations executed"
            ).unwrap(),
            migrations_duration: Histogram::new(
                "database_migrations_duration_seconds",
                "Duration of database migrations in seconds"
            ).unwrap(),
            migrations_failed: Counter::new(
                "database_migrations_failed_total",
                "Total number of failed database migrations"
            ).unwrap(),
        }
    }
}
```

---

## 9. CI/CD 통합

### 9.1 GitHub Actions 워크플로우

#### 9.1.1 마이그레이션 테스트
```yaml
# .github/workflows/migration-test.yml
name: Migration Test

on:
  pull_request:
    paths:
      - 'migrations/**'
      - 'src/**'

jobs:
  test-migration:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test_db
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: 1.90.0
        components: rustfmt, clippy
    
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features --features postgres
    
    - name: Run migrations
      run: |
        export DATABASE_URL="postgres://postgres:postgres@localhost:5432/test_db"
        sqlx migrate run
    
    - name: Verify schema
      run: |
        export DATABASE_URL="postgres://postgres:postgres@localhost:5432/test_db"
        psql "$DATABASE_URL" -c "\dt"
        psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM security.roles;"
```

#### 9.1.2 프로덕션 배포
```yaml
# .github/workflows/deploy-production.yml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: production
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup kubectl
      uses: azure/setup-kubectl@v3
      with:
        version: 'latest'
    
    - name: Configure kubectl
      run: |
        echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > kubeconfig
        export KUBECONFIG=kubeconfig
    
    - name: Run database migration
      run: |
        export DATABASE_URL="${{ secrets.DATABASE_URL }}"
        sqlx migrate run
      env:
        DATABASE_URL: ${{ secrets.DATABASE_URL }}
    
    - name: Deploy application
      run: |
        kubectl apply -f k8s/deployment.yaml
        kubectl rollout status deployment/pacs-server
```

### 9.2 ArgoCD 통합

#### 9.2.1 Application 정의
```yaml
# argocd/pacs-server-app.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: pacs-server
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/company/pacs-extension-server
    targetRevision: HEAD
    path: k8s
  destination:
    server: https://kubernetes.default.svc
    namespace: pacs-extension
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
    - CreateNamespace=true
```

#### 9.2.2 마이그레이션 전용 Application
```yaml
# argocd/migration-app.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: database-migration
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/company/pacs-extension-server
    targetRevision: HEAD
    path: k8s/migration
  destination:
    server: https://kubernetes.default.svc
    namespace: pacs-extension
  syncPolicy:
    automated: false  # 수동 실행
```

---

## 10. 모범 사례 및 권장사항

### 10.1 마이그레이션 작성 가이드라인

#### 10.1.1 DO (해야 할 것)
- **원자성**: 각 마이그레이션은 원자적으로 실행되어야 함
- **멱등성**: 같은 마이그레이션을 여러 번 실행해도 안전해야 함
- **롤백 가능**: 각 마이그레이션은 롤백 가능해야 함
- **테스트**: 마이그레이션 전후로 테스트 실행

#### 10.1.2 DON'T (하지 말아야 할 것)
- **데이터 손실**: 프로덕션 데이터를 삭제하지 말 것
- **긴 실행 시간**: 마이그레이션은 빠르게 실행되어야 함
- **의존성 무시**: 마이그레이션 간 의존성을 고려하지 않으면 안 됨
- **롤백 불가능**: 롤백할 수 없는 마이그레이션은 작성하지 말 것

### 10.2 성능 최적화

#### 10.2.1 대용량 테이블 마이그레이션
```sql
-- 배치 처리로 대용량 데이터 마이그레이션
DO $$
DECLARE
    batch_size INTEGER := 1000;
    offset_val INTEGER := 0;
    total_rows INTEGER;
BEGIN
    -- 전체 행 수 확인
    SELECT COUNT(*) INTO total_rows FROM large_table;
    
    -- 배치별 처리
    WHILE offset_val < total_rows LOOP
        INSERT INTO new_table (col1, col2, col3)
        SELECT col1, col2, col3
        FROM large_table
        ORDER BY id
        LIMIT batch_size OFFSET offset_val;
        
        offset_val := offset_val + batch_size;
        
        -- 진행 상황 로깅
        RAISE NOTICE 'Processed % of % rows', offset_val, total_rows;
    END LOOP;
END $$;
```

#### 10.2.2 인덱스 최적화
```sql
-- 인덱스 생성 최적화
CREATE INDEX CONCURRENTLY idx_large_table_column 
ON large_table(column_name);

-- 통계 업데이트
ANALYZE large_table;
```

### 10.3 보안 고려사항

#### 10.3.1 권한 관리
```sql
-- 마이그레이션 전용 사용자 생성
CREATE USER migration_user WITH PASSWORD 'secure_password';

-- 필요한 권한만 부여
GRANT CONNECT ON DATABASE pacs_db TO migration_user;
GRANT USAGE ON SCHEMA security TO migration_user;
GRANT CREATE ON SCHEMA security TO migration_user;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA security TO migration_user;
```

#### 10.3.2 감사 로깅
```sql
-- 마이그레이션 감사 테이블
CREATE TABLE migration_audit (
    id SERIAL PRIMARY KEY,
    migration_version VARCHAR(50) NOT NULL,
    executed_by VARCHAR(100) NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    rollback_executed BOOLEAN DEFAULT FALSE
);
```

---

## 11. 결론

### 11.1 선택된 전략의 장점
1. **단순성**: 구현이 간단하고 이해하기 쉬움
2. **유연성**: 환경별로 다른 전략 적용 가능
3. **안전성**: 환경 변수로 제어하여 실수 방지
4. **유지보수성**: 코드 변경 최소화

### 11.2 향후 개선 방향
1. **자동화**: 더 많은 마이그레이션 작업 자동화
2. **모니터링**: 실시간 마이그레이션 상태 모니터링
3. **롤백**: 더 안전하고 빠른 롤백 메커니즘
4. **테스트**: 마이그레이션 전용 테스트 프레임워크

### 11.3 권장사항
1. **정기 검토**: 분기별 마이그레이션 전략 검토
2. **문서화**: 모든 마이그레이션 변경사항 문서화
3. **팀 교육**: 마이그레이션 모범 사례 팀 공유
4. **자동화**: 가능한 모든 마이그레이션 작업 자동화

이 문서를 통해 안전하고 효율적인 데이터베이스 마이그레이션 환경을 구축하고 유지할 수 있습니다.
