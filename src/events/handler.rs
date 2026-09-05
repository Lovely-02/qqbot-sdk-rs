use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
};
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{Instrument, debug, info_span};

type HandlerFuture = BoxFuture<'static, Result<()>>;
type HandlerFn = dyn Fn(EventEnvelope, Arc<QQBotClient>) -> HandlerFuture + Send + Sync;
type HandlerMap = HashMap<String, Vec<Arc<dyn ErasedHandler>>>;

/// 可通过 [`EventRouter::on`] 注册的强类型事件。
pub trait Event: DeserializeOwned + Send + Sync + 'static {
    const NAME: &'static str;
}

/// 接收原始事件信封的处理器抽象，适合封装跨事件的公共逻辑。
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理一个事件。
    async fn handle(&self, event: EventEnvelope, client: Arc<QQBotClient>) -> Result<()>;
}

/// 事件信封，包含事件名称、序列号和原始数据。
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

/// 线程安全的事件处理器注册表。
#[derive(Clone, Default)]
pub struct EventRouter {
    handlers: Arc<RwLock<HandlerMap>>,
}

impl EventRouter {
    /// 创建空路由器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个实现了 [`EventHandler`] 的处理器到所有事件。
    pub async fn add_handler<H: EventHandler + 'static>(&self, handler: H) {
        let handler = Arc::new(handler);
        let wrapped = ClosureHandler {
            call_fn: Arc::new(move |event, client| {
                let handler = handler.clone();
                Box::pin(async move { handler.handle(event, client).await })
            }),
        };
        self.handlers
            .write()
            .await
            .entry("*".into())
            .or_default()
            .push(Arc::new(wrapped));
    }

    /// 注册一个强类型事件处理器。
    pub async fn on<T, F, Fut>(&self, handler: F)
    where
        T: Event,
        F: Fn(T, Arc<QQBotClient>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let wrapped = ClosureHandler {
            call_fn: Arc::new(move |event, client| {
                let result = serde_json::from_value::<T>(event.data).map_err(SdkError::from);
                let handler = handler.clone();
                Box::pin(async move { handler(result?, client).await })
            }),
        };
        self.handlers
            .write()
            .await
            .entry(T::NAME.into())
            .or_default()
            .push(Arc::new(wrapped));
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

    /// 分发一个事件；同名处理器按注册顺序执行。
    pub async fn dispatch(&self, envelope: EventEnvelope, client: Arc<QQBotClient>) -> Result<()> {
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
        let span = info_span!("event", event_type = %envelope.name, event_id = ?envelope.id, trace_id, span_id);
        for handler in handlers {
            handler
                .call(envelope.clone(), client.clone())
                .instrument(span.clone())
                .await?;
        }
        debug!(event_type = %envelope.name, "事件处理完成");
        Ok(())
    }
}
