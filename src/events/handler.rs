use super::event_display_name;
use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
    logging::EVENT_LOG_TARGET,
};
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{Instrument, debug, info, info_span};

type HandlerFuture = BoxFuture<'static, Result<()>>;
type HandlerFn = dyn Fn(EventEnvelope, Arc<QQBotClient>) -> HandlerFuture + Send + Sync;
type HandlerMap = HashMap<String, Vec<Arc<dyn ErasedHandler>>>;

/// 返回 JSON 路径对应的非空字符串字段。
fn string_at<'a>(data: &'a Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(data, |value, key| value.get(*key))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}

/// 返回 JSON 路径对应的字符串、数字或布尔值。
fn value_at(data: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(data, |value, key| value.get(*key))
        .and_then(|value| match value {
            Value::Null => None,
            Value::String(value) if value.is_empty() => None,
            Value::Array(value) if value.is_empty() => None,
            Value::Object(value) if value.is_empty() => None,
            Value::String(value) => Some(value.clone()),
            value => Some(value.to_string()),
        })
}

/// 从多个官方字段候选中取得第一个可用值。
fn first_value(data: &Value, paths: &[&[&str]]) -> Option<String> {
    paths.iter().find_map(|path| value_at(data, path))
}

/// 格式化 ID，缺失显示为“未知”。
fn label_value(value: Option<String>) -> String {
    value.unwrap_or_else(|| "未知".into())
}

/// 将官方事件转换为单行业务日志。
fn incoming_event_log(envelope: &EventEnvelope) -> Option<String> {
    let data = &envelope.data;
    let display_name = event_display_name(&envelope.name);
    let content = string_at(data, &["content"]).unwrap_or("[无文本内容]");

    match envelope.name.as_str() {
        "GROUP_AT_MESSAGE_CREATE" | "GROUP_MESSAGE_CREATE" => {
            let group_openid = label_value(value_at(data, &["group_openid"]));
            let member_openid = label_value(first_value(
                data,
                &[
                    &["author", "member_openid"],
                    &["author", "user_openid"],
                    &["author", "id"],
                ],
            ));
            Some(format!(
                "[群消息 ({group_openid})-用户({member_openid})] : {content}"
            ))
        }
        "C2C_MESSAGE_CREATE" => {
            let user_openid = label_value(first_value(
                data,
                &[
                    &["author", "user_openid"],
                    &["user_openid"],
                    &["author", "id"],
                ],
            ));
            Some(format!("[单聊消息 ({user_openid})] : {content}"))
        }
        "MESSAGE_CREATE" | "AT_MESSAGE_CREATE" => {
            let guild_id = label_value(value_at(data, &["guild_id"]));
            let channel_id = label_value(value_at(data, &["channel_id"]));
            let user_id = label_value(value_at(data, &["author", "id"]));
            Some(format!(
                "[频道消息 ({guild_id})-子频道({channel_id})-用户({user_id})] : {content}"
            ))
        }
        "DIRECT_MESSAGE_CREATE" => {
            let guild_id = label_value(value_at(data, &["guild_id"]));
            let user_id = label_value(value_at(data, &["author", "id"]));
            Some(format!(
                "[频道私信 ({guild_id})-用户({user_id})] : {content}"
            ))
        }
        "FRIEND_ADD" | "FRIEND_DEL" => {
            let openid = label_value(value_at(data, &["openid"]));
            let action = if envelope.name == "FRIEND_ADD" {
                "好友已添加"
            } else {
                "好友已删除"
            };
            Some(format!("[{display_name} ({openid})] : {action}"))
        }
        "C2C_MSG_RECEIVE" | "C2C_MSG_REJECT" => {
            let openid = label_value(value_at(data, &["openid"]));
            let action = if envelope.name == "C2C_MSG_RECEIVE" {
                "已开启主动消息接收"
            } else {
                "已关闭主动消息接收"
            };
            Some(format!("[单聊消息接收 ({openid})] : {action}"))
        }
        "GROUP_MEMBER_ADD" | "GROUP_MEMBER_REMOVE" => {
            let group = label_value(value_at(data, &["group_openid"]));
            let member = label_value(first_value(
                data,
                &[&["member_openid"], &["user_openid"], &["user_id"]],
            ));
            let action = if envelope.name == "GROUP_MEMBER_ADD" {
                "新成员加入群聊"
            } else {
                "成员退出群聊"
            };
            Some(format!(
                "[{} ({group})-用户({member})] : {action}",
                display_name
            ))
        }
        "GROUP_ADD_ROBOT" | "GROUP_DEL_ROBOT" => {
            let group = label_value(value_at(data, &["group_openid"]));
            let operator = label_value(value_at(data, &["op_member_openid"]));
            let action = if envelope.name == "GROUP_ADD_ROBOT" {
                "机器人已加入群聊"
            } else {
                "机器人已退出群聊"
            };
            Some(format!(
                "[{} ({group})-操作者({operator})] : {action}",
                display_name
            ))
        }
        "GROUP_MSG_RECEIVE" | "GROUP_MSG_REJECT" => {
            let group = label_value(value_at(data, &["group_openid"]));
            let operator = label_value(value_at(data, &["op_member_openid"]));
            let action = if envelope.name == "GROUP_MSG_RECEIVE" {
                "已开启群主动消息接收"
            } else {
                "已关闭群主动消息接收"
            };
            Some(format!(
                "[{} ({group})-操作者({operator})] : {action}",
                display_name
            ))
        }
        "GROUP_JOIN_REQUEST" => {
            let group = label_value(value_at(data, &["group_openid"]));
            let member = label_value(value_at(data, &["member_openid"]));
            let username = value_at(data, &["username"]).unwrap_or_else(|| "用户".into());
            Some(format!(
                "[入群申请 ({group})-用户({member})] : {username}申请加入群聊"
            ))
        }
        "SUBSCRIBE_MESSAGE_STATUS" => {
            let group = value_at(data, &["group_openid"]);
            let user = value_at(data, &["openid"]);
            let target = match (group, user) {
                (Some(group), Some(user)) => format!("群({group})-用户({user})"),
                (Some(group), None) => format!("群({group})"),
                (None, Some(user)) => format!("用户({user})"),
                (None, None) => "未知".into(),
            };
            let count = data
                .get("result")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            Some(format!(
                "[订阅消息授权 ({target})] : 收到{count}项授权状态变更"
            ))
        }
        "INTERACTION_CREATE" => {
            let scene = value_at(data, &["scene"]);
            let interaction = if let Some(button) = first_value(
                data,
                &[
                    &["data", "resolved", "button_id"],
                    &["data", "resolved", "button_data"],
                ],
            ) {
                format!("点击了按钮({button})")
            } else if let Some(scope) =
                value_at(data, &["data", "resolved", "authorize_data", "scope"])
            {
                format!("提交了授权({scope})")
            } else if let Some(action) = value_at(data, &["data", "resolved", "action"]) {
                format!("触发了操作({action})")
            } else if let Some(feedback) = value_at(data, &["data", "resolved", "feedback_opt"]) {
                format!("提交了反馈({feedback})")
            } else {
                "触发了互动".into()
            };
            let (kind, target, user) = match scene.as_deref() {
                Some("group") => (
                    "群互动",
                    label_value(value_at(data, &["group_openid"])),
                    label_value(first_value(
                        data,
                        &[&["group_member_openid"], &["data", "resolved", "user_id"]],
                    )),
                ),
                Some("guild") => (
                    "频道互动",
                    format!(
                        "{}-子频道({})",
                        label_value(value_at(data, &["guild_id"])),
                        label_value(value_at(data, &["channel_id"]))
                    ),
                    label_value(value_at(data, &["data", "resolved", "user_id"])),
                ),
                _ => (
                    "单聊互动",
                    label_value(value_at(data, &["user_openid"])),
                    label_value(value_at(data, &["data", "resolved", "user_id"])),
                ),
            };
            Some(format!("[{kind} ({target})-用户({user})] : {interaction}"))
        }
        "GUILD_CREATE" | "GUILD_UPDATE" | "GUILD_DELETE" => {
            let guild = label_value(value_at(data, &["id"]));
            let operator = label_value(value_at(data, &["op_user_id"]));
            let name = value_at(data, &["name"]).unwrap_or_else(|| "未命名频道".into());
            Some(format!(
                "[{} ({guild})-操作者({operator})] : {name}",
                display_name
            ))
        }
        "CHANNEL_CREATE" | "CHANNEL_UPDATE" | "CHANNEL_DELETE" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let channel = label_value(first_value(data, &[&["id"], &["channel_id"]]));
            let operator = label_value(value_at(data, &["op_user_id"]));
            let name = value_at(data, &["name"]).unwrap_or_else(|| "未命名子频道".into());
            Some(format!(
                "[{} ({guild})-子频道({channel})-操作者({operator})] : {name}",
                display_name
            ))
        }
        "GUILD_MEMBER_ADD" | "GUILD_MEMBER_UPDATE" | "GUILD_MEMBER_REMOVE" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let user = label_value(value_at(data, &["user", "id"]));
            let operator = label_value(value_at(data, &["op_user_id"]));
            let nick = value_at(data, &["nick"]);
            let detail = nick.map_or_else(String::new, |nick| format!(" : {nick}"));
            Some(format!(
                "[{} ({guild})-用户({user})-操作者({operator})]{detail}",
                display_name
            ))
        }
        "MESSAGE_DELETE" | "PUBLIC_MESSAGE_DELETE" | "DIRECT_MESSAGE_DELETE" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let channel = label_value(value_at(data, &["channel_id"]));
            let message = label_value(first_value(data, &[&["id"], &["message_id"]]));
            Some(format!(
                "[{} ({guild})-子频道({channel})-消息({message})] : 消息已撤回",
                display_name
            ))
        }
        "MESSAGE_REACTION_ADD" | "MESSAGE_REACTION_REMOVE" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let channel = label_value(value_at(data, &["channel_id"]));
            let user = label_value(value_at(data, &["user_id"]));
            let message = label_value(value_at(data, &["target", "id"]));
            let emoji = label_value(first_value(data, &[&["emoji", "name"], &["emoji", "id"]]));
            Some(format!(
                "[{} ({guild})-子频道({channel})-用户({user})] : 消息({message})表情({emoji})",
                display_name
            ))
        }
        "MESSAGE_AUDIT_PASS" | "MESSAGE_AUDIT_REJECT" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let channel = label_value(value_at(data, &["channel_id"]));
            let message = label_value(value_at(data, &["message_id"]));
            Some(format!(
                "[{} ({guild})-子频道({channel})-消息({message})] : 审核完成",
                display_name
            ))
        }
        "FORUM_THREAD_CREATE"
        | "FORUM_THREAD_UPDATE"
        | "FORUM_THREAD_DELETE"
        | "FORUM_POST_CREATE"
        | "FORUM_POST_DELETE"
        | "FORUM_REPLY_CREATE"
        | "FORUM_REPLY_DELETE"
        | "FORUM_PUBLISH_AUDIT_RESULT" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let channel = label_value(value_at(data, &["channel_id"]));
            let user = label_value(value_at(data, &["author_id"]));
            let object = first_value(
                data,
                &[
                    &["thread_info", "thread_id"],
                    &["post_info", "post_id"],
                    &["reply_info", "reply_id"],
                    &["thread_id"],
                    &["post_id"],
                    &["reply_id"],
                ],
            )
            .unwrap_or_else(|| "未知".into());
            Some(format!(
                "[{} ({guild})-子频道({channel})-用户({user})] : 对象({object})",
                display_name
            ))
        }
        "AUDIO_START" | "AUDIO_FINISH" | "AUDIO_ON_MIC" | "AUDIO_OFF_MIC" => {
            let guild = label_value(value_at(data, &["guild_id"]));
            let channel = label_value(value_at(data, &["channel_id"]));
            let user = label_value(first_value(data, &[&["user_id"], &["user_openid"]]));
            Some(format!(
                "[{} ({guild})-子频道({channel})-用户({user})] : 音频状态已变更",
                display_name
            ))
        }
        "READY" => Some(format!(
            "[网关就绪] : 会话({})",
            label_value(value_at(data, &["session_id"]))
        )),
        "RESUMED" => Some("[网关恢复连接] : 会话已恢复".into()),
        _ if display_name != "未知事件" => Some(format!("[{display_name}] : 官方事件已触发")),
        _ => None,
    }
}

/// 处理器运行时上下文。
#[derive(Clone)]
pub struct EventContext {
    client: Arc<QQBotClient>,
    pub event_id: Option<String>,
    pub event_name: String,
    pub sequence: Option<i64>,
}

impl std::fmt::Debug for EventContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EventContext")
            .field("event_id", &self.event_id)
            .field("event_name", &self.event_name)
            .field("sequence", &self.sequence)
            .finish()
    }
}

impl EventContext {
    pub(crate) fn new(envelope: &EventEnvelope, client: Arc<QQBotClient>) -> Self {
        Self {
            client,
            event_id: envelope.id.clone(),
            event_name: envelope.name.clone(),
            sequence: envelope.sequence,
        }
    }

    pub(crate) fn client(&self) -> Arc<QQBotClient> {
        self.client.clone()
    }
}

/// 可注册的强类型事件。
pub trait Event: DeserializeOwned + Send + Sync + 'static {
    const NAME: &'static str;
    const NAMES: &'static [&'static str] = &[Self::NAME];

    /// 附加运行时客户端和网关元数据。
    fn attach_context(&mut self, _context: EventContext) {}
}

/// 原始事件处理器。
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理一个事件。
    async fn handle(&self, event: EventEnvelope, client: Arc<QQBotClient>) -> Result<()>;
}

/// 事件信封。
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub id: Option<String>,
    pub name: String,
    pub sequence: Option<i64>,
    pub data: Value,
}

#[async_trait]
trait ErasedHandler: Send + Sync {
    async fn call(&self, event: EventEnvelope, client: Arc<QQBotClient>) -> Result<()>;
}

struct ClosureHandler {
    call_fn: Arc<HandlerFn>,
}

#[async_trait]
impl ErasedHandler for ClosureHandler {
    async fn call(&self, event: EventEnvelope, client: Arc<QQBotClient>) -> Result<()> {
        (self.call_fn)(event, client).await
    }
}

/// 线程安全事件路由器。
#[derive(Clone, Default)]
pub struct EventRouter {
    handlers: Arc<RwLock<HandlerMap>>,
}

impl EventRouter {
    /// 创建空路由器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册处理所有事件的处理器。
    pub async fn add_handler<H: EventHandler + 'static>(&self, handler: H) {
        let handler = Arc::new(handler);
        let wrapped: Arc<dyn ErasedHandler> = Arc::new(ClosureHandler {
            call_fn: Arc::new(move |event, client| {
                let handler = handler.clone();
                Box::pin(async move { handler.handle(event, client).await })
            }),
        });
        self.handlers
            .write()
            .await
            .entry("*".into())
            .or_default()
            .push(wrapped);
    }

    /// 注册一个强类型事件处理器。
    pub async fn on<T, F, Fut>(&self, handler: F)
    where
        T: Event,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let wrapped: Arc<dyn ErasedHandler> = Arc::new(ClosureHandler {
            call_fn: Arc::new(move |event, client| {
                let context = EventContext::new(&event, client.clone());
                let result = serde_json::from_value::<T>(event.data)
                    .map(|mut parsed| {
                        parsed.attach_context(context);
                        parsed
                    })
                    .map_err(SdkError::from);
                let handler = handler.clone();
                Box::pin(async move { handler(result?).await })
            }),
        });
        let mut handlers = self.handlers.write().await;
        for name in T::NAMES {
            handlers
                .entry((*name).into())
                .or_default()
                .push(wrapped.clone());
        }
    }

    /// 注册接收所有未知事件的原始处理器。
    pub async fn on_raw<F, Fut>(&self, handler: F)
    where
        F: Fn(EventEnvelope, Arc<QQBotClient>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let wrapped = ClosureHandler {
            call_fn: Arc::new(move |event, client| {
                let handler = handler.clone();
                Box::pin(async move { handler(event, client).await })
            }),
        };
        self.handlers
            .write()
            .await
            .entry("*".into())
            .or_default()
            .push(Arc::new(wrapped));
    }

    /// 分发事件，按注册顺序执行。
    pub async fn dispatch(&self, envelope: EventEnvelope, client: Arc<QQBotClient>) -> Result<()> {
        let display_name = event_display_name(&envelope.name);
        if let Some(event_log) = incoming_event_log(&envelope) {
            info!(target: EVENT_LOG_TARGET, "{event_log}");
        } else {
            info!(
                event_type = %envelope.name,
                event_name = %display_name,
                event_id = ?envelope.id,
                sequence = ?envelope.sequence,
                "收到事件"
            );
        }
        let mut handlers = self
            .handlers
            .read()
            .await
            .get(&envelope.name)
            .cloned()
            .unwrap_or_default();
        handlers.extend(
            self.handlers
                .read()
                .await
                .get("*")
                .cloned()
                .unwrap_or_default(),
        );
        let trace_id = crate::client::next_span_id();
        let span_id = crate::client::next_span_id();
        let span = info_span!(
            "event",
            event_type = %envelope.name,
            event_name = %display_name,
            event_id = ?envelope.id,
            trace_id,
            span_id
        );
        for handler in handlers {
            handler
                .call(envelope.clone(), client.clone())
                .instrument(span.clone())
                .await?;
        }
        debug!(
            event_type = %envelope.name,
            event_name = %display_name,
            "事件处理完成（调试详情）"
        );
        Ok(())
    }
}
