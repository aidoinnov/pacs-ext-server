use actix_web::{App, HttpResponse, web, HttpServer};
use std::net::TcpListener;

#[actix_rt::main]
async fn main() {
    // 1) Start stub QIDO server
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub");
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(|| {
        App::new().route("/rs/studies", web::get().to(|| async {
            HttpResponse::Ok().json(serde_json::json!([
                {
                    "0020000D": {"Value": ["1.2.3.4"], "vr": "UI"},
                    "00080060": {"Value": ["CT"], "vr": "CS"}
                }
            ]))
        }))
    })
    .listen(listener)
    .expect("listen")
    .run();
    actix_rt::spawn(server);

    // 2) Build Dcm4chee client pointing to stub
    let cfg = pacs_server::infrastructure::config::Dcm4cheeConfig {
        base_url: format!("http://127.0.0.1:{}", port),
        qido_path: "/rs".to_string(),
        wado_path: "/wado".to_string(),
        aet: "TEST".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
        db: None,
    };
    let qido = pacs_server::infrastructure::external::Dcm4cheeQidoClient::new(cfg);

    // 3) In-memory app with only /api/dicom/studies_raw
    let app = actix_web::test::init_service(
        App::new().service(
            web::scope("/api/dicom")
                .app_data(web::Data::new(qido))
                .route("/studies_raw", web::get().to(pacs_server::presentation::controllers::dicom_gateway_controller::get_studies_raw)),
        ),
    )
    .await;

    // 4) Call with bearer header
    let req = actix_web::test::TestRequest::get()
        .uri("/api/dicom/studies_raw")
        .insert_header(("Authorization", "Bearer dummy"))
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    if !resp.status().is_success() {
        eprintln!("Integration check failed: status={}", resp.status());
        std::process::exit(1);
    }

    let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
    if !body.is_array() || body.as_array().unwrap().is_empty() {
        eprintln!("Integration check failed: body not array or empty: {}", body);
        std::process::exit(2);
    }

    println!("dicom_gw_it OK: {} item(s)", body.as_array().unwrap().len());
}


