//! 好友关系与单聊主动消息开关事件。

use super::{Event, EventContext};
use crate::{
    entities::UserHandle,
    error::{Result, SdkError},
    models::{FriendAuthor, Message},
    segment::Sendable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    /// 返回单聊用户会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

    /// 被动回复该单聊消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    /// 返回好友用户会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

    /// 被动回复该好友消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    /// 返回单聊用户会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
