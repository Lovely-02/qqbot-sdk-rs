use crate::{
    error::Result,
    models::{Channel, Group, Guild, Message},
    segment::Sendable,
};
use std::sync::Arc;

use crate::client::QQBotClient;

/// 群会话实体，通过 `bot.group(id)` 创建。
#[derive(Clone)]
pub struct GroupHandle {
    client: Arc<QQBotClient>,
    pub id: String,
}

impl GroupHandle {
    pub(crate) fn new(client: Arc<QQBotClient>, id: impl Into<String>) -> Self {
        Self {
            client,
            id: id.into(),
        }
    }

    pub async fn send(&self, message: impl Into<Sendable>) -> Result<Message> {
        self.client
            .api()
            .messages()
            .send_group(&self.id, message)
            .await
    }

    pub async fn recall(&self, message_id: &str) -> Result<()> {
        self.client
            .api()
            .messages()
            .delete_group(&self.id, message_id)
            .await
    }

    pub async fn info(&self) -> Result<Group> {
        self.client.api().groups().get(&self.id).await
    }

    pub async fn members(&self) -> Result<serde_json::Value> {
        self.client.api().groups().members(&self.id).await
    }

    pub async fn member(&self, member_openid: &str) -> Result<serde_json::Value> {
        self.client
            .api()
            .groups()
            .member(&self.id, member_openid)
            .await
    }
}

impl std::fmt::Debug for GroupHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GroupHandle")
            .field("id", &self.id)
            .finish()
    }
}

/// 单聊用户会话实体，通过 `bot.user(id)` 创建。
#[derive(Clone)]
pub struct UserHandle {
    client: Arc<QQBotClient>,
    pub id: String,
}

impl UserHandle {
    pub(crate) fn new(client: Arc<QQBotClient>, id: impl Into<String>) -> Self {
        Self {
            client,
            id: id.into(),
        }
    }

    pub async fn send(&self, message: impl Into<Sendable>) -> Result<Message> {
        self.client
            .api()
            .messages()
            .send_c2c(&self.id, message)
            .await
    }

    pub async fn recall(&self, message_id: &str) -> Result<()> {
        self.client
            .api()
            .messages()
            .delete_c2c(&self.id, message_id)
            .await
    }
}

impl std::fmt::Debug for UserHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UserHandle")
            .field("id", &self.id)
            .finish()
    }
}

/// 群成员实体，携带群和成员 OpenID。
#[derive(Clone)]
pub struct GroupMemberHandle {
    client: Arc<QQBotClient>,
    pub group_openid: String,
    pub member_openid: String,
}

impl GroupMemberHandle {
    pub(crate) fn new(
        client: Arc<QQBotClient>,
        group_openid: impl Into<String>,
        member_openid: impl Into<String>,
    ) -> Self {
        Self {
            client,
            group_openid: group_openid.into(),
            member_openid: member_openid.into(),
        }
    }

    /// 获取群成员详情。
    pub async fn info(&self) -> Result<serde_json::Value> {
        self.client
            .api()
            .groups()
            .member(&self.group_openid, &self.member_openid)
            .await
    }
}

impl std::fmt::Debug for GroupMemberHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GroupMemberHandle")
            .field("group_openid", &self.group_openid)
            .field("member_openid", &self.member_openid)
            .finish()
    }
}

/// 频道子频道会话实体，通过 `bot.channel(id)` 创建。
#[derive(Clone)]
pub struct ChannelHandle {
    client: Arc<QQBotClient>,
    pub id: String,
}

impl ChannelHandle {
    pub(crate) fn new(client: Arc<QQBotClient>, id: impl Into<String>) -> Self {
        Self {
            client,
            id: id.into(),
        }
    }

    pub async fn send(&self, message: impl Into<Sendable>) -> Result<Message> {
        self.client
            .api()
            .messages()
            .send_channel(&self.id, message)
            .await
    }

    pub async fn recall(&self, message_id: &str, hide_tip: bool) -> Result<()> {
        self.client
            .api()
            .channels()
            .recall_message(&self.id, message_id, hide_tip)
            .await
    }

    pub async fn info(&self) -> Result<Channel> {
        self.client.api().channels().get(&self.id).await
    }
}

impl std::fmt::Debug for ChannelHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ChannelHandle")
            .field("id", &self.id)
            .finish()
    }
}

/// 频道私信会话实体，通过 `bot.direct(guild_id)` 创建。
#[derive(Clone)]
pub struct DirectHandle {
    client: Arc<QQBotClient>,
    pub guild_id: String,
}

impl DirectHandle {
    pub(crate) fn new(client: Arc<QQBotClient>, guild_id: impl Into<String>) -> Self {
        Self {
            client,
            guild_id: guild_id.into(),
        }
    }

    pub async fn send(&self, message: impl Into<Sendable>) -> Result<Message> {
        self.client
            .api()
            .messages()
            .send_dm(&self.guild_id, message)
            .await
    }

    pub async fn recall(&self, message_id: &str, hide_tip: bool) -> Result<()> {
        self.client
            .api()
            .messages()
            .delete_dm(&self.guild_id, message_id, hide_tip)
            .await
    }
}

impl std::fmt::Debug for DirectHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DirectHandle")
            .field("guild_id", &self.guild_id)
            .finish()
    }
}

/// 频道会话实体，适合在频道消息事件中使用。
#[derive(Clone)]
pub struct GuildHandle {
    client: Arc<QQBotClient>,
    pub id: String,
}

impl GuildHandle {
    pub(crate) fn new(client: Arc<QQBotClient>, id: impl Into<String>) -> Self {
        Self {
            client,
            id: id.into(),
        }
    }

    pub async fn info(&self) -> Result<Guild> {
        self.client.api().guilds().get(&self.id).await
    }

    pub async fn channels(&self) -> Result<Vec<Channel>> {
        self.client.api().guilds().channels(&self.id).await
    }
}

impl std::fmt::Debug for GuildHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GuildHandle")
            .field("id", &self.id)
            .finish()
    }
}

/// 频道成员实体，携带频道 ID 和用户 ID。
#[derive(Clone)]
pub struct GuildMemberHandle {
    client: Arc<QQBotClient>,
    pub guild_id: String,
    pub user_id: String,
}

impl GuildMemberHandle {
    pub(crate) fn new(
        client: Arc<QQBotClient>,
        guild_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            guild_id: guild_id.into(),
            user_id: user_id.into(),
        }
    }

    /// 获取频道成员详情。
    pub async fn info(&self) -> Result<serde_json::Value> {
        self.client
            .api()
            .guilds()
            .member(&self.guild_id, &self.user_id)
            .await
    }
}

impl std::fmt::Debug for GuildMemberHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GuildMemberHandle")
            .field("guild_id", &self.guild_id)
            .field("user_id", &self.user_id)
            .finish()
    }
}
