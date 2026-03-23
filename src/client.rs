use anyhow::{Context, Result};
use reqwest::header::{ACCEPT_LANGUAGE, AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

const BASE_URL: &str = "https://api.understory.io";
const TOKEN_URL: &str = "https://api.auth.understory.io/oauth2/token";

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: u64,
}

pub struct UnderstoryClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    token: Arc<RwLock<Option<String>>>,
}

impl UnderstoryClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            client_id,
            client_secret,
            token: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_token(&self) -> Result<String> {
        {
            let token = self.token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        let resp = self
            .http
            .post(TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("audience", "https://api.understory.io"),
                (
                    "scope",
                    "openid booking.read booking.write event.read experience.read marketing.read order.read webhook.read webhook.write",
                ),
            ])
            .send()
            .await
            .context("failed to request token")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("token request failed ({}): {}", status, body);
        }

        let resp = resp
            .json::<TokenResponse>()
            .await
            .context("failed to parse token response")?;

        let token = resp.access_token.clone();
        *self.token.write().await = Some(resp.access_token);
        Ok(token)
    }

    pub async fn invalidate_token(&self) {
        *self.token.write().await = None;
    }

    async fn auth_headers(&self) -> Result<HeaderMap> {
        let token = self.get_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        Ok(headers)
    }

    pub async fn get(&self, path: &str, query: &[(String, String)]) -> Result<serde_json::Value> {
        let url = format!("{BASE_URL}{path}");
        let mut headers = self.auth_headers().await?;
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en"));

        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .query(query)
            .send()
            .await
            .context("request failed")?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.invalidate_token().await;
            // Retry once
            let mut headers = self.auth_headers().await?;
            headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en"));
            let resp = self
                .http
                .get(&url)
                .headers(headers)
                .query(query)
                .send()
                .await
                .context("retry request failed")?
                .error_for_status()
                .context("API error on retry")?;
            return resp.json().await.context("failed to parse response");
        }

        let resp = resp.error_for_status().context("API error")?;
        resp.json().await.context("failed to parse response")
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{BASE_URL}{path}");
        let headers = self.auth_headers().await?;

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("API error")?;

        resp.json().await.context("failed to parse response")
    }

    pub async fn put(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{BASE_URL}{path}");
        let headers = self.auth_headers().await?;

        let resp = self
            .http
            .put(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("API error")?;

        resp.json().await.context("failed to parse response")
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{BASE_URL}{path}");
        let headers = self.auth_headers().await?;

        self.http
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("API error")?;

        Ok(())
    }
}
