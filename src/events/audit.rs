//! 消息审核相关事件。

use super::Event;
use crate::models::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
