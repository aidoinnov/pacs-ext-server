use pacs_server::infrastructure::external::dcm4chee_qido_client::Dcm4cheeQidoClient;
use pacs_server::infrastructure::config::Dcm4cheeConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 환경 변수 로드
    dotenvy::dotenv().ok();

    // 로깅 초기화
    env_logger::init();

    let base_url = std::env::var("APP_DCM4CHEEBASE_URL")
        .unwrap_or_else(|_| "https://archive.pacs.ai-do.co.kr".to_string());
    let qido_path = std::env::var("APP_DCM4CHEEQIDO_PATH")
        .unwrap_or_else(|_| "/iaid-pacs/aets/iAID_PACS/rs".to_string());

    println!("🔍 Testing Dcm4chee QIDO Client");
    println!("   Base URL: {}", base_url);
    println!("   QIDO Path: {}", qido_path);
    println!();

    let config = Dcm4cheeConfig {
        base_url,
        qido_path,
        wado_path: "/iaid-pacs/aets/iAID_PACS/wado".to_string(),
        aet: "iAID_PACS".to_string(),
        username: None,
        password: None,
        timeout_ms: 30000,
        db: None,
    };

    let client = Dcm4cheeQidoClient::new(config);

    // Keycloak에서 토큰 받기
    println!("🔐 Getting Bearer token from Keycloak...");
    let keycloak_url = "https://keycloak.pacs.ai-do.co.kr/realms/dcm4che/protocol/openid-connect/token";
    let http_client = reqwest::Client::new();
    
    let token_response = http_client
        .post(keycloak_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "password"),
            ("client_id", "pacs-extension-server"),
            ("client_secret", "vYMipExC4DCpesgWMy11FEOMWybxtpfq"),
            ("username", "iaid-pacs-admin"),
            ("password", "Qlalfqjsgh1!"),
        ])
        .send()
        .await?;

    let token_json: serde_json::Value = token_response.json().await?;
    let bearer_token = token_json["access_token"]
        .as_str()
        .ok_or("Failed to get access_token")?;

    println!("   ✅ Token received (length: {})", bearer_token.len());
    println!("   Token preview: {}...", &bearer_token[..std::cmp::min(50, bearer_token.len())]);
    println!();

    // QIDO-RS 호출 테스트
    println!("📡 Testing QIDO-RS /studies endpoint...");
    let params = vec![
        ("limit".to_string(), "5".to_string()),
        ("offset".to_string(), "0".to_string()),
    ];

    match client.qido_studies_with_bearer(Some(bearer_token), params).await {
        Ok(json) => {
            println!("   ✅ Success!");
            println!();
            println!("Response:");
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    Ok(())
}

