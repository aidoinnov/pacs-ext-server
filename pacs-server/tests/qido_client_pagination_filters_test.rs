use pacs_server::infrastructure::external::Dcm4cheeQidoClient;
use pacs_server::infrastructure::config::Dcm4cheeConfig;
use mockito::{Mock, Server};

#[tokio::test]
async fn qido_studies_propagates_filters_and_pagination() {
    // Start a mock server
    let mut server = Server::new_async().await;

    // Expect GET /dcm4chee-arc/aets/DCM4CHEE/rs/studies with query params
    let _m: Mock = server.mock("GET", "/dcm4chee-arc/aets/DCM4CHEE/rs/studies")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("Modality".into(), "CT".into()),
            mockito::Matcher::UrlEncoded("StudyDate".into(), "20240101-20241231".into()),
            mockito::Matcher::UrlEncoded("limit".into(), "10".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body("[]")
        .create();

    let cfg = Dcm4cheeConfig {
        base_url: server.url(),
        qido_path: "/dcm4chee-arc/aets/DCM4CHEE/rs".to_string(),
        wado_path: "/dcm4chee-arc/aets/DCM4CHEE/wado".to_string(),
        aet: "DCM4CHEE".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
        db: None,
    };
    let client = Dcm4cheeQidoClient::new(cfg);

    let params = vec![
        ("Modality".to_string(), "CT".to_string()),
        ("StudyDate".to_string(), "20240101-20241231".to_string()),
        ("limit".to_string(), "10".to_string()),
        ("offset".to_string(), "10".to_string()),
    ];

    let res = client.qido_studies_with_bearer(None, params).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn qido_series_propagates_filters_and_pagination() {
    let mut server = Server::new_async().await;

    let study_uid = "1.2.3.4";
    let path = format!("/dcm4chee-arc/aets/DCM4CHEE/rs/studies/{}/series", study_uid);
    let _m: Mock = server.mock("GET", path.as_str())
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("Modality".into(), "MR".into()),
            mockito::Matcher::UrlEncoded("limit".into(), "5".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body("[]")
        .create();

    let cfg = Dcm4cheeConfig {
        base_url: server.url(),
        qido_path: "/dcm4chee-arc/aets/DCM4CHEE/rs".to_string(),
        wado_path: "/dcm4chee-arc/aets/DCM4CHEE/wado".to_string(),
        aet: "DCM4CHEE".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
        db: None,
    };
    let client = Dcm4cheeQidoClient::new(cfg);

    let params = vec![
        ("Modality".to_string(), "MR".to_string()),
        ("limit".to_string(), "5".to_string()),
        ("offset".to_string(), "0".to_string()),
    ];

    let res = client.qido_series_with_bearer(None, study_uid, params).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn qido_instances_propagates_filters_and_pagination() {
    let mut server = Server::new_async().await;

    let study_uid = "1.2.840.10008.1";
    let series_uid = "1.2.840.20008.1";
    let path = format!(
        "/dcm4chee-arc/aets/DCM4CHEE/rs/studies/{}/series/{}/instances",
        study_uid, series_uid
    );
    let _m: Mock = server.mock("GET", path.as_str())
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("AccessionNumber".into(), "ACC-123".into()),
            mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body("[]")
        .create();

    let cfg = Dcm4cheeConfig {
        base_url: server.url(),
        qido_path: "/dcm4chee-arc/aets/DCM4CHEE/rs".to_string(),
        wado_path: "/dcm4chee-arc/aets/DCM4CHEE/wado".to_string(),
        aet: "DCM4CHEE".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
        db: None,
    };
    let client = Dcm4cheeQidoClient::new(cfg);

    let params = vec![
        ("AccessionNumber".to_string(), "ACC-123".to_string()),
        ("limit".to_string(), "1".to_string()),
        ("offset".to_string(), "0".to_string()),
    ];

    let res = client.qido_instances_with_bearer(None, study_uid, series_uid, params).await;
    assert!(res.is_ok());
}


