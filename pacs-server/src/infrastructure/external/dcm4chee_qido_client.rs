use crate::domain::ServiceError;
use crate::infrastructure::config::Dcm4cheeConfig;
use reqwest::{Client, Url};
use serde_json::Value;
use tracing;

#[derive(Clone)]
pub struct Dcm4cheeQidoClient {
    base_url: String,
    qido_path: String,
    username: Option<String>,
    password: Option<String>,
    http_client: Client,
    timeout_ms: u64,
}

impl Dcm4cheeQidoClient {
    pub fn new(config: Dcm4cheeConfig) -> Self {
        Self {
            base_url: config.base_url,
            qido_path: config.qido_path,
            username: config.username,
            password: config.password,
            http_client: Client::new(),
            timeout_ms: config.timeout_ms,
        }
    }

    fn build_url(&self, path: &str, query: &[(&str, &str)]) -> Result<Url, ServiceError> {
        let mut url = Url::parse(&self.base_url)
            .map_err(|e| ServiceError::ExternalServiceError(format!("Invalid base_url: {}", e)))?;
        url.set_path(path);
        let mut pairs = url.query_pairs_mut();
        for (k, v) in query {
            pairs.append_pair(k, v);
        }
        drop(pairs);
        Ok(url)
    }

    pub async fn qido_studies(&self, params: Vec<(String, String)>) -> Result<Value, ServiceError> {
        // Ensure standard response accept header
        let url = self.build_url(&self.qido_path.replace("/rs", "/rs/studies"), &[])?;
        let mut req = self
            .http_client
            .get(url)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .header("Accept", "application/json");

        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            req = req.basic_auth(u, Some(p));
        }

        if !params.is_empty() {
            let qp: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            req = req.query(&qp);
        }

        let resp = req.send().await.map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /studies failed: {}", e))
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ServiceError::ExternalServiceError(format!(
                "QIDO /studies failed ({}): {}",
                status, body
            )));
        }
        let json: Value = serde_json::from_str(&body).map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /studies parse error: {}", e))
        })?;
        Ok(json)
    }

    pub async fn qido_series(
        &self,
        study_uid: &str,
        params: Vec<(String, String)>,
    ) -> Result<Value, ServiceError> {
        let series_path = format!(
            "{}/studies/{}/series",
            self.qido_path.replace("/rs", "/rs"),
            study_uid
        );
        let url = self.build_url(&series_path, &[])?;
        let mut req = self
            .http_client
            .get(url)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .header("Accept", "application/json");

        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            req = req.basic_auth(u, Some(p));
        }

        if !params.is_empty() {
            let qp: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            req = req.query(&qp);
        }

        let resp = req.send().await.map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /series failed: {}", e))
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ServiceError::ExternalServiceError(format!(
                "QIDO /series failed ({}): {}",
                status, body
            )));
        }
        let json: Value = serde_json::from_str(&body).map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /series parse error: {}", e))
        })?;
        Ok(json)
    }

    // Bearer token relay variants (if Dcm4chee sits behind Keycloak)
    pub async fn qido_studies_with_bearer(
        &self,
        bearer_token: Option<&str>,
        params: Vec<(String, String)>,
    ) -> Result<Value, ServiceError> {
        let studies_path = if self.qido_path.ends_with("/rs") {
            format!("{}/studies", self.qido_path)
        } else {
            format!("{}/rs/studies", self.qido_path)
        };
        let url = self.build_url(&studies_path, &[])?;
        let mut req = self
            .http_client
            .get(url.clone())
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .header("Accept", "application/json");

        if let Some(token) = bearer_token {
            req = req.bearer_auth(token);
            tracing::debug!(
                "QIDO /studies: Using Bearer token (length: {})",
                token.len()
            );
            tracing::debug!("QIDO /studies: URL: {}", url);
            tracing::debug!(
                "QIDO /studies: Bearer token preview: {}...",
                &token[..std::cmp::min(50, token.len())]
            );
        } else if let (Some(u), Some(p)) = (&self.username, &self.password) {
            req = req.basic_auth(u, Some(p));
            tracing::debug!("QIDO /studies: Using Basic Auth");
        } else {
            tracing::warn!("QIDO /studies: No authentication method available");
        }

        if !params.is_empty() {
            let qp: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            req = req.query(&qp);
        }

        let resp = req.send().await.map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /studies failed: {}", e))
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ServiceError::ExternalServiceError(format!(
                "QIDO /studies failed ({}): {}",
                status, body
            )));
        }
        let json: Value = serde_json::from_str(&body).map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /studies parse error: {}", e))
        })?;
        Ok(json)
    }

    pub async fn qido_series_with_bearer(
        &self,
        bearer_token: Option<&str>,
        study_uid: &str,
        params: Vec<(String, String)>,
    ) -> Result<Value, ServiceError> {
        let series_path = if self.qido_path.ends_with("/rs") {
            format!("{}/studies/{}/series", self.qido_path, study_uid)
        } else {
            format!("{}/rs/studies/{}/series", self.qido_path, study_uid)
        };
        let url = self.build_url(&series_path, &[])?;
        let mut req = self
            .http_client
            .get(url)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .header("Accept", "application/json");

        if let Some(token) = bearer_token {
            req = req.bearer_auth(token);
        } else if let (Some(u), Some(p)) = (&self.username, &self.password) {
            req = req.basic_auth(u, Some(p));
        }

        if !params.is_empty() {
            let qp: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            req = req.query(&qp);
        }

        let resp = req.send().await.map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /series failed: {}", e))
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ServiceError::ExternalServiceError(format!(
                "QIDO /series failed ({}): {}",
                status, body
            )));
        }
        let json: Value = serde_json::from_str(&body).map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /series parse error: {}", e))
        })?;
        Ok(json)
    }

    pub async fn qido_instances_with_bearer(
        &self,
        bearer_token: Option<&str>,
        study_uid: &str,
        series_uid: &str,
        params: Vec<(String, String)>,
    ) -> Result<Value, ServiceError> {
        let inst_path = if self.qido_path.ends_with("/rs") {
            format!(
                "{}/studies/{}/series/{}/instances",
                self.qido_path, study_uid, series_uid
            )
        } else {
            format!(
                "{}/rs/studies/{}/series/{}/instances",
                self.qido_path, study_uid, series_uid
            )
        };
        let url = self.build_url(&inst_path, &[])?;
        let mut req = self
            .http_client
            .get(url)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .header("Accept", "application/json");

        if let Some(token) = bearer_token {
            req = req.bearer_auth(token);
        } else if let (Some(u), Some(p)) = (&self.username, &self.password) {
            req = req.basic_auth(u, Some(p));
        }

        if !params.is_empty() {
            let qp: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            req = req.query(&qp);
        }

        let resp = req.send().await.map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /instances failed: {}", e))
        })?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ServiceError::ExternalServiceError(format!(
                "QIDO /instances failed ({}): {}",
                status, body
            )));
        }
        let json: Value = serde_json::from_str(&body).map_err(|e| {
            ServiceError::ExternalServiceError(format!("QIDO /instances parse error: {}", e))
        })?;
        Ok(json)
    }
}
