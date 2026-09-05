//! qqbot-sdk-rs：异步 QQ 机器人 SDK。
//!
//! SDK 将 HTTP API、网关事件、Webhook、鉴权和日志配置组织成几个相互独立的模块，
//! 使用 [`QQBotClient`] 作为所有 API 调用的入口。

pub mod api;
pub mod auth;
pub mod client;
pub mod error;
pub mod events;
pub mod intents;
pub mod logging;
pub mod models;
pub mod ratelimit;

pub use api::MediaTarget;
pub use api::{
    BotApi, ChannelApi, GroupApi, GuildApi, InteractionApi, MenuApi, MessageApi, PanelApi, UserApi,
    UtilityApi,
};
pub use auth::AccessTokenManager;
pub use client::{Client, ClientConfig, QQBotClient};
pub use error::{Result, SdkError};
pub use events::{
    C2cMessageReceive, C2cMsgReceive, CallbackValidationResponse, Event, EventEnvelope,
    EventHandler, EventRouter, FriendAdd, GatewayClient, GatewayConfig, GroupAtMessageCreate,
    InteractionCreate, MessageCreateEvent, OpCode, Payload, ReadyEvent, event_display_name,
};
pub use intents::{GuildMode, Intents};
pub use logging::{Format as LogFormat, LogTarget, SakuraLogger, WorkerGuard};
pub use models::{
    Channel, Group, Guild, Keyboard, Media, Message, MessageReference, MessageRequest, User,
};
