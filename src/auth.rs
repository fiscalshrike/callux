use crate::config::Config;
use crate::error::{CalendarError, Result};
use google_calendar3::hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use std::path::Path;
use yup_oauth2::authenticator::Authenticator;
use yup_oauth2::{ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub struct AuthManager {
    config: Config,
}

impl AuthManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn get_authenticator(&self) -> Result<Authenticator<HttpsConnector<HttpConnector>>> {
        let credentials_path = self.config.expand_path(&self.config.auth.credentials_path);
        let token_cache_path = self.config.expand_path(&self.config.auth.token_cache_path);

        if !Path::new(&credentials_path).exists() {
            return Err(CalendarError::AuthenticationFailed(format!(
                "Credentials file not found at: {}",
                credentials_path
            )));
        }

        let secret = self.load_application_secret(&credentials_path)?;

        if let Some(parent) = Path::new(&token_cache_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CalendarError::ConfigError(format!("Failed to create token cache directory: {}", e))
            })?;
        }

        let authenticator =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk(&token_cache_path)
                .build()
                .await
                .map_err(|e| {
                    CalendarError::AuthenticationFailed(format!(
                        "Failed to create authenticator: {}",
                        e
                    ))
                })?;

        Ok(authenticator)
    }

    pub async fn get_token(&self) -> Result<String> {
        let authenticator = self.get_authenticator().await?;

        let scopes = &[
            "https://www.googleapis.com/auth/calendar.readonly",
            "https://www.googleapis.com/auth/calendar.events.readonly",
        ];

        let token = authenticator.token(scopes).await.map_err(|e| {
            CalendarError::AuthenticationFailed(format!("Failed to get token: {}", e))
        })?;

        Ok(token.token().unwrap_or_default().to_string())
    }

    fn load_application_secret(&self, path: &str) -> Result<ApplicationSecret> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            CalendarError::ConfigError(format!("Failed to read credentials file: {}", e))
        })?;

        let credentials: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            CalendarError::ParseError(format!("Invalid JSON in credentials file: {}", e))
        })?;

        let installed = credentials
            .get("installed")
            .or_else(|| credentials.get("web"))
            .ok_or_else(|| {
                CalendarError::ParseError(
                    "Missing 'installed' or 'web' section in credentials".to_string(),
                )
            })?;

        let client_id = installed
            .get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CalendarError::ParseError("Missing 'client_id' in credentials".to_string())
            })?;

        let client_secret = installed
            .get("client_secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CalendarError::ParseError("Missing 'client_secret' in credentials".to_string())
            })?;

        let auth_uri = installed
            .get("auth_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("https://accounts.google.com/o/oauth2/auth");

        let token_uri = installed
            .get("token_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("https://oauth2.googleapis.com/token");

        let redirect_uris = installed
            .get("redirect_uris")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| vec![s.to_string()])
            .unwrap_or_else(|| vec!["http://localhost:8080".to_string()]);

        Ok(ApplicationSecret {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            auth_uri: auth_uri.to_string(),
            token_uri: token_uri.to_string(),
            auth_provider_x509_cert_url: None,
            client_x509_cert_url: None,
            redirect_uris,
            project_id: None,
            client_email: None,
        })
    }

    pub fn create_sample_credentials(&self) -> Result<()> {
        let credentials_path = self.config.expand_path(&self.config.auth.credentials_path);

        if let Some(parent) = Path::new(&credentials_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CalendarError::ConfigError(format!("Failed to create credentials directory: {}", e))
            })?;
        }

        let sample_credentials = serde_json::json!({
            "installed": {
                "client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
                "client_secret": "YOUR_CLIENT_SECRET",
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token",
                "redirect_uris": ["http://localhost:8080"]
            }
        });

        std::fs::write(
            &credentials_path,
            serde_json::to_string_pretty(&sample_credentials).unwrap(),
        )
        .map_err(|e| {
            CalendarError::ConfigError(format!("Failed to write sample credentials: {}", e))
        })?;

        Ok(())
    }
}
