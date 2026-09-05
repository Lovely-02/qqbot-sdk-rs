//! 网关和 Webhook 事件系统。

pub mod handler;
pub mod opcode;
pub mod payload;
pub mod webhook;
pub mod websocket;

pub use handler::{Event, EventContext, EventEnvelope, EventHandler, EventRouter};
pub use opcode::OpCode;
pub use payload::Payload;
pub use webhook::{CallbackValidationResponse, Webhook, WebhookVerifier};
pub use websocket::{GatewayClient, GatewayConfig};

/// 返回网关事件的中文显示名称。
///
/// 事件日志会保留官方事件名，同时附带这个易读的中文名称；未知事件统一返回
/// `未知事件`，这样平台新增事件时仍然可以正常记录和分发。
pub fn event_display_name(event: &str) -> &'static str {
    match event {
        "GUILD_CREATE" => "加入频道",
        "GUILD_UPDATE" => "频道更新",
        "GUILD_DELETE" => "退出频道",
        "CHANNEL_CREATE" => "子频道创建",
        "CHANNEL_UPDATE" => "子频道更新",
        "CHANNEL_DELETE" => "子频道删除",
        "GUILD_MEMBER_ADD" => "成员加入",
        "GUILD_MEMBER_UPDATE" => "成员更新",
        "GUILD_MEMBER_REMOVE" => "成员移除",
        "MESSAGE_CREATE" => "频道消息",
        "MESSAGE_DELETE" => "消息撤回",
        "MESSAGE_REACTION_ADD" => "添加表情表态",
        "MESSAGE_REACTION_REMOVE" => "删除表情表态",
        "DIRECT_MESSAGE_CREATE" => "私信消息",
        "DIRECT_MESSAGE_DELETE" => "私信撤回",
        "GROUP_MEMBER_ADD" => "群成员加入",
        "GROUP_MEMBER_DEL" | "GROUP_MEMBER_REMOVE" => "群成员移除",
        "C2C_MESSAGE_CREATE" => "私聊消息",
        "FRIEND_ADD" => "添加好友",
        "FRIEND_DEL" => "删除好友",
        "C2C_MSG_REJECT" => "关闭主动消息",
        "C2C_MSG_RECEIVE" => "开启主动消息",
        "GROUP_AT_MESSAGE_CREATE" => "群@消息",
        "GROUP_MESSAGE_CREATE" => "群消息",
        "GROUP_ADD_ROBOT" => "加入群聊",
        "GROUP_DEL_ROBOT" => "移出群聊",
        "GROUP_MSG_REJECT" => "群关闭通知",
        "GROUP_MSG_RECEIVE" => "群开启通知",
        "INTERACTION_CREATE" => "互动事件",
        "MESSAGE_AUDIT_PASS" => "审核通过",
        "MESSAGE_AUDIT_REJECT" => "审核拒绝",
        "FORUM_THREAD_CREATE" => "创建主题",
        "FORUM_THREAD_UPDATE" => "更新主题",
        "FORUM_THREAD_DELETE" => "删除主题",
        "FORUM_POST_CREATE" => "创建帖子",
        "FORUM_POST_DELETE" => "删除帖子",
        "FORUM_REPLY_CREATE" => "创建回复",
        "FORUM_REPLY_DELETE" => "删除回复",
        "FORUM_PUBLISH_AUDIT_RESULT" => "审核结果",
        "AUDIO_START" => "音频开始",
        "AUDIO_FINISH" => "音频结束",
        "AUDIO_ON_MIC" => "上麦",
        "AUDIO_OFF_MIC" => "下麦",
        "AT_MESSAGE_CREATE" => "@机器人消息",
        "PUBLIC_MESSAGE_DELETE" => "公开消息删除",
        "READY" => "就绪",
        "RESUMED" => "恢复连接",
        _ => "未知事件",
    }
}

use crate::{
    entities::{ChannelHandle, DirectHandle, GroupHandle, GuildHandle, UserHandle},
    error::{Result, SdkError},
    models::{
        ArkData, Embed, FriendAuthor, Message, MessageAttachment, MessageElement, MessageScene,
        User,
    },
    segment::Sendable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// READY 网关事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadyEvent {
    pub session_id: Option<String>,
    pub user: Option<User>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for ReadyEvent {
    const NAME: &'static str = "READY";
}

/// 子频道消息事件的常用字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageCreateEvent {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub timestamp: Option<String>,
    pub seq: Option<u64>,
    pub seq_in_channel: Option<String>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    #[serde(default)]
    pub mentions: Vec<User>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for MessageCreateEvent {
    const NAME: &'static str = "MESSAGE_CREATE";
    const NAMES: &'static [&'static str] = &["MESSAGE_CREATE", "AT_MESSAGE_CREATE"];

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl MessageCreateEvent {
    /// 返回此消息对应的子频道会话实体。
    pub fn channel(&self) -> Result<ChannelHandle> {
        let channel_id = self
            .channel_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道消息缺少 channel_id".into()))?;
        Ok(self.context()?.client().channel(channel_id))
    }

    /// 返回此消息对应的频道会话实体。
    pub fn guild(&self) -> Result<GuildHandle> {
        let guild_id = self
            .guild_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道消息缺少 guild_id".into()))?;
        Ok(self.context()?.client().guild(guild_id))
    }

    /// 回复此频道消息。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let channel_id = self
            .channel_id
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("频道消息缺少 channel_id".into()))?;
        let context = self.context()?;
        context
            .client()
            .api()
            .messages()
            .reply_channel(
                channel_id,
                message,
                self.id.as_deref(),
                context.event_id.as_deref(),
            )
            .await
    }

    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }
}

/// 频道私信消息事件（`DIRECT_MESSAGE_CREATE`）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectMessageCreate {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub timestamp: Option<String>,
    pub seq: Option<u64>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for DirectMessageCreate {
    const NAME: &'static str = "DIRECT_MESSAGE_CREATE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl DirectMessageCreate {
    pub fn direct(&self) -> Result<DirectHandle> {
        let guild_id = self
            .guild_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道私信缺少 guild_id".into()))?;
        Ok(self.context()?.client().direct(guild_id))
    }

    pub fn channel(&self) -> Result<ChannelHandle> {
        let channel_id = self
            .channel_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道私信缺少 channel_id".into()))?;
        Ok(self.context()?.client().channel(channel_id))
    }

    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let guild_id = self
            .guild_id
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("频道私信缺少 guild_id".into()))?;
        let context = self.context()?;
        context
            .client()
            .api()
            .messages()
            .reply_dm(
                guild_id,
                message,
                self.id.as_deref(),
                context.event_id.as_deref(),
            )
            .await
    }

    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }
}

/// 单聊消息事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct C2cMessageReceive {
    pub id: Option<String>,
    pub user_openid: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub timestamp: Option<String>,
    pub message_type: Option<u16>,
    pub message_scene: Option<MessageScene>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for C2cMessageReceive {
    const NAME: &'static str = "C2C_MESSAGE_CREATE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl C2cMessageReceive {
    pub fn user(&self) -> Result<UserHandle> {
        let user_openid = self.user_openid().ok_or_else(|| {
            SdkError::InvalidInput("单聊事件缺少 user_openid 或 author.id".into())
        })?;
        Ok(self.context()?.client().user(user_openid))
    }

    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let user_openid = self.user_openid().ok_or_else(|| {
            SdkError::InvalidInput("单聊事件缺少 user_openid 或 author.id".into())
        })?;
        let context = self.context()?;
        context
            .client()
            .api()
            .messages()
            .reply_c2c(
                user_openid,
                message,
                self.id.as_deref(),
                context.event_id.as_deref(),
            )
            .await
    }

    fn user_openid(&self) -> Option<&str> {
        self.user_openid
            .as_deref()
            .or_else(|| {
                self.author
                    .as_ref()
                    .and_then(|user| user.user_openid.as_deref())
            })
            .or_else(|| self.author.as_ref().and_then(|user| user.openid.as_deref()))
            .or_else(|| self.author.as_ref().and_then(|user| user.id.as_deref()))
    }

    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }
}

/// 用户开启单聊主动消息接收时触发的事件（`C2C_MSG_RECEIVE`）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct C2cMsgReceive {
    pub openid: Option<String>,
    pub timestamp: Option<i64>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for C2cMsgReceive {
    const NAME: &'static str = "C2C_MSG_RECEIVE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl C2cMsgReceive {
    pub fn user(&self) -> Result<UserHandle> {
        let openid = self
            .openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(openid))
    }

    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let openid = self
            .openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        context
            .client()
            .api()
            .messages()
            .reply_c2c(openid, message, None, context.event_id.as_deref())
            .await
    }
}

/// 群聊中 @ 机器人的消息事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupAtMessageCreate {
    pub id: Option<String>,
    pub group_openid: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub timestamp: Option<String>,
    pub message_type: Option<u16>,
    pub message_scene: Option<MessageScene>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    pub mentions: Vec<User>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupAtMessageCreate {
    const NAME: &'static str = "GROUP_AT_MESSAGE_CREATE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupAtMessageCreate {
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群消息缺少 group_openid".into()))?;
        Ok(self.context()?.client().group(group_openid))
    }

    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let group_openid = self
            .group_openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("群消息缺少 group_openid".into()))?;
        let context = self.context()?;
        context
            .client()
            .api()
            .messages()
            .reply_group(
                group_openid,
                message,
                self.id.as_deref(),
                context.event_id.as_deref(),
            )
            .await
    }

    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }
}

/// 群聊普通消息事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMessageCreate {
    pub id: Option<String>,
    pub group_openid: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub timestamp: Option<String>,
    pub message_type: Option<u16>,
    pub message_scene: Option<MessageScene>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    pub mentions: Vec<User>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupMessageCreate {
    const NAME: &'static str = "GROUP_MESSAGE_CREATE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupMessageCreate {
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群消息缺少 group_openid".into()))?;
        Ok(self.context()?.client().group(group_openid))
    }

    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let group_openid = self
            .group_openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("群消息缺少 group_openid".into()))?;
        let context = self.context()?;
        context
            .client()
            .api()
            .messages()
            .reply_group(
                group_openid,
                message,
                self.id.as_deref(),
                context.event_id.as_deref(),
            )
            .await
    }

    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }
}

/// 好友添加事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FriendAdd {
    pub openid: Option<String>,
    pub timestamp: Option<i64>,
    pub scene: Option<u16>,
    pub scene_param: Option<String>,
    pub author: Option<FriendAuthor>,
    pub short_code: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for FriendAdd {
    const NAME: &'static str = "FRIEND_ADD";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl FriendAdd {
    pub fn user(&self) -> Result<UserHandle> {
        let openid = self
            .openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("好友事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(openid))
    }

    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let openid = self
            .openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("好友事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        context
            .client()
            .api()
            .messages()
            .reply_c2c(openid, message, None, context.event_id.as_deref())
            .await
    }
}

/// 用户删除机器人好友事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FriendDelete {
    pub openid: Option<String>,
    pub timestamp: Option<i64>,
    pub author: Option<FriendAuthor>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for FriendDelete {
    const NAME: &'static str = "FRIEND_DEL";
}

/// 互动事件，保留未知字段以兼容平台扩展。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionCreate {
    pub id: Option<String>,
    pub application_command: Option<Value>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for InteractionCreate {
    const NAME: &'static str = "INTERACTION_CREATE";
}

/// 事件回调的原始消息内容。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawEvent {
    pub message: Option<Message>,
    #[serde(flatten)]
    pub data: Value,
}
