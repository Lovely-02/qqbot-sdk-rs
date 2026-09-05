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
    sync::Mutex,
    time::{interval, sleep},
};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{Instrument, debug, info, info_span, warn};

/// 网关连接配置。
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 订阅的事件位掩码。
    pub intents: Intents,
    /// 分片编号和总分片数。
    pub shard: (u32, u32),
    /// 断线重连的初始等待时间。
    pub reconnect_delay: Duration,
    /// 是否自动无限重连。
    pub auto_reconnect: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            intents: Intents::for_mode(crate::intents::GuildMode::Public, true, true),
            shard: (0, 1),
            reconnect_delay: Duration::from_secs(2),
            auto_reconnect: true,
        }
    }
}

/// WebSocket 网关客户端，负责连接、鉴权、心跳、事件分发和重连。
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

impl GatewayClient {
    /// 创建网关客户端。
    pub fn new(
        client: Arc<QQBotClient>,
        router: crate::events::EventRouter,
        config: GatewayConfig,
    ) -> Self {
        Self {
            client,
            router,
            config,
            state: Arc::new(Mutex::new(GatewayState::default())),
        }
    }

    /// 连接网关并运行事件循环；断线后按配置重连。
    pub async fn run(&self) -> Result<()> {
        let mut attempts = 0u32;
        loop {
            let result = self.run_connection().await;
            if result.is_ok() || !self.config.auto_reconnect {
                return result;
            }
            attempts = attempts.saturating_add(1);
            let delay = self.config.reconnect_delay.saturating_mul(attempts.min(8));
            warn!(?delay, attempts, error = %result.as_ref().unwrap_err(), "网关断开，准备重连");
            sleep(delay).await;
        }
    }

    async fn run_connection(&self) -> Result<()> {
        let url = self.client.gateway_url().await?;
        let trace_id = crate::client::next_span_id();
        let span_id = crate::client::next_span_id();
        let span = info_span!("gateway_connection", url = %url, trace_id, span_id);
        async move {
            info!("正在连接 WebSocket 网关");
            let (mut socket, _) = connect_async(&url).await.map_err(|e| SdkError::WebSocket(e.to_string()))?;
            let mut heartbeat = interval(Duration::from_secs(3600));
            let mut heartbeat_ready = false;
            let mut sequence = self.state.lock().await.sequence;
            loop {
                tokio::select! {
                    _ = heartbeat.tick(), if heartbeat_ready => {
                        socket.send(WsMessage::Text(json!({"op": OpCode::Heartbeat as u8, "d": sequence}).to_string().into())).await.map_err(|e| SdkError::WebSocket(e.to_string()))?;
                        debug!(sequence = ?sequence, "发送心跳");
                    }
                    incoming = socket.next() => {
                        let incoming = incoming.ok_or_else(|| SdkError::WebSocket("网关连接已关闭".into()))?;
                        let incoming = incoming.map_err(|e| SdkError::WebSocket(e.to_string()))?;
                        let text = match incoming { WsMessage::Text(text) => text.to_string(), WsMessage::Binary(bytes) => String::from_utf8(bytes.to_vec()).map_err(|e| SdkError::WebSocket(e.to_string()))?, WsMessage::Ping(data) => { socket.send(WsMessage::Pong(data)).await.map_err(|e| SdkError::WebSocket(e.to_string()))?; continue }, WsMessage::Close(_) => return Err(SdkError::WebSocket("服务端关闭连接".into())), _ => continue };
                        let payload: Payload<Value> = serde_json::from_str(&text)?;
                        if let Some(seq) = payload.s { sequence = Some(seq); self.state.lock().await.sequence = Some(seq); }
                        match OpCode::from_u8(payload.op) {
                            Some(OpCode::Hello) => {
                                let hello: HelloData = serde_json::from_value(payload.d)?;
                                heartbeat = interval(Duration::from_millis(hello.heartbeat_interval.max(1000)));
                                heartbeat_ready = true;
                                let token = self.client.auth().token().await?;
                                let previous_session = self.state.lock().await.session_id.clone();
                                let resume = previous_session.is_some();
                                let auth_payload = if let Some(session_id) = previous_session.clone() {
                                    json!({"op": OpCode::Resume as u8, "d": {"token": format!("QQBot {}", token), "session_id": session_id, "seq": sequence}})
                                } else {
                                    json!({"op": OpCode::Identify as u8, "d": {"token": format!("QQBot {}", token), "intents": self.config.intents.bits(), "shard": [self.config.shard.0, self.config.shard.1], "properties": {"$os": std::env::consts::OS, "$browser": "qqbot-sdk-rs", "$device": "qqbot-sdk-rs"}}})
                                };
                                socket.send(WsMessage::Text(auth_payload.to_string().into())).await.map_err(|e| SdkError::WebSocket(e.to_string()))?;
                                info!(heartbeat_interval = hello.heartbeat_interval, resume, "收到 Hello 并发送鉴权");
                            }
                            Some(OpCode::Dispatch) => {
                                let name = payload.t.clone().unwrap_or_default();
                                if name == "READY" {
                                    if let Some(session_id) = payload.d.get("session_id").and_then(Value::as_str) { self.state.lock().await.session_id = Some(session_id.to_owned()); }
                                    info!("鉴权成功，收到 READY");
                                }
                                self.router.dispatch(EventEnvelope { id: payload.id, name, sequence: payload.s, data: payload.d }, self.client.clone()).await?;
                            }
                            Some(OpCode::Heartbeat) => {
                                socket.send(WsMessage::Text(json!({"op": 1, "d": sequence}).to_string().into())).await.map_err(|e| SdkError::WebSocket(e.to_string()))?;
                            }
                            Some(OpCode::HeartbeatAck) => debug!("收到心跳 ACK"),
                            Some(OpCode::Reconnect) => return Err(SdkError::WebSocket("服务端要求重连".into())),
                            Some(OpCode::InvalidSession) => { self.state.lock().await.session_id = None; return Err(SdkError::Auth("网关返回 Invalid Session".into())); },
                            _ => debug!(opcode = payload.op, "收到未处理的网关操作码"),
                        }
                    }
                }
            }
        }.instrument(span).await
    }
}
