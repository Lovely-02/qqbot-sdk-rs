use super::{
    C2cMessageReceive, DirectMessageCreate, Event, EventContext, EventEnvelope,
    GroupAtMessageCreate, GroupMessageCreate, MessageCreateEvent,
};
use crate::{Bot, Message, Result, SdkError, Sendable};
use std::{ops::Deref, sync::Arc};

/// 字符串订阅收到的事件；通过字段访问官方名称、原始数据和网关元数据。
#[derive(Debug, Clone)]
pub struct BotEvent {
    envelope: EventEnvelope,
    context: EventContext,
}

/// 用于取消订阅的标识，同一进程内唯一。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(pub(crate) u64);

impl Deref for BotEvent {
    type Target = EventEnvelope;

    fn deref(&self) -> &Self::Target {
        &self.envelope
    }
}

impl BotEvent {
    pub(crate) fn new(envelope: EventEnvelope, bot: Arc<Bot>) -> Self {
        let context = EventContext::new(&envelope, bot);
        Self { envelope, context }
    }

    /// 返回当前事件的 Bot，可用于创建群、用户、频道等会话实体。
    pub fn bot(&self) -> Arc<Bot> {
        self.context.client()
    }

    /// 转为强类型事件，并保留回复等辅助方法需要的运行时上下文。
    ///
    /// # 错误
    /// 类型与官方事件名称不匹配时返回参数错误；字段解析失败时返回序列化错误。
    pub fn parse<T: Event>(&self) -> Result<T> {
        if !T::NAMES.contains(&self.name.as_str()) {
            return Err(SdkError::InvalidInput(format!(
                "事件 {} 不能解析为 {}",
                self.name,
                T::NAME
            )));
        }
        let mut event: T = serde_json::from_value(self.data.clone())?;
        event.attach_context(self.context.clone());
        Ok(event)
    }

    /// 回复群聊、单聊、频道或频道私信消息。
    ///
    /// # 错误
    /// 非消息事件返回参数错误；其他错误与对应强类型事件的 `reply` 一致。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let message = message.into();
        match self.name.as_str() {
            "GROUP_AT_MESSAGE_CREATE" => self.parse::<GroupAtMessageCreate>()?.reply(message).await,
            "GROUP_MESSAGE_CREATE" => self.parse::<GroupMessageCreate>()?.reply(message).await,
            "C2C_MESSAGE_CREATE" => self.parse::<C2cMessageReceive>()?.reply(message).await,
            "DIRECT_MESSAGE_CREATE" => self.parse::<DirectMessageCreate>()?.reply(message).await,
            "MESSAGE_CREATE" | "AT_MESSAGE_CREATE" => {
                self.parse::<MessageCreateEvent>()?.reply(message).await
            }
            _ => Err(SdkError::InvalidInput(format!(
                "事件 {} 不是消息事件",
                self.name
            ))),
        }
    }
}
