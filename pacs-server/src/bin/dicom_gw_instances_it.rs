use actix_web::{App, HttpResponse, web, HttpServer};
use std::net::TcpListener;

#[actix_rt::main]
async fn main() {
    let study_uid = "1.2.3.4";

    // Stub QIDO server for /rs/studies/{uid}/instances
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub");
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(move || {
        App::new().route("/rs/studies/{study_uid}/instances", web::get().to(|| async {
            HttpResponse::Ok().json(serde_json::json!([
                {
                    "00080018": {"Value": ["9.9.9.9"], "vr": "UI"},
                    "0020000E": {"Value": ["7.7.7.7"], "vr": "UI"}
                }
            ]))
        }))
    })
    .listen(listener)
    .expect("listen")
    .run();
    actix_rt::spawn(server);

    // Dcm4chee client pointing to stub
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

    // App exposing /api/dicom/studies/{uid}/instances
    let app = actix_web::test::init_service(
        App::new().service(
            web::scope("/api/dicom")
                .app_data(web::Data::new(qido))
                .route(
                    "/studies/{study_uid}/instances",
                    web::get().to(pacs_server::presentation::controllers::dicom_gateway_controller::get_instances),
                ),
        ),
    )
    .await;

    // Call
    let url = format!("/api/dicom/studies/{}/instances?limit=1", study_uid);
    let req = actix_web::test::TestRequest::get()
        .uri(&url)
        .insert_header(("Authorization", "Bearer dummy"))
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    if !resp.status().is_success() {
        eprintln!("Integration check (instances) failed: status={}", resp.status());
        std::process::exit(1);
    }

    let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
    if !body.is_array() || body.as_array().unwrap().is_empty() {
        eprintln!("Integration check (instances) failed: body not array or empty: {}", body);
        std::process::exit(2);
    }

    println!("dicom_gw_instances_it OK: {} item(s)", body.as_array().unwrap().len());
}


