//! qqbot-sdk-rs：异步 QQ 机器人 SDK。
//!
//! SDK 将 HTTP API、网关事件、Webhook、鉴权和日志配置组织成几个相互独立的模块，
//! 使用 [`QQBotClient`] 作为所有 API 调用的入口。

pub mod api;
pub mod auth;
pub mod client;
pub mod entities;
pub mod error;
pub mod events;
pub mod intents;
pub mod logging;
pub mod models;
pub mod ratelimit;
pub mod segment;

pub use api::MediaTarget;
pub use api::{
    BotApi, ChannelApi, GroupApi, GuildApi, InteractionApi, MenuApi, MessageApi, PanelApi, UserApi,
    UtilityApi,
};
pub use auth::AccessTokenManager;
pub use client::{Bot, Client, ClientConfig, QQBotClient};
pub use entities::{ChannelHandle, DirectHandle, GroupHandle, GuildHandle, UserHandle};
pub use error::{Result, SdkError};
pub use events::{
    C2cMessageReceive, C2cMsgReceive, CallbackValidationResponse, DirectMessageCreate, Event,
    EventContext, EventEnvelope, EventHandler, EventRouter, FriendAdd, FriendDelete, GatewayClient,
    GatewayConfig, GroupAtMessageCreate, GroupMessageCreate, InteractionCreate, MessageCreateEvent,
    OpCode, Payload, ReadyEvent, event_display_name,
};
pub use intents::{GuildMode, Intents};
pub use logging::{Format as LogFormat, LogTarget, SakuraLogger, WorkerGuard};
pub use models::{
    Ark, ArkData, Channel, Embed, FriendAuthor, Group, Guild, InputNotify, Keyboard, Media,
    Message, MessageAttachment, MessageElement, MessageExtInfo, MessageReference, MessageRequest,
    MessageScene, User,
};
pub use segment::{MediaSegment, MediaSource, MessageBuilder, MessageSegment, Sendable};
