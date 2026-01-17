// CLI 파싱을 위한 clap 라이브러리
// - Parser: 전체 CLI 구조 정의
// - Subcommand: 하위 명령어(job) 정의
use clap::{Parser, Subcommand};

// GC 관련 Application Service
// - GcService: GC 서비스 인터페이스
// - GcServiceImpl: 실제 구현체
// - ObjectStorageService: 오브젝트 스토리지 trait
// - ObjectStorageServiceFactory: S3 등 오브젝트 스토리지 생성 팩토리
use pacs_server::application::services::{GcService, GcServiceImpl, ObjectStorageService, ObjectStorageServiceFactory};
// GC 관련 Repository 구현체
// - GcRepositoryImpl: 어노테이션 조회/상태 변경
// - GcLogRepositoryImpl: GC 실행 로그 저장
use pacs_server::infrastructure::repositories::{GcRepositoryImpl, GcLogRepositoryImpl};
// PostgreSQL 커넥션 풀 설정
use sqlx::postgres::PgPoolOptions;
// Arc: 여러 스레드/서비스에서 안전하게 공유하기 위한 참조 카운트 포인터
use std::sync::Arc;

/// ==============================
/// CLI 최상위 구조
/// ==============================
///
/// gc_runner 라는 이름의 CLI 프로그램 정의
/// 예:
///   gc_runner timeout-pending --grace-days 3
///   gc_runner cleanup-failed --dry-run
///
#[derive(Parser)]
#[command(name = "gc_runner")]
#[command(about = "PACS GC Batch Job Runner", long_about = None)]
struct Cli {
    /// 실행할 하위 명령어 (Job A / Job B)
    #[command(subcommand)]
    command: Commands,
}

/// ==============================
/// 하위 명령어 정의
/// ==============================
///
/// GC 배치 작업은 크게 두 가지:
/// - Job A: PENDING 상태가 너무 오래 지속된 항목 처리
/// - Job B: FAILED 상태의 스냅샷 정리
///
#[derive(Subcommand)]
enum Commands {
    /// Job A: PENDING 상태 타임아웃 처리
    /// - 업로드/처리가 완료되지 않은 채 오래 방치된 항목을 정리
    TimeoutPending {
        /// Grace period (days)
        /// - PENDING 상태로 이 기간 이상 유지된 것만 대상
        #[arg(long, default_value = "3")]
        grace_days: i32,

        /// Batch size
        /// - 한 번에 처리할 최대 개수 (대량 삭제 방지)
        #[arg(long, default_value = "1000")]
        batch_size: i32,

        /// Dry-run mode
        /// - true: 실제 삭제/변경 없이 로그만 기록
        /// - false: 실제 처리 수행
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Job B: FAILED 상태 스냅샷 정리
    /// - 이미 실패한 어노테이션의 S3 스냅샷 파일 삭제
    CleanupFailed {
        /// Grace period (days)
        /// - FAILED 이후 일정 기간이 지난 것만 삭제 대상
        #[arg(long, default_value = "7")]
        grace_days: i32,

        /// Batch size
        #[arg(long, default_value = "1000")]
        batch_size: i32,

        /// Dry-run mode
        /// - true: 삭제 대상 확인용
        /// - false: 실제 S3 삭제 수행
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },
}

/// ==============================
/// Advisory Lock 헬퍼
/// ==============================
/// PostgreSQL Advisory Lock을 사용하여 동시 실행 방지
/// - lock_id: 각 job마다 고유한 ID 사용
///   - Job A (timeout-pending): 1001
///   - Job B (cleanup-failed): 1002
/// - 반환값: true = 락 획득 성공, false = 이미 실행 중
async fn try_acquire_lock(pool: &sqlx::PgPool, lock_id: i64) -> Result<bool, sqlx::Error> {
    let result: (bool,) = sqlx::query_as(
        "SELECT pg_try_advisory_lock($1)"
    )
    .bind(lock_id)
    .fetch_one(pool)
    .await?;

    Ok(result.0)
}

/// Advisory Lock 해제
async fn release_lock(pool: &sqlx::PgPool, lock_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(lock_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// ==============================
/// 프로그램 진입점
/// ==============================
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------
    // 1️⃣ 로깅 초기화
    // ------------------------------
    // env_logger 사용
    // - RUST_LOG 환경변수로 로그 레벨 제어 가능
    env_logger::init();

    // ------------------------------
    // 2️⃣ CLI 인자 파싱
    // ------------------------------
    let cli = Cli::parse();

    // ------------------------------
    // 3️⃣ 환경 변수 로드
    // ------------------------------
    // - 배치 실행 환경에서 반드시 주입되어야 함
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let s3_bucket = std::env::var("S3_BUCKET")
        .expect("S3_BUCKET must be set");
    let s3_region = std::env::var("S3_REGION")
        .unwrap_or_else(|_| "us-east-1".to_string());
    let s3_access_key = std::env::var("S3_ACCESS_KEY")
        .expect("S3_ACCESS_KEY must be set");
    let s3_secret_key = std::env::var("S3_SECRET_KEY")
        .expect("S3_SECRET_KEY must be set");

    // ------------------------------
    // 4️⃣ DB 커넥션 풀 생성
    // ------------------------------
    // - sqlx PgPool 사용
    // - max_connections: 동시에 사용할 최대 DB 커넥션 수
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // 여러 Repository에서 공유하므로 Arc로 감쌈        
    let pool = Arc::new(pool);

    // ------------------------------
    // 5️⃣ Object Storage(S3) 서비스 생성
    // ------------------------------
    // Factory 패턴 사용
    // - 추후 S3 외 다른 스토리지(GCS, MinIO 등) 확장 가능
    let object_storage = ObjectStorageServiceFactory::create(
        "s3",
        &s3_bucket,
        &s3_region,
        "",  // endpoint (S3는 불필요)
        &s3_access_key,
        &s3_secret_key,
    )
    .await?;
    // Box<dyn ObjectStorageService>를 Arc<dyn ObjectStorageService>로 변환
    let object_storage: Arc<dyn ObjectStorageService> = Arc::from(object_storage);

    // ------------------------------
    // 6️⃣ Repository 생성
    // ------------------------------
    // - DB 접근 계층
    // - Arc로 감싸 여러 서비스에서 공유
    let gc_repository = Arc::new(GcRepositoryImpl::new(pool.clone()));
    let gc_log_repository = Arc::new(GcLogRepositoryImpl::new(pool.clone()));

    // ------------------------------
    // 7️⃣ GC Service 생성
    // ------------------------------
    // - 실제 비즈니스 로직 담당
    let gc_service = GcServiceImpl::new(
        gc_repository,
        gc_log_repository,
        object_storage,
    );

    // ------------------------------
    // 8️⃣ CLI 명령어에 따른 작업 실행
    // ------------------------------
    match cli.command {
        // --------------------------
        // Job A 실행
        // --------------------------
        Commands::TimeoutPending { grace_days, batch_size, dry_run } => {
            const LOCK_ID: i64 = 1001; // Job A 전용 락 ID

            println!("🔄 Running Job A: Timeout Pending Snapshots");
            println!("   Grace Days: {}", grace_days);
            println!("   Batch Size: {}", batch_size);
            println!("   Dry-run: {}", dry_run);

            // Advisory Lock 획득 시도
            if !try_acquire_lock(&pool, LOCK_ID).await? {
                eprintln!("⚠️  Another instance of Job A is already running. Exiting.");
                std::process::exit(1);
            }

            println!("🔒 Lock acquired");

            // GC 서비스 호출
            let results = gc_service
                .timeout_pending_snapshots(grace_days, batch_size, dry_run)
                .await;

            // 락 해제 (결과와 무관하게 항상 해제)
            release_lock(&pool, LOCK_ID).await?;
            println!("🔓 Lock released");

            let results = results?;

            // 성공/실패 건수 집계
            let success_count = results.iter().filter(|r| r.success).count();
            let failed_count = results.len() - success_count;

            // 실행 결과 출력
            println!("✅ Job A completed:");
            println!("   Total: {}", results.len());
            println!("   Success: {}", success_count);
            println!("   Failed: {}", failed_count);
        }

        // --------------------------
        // Job B 실행
        // --------------------------
        Commands::CleanupFailed { grace_days, batch_size, dry_run } => {
            const LOCK_ID: i64 = 1002; // Job B 전용 락 ID

            println!("🗑️  Running Job B: Cleanup Failed Snapshots");
            println!("   Grace Days: {}", grace_days);
            println!("   Batch Size: {}", batch_size);
            println!("   Dry-run: {}", dry_run);

            // Advisory Lock 획득 시도
            if !try_acquire_lock(&pool, LOCK_ID).await? {
                eprintln!("⚠️  Another instance of Job B is already running. Exiting.");
                std::process::exit(1);
            }

            println!("🔒 Lock acquired");

            let results = gc_service
                .cleanup_failed_snapshots(grace_days, batch_size, dry_run)
                .await;

            // 락 해제 (결과와 무관하게 항상 해제)
            release_lock(&pool, LOCK_ID).await?;
            println!("🔓 Lock released");

            let results = results?;

            let success_count = results.iter().filter(|r| r.success).count();
            let failed_count = results.len() - success_count;

            println!("✅ Job B completed:");
            println!("   Total: {}", results.len());
            println!("   Success: {}", success_count);
            println!("   Failed: {}", failed_count);
        }
    }

    Ok(())
}