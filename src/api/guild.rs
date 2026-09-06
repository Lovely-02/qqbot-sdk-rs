use super::{optional_query, segment};
use crate::{
    client::QQBotClient,
    error::Result,
    models::{Channel, Guild},
};
use reqwest::Method;
use serde_json::Value;

/// 频道和子频道管理 API。
pub struct GuildApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> GuildApi<'a> {
    /// 获取频道详情。
    pub async fn get(&self, guild_id: &str) -> Result<Guild> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}", segment(guild_id)),
                Option::<&Value>::None,
            )
            .await
    }

    /// 获取频道下所有子频道。
    pub async fn channels(&self, guild_id: &str) -> Result<Vec<Channel>> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}/channels", segment(guild_id)),
                Option::<&Value>::None,
            )
            .await
    }

    /// 创建子频道。请求字段按官方文档传入。
    pub async fn create_channel(&self, guild_id: &str, body: &Value) -> Result<Channel> {
        self.client
            .request_json(
                Method::POST,
                &format!("/guilds/{}/channels", segment(guild_id)),
                Some(body),
            )
            .await
    }

    /// 发布频道公告。
    pub async fn create_announcement(&self, guild_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/guilds/{}/announces", segment(guild_id)),
                Some(body),
            )
            .await
    }
    /// 删除频道公告。
    pub async fn delete_announcement(&self, guild_id: &str, message_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/guilds/{}/announces/{}",
                    segment(guild_id),
                    segment(message_id)
                ),
                None,
            )
            .await
    }

    /// 获取频道 API 权限。
    pub async fn api_permissions(&self, guild_id: &str) -> Result<Value> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}/api_permission", segment(guild_id)),
                Option::<&Value>::None,
            )
            .await
    }
    /// 申请频道 API 权限。
    pub async fn request_api_permission(&self, guild_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/guilds/{}/api_permission/demand", segment(guild_id)),
                Some(body),
            )
            .await
    }

    /// 获取频道角色列表。
    pub async fn roles(&self, guild_id: &str) -> Result<Value> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}/roles", segment(guild_id)),
                Option::<&Value>::None,
            )
            .await
    }
    /// 创建频道角色。
    pub async fn create_role(&self, guild_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/guilds/{}/roles", segment(guild_id)),
                Some(body),
            )
            .await
    }
    /// 更新频道角色。
    pub async fn update_role(&self, guild_id: &str, role_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PATCH,
                &format!("/guilds/{}/roles/{}", segment(guild_id), segment(role_id)),
                Some(body),
            )
            .await
    }
    /// 删除频道角色。
    pub async fn delete_role(&self, guild_id: &str, role_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/guilds/{}/roles/{}", segment(guild_id), segment(role_id)),
                None,
            )
            .await
    }
    /// 给频道成员授予角色。
    pub async fn add_member_role(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!(
                    "/guilds/{}/members/{}/roles/{}",
                    segment(guild_id),
                    segment(user_id),
                    segment(role_id)
                ),
                Some(body),
            )
            .await
    }
    /// 移除频道成员角色。
    pub async fn remove_member_role(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/guilds/{}/members/{}/roles/{}",
                    segment(guild_id),
                    segment(user_id),
                    segment(role_id)
                ),
                None,
            )
            .await
    }

    /// 移除频道成员角色并传入请求体。
    pub async fn remove_member_role_with_body(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
        body: &Value,
    ) -> Result<()> {
        self.client
            .request_empty(
                Method::DELETE,
                &format!(
                    "/guilds/{}/members/{}/roles/{}",
                    segment(guild_id),
                    segment(user_id),
                    segment(role_id)
                ),
                Some(body),
            )
            .await
    }
    /// 获取频道成员列表。
    pub async fn members(&self, guild_id: &str) -> Result<Value> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}/members", segment(guild_id)),
                Option::<&Value>::None,
            )
            .await
    }

    /// 获取频道成员列表并传入官方分页参数。
    pub async fn members_page(
        &self,
        guild_id: &str,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Value> {
        let query = optional_query([
            ("after", after.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                &format!("/guilds/{}/members", segment(guild_id)),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 获取频道成员详情。
    pub async fn member(&self, guild_id: &str, user_id: &str) -> Result<Value> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}/members/{}", segment(guild_id), segment(user_id)),
                Option::<&Value>::None,
            )
            .await
    }
    /// 移除频道成员。
    pub async fn remove_member(&self, guild_id: &str, user_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/guilds/{}/members/{}", segment(guild_id), segment(user_id)),
                None,
            )
            .await
    }

    /// 移除频道成员并传入踢出选项。
    pub async fn remove_member_with_options(
        &self,
        guild_id: &str,
        user_id: &str,
        body: &Value,
    ) -> Result<()> {
        self.client
            .request_empty(
                Method::DELETE,
                &format!("/guilds/{}/members/{}", segment(guild_id), segment(user_id)),
                Some(body),
            )
            .await
    }
    /// 获取角色下的成员列表。
    pub async fn role_members(&self, guild_id: &str, role_id: &str) -> Result<Value> {
        self.client
            .request_json(
                Method::GET,
                &format!(
                    "/guilds/{}/roles/{}/members",
                    segment(guild_id),
                    segment(role_id)
                ),
                Option::<&Value>::None,
            )
            .await
    }

    /// 获取指定角色下的成员列表并传入官方分页参数。
    pub async fn role_members_page(
        &self,
        guild_id: &str,
        role_id: &str,
        start_index: Option<u16>,
        limit: Option<u16>,
    ) -> Result<Value> {
        let query = optional_query([
            ("start_index", start_index.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                &format!(
                    "/guilds/{}/roles/{}/members",
                    segment(guild_id),
                    segment(role_id)
                ),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 设置单个成员禁言。
    pub async fn mute_member(&self, guild_id: &str, user_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PATCH,
                &format!(
                    "/guilds/{}/members/{}/mute",
                    segment(guild_id),
                    segment(user_id)
                ),
                Some(body),
            )
            .await
    }
    /// 设置频道全员禁言。
    pub async fn mute_guild(&self, guild_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PATCH,
                &format!("/guilds/{}/mute", segment(guild_id)),
                Some(body),
            )
            .await
    }
    /// 设置频道多成员禁言。
    pub async fn mute_members(&self, guild_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PATCH,
                &format!("/guilds/{}/mute", segment(guild_id)),
                Some(body),
            )
            .await
    }
}
