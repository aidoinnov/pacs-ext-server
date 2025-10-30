use std::env;

#[actix_rt::main]
async fn main() {
    // Read Keycloak and Gateway settings from environment
    let kc_base = env::var("KEYCLOAK_BASE_URL").expect("KEYCLOAK_BASE_URL is required");
    let realm = env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "dcm4che".to_string());
    let client_id = env::var("KEYCLOAK_CLIENT_ID").expect("KEYCLOAK_CLIENT_ID is required");
    let username = env::var("KEYCLOAK_USERNAME").ok();
    let password = env::var("KEYCLOAK_PASSWORD").ok();
    let client_secret = env::var("KEYCLOAK_CLIENT_SECRET").ok();
    let gateway = env::var("GATEWAY_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Obtain token via password grant (preferred) or client_credentials fallback
    let token_endpoint = format!("{}/realms/{}/protocol/openid-connect/token", kc_base.trim_end_matches('/'), realm);
    let http = reqwest::Client::new();

    let access_token = if let (Some(u), Some(p)) = (username, password) {
        // Resource Owner Password Credentials grant
        let mut form = vec![
            ("grant_type", "password"),
            ("client_id", client_id.as_str()),
            ("username", u.as_str()),
            ("password", p.as_str()),
        ];
        if let Some(sec) = client_secret.as_ref() { form.push(("client_secret", sec.as_str())); }
        let resp = http.post(&token_endpoint).form(&form).send().await.expect("token request failed");
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            eprintln!("Token error (password grant): {}", body);
            std::process::exit(1);
        }
        let v: serde_json::Value = resp.json().await.expect("token json parse failed");
        v.get("access_token").and_then(|v| v.as_str()).expect("missing access_token").to_string()
    } else {
        // Client Credentials grant
        let mut form = vec![
            ("grant_type", "client_credentials"),
            ("client_id", client_id.as_str()),
        ];
        if let Some(sec) = client_secret.as_ref() { form.push(("client_secret", sec.as_str())); }
        let resp = http.post(&token_endpoint).form(&form).send().await.expect("token request failed");
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            eprintln!("Token error (client credentials): {}", body);
            std::process::exit(1);
        }
        let v: serde_json::Value = resp.json().await.expect("token json parse failed");
        v.get("access_token").and_then(|v| v.as_str()).expect("missing access_token").to_string()
    };

    // Call gateway endpoints with Bearer token
    let auth = format!("Bearer {}", access_token);

    // studies
    let studies_url = format!("{}/api/dicom/studies?project_id=1&limit=1", gateway.trim_end_matches('/')); 
    let resp = http.get(&studies_url).header("Authorization", &auth).send().await.expect("studies request failed");
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!("/studies failed: {} {}", status, body);
        std::process::exit(2);
    }
    let studies: serde_json::Value = resp.json().await.expect("studies json parse failed");
    println!("studies OK: {}", studies);

    // If we have at least one study UID, try series and instances
    if let Some(uid) = studies.as_array().and_then(|a| a.get(0))
        .and_then(|item| item.get("0020000D")).and_then(|t| t.get("Value"))
        .and_then(|v| v.as_array()).and_then(|arr| arr.get(0)).and_then(|v| v.as_str())
    {
        let series_url = format!("{}/api/dicom/studies/{}/series?project_id=1&limit=1", gateway.trim_end_matches('/'), uid);
        let resp = http.get(&series_url).header("Authorization", &auth).send().await.expect("series request failed");
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            eprintln!("/series failed: {} {}", status, body);
            std::process::exit(3);
        }
        let series: serde_json::Value = resp.json().await.expect("series json parse failed");
        println!("series OK: {}", series);

        // Get series_uid first
        let series_uid = series.as_array().and_then(|a| a.get(0))
            .and_then(|item| item.get("0020000E")).and_then(|t| t.get("Value"))
            .and_then(|v| v.as_array()).and_then(|arr| arr.get(0)).and_then(|v| v.as_str());
        
        if let Some(sid) = series_uid {
            let instances_url = format!("{}/api/dicom/studies/{}/series/{}/instances?project_id=1&limit=1", gateway.trim_end_matches('/'), uid, sid);
            let resp = http.get(&instances_url).header("Authorization", &auth).send().await.expect("instances request failed");
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!("/instances failed: {} {}", status, body);
                std::process::exit(4);
            }
            let instances: serde_json::Value = resp.json().await.expect("instances json parse failed");
            println!("instances OK: {}", instances);
        } else {
            eprintln!("No SeriesInstanceUID found in /series response to proceed with instances.");
        }
    } else {
        eprintln!("No StudyInstanceUID found in /studies response to proceed with series/instances.");
    }
}


