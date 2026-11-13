//! xAI Grok OAuth, as in Hermes' `xai-oauth` provider.

use super::{OAuthError, OAuthProvider, OAuthTokens};
use async_trait::async_trait;
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use chrono::Utc;
use std::collections::BTreeMap;

// Public client ID shared with Hermes and grok-cli.
const DEFAULT_CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
const DEFAULT_TOKEN_URL: &str = "https://auth.x.ai/oauth2/token";
const UPSTREAM_URL: &str = "https://api.x.ai";

#[derive(Debug)]
pub struct XaiProvider {
    client_id: String,
    token_url: String,
    http_client: reqwest::Client,
}

impl XaiProvider {
    pub fn new(client_id: Option<&str>, token_url: Option<&str>) -> Self {
        Self {
            client_id: client_id.unwrap_or(DEFAULT_CLIENT_ID).to_string(),
            token_url: token_url.unwrap_or(DEFAULT_TOKEN_URL).to_string(),
            http_client: reqwest::Client::builder()
                .user_agent(format!("ClawShell/{}", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub fn from_config(config: &super::OAuthProviderConfig) -> Self {
        Self::new(config.client_id.as_deref(), config.token_url.as_deref())
    }
}

fn parse_token_response(
    json: &serde_json::Value,
    fallback_refresh: &str,
) -> Result<OAuthTokens, OAuthError> {
    let access_token = json
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OAuthError::RefreshFailed("missing access_token in response".to_string()))?
        .to_string();
    let refresh_token = json
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback_refresh)
        .to_string();
    let id_token = json
        .get("id_token")
        .and_then(|v| v.as_str())
        .map(String::from);
    let expires_at = json
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

    Ok(OAuthTokens {
        access_token,
        refresh_token: Some(refresh_token),
        id_token,
        expires_at,
        account_id: None,
        extra: BTreeMap::new(),
    })
}

#[async_trait]
impl OAuthProvider for XaiProvider {
    fn id(&self) -> &str {
        "xai"
    }

    fn display_name(&self) -> &str {
        "xAI Grok (OAuth)"
    }

    async fn login_browser(&self, _callback_port: u16) -> Result<OAuthTokens, OAuthError> {
        Err(OAuthError::LoginFailed(
            "xAI login is not supported here; run `hermes auth add xai-oauth` and paste the tokens"
                .to_string(),
        ))
    }

    async fn login_headless(&self) -> Result<OAuthTokens, OAuthError> {
        self.login_browser(0).await
    }

    async fn refresh(&self, refresh_token: &str) -> Result<OAuthTokens, OAuthError> {
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", self.client_id.as_str()),
            ("refresh_token", refresh_token),
        ];
        let resp = self
            .http_client
            .post(&self.token_url)
            .header(axum::http::header::ACCEPT, "application/json")
            .form(&params)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(OAuthError::RefreshFailed(format!(
                "refresh failed ({status}): {body}"
            )));
        }
        let json: serde_json::Value = resp.json().await?;
        parse_token_response(&json, refresh_token)
    }

    fn inject_auth(&self, headers: &mut HeaderMap, access_token: &str) -> Result<(), OAuthError> {
        headers.insert(AUTHORIZATION, format!("Bearer {access_token}").parse()?);
        Ok(())
    }

    fn upstream_url(&self, _tokens: &OAuthTokens) -> Option<String> {
        Some(UPSTREAM_URL.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let provider = XaiProvider::new(None, None);
        assert_eq!(provider.id(), "xai");
        assert_eq!(provider.client_id, DEFAULT_CLIENT_ID);
        assert_eq!(provider.token_url, DEFAULT_TOKEN_URL);
    }

    #[test]
    fn test_parse_rotated_refresh_token() {
        let json = serde_json::json!({
            "access_token": "new-access",
            "refresh_token": "new-refresh",
            "expires_in": 21600
        });
        let tokens = parse_token_response(&json, "old-refresh").unwrap();
        assert_eq!(tokens.access_token, "new-access");
        assert_eq!(tokens.refresh_token.as_deref(), Some("new-refresh"));
        assert!(tokens.expires_at.is_some());
    }

    #[test]
    fn test_parse_keeps_refresh_token_when_not_rotated() {
        let json = serde_json::json!({"access_token": "a"});
        let tokens = parse_token_response(&json, "old-refresh").unwrap();
        assert_eq!(tokens.refresh_token.as_deref(), Some("old-refresh"));
    }

    #[test]
    fn test_parse_missing_access_token() {
        let json = serde_json::json!({"refresh_token": "r"});
        assert!(parse_token_response(&json, "old").is_err());
    }

    #[test]
    fn test_inject_auth_and_upstream() {
        let provider = XaiProvider::new(None, None);
        let mut headers = HeaderMap::new();
        provider.inject_auth(&mut headers, "tok").unwrap();
        assert_eq!(headers.get("authorization").unwrap(), "Bearer tok");
        let tokens = parse_token_response(&serde_json::json!({"access_token": "a"}), "r").unwrap();
        assert_eq!(
            provider.upstream_url(&tokens).as_deref(),
            Some("https://api.x.ai")
        );
        assert_eq!(provider.rewrite_request_path("/v1/chat/completions"), None);
    }
}
