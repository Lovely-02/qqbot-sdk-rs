use crate::{
    error::{Result, SdkError},
    logging::API_ERROR_LOG_TARGET,
    models::{AccessTokenResponse, ApiErrorBody},
};
use reqwest::Client;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{debug, error};

#[derive(Debug, Clone)]
struct CachedToken {
    value: String,
    expires_at: Instant,
}

/// 管理 AccessToken，并自动缓存刷新。
#[derive(Clone)]
pub struct AccessTokenManager {
    app_id: Arc<str>,
    app_secret: Arc<str>,
    base_url: Arc<str>,
    http: Client,
    cached: Arc<Mutex<Option<CachedToken>>>,
}

impl AccessTokenManager {
    /// 创建鉴权管理器。
    pub fn new(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        base_url: impl Into<String>,
        http: Client,
    ) -> Self {
        Self {
            app_id: Arc::from(app_id.into()),
            app_secret: Arc::from(app_secret.into()),
            base_url: Arc::from(base_url.into()),
            http,
            cached: Arc::new(Mutex::new(None)),
        }
    }

    /// 返回 AppID。
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// 获取有效的 AccessToken。
    pub async fn token(&self) -> Result<String> {
        {
            let cached = self.cached.lock().await;
            if let Some(token) = cached
                .as_ref()
                .filter(|token| token.expires_at > Instant::now())
            {
                return Ok(token.value.clone());
            }
        }

        let mut cached = self.cached.lock().await;
        if let Some(token) = cached
            .as_ref()
            .filter(|token| token.expires_at > Instant::now())
        {
            return Ok(token.value.clone());
        }

        let url = format!(
            "{}/app/getAppAccessToken",
            self.base_url.trim_end_matches('/')
        );
        debug!(%url, "刷新 QQ AccessToken");
        let response = match self
            .http
            .post(url)
            .json(&json!({
                "appId": self.app_id.as_ref(),
                "clientSecret": self.app_secret.as_ref(),
            }))
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                error!(target: API_ERROR_LOG_TARGET, "{error}");
                return Err(SdkError::from(error));
            }
        };
        let status = response.status();
        let bytes = response.bytes().await.map_err(|error| {
            error!(target: API_ERROR_LOG_TARGET, "{error}");
            SdkError::from(error)
        })?;
        if !status.is_success() {
            let body = serde_json::from_slice::<ApiErrorBody>(&bytes).unwrap_or_default();
            let error = SdkError::Api {
                status: status.as_u16(),
                code: body.err_code.or(body.code).unwrap_or(-1),
                message: body
                    .message
                    .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned()),
            };
            error!(target: API_ERROR_LOG_TARGET, "{}", String::from_utf8_lossy(&bytes));
            return Err(error);
        }
        let token: AccessTokenResponse = serde_json::from_slice(&bytes).map_err(|error| {
            error!(target: API_ERROR_LOG_TARGET, "{}", String::from_utf8_lossy(&bytes));
            SdkError::from(error)
        })?;
        if token.access_token.is_empty() {
            return Err(SdkError::Auth("AccessToken 响应为空".into()));
        }
        let lifetime = Duration::from_secs(token.expires_in.max(60));
        let refresh_margin = Duration::from_secs(60).min(lifetime / 2);
        *cached = Some(CachedToken {
            value: token.access_token.clone(),
            expires_at: Instant::now() + lifetime - refresh_margin,
        });
        Ok(token.access_token)
    }

    /// 清除 Token 缓存。
    pub async fn invalidate(&self) {
        *self.cached.lock().await = None;
    }
}
