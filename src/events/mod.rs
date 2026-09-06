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

/// 返回网关事件的中文名称。
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
        "GROUP_MEMBER_REMOVE" => "群成员移除",
        "GROUP_JOIN_REQUEST" => "用户申请加群",
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
        "SUBSCRIBE_MESSAGE_STATUS" => "订阅消息授权状态变更",
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
    entities::{
        ChannelHandle, DirectHandle, GroupHandle, GroupMemberHandle, GuildHandle,
        GuildMemberHandle, UserHandle,
    },
    error::{Result, SdkError},
    models::{
        ArkData, Embed, FriendAuthor, GuildMember, Message, MessageAttachment, MessageElement,
        MessageExtInfo, MessageReference, MessageScene, User,
    },
    segment::Sendable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// READY 网关事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadyEvent {
    pub version: Option<u32>,
    pub session_id: Option<String>,
    pub user: Option<User>,
    pub shard: Option<[u32; 2]>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for ReadyEvent {
    const NAME: &'static str = "READY";
}

/// 子频道消息的常用字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageCreateEvent {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub member: Option<GuildMember>,
    pub timestamp: Option<String>,
    pub seq: Option<u64>,
    pub seq_in_channel: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: Option<u8>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    #[serde(default)]
    pub mentions: Vec<User>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    pub pinned: Option<bool>,
    pub flags: Option<u64>,
    pub message_type: Option<u16>,
    pub message_scene: Option<MessageScene>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
    pub message_reference: Option<MessageReference>,
    pub ext_info: Option<MessageExtInfo>,
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

/// 频道私信消息事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectMessageCreate {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    pub member: Option<GuildMember>,
    pub timestamp: Option<String>,
    pub seq: Option<u64>,
    pub seq_in_channel: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: Option<u8>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    #[serde(default)]
    pub mentions: Vec<User>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    pub pinned: Option<bool>,
    pub flags: Option<u64>,
    pub message_type: Option<u16>,
    pub message_scene: Option<MessageScene>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
    pub message_reference: Option<MessageReference>,
    pub ext_info: Option<MessageExtInfo>,
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
            .or_else(|| self.author.as_ref().and_then(|user| user.id.as_deref()))
    }

    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }
}

/// 开启单聊主动消息接收事件。
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

/// 群成员加入事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMemberAdd {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub member_openid: Option<String>,
    pub user_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupMemberAdd {
    const NAME: &'static str = "GROUP_MEMBER_ADD";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupMemberAdd {
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }

    pub fn user(&self) -> Result<UserHandle> {
        let id = self
            .user_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 user_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(id))
    }

    /// 返回发生变更的群成员实体。
    pub fn member(&self) -> Result<GroupMemberHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let member_openid = self
            .member_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 member_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group_member(group_openid, member_openid))
    }
}

/// 群成员退出事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMemberRemove {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub member_openid: Option<String>,
    pub user_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupMemberRemove {
    const NAME: &'static str = "GROUP_MEMBER_REMOVE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupMemberRemove {
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }

    pub fn user(&self) -> Result<UserHandle> {
        let id = self
            .user_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 user_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(id))
    }

    /// 返回发生变更的群成员实体。
    pub fn member(&self) -> Result<GroupMemberHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let member_openid = self
            .member_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 member_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group_member(group_openid, member_openid))
    }
}

/// 机器人加入群聊事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupAddRobot {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub op_member_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupAddRobot {
    const NAME: &'static str = "GROUP_ADD_ROBOT";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupAddRobot {
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }

    /// 使用事件 ID 发送机器人入群欢迎消息。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let group_openid = self
            .group_openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        context
            .client()
            .api()
            .messages()
            .reply_group(group_openid, message, None, context.event_id.as_deref())
            .await
    }
}

/// 机器人退出群聊事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupDelRobot {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub op_member_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupDelRobot {
    const NAME: &'static str = "GROUP_DEL_ROBOT";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupDelRobot {
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }
}

/// 群聊消息接收开关变更事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMessageSetting {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub op_member_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl GroupMessageSetting {
    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }

    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        Ok(self.context()?.client().group(group_openid))
    }

    /// 群聊消息接收开启后，使用事件 ID 发送一条被动消息。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let context = self.context()?;
        if context.event_name != "GROUP_MSG_RECEIVE" {
            return Err(SdkError::InvalidInput(
                "GROUP_MSG_REJECT 事件不支持被动回复".into(),
            ));
        }
        let group_openid = self
            .group_openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        context
            .client()
            .api()
            .messages()
            .reply_group(group_openid, message, None, context.event_id.as_deref())
            .await
    }
}

impl Event for GroupMessageSetting {
    const NAME: &'static str = "GROUP_MSG_RECEIVE";
    const NAMES: &'static [&'static str] = &["GROUP_MSG_RECEIVE", "GROUP_MSG_REJECT"];

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

/// 用户申请加群事件中的问答项。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewQuestion {
    pub question: Option<String>,
    pub answer: Option<String>,
}

/// 用户申请加群事件中的验证信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JoinVerifyInfo {
    pub method: Option<String>,
    pub verify_message: Option<String>,
    #[serde(default)]
    pub review_qa_list: Vec<ReviewQuestion>,
}

/// 用户申请加群事件中的自动审批信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JoinAutoApproved {
    pub strategy_id: Option<String>,
}

/// 用户申请加群事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupJoinRequest {
    pub group_openid: Option<String>,
    pub join_request_id: Option<String>,
    pub risk_tips: Option<String>,
    pub union_openid: Option<String>,
    pub member_openid: Option<String>,
    pub username: Option<String>,
    pub apply_at: Option<String>,
    pub apply_source: Option<String>,
    pub invited_by: Option<String>,
    pub bot: Option<bool>,
    pub verify_info: Option<JoinVerifyInfo>,
    pub auto_approved: Option<JoinAutoApproved>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupJoinRequest {
    const NAME: &'static str = "GROUP_JOIN_REQUEST";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupJoinRequest {
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("入群申请缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(group_openid))
    }

    /// 返回申请入群的成员实体。
    pub fn member(&self) -> Result<GroupMemberHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("入群申请缺少 group_openid".into()))?;
        let member_openid = self
            .member_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("入群申请缺少 member_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group_member(group_openid, member_openid))
    }
}

/// 关闭单聊主动消息接收事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct C2cMsgReject {
    pub timestamp: Option<i64>,
    pub openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for C2cMsgReject {
    const NAME: &'static str = "C2C_MSG_REJECT";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl C2cMsgReject {
    pub fn user(&self) -> Result<UserHandle> {
        let openid = self
            .openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("单聊事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(openid))
    }
}

/// 订阅消息模板的授权结果。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscribeMessageTemplateResult {
    pub template_id: Option<u32>,
    pub custom_template_id: Option<String>,
    pub op: Option<u8>,
    pub subscribe_id: Option<String>,
    pub subscribe_ts: Option<i64>,
    pub update_ts: Option<i64>,
}

/// 订阅消息授权状态变更事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscribeMessageStatus {
    pub group_openid: Option<String>,
    pub openid: Option<String>,
    #[serde(default)]
    pub result: Vec<SubscribeMessageTemplateResult>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for SubscribeMessageStatus {
    const NAME: &'static str = "SUBSCRIBE_MESSAGE_STATUS";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl SubscribeMessageStatus {
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("订阅事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(group_openid))
    }

    pub fn user(&self) -> Result<UserHandle> {
        let openid = self
            .openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("订阅事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(openid))
    }
}

/// 频道变更事件的公共字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuildEvent {
    pub id: Option<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub owner_id: Option<String>,
    pub member_count: Option<u64>,
    pub max_members: Option<u64>,
    pub description: Option<String>,
    pub joined_at: Option<String>,
    pub op_user_id: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// 子频道变更事件的公共字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelEvent {
    pub id: Option<String>,
    pub guild_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<u32>,
    #[serde(rename = "sub_type")]
    pub sub_type: Option<u32>,
    pub position: Option<u32>,
    pub owner_id: Option<String>,
    pub op_user_id: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

macro_rules! define_guild_event {
    ($type_name:ident, $event_name:literal) => {
        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct $type_name {
            #[serde(flatten)]
            pub data: GuildEvent,
            #[serde(skip)]
            context: Option<EventContext>,
        }

        impl Event for $type_name {
            const NAME: &'static str = $event_name;

            fn attach_context(&mut self, context: EventContext) {
                self.context = Some(context);
            }
        }

        impl $type_name {
            pub fn guild(&self) -> Result<GuildHandle> {
                let id = self
                    .data
                    .id
                    .clone()
                    .ok_or_else(|| SdkError::InvalidInput("频道事件缺少 id".into()))?;
                let context = self
                    .context
                    .as_ref()
                    .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
                Ok(context.client().guild(id))
            }
        }
    };
}

define_guild_event!(GuildCreate, "GUILD_CREATE");
define_guild_event!(GuildUpdate, "GUILD_UPDATE");
define_guild_event!(GuildDelete, "GUILD_DELETE");

macro_rules! define_channel_event {
    ($type_name:ident, $event_name:literal) => {
        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct $type_name {
            #[serde(flatten)]
            pub data: ChannelEvent,
            #[serde(skip)]
            context: Option<EventContext>,
        }

        impl Event for $type_name {
            const NAME: &'static str = $event_name;

            fn attach_context(&mut self, context: EventContext) {
                self.context = Some(context);
            }
        }

        impl $type_name {
            pub fn channel(&self) -> Result<ChannelHandle> {
                let id = self
                    .data
                    .id
                    .clone()
                    .ok_or_else(|| SdkError::InvalidInput("子频道事件缺少 id".into()))?;
                let context = self
                    .context
                    .as_ref()
                    .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
                Ok(context.client().channel(id))
            }

            pub fn guild(&self) -> Result<GuildHandle> {
                let id = self
                    .data
                    .guild_id
                    .clone()
                    .ok_or_else(|| SdkError::InvalidInput("子频道事件缺少 guild_id".into()))?;
                let context = self
                    .context
                    .as_ref()
                    .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
                Ok(context.client().guild(id))
            }
        }
    };
}

define_channel_event!(ChannelCreate, "CHANNEL_CREATE");
define_channel_event!(ChannelUpdate, "CHANNEL_UPDATE");
define_channel_event!(ChannelDelete, "CHANNEL_DELETE");

/// 频道成员变更事件的公共字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuildMemberEvent {
    pub guild_id: Option<String>,
    pub user: Option<User>,
    pub nick: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    pub joined_at: Option<String>,
    pub op_user_id: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

macro_rules! define_guild_member_event {
    ($type_name:ident, $event_name:literal) => {
        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct $type_name {
            #[serde(flatten)]
            pub data: GuildMemberEvent,
            #[serde(skip)]
            context: Option<EventContext>,
        }

        impl Event for $type_name {
            const NAME: &'static str = $event_name;

            fn attach_context(&mut self, context: EventContext) {
                self.context = Some(context);
            }
        }

        impl $type_name {
            /// 返回发生变更的频道会话实体。
            pub fn guild(&self) -> Result<GuildHandle> {
                let id =
                    self.data.guild_id.clone().ok_or_else(|| {
                        SdkError::InvalidInput("频道成员事件缺少 guild_id".into())
                    })?;
                let context = self
                    .context
                    .as_ref()
                    .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
                Ok(context.client().guild(id))
            }

            /// 返回发生变更的频道成员实体。
            pub fn member(&self) -> Result<GuildMemberHandle> {
                let guild_id =
                    self.data.guild_id.clone().ok_or_else(|| {
                        SdkError::InvalidInput("频道成员事件缺少 guild_id".into())
                    })?;
                let user_id = self
                    .data
                    .user
                    .as_ref()
                    .and_then(|user| user.id.clone())
                    .ok_or_else(|| SdkError::InvalidInput("频道成员事件缺少 user.id".into()))?;
                let context = self
                    .context
                    .as_ref()
                    .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
                Ok(context.client().guild_member(guild_id, user_id))
            }
        }
    };
}

define_guild_member_event!(GuildMemberAdd, "GUILD_MEMBER_ADD");
define_guild_member_event!(GuildMemberUpdate, "GUILD_MEMBER_UPDATE");
define_guild_member_event!(GuildMemberRemove, "GUILD_MEMBER_REMOVE");

/// 互动事件中的消息场景信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionMessageScene {
    #[serde(default)]
    pub ext: Vec<String>,
}

/// 互动事件中的授权数据。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionAuthorizeData {
    pub opt_scene: Option<String>,
    pub scope: Option<String>,
}

/// 互动事件解析后的数据。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionResolved {
    pub button_data: Option<String>,
    pub button_id: Option<String>,
    pub user_id: Option<String>,
    pub feature_id: Option<String>,
    pub message_id: Option<String>,
    pub feedback_opt: Option<String>,
    pub checked: Option<u8>,
    pub action: Option<String>,
    pub message_scene: Option<InteractionMessageScene>,
    pub authorize_data: Option<InteractionAuthorizeData>,
}

/// 互动事件内部数据。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionData {
    pub r#type: Option<u8>,
    pub resolved: Option<InteractionResolved>,
}

/// 互动事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionCreate {
    pub id: Option<String>,
    pub r#type: Option<u8>,
    pub scene: Option<String>,
    pub chat_type: Option<u8>,
    pub timestamp: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_openid: Option<String>,
    pub group_openid: Option<String>,
    pub group_member_openid: Option<String>,
    pub data: Option<InteractionData>,
    pub version: Option<u32>,
    pub application_id: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for InteractionCreate {
    const NAME: &'static str = "INTERACTION_CREATE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl InteractionCreate {
    /// 回复需要被动消息的群聊或单聊互动事件。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        let event_id = context
            .event_id
            .as_deref()
            .or(self.id.as_deref())
            .ok_or_else(|| SdkError::InvalidInput("互动事件缺少 event_id".into()))?;
        match self.scene.as_deref() {
            Some("c2c") => {
                let openid = self
                    .user_openid
                    .as_deref()
                    .ok_or_else(|| SdkError::InvalidInput("单聊互动缺少 user_openid".into()))?;
                context
                    .client()
                    .api()
                    .messages()
                    .reply_c2c(openid, message, None, Some(event_id))
                    .await
            }
            Some("group") => {
                let openid = self
                    .group_openid
                    .as_deref()
                    .ok_or_else(|| SdkError::InvalidInput("群聊互动缺少 group_openid".into()))?;
                context
                    .client()
                    .api()
                    .messages()
                    .reply_group(openid, message, None, Some(event_id))
                    .await
            }
            _ => Err(SdkError::InvalidInput(
                "频道互动请使用 interactions().respond()，不能发送被动消息".into(),
            )),
        }
    }

    /// 回复需要确认的按钮或快捷菜单互动。
    pub async fn respond(&self, body: &Value) -> Result<Value> {
        let interaction_id = self
            .id
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("互动事件缺少 interaction_id".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        context
            .client()
            .api()
            .interactions()
            .respond(interaction_id, body)
            .await
    }

    pub fn user(&self) -> Result<UserHandle> {
        let openid = self.user_openid.clone().ok_or_else(|| {
            SdkError::InvalidInput("互动事件仅在 user_openid 存在时才能创建用户会话".into())
        })?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(openid))
    }

    pub fn group(&self) -> Result<GroupHandle> {
        let openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("互动事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(openid))
    }

    pub fn channel(&self) -> Result<ChannelHandle> {
        let id = self
            .channel_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("互动事件缺少 channel_id".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().channel(id))
    }

    pub fn guild(&self) -> Result<GuildHandle> {
        let id = self
            .guild_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("互动事件缺少 guild_id".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().guild(id))
    }
}

/// 频道消息审核结果。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageAudited {
    pub audit_id: Option<String>,
    pub audit_time: Option<String>,
    pub channel_id: Option<String>,
    pub create_time: Option<String>,
    pub guild_id: Option<String>,
    pub message_id: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// 消息审核通过事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageAuditPass {
    #[serde(flatten)]
    pub data: MessageAudited,
}

impl Event for MessageAuditPass {
    const NAME: &'static str = "MESSAGE_AUDIT_PASS";
}

/// 消息审核拒绝事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageAuditReject {
    #[serde(flatten)]
    pub data: MessageAudited,
}

impl Event for MessageAuditReject {
    const NAME: &'static str = "MESSAGE_AUDIT_REJECT";
}

/// 事件回调原始内容。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawEvent {
    pub message: Option<Message>,
    #[serde(flatten)]
    pub data: Value,
}
