use crate::{client::QQBotClient, error::Result};
use reqwest::Method;
use serde_json::Value;

/// 通用工具 API。
pub struct UtilityApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> UtilityApi<'a> {
    /// 生成 QQ 资源跳转链接。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn generate_url_link(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(Method::POST, "/v2/generate_url_link", Some(body))
            .await
    }
}
