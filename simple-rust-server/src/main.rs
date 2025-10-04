use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting on http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
