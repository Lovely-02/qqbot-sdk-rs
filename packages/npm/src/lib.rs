use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;
use qqbot_sdk_core::{Bot, BotMode, EventRouter, GatewayClient, GatewayConfig, Intents, Sendable};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::watch;

fn js_error(error: impl std::fmt::Display) -> Error {
    Error::from_reason(error.to_string())
}

#[napi(object)]
pub struct BotOptions {
    pub app_id: String,
    pub app_secret: String,
    pub mode: Option<String>,
    pub intents: Option<u32>,
}

#[napi(js_name = "NativeBot")]
pub struct NodeBot {
    client: Arc<Bot>,
    intents: Option<u32>,
    started: AtomicBool,
    shutdown: watch::Sender<bool>,
}

#[napi]
impl NodeBot {
    #[napi(constructor)]
    pub fn new(options: BotOptions) -> Result<Self> {
        if options.app_id.trim().is_empty() || options.app_secret.trim().is_empty() {
            return Err(Error::from_reason("appId and appSecret must not be empty"));
        }
        let mode = match options.mode.as_deref().unwrap_or("public") {
            "public" => BotMode::PublicWebSocket,
            "private" => BotMode::PrivateWebSocket,
            _ => return Err(Error::from_reason("mode must be public or private")),
        };
        let (shutdown, _) = watch::channel(false);
        Ok(Self {
            client: Arc::new(Bot::new(options.app_id, options.app_secret, mode).map_err(js_error)?),
            intents: options.intents,
            started: AtomicBool::new(false),
            shutdown,
        })
    }

    /// 仅运行一次：停止时兑现 Promise，网关发生不可恢复的错误时拒绝 Promise。
    #[napi]
    pub async fn run(&self, callback: ThreadsafeFunction<String, ()>) -> Result<()> {
        if self.started.swap(true, Ordering::AcqRel) {
            return Err(Error::from_reason(
                "Bot already started; create a new Bot after stop",
            ));
        }
        let mut shutdown = self.shutdown.subscribe();
        if *shutdown.borrow_and_update() {
            return Ok(());
        }
        let router = EventRouter::new();
        let callback = Arc::new(callback);
        router.on_raw(move |event, _| {
            let callback = Arc::clone(&callback);
            async move {
                let payload = json!({
                    "id": event.id, "name": event.name,
                    "sequence": event.sequence.map(|value| value.to_string()), "data": event.data,
                });
                callback.call_async(Ok(payload.to_string())).await.map_err(|error| {
                    qqbot_sdk_core::SdkError::InvalidInput(format!("Node event callback failed: {error}"))
                })?;
                Ok(())
            }
        }).await;
        let mut config = GatewayConfig::for_bot(&self.client);
        if let Some(bits) = self.intents {
            config.intents = Intents::from_bits(u64::from(bits));
        }
        let gateway =
            GatewayClient::new(Arc::clone(&self.client), router, config).map_err(js_error)?;
        tokio::select! {
            result = gateway.run() => result.map_err(js_error),
            _ = shutdown.changed() => Ok(()),
        }
    }

    #[napi]
    pub fn stop(&self) {
        self.shutdown.send_replace(true);
    }

    #[napi]
    pub async fn send(&self, kind: String, id: String, body: Value) -> Result<Value> {
        if id.trim().is_empty() {
            return Err(Error::from_reason("target id must not be empty"));
        }
        let message = match body {
            Value::String(text) => Sendable::from(text),
            Value::Object(_) => Sendable::from(
                serde_json::from_value::<qqbot_sdk_core::MessageRequest>(body).map_err(js_error)?,
            ),
            _ => {
                return Err(Error::from_reason(
                    "message must be text or a MessageRequest object",
                ));
            }
        };
        let api = self.client.api();
        let messages = api.messages();
        let result = match kind.as_str() {
            "user" => messages.send_c2c(&id, message).await,
            "group" => messages.send_group(&id, message).await,
            "channel" => messages.send_channel(&id, message).await,
            "direct" => messages.send_dm(&id, message).await,
            _ => return Err(Error::from_reason("unknown target kind")),
        }
        .map_err(js_error)?;
        serde_json::to_value(result).map_err(js_error)
    }
}
