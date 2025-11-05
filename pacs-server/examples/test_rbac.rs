/// RBAC 로직 테스트 프로그램
/// 
/// 이 프로그램은 새로운 RBAC 로직이 올바르게 동작하는지 테스트합니다.
/// 
/// 실행 방법:
/// ```bash
/// cargo run --example test_rbac
/// ```

use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 환경 변수 로드
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

    // 테스트 시나리오 실행
    println!("📋 RBAC 로직 테스트 시작\n");
    println!("{}", "=".repeat(80));

    // 시나리오 1: User 1 - 기본 접근 (모든 Study 접근 가능)
    println!("\n🧪 시나리오 1: User 1 - 기본 접근 (project_data_access에 레코드 없음)");
    println!("{}", "-".repeat(80));
    
    let user1_studies: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT 
            s.study_uid,
            s.study_description,
            CASE 
                WHEN EXISTS (
                    SELECT 1 FROM project_data_access pda
                    WHERE pda.user_id = u.id 
                      AND pda.project_id = p.id
                      AND pda.study_id = s.id
                      AND pda.status = 'DENIED'
                ) THEN '❌ DENIED'
                WHEN EXISTS (
                    SELECT 1 FROM project_data_access pda
                    WHERE pda.user_id = u.id 
                      AND pda.project_id = p.id
                      AND pda.study_id = s.id
                      AND pda.status = 'APPROVED'
                ) THEN '✅ APPROVED'
                ELSE '✅ DEFAULT'
            END as access_status
        FROM security_user u
        CROSS JOIN security_project p
        CROSS JOIN project_data_study s
        INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
        WHERE u.username = 'test_user_1'
          AND p.name = 'Test Project'
        ORDER BY s.study_uid"
    )
    .fetch_all(&pool)
    .await?;

    for (study_uid, description, status) in user1_studies {
        println!("  {} | {} | {}", study_uid, description, status);
    }

    // 시나리오 2: User 2 - Study 100 거부
    println!("\n🧪 시나리오 2: User 2 - Study 100 거부");
    println!("{}", "-".repeat(80));
    
    let user2_studies: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT 
            s.study_uid,
            s.study_description,
            CASE 
                WHEN EXISTS (
                    SELECT 1 FROM project_data_access pda
                    WHERE pda.user_id = u.id 
                      AND pda.project_id = p.id
                      AND pda.study_id = s.id
                      AND pda.status = 'DENIED'
                ) THEN '❌ DENIED'
                WHEN EXISTS (
                    SELECT 1 FROM project_data_access pda
                    WHERE pda.user_id = u.id 
                      AND pda.project_id = p.id
                      AND pda.study_id = s.id
                      AND pda.status = 'APPROVED'
                ) THEN '✅ APPROVED'
                ELSE '✅ DEFAULT'
            END as access_status
        FROM security_user u
        CROSS JOIN security_project p
        CROSS JOIN project_data_study s
        INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
        WHERE u.username = 'test_user_2'
          AND p.name = 'Test Project'
        ORDER BY s.study_uid"
    )
    .fetch_all(&pool)
    .await?;

    for (study_uid, description, status) in user2_studies {
        println!("  {} | {} | {}", study_uid, description, status);
    }

    // 시나리오 3: User 3 - Study 101 명시적 승인
    println!("\n🧪 시나리오 3: User 3 - Study 101 명시적 승인");
    println!("{}", "-".repeat(80));
    
    let user3_studies: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT 
            s.study_uid,
            s.study_description,
            CASE 
                WHEN EXISTS (
                    SELECT 1 FROM project_data_access pda
                    WHERE pda.user_id = u.id 
                      AND pda.project_id = p.id
                      AND pda.study_id = s.id
                      AND pda.status = 'DENIED'
                ) THEN '❌ DENIED'
                WHEN EXISTS (
                    SELECT 1 FROM project_data_access pda
                    WHERE pda.user_id = u.id 
                      AND pda.project_id = p.id
                      AND pda.study_id = s.id
                      AND pda.status = 'APPROVED'
                ) THEN '✅ APPROVED'
                ELSE '✅ DEFAULT'
            END as access_status
        FROM security_user u
        CROSS JOIN security_project p
        CROSS JOIN project_data_study s
        INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
        WHERE u.username = 'test_user_3'
          AND p.name = 'Test Project'
        ORDER BY s.study_uid"
    )
    .fetch_all(&pool)
    .await?;

    for (study_uid, description, status) in user3_studies {
        println!("  {} | {} | {}", study_uid, description, status);
    }

    println!("\n{}", "=".repeat(80));
    println!("\n✅ 테스트 완료!");
    
    println!("\n📊 예상 결과:");
    println!("  User 1: 모든 Study ✅ DEFAULT (프로젝트 멤버 기본 허용)");
    println!("  User 2: Study 100 ❌ DENIED, 나머지 ✅ DEFAULT");
    println!("  User 3: Study 101 ✅ APPROVED, 나머지 ✅ DEFAULT");

    Ok(())
}

