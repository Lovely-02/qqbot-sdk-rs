//! 频道、子频道与频道成员变更事件。

use super::{Event, EventContext};
use crate::{
    entities::{ChannelHandle, GuildHandle, GuildMemberHandle},
    error::{Result, SdkError},
    models::User,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
            /// 返回频道会话实体。
            ///
            /// # Errors
            /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
            /// 返回子频道会话实体。
            ///
            /// # Errors
            /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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

            /// 返回子频道所属频道实体。
            ///
            /// # Errors
            /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
            /// 返回成员所属频道实体。
            ///
            /// # Errors
            /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
            ///
            /// # Errors
            /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
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
