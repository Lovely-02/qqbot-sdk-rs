use crate::{
    client::QQBotClient,
    error::Result,
    models::{Guild, User},
};
use reqwest::Method;
use serde_json::Value;

use super::optional_query;

/// 机器人自身信息和可访问频道 API。
pub struct BotApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> BotApi<'a> {
    /// 获取当前机器人用户信息。
    pub async fn me(&self) -> Result<User> {
        self.client
            .request_json(
                Method::GET,
                "/users/@me",
                Option::<&serde_json::Value>::None,
            )
            .await
    }

    /// 获取当前机器人可访问的频道列表。
    pub async fn guilds(&self, after: Option<&str>, limit: Option<u16>) -> Result<Vec<Guild>> {
        let query = optional_query([
            ("after", after.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                "/users/@me/guilds",
                Option::<&serde_json::Value>::None,
                &query,
            )
            .await
    }

    /// 获取当前机器人可访问的频道列表（`guilds` 的语义别名）。
    pub async fn list_guilds(&self, after: Option<&str>, limit: Option<u16>) -> Result<Vec<Guild>> {
        self.guilds(after, limit).await
    }

    /// 创建频道私信（返回 `guild_id` 和 `channel_id`）。
    pub async fn create_dm(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(Method::POST, "/users/@me/dms", Some(body))
            .await
    }
}
