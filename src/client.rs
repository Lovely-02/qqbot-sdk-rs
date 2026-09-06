use crate::{
    api::{
        BotApi, ChannelApi, GroupApi, GuildApi, InteractionApi, MenuApi, MessageApi, PanelApi,
        UserApi, UtilityApi,
    },
    auth::AccessTokenManager,
    entities::{
        ChannelHandle, DirectHandle, GroupHandle, GroupMemberHandle, GuildHandle,
        GuildMemberHandle, UserHandle,
    },
    error::{Result, SdkError},
    intents::{GuildMode, Intents},
    logging::API_ERROR_LOG_TARGET,
    models::{ApiErrorBody, GatewayBotResponse},
    ratelimit::RateLimiter,
};
use reqwest::{Client as HttpClient, Method, RequestBuilder, Response};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tracing::{Instrument, debug, error, info_span};

static NEXT_SPAN_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_span_id() -> u64 {
    NEXT_SPAN_ID.fetch_add(1, Ordering::Relaxed)
}

/// 事件接入方式。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventTransport {
    /// WebSocket 接入。
    WebSocket,
    /// Webhook 接入。
    Webhook,
}

/// Bot 的域类型与事件接入方式。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BotMode {
    /// 公域 Bot + WebSocket。
    PublicWebSocket,
    /// 私域 Bot + WebSocket。
    PrivateWebSocket,
    /// 公域 Bot + Webhook。
    PublicWebhook,
    /// 私域 Bot + Webhook。
    PrivateWebhook,
}

impl BotMode {
    /// 返回公域或私域模式。
    pub const fn guild_mode(self) -> GuildMode {
        match self {
            Self::PublicWebSocket | Self::PublicWebhook => GuildMode::Public,
            Self::PrivateWebSocket | Self::PrivateWebhook => GuildMode::Private,
        }
    }

    /// 返回事件接入方式。
    pub const fn event_transport(self) -> EventTransport {
        match self {
            Self::PublicWebSocket | Self::PrivateWebSocket => EventTransport::WebSocket,
            Self::PublicWebhook | Self::PrivateWebhook => EventTransport::Webhook,
        }
    }

    /// 是否使用 WebSocket。
    pub const fn is_websocket(self) -> bool {
        matches!(self.event_transport(), EventTransport::WebSocket)
    }

    /// 是否使用 Webhook。
    pub const fn is_webhook(self) -> bool {
        matches!(self.event_transport(), EventTransport::Webhook)
    }
}

/// 客户端配置。
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Bot 模式。
    pub mode: BotMode,
    /// QQ API 根地址。
    pub api_base_url: String,
    /// 网关地址；为空时先调用 `/gateway`。
    pub gateway_url: Option<String>,
    /// 单次 HTTP 请求超时。
    pub request_timeout: Duration,
    /// 主动消息 QPS 限制。
    pub bot_qps: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            mode: BotMode::PublicWebSocket,
            api_base_url: "https://api.bot.qq.com".into(),
            gateway_url: None,
            request_timeout: Duration::from_secs(20),
            bot_qps: 5,
        }
    }
}

/// QQ Bot 客户端。
#[derive(Clone)]
pub struct QQBotClient {
    pub(crate) http: HttpClient,
    pub(crate) auth: AccessTokenManager,
    pub(crate) config: Arc<ClientConfig>,
    pub(crate) limiter: RateLimiter,
}

/// 客户端别名。
pub type Client = QQBotClient;
/// Bot 风格别名。
pub type Bot = QQBotClient;

impl QQBotClient {
    /// 创建客户端。
    pub fn new(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        mode: BotMode,
    ) -> Result<Self> {
        Self::with_config(
            app_id,
            app_secret,
            ClientConfig {
                mode,
                ..Default::default()
            },
        )
    }

    /// 按配置创建客户端。
    pub fn with_config(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        config: ClientConfig,
    ) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(config.request_timeout)
            .user_agent("qqbot-sdk-rs/0.0.1")
            .build()?;
        let auth = AccessTokenManager::new(
            app_id,
            app_secret,
            config.api_base_url.clone(),
            http.clone(),
        );
        let limiter = RateLimiter::new(config.bot_qps.max(1), Duration::from_secs(1));
        Ok(Self {
            http,
            auth,
            config: Arc::new(config),
            limiter,
        })
    }

    /// 返回 API 入口。
    pub fn api(&self) -> Api<'_> {
        Api { client: self }
    }

    /// 返回鉴权管理器。
    pub fn auth(&self) -> &AccessTokenManager {
        &self.auth
    }

    /// 返回 Bot 模式。
    pub fn mode(&self) -> BotMode {
        self.config.mode
    }

    /// 返回公域或私域模式。
    pub fn guild_mode(&self) -> GuildMode {
        self.config.mode.guild_mode()
    }

    /// 返回事件接入方式。
    pub fn event_transport(&self) -> EventTransport {
        self.config.mode.event_transport()
    }

    /// 生成默认网关订阅；特殊 Intents 需按权限手动加入。
    pub fn default_intents(&self) -> Intents {
        Intents::for_mode(self.guild_mode(), false, false)
    }

    /// 创建群会话实体。
    pub fn group(&self, id: impl Into<String>) -> GroupHandle {
        GroupHandle::new(Arc::new(self.clone()), id)
    }

    /// 创建群成员实体。
    pub fn group_member(
        &self,
        group_openid: impl Into<String>,
        member_openid: impl Into<String>,
    ) -> GroupMemberHandle {
        GroupMemberHandle::new(Arc::new(self.clone()), group_openid, member_openid)
    }

    /// 创建单聊会话实体。
    pub fn user(&self, id: impl Into<String>) -> UserHandle {
        UserHandle::new(Arc::new(self.clone()), id)
    }

    /// 创建子频道会话实体。
    pub fn channel(&self, id: impl Into<String>) -> ChannelHandle {
        ChannelHandle::new(Arc::new(self.clone()), id)
    }

    /// 创建频道私信实体。
    pub fn direct(&self, guild_id: impl Into<String>) -> DirectHandle {
        DirectHandle::new(Arc::new(self.clone()), guild_id)
    }

    /// 创建频道会话实体。
    pub fn guild(&self, id: impl Into<String>) -> GuildHandle {
        GuildHandle::new(Arc::new(self.clone()), id)
    }

    /// 创建频道成员实体。
    pub fn guild_member(
        &self,
        guild_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> GuildMemberHandle {
        GuildMemberHandle::new(Arc::new(self.clone()), guild_id, user_id)
    }

    /// 发送单聊消息。
    pub async fn send_c2c_message(
        &self,
        user_openid: &str,
        message: impl Into<crate::segment::Sendable>,
    ) -> Result<crate::models::Message> {
        self.api().messages().send_c2c(user_openid, message).await
    }

    /// 发送群消息。
    pub async fn send_group_message(
        &self,
        group_openid: &str,
        message: impl Into<crate::segment::Sendable>,
    ) -> Result<crate::models::Message> {
        self.api()
            .messages()
            .send_group(group_openid, message)
            .await
    }

    /// 发送子频道消息。
    pub async fn send_channel_message(
        &self,
        channel_id: &str,
        message: impl Into<crate::segment::Sendable>,
    ) -> Result<crate::models::Message> {
        self.api()
            .messages()
            .send_channel(channel_id, message)
            .await
    }

    /// 获取网关地址。
    pub async fn gateway_url(&self) -> Result<String> {
        if let Some(url) = &self.config.gateway_url {
            return Ok(url.clone());
        }
        let response: GatewayResponse = self
            .request_json(Method::GET, "/gateway", Option::<&Value>::None)
            .await?;
        response
            .url
            .ok_or_else(|| SdkError::WebSocket("/gateway 响应缺少 url".into()))
    }

    /// 获取 `/gateway/bot` 的 WSS 地址。
    pub async fn gateway_url_bot(&self) -> Result<String> {
        Ok(self.gateway_bot().await?.url)
    }

    /// 获取 `/gateway/bot` 的完整响应。
    pub async fn gateway_bot(&self) -> Result<GatewayBotResponse> {
        self.request_json(Method::GET, "/gateway/bot", Option::<&Value>::None)
            .await
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!(
            "{}{}",
            self.config.api_base_url.trim_end_matches('/'),
            if path.starts_with('/') {
                path.to_owned()
            } else {
                format!("/{path}")
            }
        )
    }

    pub(crate) async fn request_json<B, T>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.limiter.acquire("bot").await?;
        let token = self.auth.token().await?;
        let trace_id = next_span_id();
        let span_id = next_span_id();
        let span = info_span!("qq_http", method = %method, path, trace_id, span_id);
        async move {
            let mut request = self
                .http
                .request(method.clone(), self.url(path))
                .header("Authorization", format!("QQBot {token}"));
            if let Some(body) = body {
                request = request.json(body);
            }
            let response = self.send_request(request).await?;
            self.decode_response(response).await
        }
        .instrument(span)
        .await
    }

    pub(crate) async fn request_json_query<B, T, Q>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        query: &Q,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        self.limiter.acquire("bot").await?;
        let token = self.auth.token().await?;
        let trace_id = next_span_id();
        let span_id = next_span_id();
        let span = info_span!("qq_http", method = %method, path, trace_id, span_id);
        async move {
            let mut request = self
                .http
                .request(method.clone(), self.url(path))
                .header("Authorization", format!("QQBot {token}"))
                .query(query);
            if let Some(body) = body {
                request = request.json(body);
            }
            let response = self.send_request(request).await?;
            self.decode_response(response).await
        }
        .instrument(span)
        .await
    }

    pub(crate) async fn request_multipart<T>(
        &self,
        method: Method,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.limiter.acquire("bot").await?;
        let token = self.auth.token().await?;
        let trace_id = next_span_id();
        let span_id = next_span_id();
        let span = info_span!("qq_http", method = %method, path, trace_id, span_id);
        async move {
            let request = self
                .http
                .request(method.clone(), self.url(path))
                .header("Authorization", format!("QQBot {token}"))
                .multipart(form);
            let response = self.send_request(request).await?;
            self.decode_response(response).await
        }
        .instrument(span)
        .await
    }

    pub(crate) async fn request_empty<B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<()> {
        let _: Value = self.request_json(method, path, body).await?;
        Ok(())
    }

    /// 发送 QQ API 请求并记录网络错误。
    async fn send_request(&self, request: RequestBuilder) -> Result<Response> {
        request.send().await.map_err(|error| {
            error!(target: API_ERROR_LOG_TARGET, "{error}");
            SdkError::from(error)
        })
    }

    /// 解析 QQ API 响应并记录错误。
    async fn decode_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let bytes = response.bytes().await.map_err(|error| {
            error!(target: API_ERROR_LOG_TARGET, "{error}");
            SdkError::from(error)
        })?;
        debug!(status = status.as_u16(), size = bytes.len(), "QQ API 响应");
        if !status.is_success() {
            let body = serde_json::from_slice::<ApiErrorBody>(&bytes).unwrap_or_default();
            let error = SdkError::Api {
                status: status.as_u16(),
                code: body.err_code.or(body.code).unwrap_or(-1),
                message: body
                    .message
                    .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned()),
            };
            error!(target: API_ERROR_LOG_TARGET, "{}", String::from_utf8_lossy(&bytes));
            return Err(error);
        }
        if bytes.is_empty() {
            return serde_json::from_value(Value::Null).map_err(|error| {
                error!(target: API_ERROR_LOG_TARGET, "{error}");
                SdkError::from(error)
            });
        }
        serde_json::from_slice(&bytes).map_err(|error| {
            error!(target: API_ERROR_LOG_TARGET, "{}", String::from_utf8_lossy(&bytes));
            SdkError::from(error)
        })
    }
}

/// API 入口。
pub struct Api<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> Api<'a> {
    /// 机器人自身 API。
    pub fn bot(&self) -> BotApi<'a> {
        BotApi {
            client: self.client,
        }
    }
    /// 频道管理 API。
    pub fn guilds(&self) -> GuildApi<'a> {
        GuildApi {
            client: self.client,
        }
    }
    /// 消息 API。
    pub fn messages(&self) -> MessageApi<'a> {
        MessageApi {
            client: self.client,
        }
    }
    /// 用户 API。
    pub fn users(&self) -> UserApi<'a> {
        UserApi {
            client: self.client,
        }
    }
    /// 群 API。
    pub fn groups(&self) -> GroupApi<'a> {
        GroupApi {
            client: self.client,
        }
    }
    /// 子频道 API。
    pub fn channels(&self) -> ChannelApi<'a> {
        ChannelApi {
            client: self.client,
        }
    }
    /// 互动回调 API。
    pub fn interactions(&self) -> InteractionApi<'a> {
        InteractionApi {
            client: self.client,
        }
    }
    /// 自定义菜单 API。
    pub fn menu(&self) -> MenuApi<'a> {
        MenuApi {
            client: self.client,
        }
    }
    /// 面板 API。
    pub fn panels(&self) -> PanelApi<'a> {
        PanelApi {
            client: self.client,
        }
    }
    /// 通用工具 API。
    pub fn utility(&self) -> UtilityApi<'a> {
        UtilityApi {
            client: self.client,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct GatewayResponse {
    url: Option<String>,
}
