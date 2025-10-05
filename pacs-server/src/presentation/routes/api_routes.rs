use actix_web::web;

pub fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1"));
}
