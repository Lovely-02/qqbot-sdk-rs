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

impl CachedToken {
    fn new(value: String, expires_in: u64, requested_at: Instant) -> Result<Self> {
        if expires_in == 0 {
            return Err(SdkError::Auth("AccessToken 有效期必须大于 0".into()));
        }
        let lifetime = Duration::from_secs(expires_in);
        let refresh_margin = Duration::from_secs(60).min(lifetime / 2);
        let expires_at = requested_at
            .checked_add(lifetime - refresh_margin)
            .ok_or_else(|| SdkError::Auth("AccessToken 有效期超出可表示范围".into()))?;
        if expires_at <= Instant::now() {
            return Err(SdkError::Auth(
                "AccessToken 在请求期间已到达刷新时间".into(),
            ));
        }
        Ok(Self { value, expires_at })
    }
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
    ///
    /// # Errors
    /// 请求失败、响应解析失败、Token 为空、有效期无效或已到刷新时间时返回 `SdkError`。
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
        // 从请求开始计时，避免网络耗时延长本地缓存的有效期。
        let requested_at = Instant::now();
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
        let trace_id = response
            .headers()
            .get("X-Tps-trace-ID")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = response.bytes().await.map_err(|error| {
            error!(target: API_ERROR_LOG_TARGET, "{error}");
            SdkError::from(error)
        })?;
        let body = serde_json::from_slice::<ApiErrorBody>(&bytes).unwrap_or_default();
        let error_code = body.nonzero_code();
        if !status.is_success() || error_code.is_some() {
            let error = SdkError::Api {
                status: status.as_u16(),
                code: error_code.or(body.err_code).or(body.code).unwrap_or(-1),
                message: body
                    .message
                    .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned()),
                trace_id: body.trace_id.or(trace_id),
                data: body.data,
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
        *cached = Some(CachedToken::new(
            token.access_token.clone(),
            token.expires_in,
            requested_at,
        )?);
        Ok(token.access_token)
    }

    /// 清除 Token 缓存。
    pub async fn invalidate(&self) {
        *self.cached.lock().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_deadline_respects_short_and_normal_lifetimes() {
        let started = Instant::now();
        for (seconds, cached_for) in [
            (1, Duration::from_millis(500)),
            (7200, Duration::from_secs(7140)),
        ] {
            let token = CachedToken::new("token".into(), seconds, started).unwrap();
            assert_eq!(token.expires_at.duration_since(started), cached_for);
        }
    }

    #[test]
    fn invalid_or_elapsed_lifetimes_return_auth_errors() {
        for (seconds, started) in [
            (0, Instant::now()),
            (u64::MAX, Instant::now()),
            (1, Instant::now() - Duration::from_secs(2)),
        ] {
            assert!(matches!(
                CachedToken::new("token".into(), seconds, started),
                Err(SdkError::Auth(_))
            ));
        }
    }
}
