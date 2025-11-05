/// 테스트 데이터 준비 프로그램

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
    println!("🚀 테스트 데이터 준비 중...\n");

    // 프로젝트 생성
    println!("1️⃣ 프로젝트 생성...");
    sqlx::query(
        "INSERT INTO security_project (name, description, sponsor, start_date, end_date, status, is_active)
         VALUES ('Test Project', 'RBAC 테스트용 프로젝트', 'Test Sponsor', CURRENT_DATE, CURRENT_DATE + INTERVAL '1 year', 'ACTIVE', true)
         ON CONFLICT DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to create project");

    // 사용자 생성
    println!("2️⃣ 사용자 생성...");
    sqlx::query(
        "INSERT INTO security_user (keycloak_id, username, email, full_name)
         VALUES
             ('00000000-0000-0000-0000-000000000001'::uuid, 'test_user_1', 'user1@test.com', 'Test User 1'),
             ('00000000-0000-0000-0000-000000000002'::uuid, 'test_user_2', 'user2@test.com', 'Test User 2'),
             ('00000000-0000-0000-0000-000000000003'::uuid, 'test_user_3', 'user3@test.com', 'Test User 3')
         ON CONFLICT (keycloak_id) DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to create users");

    // 프로젝트 멤버십 추가
    println!("3️⃣ 프로젝트 멤버십 추가...");
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id)
         SELECT u.id, p.id
         FROM security_user u, security_project p
         WHERE u.username IN ('test_user_1', 'test_user_2', 'test_user_3')
           AND p.name = 'Test Project'
         ON CONFLICT DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to add project membership");

    // Study 데이터 생성
    println!("4️⃣ Study 데이터 생성...");
    sqlx::query(
        "INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
         VALUES 
             ('1.2.3.100', 'Test Study 100', 'P001', 'Patient 001', '2024-01-01'),
             ('1.2.3.101', 'Test Study 101', 'P002', 'Patient 002', '2024-01-02'),
             ('1.2.3.102', 'Test Study 102', 'P003', 'Patient 003', '2024-01-03')
         ON CONFLICT (study_uid) DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to create studies");

    // project_data에 Study 매핑
    println!("5️⃣ 프로젝트에 Study 매핑...");
    sqlx::query(
        "INSERT INTO project_data (project_id, resource_level, study_id)
         SELECT p.id, 'STUDY', s.id
         FROM security_project p, project_data_study s
         WHERE p.name = 'Test Project'
           AND s.study_uid IN ('1.2.3.100', '1.2.3.101', '1.2.3.102')
         ON CONFLICT DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to map studies to project");

    // User 2에게 Study 100 거부
    println!("6️⃣ User 2에게 Study 100 거부 설정...");
    sqlx::query(
        "INSERT INTO project_data_access (project_id, user_id, resource_level, study_id, status, project_data_id)
         SELECT p.id, u.id, 'STUDY', s.id, 'DENIED', pd.id
         FROM security_project p
         CROSS JOIN security_user u
         CROSS JOIN project_data_study s
         INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
         WHERE p.name = 'Test Project'
           AND u.username = 'test_user_2'
           AND s.study_uid = '1.2.3.100'
         ON CONFLICT DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to deny study for user 2");

    // User 3에게 Study 101 명시적 승인
    println!("7️⃣ User 3에게 Study 101 명시적 승인 설정...");
    sqlx::query(
        "INSERT INTO project_data_access (project_id, user_id, resource_level, study_id, status, project_data_id)
         SELECT p.id, u.id, 'STUDY', s.id, 'APPROVED', pd.id
         FROM security_project p
         CROSS JOIN security_user u
         CROSS JOIN project_data_study s
         INNER JOIN project_data pd ON pd.study_id = s.id AND pd.project_id = p.id
         WHERE p.name = 'Test Project'
           AND u.username = 'test_user_3'
           AND s.study_uid = '1.2.3.101'
         ON CONFLICT DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to approve study for user 3");

    println!("\n✅ 테스트 데이터 준비 완료!");
}

