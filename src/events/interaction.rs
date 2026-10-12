//! 互动事件。

use super::{Event, EventContext};
use crate::{
    entities::{ChannelHandle, GroupHandle, GuildHandle, UserHandle},
    error::{Result, SdkError},
    models::Message,
    segment::Sendable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

    /// 返回互动用户的单聊会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

    /// 返回互动群聊的群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

    /// 返回互动子频道的会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

    /// 返回互动频道实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
