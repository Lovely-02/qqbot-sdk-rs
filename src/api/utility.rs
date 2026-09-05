use crate::{client::QQBotClient, error::Result};
use reqwest::Method;
use serde_json::Value;

/// 通用工具 API。
pub struct UtilityApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> UtilityApi<'a> {
    /// 生成 QQ 资源跳转链接。
    pub async fn generate_url_link(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(Method::POST, "/v2/generate_url_link", Some(body))
            .await
    }
}
