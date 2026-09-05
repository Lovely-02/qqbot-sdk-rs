//! 网关和 Webhook 事件系统。

pub mod handler;
pub mod opcode;
pub mod payload;
pub mod webhook;
pub mod websocket;

pub use handler::{Event, EventEnvelope, EventHandler, EventRouter};
pub use opcode::OpCode;
pub use payload::Payload;
pub use webhook::{CallbackValidationResponse, Webhook, WebhookVerifier};
pub use websocket::{GatewayClient, GatewayConfig};

use crate::models::{Message, User};
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
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for MessageCreateEvent {
    const NAME: &'static str = "MESSAGE_CREATE";
}

/// 单聊消息事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct C2cMessageReceive {
    pub id: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for C2cMessageReceive {
    const NAME: &'static str = "C2C_MESSAGE_CREATE";
}

/// 新版网关使用的单聊消息事件名称（`C2C_MSG_RECEIVE`）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct C2cMsgReceive {
    pub id: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for C2cMsgReceive {
    const NAME: &'static str = "C2C_MSG_RECEIVE";
}

/// 群聊中 @ 机器人的消息事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupAtMessageCreate {
    pub id: Option<String>,
    pub group_openid: Option<String>,
    pub content: Option<String>,
    pub author: Option<User>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for GroupAtMessageCreate {
    const NAME: &'static str = "GROUP_AT_MESSAGE_CREATE";
}

/// 好友添加事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FriendAdd {
    pub user_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

impl Event for FriendAdd {
    const NAME: &'static str = "FRIEND_ADD";
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
