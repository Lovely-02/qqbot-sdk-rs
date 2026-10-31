//! 频道撤回、表态、论坛及音频通知。未知字段原样保留。
use super::Event;
use crate::models::{Message, User};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

fn optional_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    match Option::<Value>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(Value::Number(value)) if value.is_u64() => Ok(Some(value.to_string())),
        Some(_) => Err(serde::de::Error::custom("ID 必须是字符串或无符号整数")),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageDeleteData {
    pub message: Option<Message>,
    pub op_user: Option<User>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReactionTarget {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<u32>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReactionEmoji {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<u32>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageReactionData {
    pub user_id: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub target: Option<ReactionTarget>,
    pub emoji: Option<ReactionEmoji>,
    #[serde(flatten)]
    pub extra: Value,
}

/// 论坛文档的 title/content 同时存在字符串说明和富文本数组示例，保留两种表示。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForumContent {
    pub thread_id: Option<String>,
    pub post_id: Option<String>,
    pub reply_id: Option<String>,
    pub title: Option<Value>,
    pub content: Option<Value>,
    pub date_time: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForumEventData {
    #[serde(default, deserialize_with = "optional_id")]
    pub guild_id: Option<String>,
    #[serde(default, deserialize_with = "optional_id")]
    pub channel_id: Option<String>,
    #[serde(default, deserialize_with = "optional_id")]
    pub author_id: Option<String>,
    pub thread_info: Option<ForumContent>,
    pub post_info: Option<ForumContent>,
    pub reply_info: Option<ForumContent>,
    pub thread_id: Option<String>,
    pub post_id: Option<String>,
    pub reply_id: Option<String>,
    #[serde(rename = "type")]
    pub audit_type: Option<u32>,
    pub result: Option<u32>,
    pub err_msg: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioActionData {
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub audio_url: Option<String>,
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioOrLiveMemberData {
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub channel_type: Option<u32>,
    pub user_id: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

macro_rules! events {
    ($data:ty; $($name:ident => $event:literal),+ $(,)?) => {$(
        #[derive(Debug, Clone, Default, Serialize, Deserialize)]
        pub struct $name {
            #[serde(flatten)]
            pub data: $data,
        }
        impl Event for $name { const NAME: &'static str = $event; }
    )+};
}

events!(MessageDeleteData;
    MessageDelete => "MESSAGE_DELETE",
    PublicMessageDelete => "PUBLIC_MESSAGE_DELETE",
    DirectMessageDelete => "DIRECT_MESSAGE_DELETE",
);

events!(MessageReactionData;
    MessageReactionAdd => "MESSAGE_REACTION_ADD",
    MessageReactionRemove => "MESSAGE_REACTION_REMOVE",
);

events!(ForumEventData;
    ForumThreadCreate => "FORUM_THREAD_CREATE",
    ForumThreadUpdate => "FORUM_THREAD_UPDATE",
    ForumThreadDelete => "FORUM_THREAD_DELETE",
    ForumPostCreate => "FORUM_POST_CREATE",
    ForumPostDelete => "FORUM_POST_DELETE",
    ForumReplyCreate => "FORUM_REPLY_CREATE",
    ForumReplyDelete => "FORUM_REPLY_DELETE",
    ForumPublishAuditResult => "FORUM_PUBLISH_AUDIT_RESULT",
    OpenForumThreadCreate => "OPEN_FORUM_THREAD_CREATE",
    OpenForumThreadUpdate => "OPEN_FORUM_THREAD_UPDATE",
    OpenForumThreadDelete => "OPEN_FORUM_THREAD_DELETE",
    OpenForumPostCreate => "OPEN_FORUM_POST_CREATE",
    OpenForumPostDelete => "OPEN_FORUM_POST_DELETE",
    OpenForumReplyCreate => "OPEN_FORUM_REPLY_CREATE",
    OpenForumReplyDelete => "OPEN_FORUM_REPLY_DELETE",
);

events!(AudioActionData;
    AudioStart => "AUDIO_START",
    AudioFinish => "AUDIO_FINISH",
    AudioOnMic => "AUDIO_ON_MIC",
    AudioOffMic => "AUDIO_OFF_MIC",
);

events!(AudioOrLiveMemberData;
    AudioOrLiveChannelMemberEnter => "AUDIO_OR_LIVE_CHANNEL_MEMBER_ENTER",
    AudioOrLiveChannelMemberExit => "AUDIO_OR_LIVE_CHANNEL_MEMBER_EXIT",
);
