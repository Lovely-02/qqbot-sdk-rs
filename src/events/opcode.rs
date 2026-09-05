use serde::{Deserialize, Serialize};

/// QQ 网关操作码。
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
#[repr(u8)]
pub enum OpCode {
    /// 服务端推送事件。
    Dispatch = 0,
    /// 心跳。
    Heartbeat = 1,
    /// 客户端鉴权。
    Identify = 2,
    /// 恢复会话。
    Resume = 6,
    /// 服务端要求重连。
    Reconnect = 7,
    /// 会话无效。
    InvalidSession = 9,
    /// 连接建立后的 Hello。
    Hello = 10,
    /// 心跳确认。
    HeartbeatAck = 11,
    /// HTTP 回调确认。
    HttpCallbackAck = 12,
    /// 回调地址验证。
    CallbackVerify = 13,
}

impl OpCode {
    /// 将平台数值转换为操作码，未知值返回 `None`。
    pub fn from_u8(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::Dispatch,
            1 => Self::Heartbeat,
            2 => Self::Identify,
            6 => Self::Resume,
            7 => Self::Reconnect,
            9 => Self::InvalidSession,
            10 => Self::Hello,
            11 => Self::HeartbeatAck,
            12 => Self::HttpCallbackAck,
            13 => Self::CallbackVerify,
            _ => return None,
        })
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or_else(|| format!("未知网关操作码: {value}"))
    }
}
