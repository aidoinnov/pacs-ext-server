use actix_web::{test, web, App};
use pacs_server::infrastructure::auth::JwtService;
use pacs_server::infrastructure::config::JwtConfig;
use pacs_server::infrastructure::repositories::UserRepositoryImpl;
use pacs_server::presentation::controllers::user_controller::get_me;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

#[actix_web::test]
#[ignore]
async fn get_me_returns_profile_with_valid_token() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect DB");

    let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone()));

    // Insert test user
    let keycloak_id = Uuid::new_v4();
    let username = format!("me_user_{}", Uuid::new_v4());
    let email = format!("{}@example.com", username);
    let rec: (i32,) = sqlx::query_as(
        "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(keycloak_id)
    .bind(&username)
    .bind(&email)
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id = rec.0;

    // Jwt service and token
    let jwt = JwtService::new(&JwtConfig {
        secret: "test-secret-key-for-jwt-token-generation-and-validation".to_string(),
        expiration_hours: 24,
    });
    let claims = pacs_server::infrastructure::auth::Claims::new(
        user_id,
        keycloak_id,
        username.clone(),
        email.clone(),
        24,
    );
    let token = jwt.create_token(&claims).unwrap();

    // Minimal app with the route
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(Arc::new(jwt)))
            .app_data(web::Data::new(user_repo.clone()))
            .service(web::resource("/api/users/me").route(web::get().to(get_me))),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/users/me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"].as_i64().unwrap() as i32, user_id);

    // cleanup
    let _ = sqlx::query("DELETE FROM security_user WHERE id = $1").bind(user_id).execute(&pool).await;
}


