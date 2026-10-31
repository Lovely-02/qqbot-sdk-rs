use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
    events::{EventEnvelope, OpCode, Payload},
    intents::Intents,
    models::HelloData,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
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
    coordinator: Option<Arc<super::shards::SessionStartCoordinator>>,
    coordinated_url: Option<String>,
    running: Arc<Mutex<()>>,
    session_invalidated: AtomicBool,
}

pub(super) type RunGuard = Arc<tokio::sync::OwnedMutexGuard<()>>;

// 分发任务可能比 run future 活得更久，必须共同持有守卫直到清理结束。
struct RunLease {
    _gateway: tokio::sync::OwnedMutexGuard<()>,
    _manager: Option<RunGuard>,
}

struct DispatcherGuard(tokio::task::AbortHandle);

impl Drop for DispatcherGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Default)]
struct GatewayState {
    session_id: Option<String>,
    sequence: Option<i64>,
}

impl GatewayState {
    fn resume_session(&mut self) -> Option<(String, i64)> {
        let resume = self
            .session_id
            .as_deref()
            .filter(|id| !id.is_empty())
            .zip(self.sequence)
            .map(|(id, sequence)| (id.to_owned(), sequence));
        if resume.is_none() {
            // READY 尚未处理完时没有可恢复的检查点，不能携带 null 序号或旧序号。
            self.session_id = None;
            self.sequence = None;
        }
        resume
    }
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

impl QueuedItem {
    async fn drain(sender: &mpsc::Sender<Self>, deadline: Duration) -> bool {
        // 队列已满时发送屏障也会阻塞，必须与等待确认共享超时。
        timeout(deadline, async {
            let (done_tx, done_rx) = oneshot::channel();
            sender.send(Self::Barrier(done_tx)).await.ok()?;
            done_rx.await.ok()
        })
        .await
        .is_ok_and(|result| result.is_some())
    }
}

impl From<SdkError> for GatewayFailure {
    fn from(error: SdkError) -> Self {
        let retry = !matches!(error, SdkError::InvalidInput(_));
        Self {
            error,
            retry,
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
    ///
    /// # Errors
    /// Bot 模式不是 WebSocket、Intents 超出公私域订阅限制或 `shard` 参数非法时返回 `SdkError::InvalidInput`。
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
            Some(crate::intents::GuildMode::Public | crate::intents::GuildMode::Private) | None => {
            }
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
            coordinator: None,
            coordinated_url: None,
            running: Arc::default(),
            session_invalidated: AtomicBool::new(false),
        })
    }

    pub(super) fn coordinate_starts(
        &mut self,
        coordinator: Arc<super::shards::SessionStartCoordinator>,
        url: String,
    ) {
        self.coordinator = Some(coordinator);
        self.coordinated_url = Some(url);
    }

    /// 连接网关并运行事件循环，支持重连。
    ///
    /// # Errors
    /// 已在运行或清理时返回 `SdkError::InvalidInput`；网关鉴权、协议或事件分发失败时返回对应错误。
    /// 启用自动重连时持续重试，只有不可恢复的错误才返回。
    pub async fn run(&self) -> Result<()> {
        self.run_with_manager(None).await
    }

    pub(super) async fn run_with_manager(&self, manager: Option<RunGuard>) -> Result<()> {
        let gateway = self
            .running
            .clone()
            .try_lock_owned()
            .map_err(|_| SdkError::InvalidInput("当前网关正在运行或清理中".into()))?;
        let running = Arc::new(RunLease {
            _gateway: gateway,
            _manager: manager,
        });
        let mut attempts = 0u32;
        // 一次运行中的重连复用发现结果；再次调用 run 时允许重新发现地址。
        let discovered_url = tokio::sync::OnceCell::new();
        loop {
            if self.session_invalidated.load(Ordering::Acquire) {
                self.clear_session().await;
                // 清理完成后才撤销标记，取消清理也不会丢失失效通知。
                self.session_invalidated.store(false, Ordering::Release);
            }
            match self.run_connection(&discovered_url, &running).await {
                Ok(()) => continue,
                Err(failure) => {
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

    async fn run_connection(
        &self,
        discovered_url: &tokio::sync::OnceCell<String>,
        running: &Arc<RunLease>,
    ) -> std::result::Result<(), GatewayFailure> {
        let (event_tx, mut event_rx) = mpsc::channel::<QueuedItem>(256);
        let router = self.router.clone();
        let client = self.client.clone();
        let state = self.state.clone();
        let dispatch_failure = Arc::new(Mutex::new(None::<String>));
        let dispatch_failure_for_task = dispatch_failure.clone();
        let running = running.clone();
        let dispatcher = tokio::spawn(async move {
            let _running = running;
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

        let _dispatcher_guard = DispatcherGuard(dispatcher.abort_handle());
        let result = self
            .run_connection_loop(&event_tx, &dispatch_failure, discovered_url)
            .await;
        if result.as_ref().is_err_and(|failure| failure.clear_session) {
            // 必须在排空队列的第一个 await 之前保存，供取消后的下一次运行处理。
            self.session_invalidated.store(true, Ordering::Release);
        }
        let barrier_completed = QueuedItem::drain(&event_tx, Duration::from_secs(30)).await;
        if result.as_ref().is_err_and(|failure| !failure.retry) {
            // 分发器失败不能把封禁、权限等不可恢复的协议错误改成重试。
            dispatcher.abort();
            let _ = dispatcher.await;
            return result;
        }
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
        discovered_url: &tokio::sync::OnceCell<String>,
    ) -> std::result::Result<(), GatewayFailure> {
        let url = match &self.coordinated_url {
            Some(url) => url.clone(),
            None => discovered_url
                .get_or_try_init(|| self.client.gateway_url())
                .await?
                .clone(),
        };
        let trace_id = crate::client::next_span_id();
        let span_id = crate::client::next_span_id();
        let span = info_span!("gateway_connection", url = %url, trace_id, span_id);
        async move {
            // 配额和启动桶在连接前预约，避免远端连接在漫长的配额等待中失效。
            let resume_session = self.state.lock().await.resume_session();
            let needs_identify = resume_session.is_none();
            let mut start_permit = if needs_identify {
                match &self.coordinator {
                    Some(coordinator) => Some(coordinator.acquire(self.config.shard.0).await?),
                    None => None,
                }
            } else { None };
            info!("正在连接 WebSocket 网关");
            let (mut socket, _) = timeout(self.client.config.request_timeout, connect_async(&url))
                .await
                .map_err(|_| SdkError::Timeout("WebSocket 握手".into()))?
                .map_err(|error| SdkError::WebSocket(error.to_string()))?;
            let mut phase_deadline = tokio::time::Instant::now().checked_add(self.client.config.request_timeout)
                .ok_or_else(|| SdkError::InvalidInput("网关 Hello 超时时间超出范围".into()))?;
            let mut received_hello = false;
            let mut heartbeat = interval(Duration::from_secs(3600));
            heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
            heartbeat.tick().await;
            let mut heartbeat_interval = Duration::from_secs(3600);
            let mut heartbeat_ready = false;
            let mut heartbeat_ack_pending = false;
            let mut authenticated = false;

            loop {
                tokio::select! {
                    _ = tokio::time::sleep_until(phase_deadline), if !authenticated => {
                        let phase = if received_hello { "等待网关鉴权完成" } else { "等待网关 Hello" };
                        return Err(SdkError::Timeout(phase.into()).into());
                    }
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
                                if received_hello {
                                    return Err(SdkError::WebSocket("重复收到网关 Hello".into()).into());
                                }
                                received_hello = true;
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

                                let resume = resume_session.is_some();
                                let token = self.client.auth().token().await?;
                                let auth_payload = if let Some((session_id, sequence)) = &resume_session {
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
                                if let Some(permit) = start_permit.as_mut() {
                                    permit.commit()?;
                                }
                                socket
                                    .send(WsMessage::Text(auth_payload.to_string().into()))
                                    .await
                                    .map_err(|error| SdkError::WebSocket(error.to_string()))?;
                                drop(start_permit.take());
                                phase_deadline = tokio::time::Instant::now().checked_add(self.client.config.request_timeout)
                                    .ok_or_else(|| SdkError::InvalidInput("网关鉴权超时时间超出范围".into()))?;
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
                            Some(OpCode::Identify | OpCode::Resume | OpCode::HttpCallbackAck | OpCode::CallbackVerify)
                            | None => debug!(opcode = payload.op, "收到未处理的网关操作码"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn drain_times_out_even_when_queue_is_full() {
        let (sender, _receiver) = mpsc::channel(1);
        let (done, _ack) = oneshot::channel();
        sender.send(QueuedItem::Barrier(done)).await.unwrap();
        let drained = timeout(
            Duration::from_secs(1),
            QueuedItem::drain(&sender, Duration::from_millis(10)),
        )
        .await
        .expect("满队列的屏障发送也必须受超时约束");
        assert!(!drained);
    }

    #[tokio::test]
    async fn drain_waits_for_acknowledgement() {
        let (sender, mut receiver) = mpsc::channel(1);
        let worker = tokio::spawn(async move {
            if let Some(QueuedItem::Barrier(done)) = receiver.recv().await {
                done.send(()).unwrap();
            }
        });
        assert!(QueuedItem::drain(&sender, Duration::from_secs(1)).await);
        worker.await.unwrap();
    }

    #[tokio::test]
    async fn drain_fails_if_dispatcher_is_closed() {
        let (sender, receiver) = mpsc::channel(1);
        drop(receiver);
        assert!(!QueuedItem::drain(&sender, Duration::from_secs(1)).await);
    }
}
