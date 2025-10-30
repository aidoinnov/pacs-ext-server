use actix_web::HttpRequest;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use std::sync::Arc;
use uuid::Uuid;
#[allow(unused_imports)]
use tracing::{debug, warn};

use crate::infrastructure::auth::JwtService;
use crate::domain::repositories::UserRepository; // bring trait into scope for method resolution
use crate::infrastructure::repositories::UserRepositoryImpl;

/// Extract application `user_id` from Authorization bearer token.
/// 1) Try validating as our JWT and get user_id from claims
/// 2) If that fails, treat as Keycloak token: decode payload `sub` and look up user by keycloak_id
pub async fn extract_user_id_from_request(
    req: &HttpRequest,
    jwt: &Arc<JwtService>,
    user_repo: &Arc<UserRepositoryImpl>,
) -> Option<i32> {
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h,
        None => {
            warn!("Auth: missing Authorization header");
            return None;
        }
    };
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            warn!("Auth: invalid Authorization header (non-UTF8)");
            return None;
        }
    };
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
    // Try our own JWT first
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

    // Fallback: Keycloak access token → decode `sub` → DB lookup
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

/// Decode Keycloak JWT and return `sub` claim (without signature verification)
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


