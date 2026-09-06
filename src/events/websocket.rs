use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
    events::{EventEnvelope, OpCode, Payload},
    intents::Intents,
    models::HelloData,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, mpsc, oneshot},
    time::{MissedTickBehavior, interval, sleep, timeout},
};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{Instrument, debug, info, info_span, warn};

/// 网关配置。
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 订阅的事件位掩码。
    pub intents: Intents,
    /// 分片编号和总分片数。
    pub shard: (u32, u32),
    /// 断线重连的初始等待时间。
    pub reconnect_delay: Duration,
    /// 是否自动重连。
    pub auto_reconnect: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            intents: Intents::for_mode(crate::intents::GuildMode::Public, false, false),
            shard: (0, 1),
            reconnect_delay: Duration::from_secs(5),
            auto_reconnect: true,
        }
    }
}

impl GatewayConfig {
    /// 按 Bot 模式创建网关配置。
    pub fn for_bot(client: &QQBotClient) -> Self {
        Self {
            intents: client.default_intents(),
            ..Default::default()
        }
    }
}

/// WebSocket 网关客户端。
pub struct GatewayClient {
    client: Arc<QQBotClient>,
    router: crate::events::EventRouter,
    config: GatewayConfig,
    state: Arc<Mutex<GatewayState>>,
}

#[derive(Default)]
struct GatewayState {
    session_id: Option<String>,
    sequence: Option<i64>,
}

#[derive(Debug)]
struct GatewayFailure {
    error: SdkError,
    retry: bool,
    clear_session: bool,
    authenticated: bool,
}

enum QueuedItem {
    Event(EventEnvelope),
    Barrier(oneshot::Sender<()>),
}

impl From<SdkError> for GatewayFailure {
    fn from(error: SdkError) -> Self {
        Self {
            error,
            retry: true,
            clear_session: false,
            authenticated: false,
        }
    }
}

impl From<serde_json::Error> for GatewayFailure {
    fn from(error: serde_json::Error) -> Self {
        SdkError::from(error).into()
    }
}

impl GatewayClient {
    /// 创建网关客户端。
    pub fn new(
        client: Arc<QQBotClient>,
        router: crate::events::EventRouter,
        config: GatewayConfig,
    ) -> Result<Self> {
        if !client.mode().is_websocket() {
            return Err(SdkError::InvalidInput(
                "当前 Bot 模式不是 WebSocket，不能创建 GatewayClient".into(),
            ));
        }
        match client.guild_mode() {
            Some(crate::intents::GuildMode::Public)
                if config.intents.contains(Intents::GUILD_MESSAGES)
                    || config.intents.contains(Intents::FORUMS_EVENT) =>
            {
                return Err(SdkError::InvalidInput(
                    "公域 Bot 不能订阅 GUILD_MESSAGES 或 FORUMS_EVENT".into(),
                ));
            }
            Some(crate::intents::GuildMode::Private)
                if config.intents.contains(Intents::PUBLIC_GUILD_MESSAGES) =>
            {
                return Err(SdkError::InvalidInput(
                    "私域 Bot 请使用 GUILD_MESSAGES 订阅频道消息".into(),
                ));
            }
            _ => {}
        }
        if config.shard.1 == 0 || config.shard.0 >= config.shard.1 {
            return Err(SdkError::InvalidInput(
                "shard 必须满足 shard_id < shard_count，且 shard_count 不能为 0".into(),
            ));
        }
        Ok(Self {
            client,
            router,
            config,
            state: Arc::new(Mutex::new(GatewayState::default())),
        })
    }

    /// 连接网关并运行事件循环，支持重连。
    pub async fn run(&self) -> Result<()> {
        let mut attempts = 0u32;
        loop {
            match self.run_connection().await {
                Ok(()) => continue,
                Err(failure) => {
                    if failure.clear_session {
                        self.clear_session().await;
                    }
                    if !failure.retry || !self.config.auto_reconnect {
                        break Err(failure.error);
                    }

                    attempts = if failure.authenticated {
                        1
                    } else {
                        attempts.saturating_add(1).max(1)
                    };
                    let delay = self.config.reconnect_delay.saturating_mul(attempts.min(8));
                    warn!(
                        ?delay,
                        attempts,
                        error = %failure.error,
                        "网关断开，准备重连"
                    );
                    sleep(delay).await;
                }
            }
        }
    }

    async fn clear_session(&self) {
        let mut state = self.state.lock().await;
        state.session_id = None;
        state.sequence = None;
    }

    async fn run_connection(&self) -> std::result::Result<(), GatewayFailure> {
        let (event_tx, mut event_rx) = mpsc::channel::<QueuedItem>(256);
        let router = self.router.clone();
        let client = self.client.clone();
        let state = self.state.clone();
        let dispatch_failure = Arc::new(Mutex::new(None::<String>));
        let dispatch_failure_for_task = dispatch_failure.clone();
        let dispatcher = tokio::spawn(async move {
            while let Some(item) = event_rx.recv().await {
                match item {
                    QueuedItem::Event(envelope) => {
                        let sequence = envelope.sequence;
                        match router.dispatch(envelope, client.clone()).await {
                            Ok(()) => {
                                if let Some(sequence) = sequence {
                                    state.lock().await.sequence = Some(sequence);
                                }
                            }
                            Err(error) => {
                                warn!(%error, sequence = ?sequence, "事件处理失败，保留序列号并停止分发");
                                *dispatch_failure_for_task.lock().await = Some(error.to_string());
                                break;
                            }
                        }
                    }
                    QueuedItem::Barrier(done) => {
                        let _ = done.send(());
                    }
                }
            }
        });

        let result = self.run_connection_loop(&event_tx, &dispatch_failure).await;
        let (done_tx, done_rx) = oneshot::channel();
        let barrier_sent = event_tx.send(QueuedItem::Barrier(done_tx)).await.is_ok();
        let barrier_completed = if barrier_sent {
            timeout(Duration::from_secs(30), done_rx)
                .await
                .is_ok_and(|result| result.is_ok())
        } else {
            false
        };
        if !barrier_completed {
            if let Some(error) = dispatch_failure.lock().await.clone() {
                dispatcher.abort();
                let _ = dispatcher.await;
                return Err(GatewayFailure {
                    error: SdkError::WebSocket(format!("事件处理失败: {error}")),
                    retry: true,
                    clear_session: false,
                    authenticated: true,
                });
            }
            // 清理处理器，避免断线任务悬挂。
            dispatcher.abort();
            let _ = dispatcher.await;
            return result;
        }
        if let Some(error) = dispatch_failure.lock().await.clone() {
            dispatcher.abort();
            let _ = dispatcher.await;
            return Err(GatewayFailure {
                error: SdkError::WebSocket(format!("事件处理失败: {error}")),
                retry: true,
                clear_session: false,
                authenticated: true,
            });
        }
        drop(event_tx);
        let _ = dispatcher.await;
        if let Some(error) = dispatch_failure.lock().await.clone() {
            return Err(GatewayFailure {
                error: SdkError::WebSocket(format!("事件处理失败: {error}")),
                retry: true,
                clear_session: false,
                authenticated: true,
            });
        }
        result
    }

    async fn run_connection_loop(
        &self,
        event_tx: &mpsc::Sender<QueuedItem>,
        dispatch_failure: &Arc<Mutex<Option<String>>>,
    ) -> std::result::Result<(), GatewayFailure> {
        let url = self.client.gateway_url().await?;
        let trace_id = crate::client::next_span_id();
        let span_id = crate::client::next_span_id();
        let span = info_span!("gateway_connection", url = %url, trace_id, span_id);
        async move {
            info!("正在连接 WebSocket 网关");
            let (mut socket, _) = connect_async(&url)
                .await
                .map_err(|error| SdkError::WebSocket(error.to_string()))?;
            let mut heartbeat = interval(Duration::from_secs(3600));
            heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
            heartbeat.tick().await;
            let mut heartbeat_interval = Duration::from_secs(3600);
            let mut heartbeat_ready = false;
            let mut heartbeat_ack_pending = false;
            let mut authenticated = false;

            loop {
                tokio::select! {
                    _ = heartbeat.tick(), if heartbeat_ready => {
                        if heartbeat_ack_pending {
                            return Err(GatewayFailure {
                                error: SdkError::WebSocket("心跳 ACK 超时，主动重连".into()),
                                retry: true,
                                clear_session: false,
                                authenticated,
                            });
                        }
                        let sequence = self.state.lock().await.sequence;
                        socket
                            .send(WsMessage::Text(json!({"op": OpCode::Heartbeat as u8, "d": sequence}).to_string().into()))
                            .await
                            .map_err(|error| SdkError::WebSocket(error.to_string()))?;
                        heartbeat_ack_pending = true;
                        debug!(sequence = ?sequence, "发送心跳");
                    }
                    incoming = socket.next() => {
                        let incoming = incoming.ok_or_else(|| SdkError::WebSocket("网关连接已关闭".into()))?;
                        let incoming = incoming.map_err(|error| SdkError::WebSocket(error.to_string()))?;
                        let text = match incoming {
                            WsMessage::Text(text) => text.to_string(),
                            WsMessage::Binary(bytes) => String::from_utf8(bytes.to_vec())
                                .map_err(|error| SdkError::WebSocket(error.to_string()))?,
                            WsMessage::Ping(data) => {
                                socket
                                    .send(WsMessage::Pong(data))
                                    .await
                                    .map_err(|error| SdkError::WebSocket(error.to_string()))?;
                                continue;
                            }
                            WsMessage::Close(frame) => {
                                let (code, reason) = frame
                                    .map(|frame| (Some(u16::from(frame.code)), frame.reason.to_string()))
                                    .unwrap_or((None, String::new()));
                                warn!(?code, %reason, "服务端关闭 WebSocket 连接");
                                return Err(close_failure(code, reason, authenticated));
                            }
                            _ => continue,
                        };

                        debug!(raw_content = %text, "原始内容");
                        let payload: Payload<Value> = serde_json::from_str(&text)?;
                        match OpCode::from_u8(payload.op) {
                            Some(OpCode::Hello) => {
                                let hello: HelloData = serde_json::from_value(payload.d)?;
                                if hello.heartbeat_interval == 0 {
                                    return Err(GatewayFailure {
                                        error: SdkError::WebSocket(
                                            "Hello 中的 heartbeat_interval 不能为 0".into(),
                                        ),
                                        retry: true,
                                        clear_session: false,
                                        authenticated,
                                    });
                                }
                                heartbeat_interval = Duration::from_millis(hello.heartbeat_interval);
                                heartbeat = interval(heartbeat_interval);
                                heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
                                heartbeat.tick().await;
                                heartbeat_ready = false;
                                heartbeat_ack_pending = false;

                                let token = self.client.auth().token().await?;
                                let state = self.state.lock().await;
                                let session_id = state.session_id.clone();
                                let sequence = state.sequence;
                                drop(state);
                                let resume = session_id.is_some();
                                let auth_payload = if let Some(session_id) = session_id {
                                    json!({
                                        "op": OpCode::Resume as u8,
                                        "d": {
                                            "token": format!("QQBot {token}"),
                                            "session_id": session_id,
                                            "seq": sequence,
                                        }
                                    })
                                } else {
                                    json!({
                                        "op": OpCode::Identify as u8,
                                        "d": {
                                            "token": format!("QQBot {token}"),
                                            "intents": self.config.intents.bits(),
                                            "shard": [self.config.shard.0, self.config.shard.1],
                                            "properties": {
                                                "$os": std::env::consts::OS,
                                                "$browser": "qqbot-sdk-rs",
                                                "$device": "qqbot-sdk-rs"
                                            }
                                        }
                                    })
                                };
                                socket
                                    .send(WsMessage::Text(auth_payload.to_string().into()))
                                    .await
                                    .map_err(|error| SdkError::WebSocket(error.to_string()))?;
                                info!(heartbeat_interval = hello.heartbeat_interval, resume, "收到 Hello 并发送鉴权");
                            }
                            Some(OpCode::Dispatch) => {
                                let name = payload.t.clone().unwrap_or_default();
                                if name == "READY" {
                                    if let Some(session_id) = payload.d.get("session_id").and_then(Value::as_str) {
                                        self.state.lock().await.session_id = Some(session_id.to_owned());
                                    }
                                    heartbeat = interval(heartbeat_interval);
                                    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
                                    heartbeat.tick().await;
                                    heartbeat_ready = true;
                                    heartbeat_ack_pending = false;
                                    authenticated = true;
                                    info!("鉴权成功，收到 READY");
                                } else if name == "RESUMED" {
                                    heartbeat = interval(heartbeat_interval);
                                    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
                                    heartbeat.tick().await;
                                    heartbeat_ready = true;
                                    heartbeat_ack_pending = false;
                                    authenticated = true;
                                    info!("会话恢复成功，收到 RESUMED");
                                }
                                event_tx
                                    .send(QueuedItem::Event(EventEnvelope {
                                        id: payload.id,
                                        name,
                                        sequence: payload.s,
                                        data: payload.d,
                                    }))
                                    .await
                                    .map_err(|_| {
                                        let failure = dispatch_failure
                                            .try_lock()
                                            .ok()
                                            .and_then(|value| value.clone())
                                            .unwrap_or_else(|| "事件分发器已停止".into());
                                        SdkError::WebSocket(format!("事件处理失败: {failure}"))
                                    })?;
                            }
                            Some(OpCode::Heartbeat) => {
                                let sequence = self.state.lock().await.sequence;
                                socket
                                    .send(WsMessage::Text(json!({"op": OpCode::Heartbeat as u8, "d": sequence}).to_string().into()))
                                    .await
                                    .map_err(|error| SdkError::WebSocket(error.to_string()))?;
                                heartbeat_ack_pending = true;
                            }
                            Some(OpCode::HeartbeatAck) => {
                                heartbeat_ack_pending = false;
                                debug!("收到心跳 ACK");
                            }
                            Some(OpCode::Reconnect) => {
                                return Err(GatewayFailure {
                                    error: SdkError::WebSocket("服务端要求重连".into()),
                                    retry: true,
                                    clear_session: false,
                                    authenticated,
                                });
                            }
                            Some(OpCode::InvalidSession) => {
                                return Err(GatewayFailure {
                                    error: SdkError::Auth("网关返回 Invalid Session".into()),
                                    retry: true,
                                    clear_session: true,
                                    authenticated,
                                });
                            }
                            _ => debug!(opcode = payload.op, "收到未处理的网关操作码"),
                        }
                    }
                    _ = event_tx.closed() => {
                        let error = dispatch_failure
                            .lock()
                            .await
                            .clone()
                            .unwrap_or_else(|| "事件分发器已停止".into());
                        return Err(GatewayFailure {
                            error: SdkError::WebSocket(format!("事件处理失败: {error}")),
                            retry: true,
                            clear_session: false,
                            authenticated,
                        });
                    }
                }
            }
        }
        .instrument(span)
        .await
    }
}

fn close_failure(code: Option<u16>, reason: String, authenticated: bool) -> GatewayFailure {
    let message = match code {
        Some(code) if reason.is_empty() => format!("服务端关闭连接，关闭码: {code}"),
        Some(code) => format!("服务端关闭连接，关闭码: {code}，原因: {reason}"),
        None if reason.is_empty() => "服务端关闭连接".into(),
        None => format!("服务端关闭连接，原因: {reason}"),
    };
    let clear_session = matches!(
        code,
        Some(4001 | 4002 | 4006 | 4007 | 4010 | 4011 | 4012 | 4013 | 4014 | 4914 | 4915)
    ) || code.is_some_and(|value| (4900..=4913).contains(&value));
    let retry = !matches!(
        code,
        Some(4001 | 4002 | 4010 | 4011 | 4012 | 4013 | 4014 | 4914 | 4915)
    );
    GatewayFailure {
        error: SdkError::WebSocket(message),
        retry,
        clear_session,
        authenticated,
    }
}
