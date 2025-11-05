/// 마이그레이션 020 실행 프로그램

use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL must be set");

    println!("🔗 데이터베이스 연결 중...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ 데이터베이스 연결 성공!\n");

    // 마이그레이션 파일 읽기
    println!("📄 마이그레이션 파일 읽는 중...");
    let migration_sql = fs::read_to_string("migrations/020_refactor_project_data_hierarchy.sql")
        .expect("Failed to read migration file");
    
    println!("🚀 마이그레이션 실행 중...\n");

    // SQL 실행
    match sqlx::raw_sql(&migration_sql).execute(&pool).await {
        Ok(_) => {
            println!("✅ 마이그레이션 020 실행 완료!");
        }
        Err(e) => {
            println!("❌ 마이그레이션 실행 실패: {}", e);
            panic!("Migration failed");
        }
    }

    // 결과 확인
    println!("\n📋 project_data 테이블 구조 확인:");
    println!("{}", "-".repeat(80));

    let columns: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_name = 'project_data'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch columns");

    for (column_name, data_type, is_nullable) in columns {
        println!("  {} | {} | nullable: {}", column_name, data_type, is_nullable);
    }

    println!("\n📋 project_data_study 테이블에 project_id 있는지 확인:");
    println!("{}", "-".repeat(80));

    let has_project_id: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = 'project_data_study' AND column_name = 'project_id'
        )"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check project_id");

    if has_project_id {
        println!("  ❌ project_id 필드가 아직 존재합니다!");
    } else {
        println!("  ✅ project_id 필드가 제거되었습니다!");
    }
}

