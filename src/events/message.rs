//! 消息类强类型事件（频道、私信、单聊、群聊消息）。

use super::{Event, EventContext};
use crate::{
    entities::{ChannelHandle, DirectHandle, GroupHandle, GuildHandle, UserHandle},
    error::{Result, SdkError},
    models::{
        ArkData, Embed, GuildMember, Message, MessageAttachment, MessageElement, MessageExtInfo,
        MessageReference, MessageScene, User,
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
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn channel(&self) -> Result<ChannelHandle> {
        let channel_id = self
            .channel_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道消息缺少 channel_id".into()))?;
        Ok(self.context()?.client().channel(channel_id))
    }

    /// 返回此消息对应的频道会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn guild(&self) -> Result<GuildHandle> {
        let guild_id = self
            .guild_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道消息缺少 guild_id".into()))?;
        Ok(self.context()?.client().guild(guild_id))
    }

    /// 回复此频道消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    /// 返回频道私信会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn direct(&self) -> Result<DirectHandle> {
        let guild_id = self
            .guild_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道私信缺少 guild_id".into()))?;
        Ok(self.context()?.client().direct(guild_id))
    }

    /// 返回频道私信所在子频道实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn channel(&self) -> Result<ChannelHandle> {
        let channel_id = self
            .channel_id
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("频道私信缺少 channel_id".into()))?;
        Ok(self.context()?.client().channel(channel_id))
    }

    /// 被动回复该私信。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    /// 返回单聊用户会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn user(&self) -> Result<UserHandle> {
        let user_openid = self.user_openid().ok_or_else(|| {
            SdkError::InvalidInput("单聊事件缺少 user_openid 或 author.id".into())
        })?;
        Ok(self.context()?.client().user(user_openid))
    }

    /// 被动回复该单聊消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群消息缺少 group_openid".into()))?;
        Ok(self.context()?.client().group(group_openid))
    }

    /// 被动回复该群消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群消息缺少 group_openid".into()))?;
        Ok(self.context()?.client().group(group_openid))
    }

    /// 被动回复该群消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
