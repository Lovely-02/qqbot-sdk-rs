use super::{optional_query, segment};
use crate::{
    client::QQBotClient,
    error::Result,
    models::{Channel, Message},
};
use reqwest::Method;
use serde_json::Value;

/// 子频道相关 API。
pub struct ChannelApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> ChannelApi<'a> {
    /// 获取子频道信息。
    pub async fn get(&self, channel_id: &str) -> Result<Channel> {
        self.client
            .request_json(
                Method::GET,
                &format!("/channels/{}", segment(channel_id)),
                Option::<&serde_json::Value>::None,
            )
            .await
    }

    /// 获取频道下的子频道列表。
    pub async fn list(&self, guild_id: &str) -> Result<Vec<Channel>> {
        self.client
            .request_json(
                Method::GET,
                &format!("/guilds/{}/channels", segment(guild_id)),
                Option::<&serde_json::Value>::None,
            )
            .await
    }

    /// 更新子频道。
    pub async fn update(&self, channel_id: &str, body: &Value) -> Result<Channel> {
        self.client
            .request_json(
                Method::PATCH,
                &format!("/channels/{}", segment(channel_id)),
                Some(body),
            )
            .await
    }
    /// 删除子频道。
    pub async fn delete(&self, channel_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/channels/{}", segment(channel_id)),
                None,
            )
            .await
    }

    /// 获取子频道中的一条消息。
    pub async fn get_message(&self, channel_id: &str, message_id: &str) -> Result<Message> {
        self.client
            .request_json(
                Method::GET,
                &format!(
                    "/channels/{}/messages/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                Option::<&Value>::None,
            )
            .await
    }

    /// 修改频道中的 Markdown/键盘消息。
    pub async fn update_message(
        &self,
        channel_id: &str,
        message_id: &str,
        body: &Value,
    ) -> Result<Message> {
        self.client
            .request_json(
                Method::PATCH,
                &format!(
                    "/channels/{}/messages/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                Some(body),
            )
            .await
    }

    /// 获取语音子频道中的在线成员。
    pub async fn voice_members(&self, channel_id: &str) -> Result<Value> {
        self.get_value(&format!("/channels/{}/voice/members", segment(channel_id)))
            .await
    }

    /// 发布子频道公告。
    pub async fn create_announcement(&self, channel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/channels/{}/announces", segment(channel_id)),
                Some(body),
            )
            .await
    }
    /// 删除子频道公告。
    pub async fn delete_announcement(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/channels/{}/announces/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                None,
            )
            .await
    }
    /// 音频控制。
    pub async fn audio_control(&self, channel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/channels/{}/audio", segment(channel_id)),
                Some(body),
            )
            .await
    }
    /// 开启麦克风。
    pub async fn enable_mic(&self, channel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!("/channels/{}/mic", segment(channel_id)),
                Some(body),
            )
            .await
    }
    /// 关闭麦克风。
    pub async fn disable_mic(&self, channel_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/channels/{}/mic", segment(channel_id)),
                None,
            )
            .await
    }
    /// 获取帖子列表。
    pub async fn list_threads(&self, channel_id: &str) -> Result<Value> {
        self.get_value(&format!("/channels/{}/threads", segment(channel_id)))
            .await
    }
    /// 获取单个帖子。
    pub async fn get_thread(&self, channel_id: &str, thread_id: &str) -> Result<Value> {
        self.get_value(&format!(
            "/channels/{}/threads/{}",
            segment(channel_id),
            segment(thread_id)
        ))
        .await
    }
    /// 创建帖子。
    pub async fn create_thread(&self, channel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!("/channels/{}/threads", segment(channel_id)),
                Some(body),
            )
            .await
    }
    /// 兼容旧调用的帖子写入别名。
    ///
    /// QQ 官方没有独立的帖子更新接口；此方法与 [`Self::create_thread`] 一样使用
    /// `PUT /channels/{channel_id}/threads`，新代码请直接调用 `create_thread`。
    #[deprecated(note = "QQ API 没有独立的帖子更新接口，请使用 create_thread")]
    pub async fn update_thread(&self, channel_id: &str, body: &Value) -> Result<Value> {
        self.create_thread(channel_id, body).await
    }

    /// 设置消息为精华/置顶；官方接口请求体为空对象。
    pub async fn pin(&self, channel_id: &str, message_id: &str) -> Result<Value> {
        let body = serde_json::json!({});
        self.client
            .request_json(
                Method::PUT,
                &format!(
                    "/channels/{}/pins/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                Some(&body),
            )
            .await
    }
    /// 删除帖子。
    pub async fn delete_thread(&self, channel_id: &str, thread_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/channels/{}/threads/{}",
                    segment(channel_id),
                    segment(thread_id)
                ),
                None,
            )
            .await
    }
    /// 获取精华/置顶消息。
    pub async fn list_pins(&self, channel_id: &str) -> Result<Value> {
        self.get_value(&format!("/channels/{}/pins", segment(channel_id)))
            .await
    }
    /// 设置消息为精华/置顶。
    pub async fn pin_message(
        &self,
        channel_id: &str,
        message_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!(
                    "/channels/{}/pins/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                Some(body),
            )
            .await
    }
    /// 删除消息精华/置顶状态。
    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/channels/{}/pins/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                None,
            )
            .await
    }
    /// 创建日程。
    pub async fn create_schedule(&self, channel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/channels/{}/schedules", segment(channel_id)),
                Some(body),
            )
            .await
    }
    /// 获取日程列表。
    pub async fn list_schedules(&self, channel_id: &str) -> Result<Value> {
        self.get_value(&format!("/channels/{}/schedules", segment(channel_id)))
            .await
    }

    /// 获取日程列表，可选 `since` 时间戳过滤结束时间。
    pub async fn list_schedules_since(
        &self,
        channel_id: &str,
        since: Option<&str>,
    ) -> Result<Value> {
        let query = optional_query([("since", since.map(str::to_owned))]);
        self.client
            .request_json_query(
                Method::GET,
                &format!("/channels/{}/schedules", segment(channel_id)),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 获取日程详情。
    pub async fn get_schedule(&self, channel_id: &str, schedule_id: &str) -> Result<Value> {
        self.get_value(&format!(
            "/channels/{}/schedules/{}",
            segment(channel_id),
            segment(schedule_id)
        ))
        .await
    }
    /// 更新日程。
    pub async fn update_schedule(
        &self,
        channel_id: &str,
        schedule_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PATCH,
                &format!(
                    "/channels/{}/schedules/{}",
                    segment(channel_id),
                    segment(schedule_id)
                ),
                Some(body),
            )
            .await
    }
    /// 删除日程。
    pub async fn delete_schedule(&self, channel_id: &str, schedule_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/channels/{}/schedules/{}",
                    segment(channel_id),
                    segment(schedule_id)
                ),
                None,
            )
            .await
    }
    /// 添加消息表情回应。
    pub async fn add_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        reaction_type: &str,
        reaction_id: &str,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!(
                    "/channels/{}/messages/{}/reactions/{}/{}",
                    segment(channel_id),
                    segment(message_id),
                    segment(reaction_type),
                    segment(reaction_id)
                ),
                Option::<&Value>::None,
            )
            .await
    }

    /// 删除消息表情回应。
    pub async fn remove_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        reaction_type: &str,
        reaction_id: &str,
    ) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!(
                    "/channels/{}/messages/{}/reactions/{}/{}",
                    segment(channel_id),
                    segment(message_id),
                    segment(reaction_type),
                    segment(reaction_id)
                ),
                None,
            )
            .await
    }

    /// 查询消息表情回应用户列表。
    pub async fn list_reactions(
        &self,
        channel_id: &str,
        message_id: &str,
        reaction_type: &str,
        reaction_id: &str,
        cookie: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Value> {
        let query = optional_query([
            ("cookie", cookie.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                &format!(
                    "/channels/{}/messages/{}/reactions/{}/{}",
                    segment(channel_id),
                    segment(message_id),
                    segment(reaction_type),
                    segment(reaction_id)
                ),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 撤回消息，`hide_tip` 对应官方 `hidetip` 查询参数。
    pub async fn recall_message(
        &self,
        channel_id: &str,
        message_id: &str,
        hide_tip: bool,
    ) -> Result<()> {
        let value = if hide_tip { "true" } else { "false" };
        self.client
            .request_json_query::<Value, Value, _>(
                Method::DELETE,
                &format!(
                    "/channels/{}/messages/{}",
                    segment(channel_id),
                    segment(message_id)
                ),
                None,
                &[("hidetip", value)],
            )
            .await
            .map(|_| ())
    }
    /// 获取频道消息频率设置。
    pub async fn message_setting(&self, guild_id: &str) -> Result<Value> {
        self.get_value(&format!("/guilds/{}/message/setting", segment(guild_id)))
            .await
    }
    /// 获取频道在线人数。
    pub async fn online_numbers(&self, channel_id: &str) -> Result<Value> {
        self.get_value(&format!("/channels/{}/online_nums", segment(channel_id)))
            .await
    }

    /// 获取频道成员在子频道上的 API 权限。
    pub async fn member_permissions(&self, channel_id: &str, user_id: &str) -> Result<Value> {
        self.get_value(&format!(
            "/channels/{}/members/{}/permissions",
            segment(channel_id),
            segment(user_id)
        ))
        .await
    }
    /// 获取频道角色在子频道上的 API 权限。
    pub async fn role_permissions(&self, channel_id: &str, role_id: &str) -> Result<Value> {
        self.get_value(&format!(
            "/channels/{}/roles/{}/permissions",
            segment(channel_id),
            segment(role_id)
        ))
        .await
    }
    /// 更新频道成员 API 权限。
    pub async fn update_member_permissions(
        &self,
        channel_id: &str,
        user_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!(
                    "/channels/{}/members/{}/permissions",
                    segment(channel_id),
                    segment(user_id)
                ),
                Some(body),
            )
            .await
    }
    /// 更新频道角色 API 权限。
    pub async fn update_role_permissions(
        &self,
        channel_id: &str,
        role_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!(
                    "/channels/{}/roles/{}/permissions",
                    segment(channel_id),
                    segment(role_id)
                ),
                Some(body),
            )
            .await
    }

    async fn get_value(&self, path: &str) -> Result<Value> {
        self.client
            .request_json(Method::GET, path, Option::<&Value>::None)
            .await
    }
}
