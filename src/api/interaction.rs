use super::segment;
use crate::{client::QQBotClient, error::Result};
use reqwest::Method;
use serde_json::Value;

/// 互动事件回调 API。
pub struct InteractionApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> InteractionApi<'a> {
    /// 响应按钮、命令等互动事件。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn respond(&self, interaction_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!("/interactions/{}", segment(interaction_id)),
                Some(body),
            )
            .await
    }
}
