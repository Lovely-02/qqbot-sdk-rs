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

    /// 获取可访问频道列表，支持 `before`、`after`、`limit` 分页。
    pub async fn guilds(
        &self,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<Guild>> {
        let query = optional_query([
            ("before", before.map(str::to_owned)),
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

    /// 创建频道私信（返回 `guild_id` 和 `channel_id`）。
    pub async fn create_dm(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(Method::POST, "/users/@me/dms", Some(body))
            .await
    }
}
