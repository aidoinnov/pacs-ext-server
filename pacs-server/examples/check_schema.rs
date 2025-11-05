/// 스키마 확인 프로그램

use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL must be set");

    println!("🔗 데이터베이스 연결 중...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✅ 데이터베이스 연결 성공!\n");

    // project_data 테이블 확인
    println!("📋 project_data 테이블 구조:");
    println!("{}", "-".repeat(80));
    
    let columns: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_name = 'project_data'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await?;

    if columns.is_empty() {
        println!("❌ project_data 테이블이 존재하지 않습니다!");
    } else {
        for (column_name, data_type, is_nullable) in columns {
            println!("  {} | {} | nullable: {}", column_name, data_type, is_nullable);
        }
    }

    // project_data_study 테이블 확인
    println!("\n📋 project_data_study 테이블 구조:");
    println!("{}", "-".repeat(80));
    
    let study_columns: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_name = 'project_data_study'
         ORDER BY ordinal_position"
    )
    .fetch_all(&pool)
    .await?;

    for (column_name, data_type, is_nullable) in study_columns {
        println!("  {} | {} | nullable: {}", column_name, data_type, is_nullable);
    }

    Ok(())
}

