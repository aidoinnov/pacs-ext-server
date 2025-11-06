//! Token extraction helpers used by presentation controllers.
//!
//! This module provides utilities to resolve the application `user_id` from an
//! incoming HTTP request. The resolution follows two paths:
//! - Internal JWT: validate the token and read `sub` as our numeric user id
//! - Keycloak JWT: base64url-decode payload, read `sub` (UUID), then map to
//!   local user via `security_user.keycloak_id`
//!
//! Security notes:
//! - For Keycloak tokens we only decode payload to read `sub`; signature
//!   verification is not performed here because upstream (Keycloak-protected
//!   dcm4chee) already enforces it and our gateway primarily needs the local
//!   user mapping. If signature verification becomes a requirement at the
//!   gateway, wire a proper Keycloak public-key verifier before using `sub`.
//! - Always prefer the internal JWT path when available.

use actix_web::HttpRequest;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use std::sync::Arc;
use uuid::Uuid;
#[allow(unused_imports)]
use tracing::{debug, warn};

use crate::infrastructure::auth::JwtService;
use crate::domain::repositories::UserRepository; // bring trait into scope for method resolution
use crate::infrastructure::repositories::UserRepositoryImpl;

/// Extract application `user_id` from the `Authorization: Bearer ...` header.
///
/// Resolution order:
/// 1) Internal JWT: validate and parse claims → `claims.user_id()`
/// 2) Keycloak JWT: decode payload without signature verification → read `sub`
///    (UUID) → `security_user.keycloak_id` lookup → local `user.id`
///
/// Returns `Some(user_id)` on success, otherwise `None`.
pub async fn extract_user_id_from_request(
    req: &HttpRequest,
    jwt: &Arc<JwtService>,
    user_repo: &Arc<UserRepositoryImpl>,
) -> Option<i32> {
    // 1) Read Authorization header
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h,
        None => {
            warn!("Auth: missing Authorization header");
            return None;
        }
    };
    // 2) Convert to &str
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            warn!("Auth: invalid Authorization header (non-UTF8)");
            return None;
        }
    };
    // 3) Extract bearer token
    let token = match auth_str.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            warn!("Auth: Authorization header present but not Bearer");
            return None;
        }
    };
    debug!(
        "Auth: bearer token received (len={} preview={}...)",
        token.len(),
        &token[..std::cmp::min(32, token.len())]
    );
    // 4) Try internal JWT path first
    match jwt.validate_token(token) {
        Ok(claims) => match claims.user_id() {
            Ok(uid) => {
                debug!("Auth: validated as internal JWT, user_id={}", uid);
                return Some(uid);
            }
            Err(e) => {
                warn!("Auth: internal JWT claims present but user_id error: {:?}", e);
            }
        },
        Err(e) => {
            debug!("Auth: not internal JWT (will try Keycloak): {}", e);
        }
    }

    // 5) Fallback: Keycloak token → decode `sub` → DB mapping by keycloak_id
    if let Some(keycloak_sub) = decode_keycloak_token_sub(token) {
        debug!("Auth: decoded Keycloak sub={}", keycloak_sub);
        match Uuid::parse_str(&keycloak_sub) {
            Ok(keycloak_id) => match user_repo.find_by_keycloak_id(keycloak_id).await {
                Ok(Some(user)) => {
                    debug!("Auth: mapped Keycloak user to local user_id={}", user.id);
                    return Some(user.id);
                }
                Ok(None) => {
                    warn!("Auth: no local user mapped for keycloak_id={}", keycloak_id);
                }
                Err(e) => {
                    warn!("Auth: DB error while looking up keycloak_id: {}", e);
                }
            },
            Err(e) => {
                warn!("Auth: invalid Keycloak sub (UUID parse failed): {}", e);
            }
        }
    } else {
        warn!("Auth: failed to decode Keycloak token sub");
    }

    None
}

/// Decode Keycloak JWT payload and return the `sub` claim as `String`.
///
/// Notes:
/// - This function does not verify the token signature. It is intended only to
///   extract the `sub` claim for mapping to a local user. Perform verification
///   upstream if required.
/// - Returns `None` if the token format is invalid or the claim is missing.
pub fn decode_keycloak_token_sub(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = parts[1];
    let mut padded = payload.to_string();
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    let decoded = URL_SAFE.decode(&padded).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get("sub")?.as_str().map(|s| s.to_string())
}


