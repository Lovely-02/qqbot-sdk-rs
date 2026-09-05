use crate::{client::QQBotClient, error::Result};
use reqwest::Method;
use serde_json::Value;

/// 自定义菜单 API。
pub struct MenuApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> MenuApi<'a> {
    /// 获取当前机器人菜单。
    pub async fn get(&self) -> Result<Value> {
        self.client
            .request_json(Method::GET, "/v2/menu", Option::<&Value>::None)
            .await
    }
    /// 创建或覆盖机器人菜单。
    pub async fn put(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(Method::PUT, "/v2/menu", Some(body))
            .await
    }
}
