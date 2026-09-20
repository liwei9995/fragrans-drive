use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::api::error::AppError;
use crate::config::Config;
use crate::domain::user::{StoredPasskey, User};

pub struct WebauthnService {
    webauthn: Arc<Webauthn>,
    reg_states: Arc<RwLock<HashMap<String, (PasskeyRegistration, Instant)>>>,
    auth_states: Arc<RwLock<HashMap<String, (PasskeyAuthentication, Instant)>>>,
}

impl WebauthnService {
    pub fn new(config: &Config) -> Result<Self, AppError> {
        let (rp_id, rp_origin) = Self::parse_rp_config(&config.domain)?;

        let mut builder = WebauthnBuilder::new(&rp_id, &rp_origin)
            .map_err(|e| AppError::InternalError(format!("WebauthnBuilder init failed: {}", e)))?
            .rp_name("Fragrans Drive");

        // Allow localhost origin for local development if not already the rp_origin
        if let Ok(dev_origin) = Url::parse("http://localhost:5173") {
            if dev_origin != rp_origin && &rp_id == "localhost" {
                builder = builder.append_allowed_origin(&dev_origin);
            }
        }

        let webauthn = builder
            .build()
            .map_err(|e| AppError::InternalError(format!("Webauthn build failed: {}", e)))?;

        Ok(Self {
            webauthn: Arc::new(webauthn),
            reg_states: Arc::new(RwLock::new(HashMap::new())),
            auth_states: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn parse_rp_config(domain: &str) -> Result<(String, Url), AppError> {
        if let Ok(url) = Url::parse(domain) {
            let host = url
                .host_str()
                .ok_or_else(|| AppError::InternalError("Missing host in domain".to_string()))?
                .to_string();
            return Ok((host, url));
        }

        let default_url = Url::parse("http://localhost:5173")
            .map_err(|e| AppError::InternalError(format!("Invalid fallback URL: {}", e)))?;
        Ok(("localhost".to_string(), default_url))
    }

    pub async fn start_registration(
        &self,
        user: &User,
    ) -> Result<(CreationChallengeResponse, String), AppError> {
        let user_id = user
            .id
            .ok_or_else(|| AppError::InternalError("User missing ID".to_string()))?;
        let mut bytes = [0u8; 16];
        bytes[..12].copy_from_slice(&user_id.bytes());
        let user_uuid = Uuid::from_bytes(bytes);

        let display_name = if !user.first_name.is_empty() || !user.last_name.is_empty() {
            format!("{} {}", user.first_name, user.last_name).trim().to_string()
        } else {
            user.email.clone()
        };

        let exclude_credentials = if user.passkeys.is_empty() {
            None
        } else {
            let mut list = Vec::new();
            for sp in &user.passkeys {
                if let Ok(pk) = sp.get_passkey() {
                    list.push(pk.cred_id().clone());
                }
            }
            if list.is_empty() {
                None
            } else {
                Some(list)
            }
        };

        let (challenge, reg_state) = self
            .webauthn
            .start_passkey_registration(user_uuid, &user.email, &display_name, exclude_credentials)
            .map_err(|e| AppError::InternalError(format!("Failed to start passkey registration: {}", e)))?;

        let session_id = Uuid::new_v4().to_string();
        {
            let mut lock = self.reg_states.write().await;
            lock.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(300));
            lock.insert(session_id.clone(), (reg_state, Instant::now()));
        }

        Ok((challenge, session_id))
    }

    pub async fn finish_registration(
        &self,
        session_id: &str,
        req: &RegisterPublicKeyCredential,
        device_name: Option<String>,
    ) -> Result<StoredPasskey, AppError> {
        let reg_state = {
            let mut lock = self.reg_states.write().await;
            match lock.remove(session_id) {
                Some((state, time)) => {
                    if time.elapsed() > Duration::from_secs(300) {
                        return Err(AppError::BadRequest(
                            "Registration ceremony timed out".to_string(),
                        ));
                    }
                    state
                }
                None => {
                    return Err(AppError::BadRequest(
                        "Invalid or expired registration session".to_string(),
                    ));
                }
            }
        };

        let passkey = self
            .webauthn
            .finish_passkey_registration(req, &reg_state)
            .map_err(|e| AppError::BadRequest(format!("Registration verification failed: {}", e)))?;

        let passkey_json = serde_json::to_string(&passkey)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize passkey: {}", e)))?;

        let cred_id_str = req.id.to_string();
        let name = device_name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| "Touch ID / Passkey".to_string());

        Ok(StoredPasskey {
            id: cred_id_str,
            name,
            passkey_json,
            created_at: Some(chrono::Utc::now()),
        })
    }

    pub async fn start_authentication(
        &self,
        user: Option<&User>,
    ) -> Result<(RequestChallengeResponse, String), AppError> {
        let passkeys = if let Some(u) = user {
            let mut list = Vec::new();
            for sp in &u.passkeys {
                if let Ok(pk) = sp.get_passkey() {
                    list.push(pk);
                }
            }
            list
        } else {
            Vec::new()
        };

        let (challenge, auth_state) = self
            .webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| {
                AppError::InternalError(format!("Failed to start passkey authentication: {}", e))
            })?;

        let session_id = Uuid::new_v4().to_string();
        {
            let mut lock = self.auth_states.write().await;
            lock.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(300));
            lock.insert(session_id.clone(), (auth_state, Instant::now()));
        }

        Ok((challenge, session_id))
    }

    pub async fn finish_authentication(
        &self,
        session_id: &str,
        req: &PublicKeyCredential,
    ) -> Result<AuthenticationResult, AppError> {
        let auth_state = {
            let mut lock = self.auth_states.write().await;
            match lock.remove(session_id) {
                Some((state, time)) => {
                    if time.elapsed() > Duration::from_secs(300) {
                        return Err(AppError::BadRequest(
                            "Authentication ceremony timed out".to_string(),
                        ));
                    }
                    state
                }
                None => {
                    return Err(AppError::BadRequest(
                        "Invalid or expired authentication session".to_string(),
                    ));
                }
            }
        };

        let auth_result = self
            .webauthn
            .finish_passkey_authentication(req, &auth_state)
            .map_err(|e| AppError::Unauthorized(format!("Touch ID authentication failed: {}", e)))?;

        Ok(auth_result)
    }
}
