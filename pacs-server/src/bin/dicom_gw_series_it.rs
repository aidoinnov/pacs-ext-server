use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

#[actix_rt::main]
async fn main() {
    // fixed study UID for stub
    let study_uid = "1.2.3.4";

    // 1) Start stub QIDO server for /rs/{study_uid}/series
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub");
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(move || {
        App::new().route(
            "/rs/studies/{study_uid}/series",
            web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!([
                    {
                        "0020000E": {"Value": ["7.7.7.7"], "vr": "UI"},
                        "00080060": {"Value": ["CT"], "vr": "CS"}
                    }
                ]))
            }),
        )
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

    // 3) In-memory app with /api/dicom/studies/{uid}/series
    let app = actix_web::test::init_service(
        App::new().service(
            web::scope("/api/dicom")
                .app_data(web::Data::new(qido))
                .route(
                "/studies/{study_uid}/series",
                web::get().to(
                    pacs_server::presentation::controllers::dicom_gateway_controller::get_series,
                ),
            ),
        ),
    )
    .await;

    // 4) Call with bearer header and limit
    let url = format!("/api/dicom/studies/{}/series?limit=1", study_uid);
    let req = actix_web::test::TestRequest::get()
        .uri(&url)
        .insert_header(("Authorization", "Bearer dummy"))
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    if !resp.status().is_success() {
        eprintln!(
            "Integration check (series) failed: status={}",
            resp.status()
        );
        std::process::exit(1);
    }

    let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
    if !body.is_array() || body.as_array().unwrap().is_empty() {
        eprintln!(
            "Integration check (series) failed: body not array or empty: {}",
            body
        );
        std::process::exit(2);
    }

    println!(
        "dicom_gw_series_it OK: {} item(s)",
        body.as_array().unwrap().len()
    );
}
