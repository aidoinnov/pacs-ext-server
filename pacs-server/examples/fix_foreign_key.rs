/// 외래 키 수정 프로그램

use sqlx::postgres::PgPoolOptions;
use std::env;

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
    println!("🚀 외래 키 수정 중...\n");

    // 기존 외래 키 삭제
    println!("1️⃣ 기존 외래 키 삭제...");
    sqlx::query(
        "ALTER TABLE project_data_access DROP CONSTRAINT IF EXISTS project_data_access_project_data_id_fkey"
    )
    .execute(&pool)
    .await
    .expect("Failed to drop foreign key");

    // 새로운 외래 키 추가
    println!("2️⃣ 새로운 외래 키 추가...");
    sqlx::query(
        "ALTER TABLE project_data_access 
         ADD CONSTRAINT project_data_access_project_data_id_fkey 
         FOREIGN KEY (project_data_id) REFERENCES project_data(id) ON DELETE CASCADE"
    )
    .execute(&pool)
    .await
    .expect("Failed to add foreign key");

    println!("\n✅ 외래 키 수정 완료!");
}

