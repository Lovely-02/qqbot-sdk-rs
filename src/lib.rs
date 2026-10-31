//! qqbot-sdk-rs：异步 QQ 机器人 SDK。

pub mod api;
pub mod auth;
pub mod client;
mod endpoint_limits;
pub mod entities;
pub mod error;
pub mod events;
pub mod intents;
pub mod logging;
pub mod models;
pub mod ratelimit;
mod reply;
pub mod segment;

pub use api::MaterialApi;
pub use api::MediaTarget;
pub use api::material::{
    ImageUploadCompletion, ImageUploadPreparation, ImageUploadRequest, MaterialImage,
    MaterialImageEntry, MaterialImagePage,
};
pub use api::{
    BotApi, ChannelApi, GroupApi, GuildApi, InteractionApi, MenuApi, MessageApi, PanelApi, UserApi,
    UtilityApi,
};
pub use auth::AccessTokenManager;
pub use client::{Bot, BotMode, Client, ClientConfig, EventTransport, QQBotClient};
pub use entities::{
    ChannelHandle, DirectHandle, GroupHandle, GroupMemberHandle, GuildHandle, GuildMemberHandle,
    UserHandle,
};
pub use error::{Result, SdkError};
pub use events::GatewayShardManager;
pub use events::notifications::*;
pub use events::{
    C2cMessageReceive, C2cMsgReceive, C2cMsgReject, CallbackValidationResponse, ChannelCreate,
    ChannelDelete, ChannelEvent, ChannelUpdate, DirectMessageCreate, Event, EventContext,
    EventEnvelope, EventHandler, EventRouter, FriendAdd, FriendDelete, GatewayClient,
    GatewayConfig, GroupAddRobot, GroupAtMessageCreate, GroupDelRobot, GroupJoinRequest,
    GroupMemberAdd, GroupMemberRemove, GroupMessageCreate, GroupMessageSetting, GuildCreate,
    GuildDelete, GuildEvent, GuildMemberAdd, GuildMemberEvent, GuildMemberRemove,
    GuildMemberUpdate, GuildUpdate, InteractionAuthorizeData, InteractionCreate, InteractionData,
    InteractionMessageScene, InteractionResolved, JoinAutoApproved, JoinVerifyInfo,
    MessageAuditPass, MessageAuditReject, MessageAudited, MessageCreateEvent, OpCode, Payload,
    ReadyEvent, ReviewQuestion, SubscribeMessageStatus, SubscribeMessageTemplateResult,
    event_display_name,
};
pub use events::{Webhook, WebhookVerifier};
pub use intents::{GuildMode, Intents};
pub use logging::{Format as LogFormat, LogTarget, SakuraLogger, WorkerGuard};
pub use models::{
    Ark, ArkData, Channel, Embed, FriendAuthor, GatewayBotResponse, Group, Guild, InputNotify,
    Keyboard, Media, Message, MessageAttachment, MessageElement, MessageExtInfo, MessageReference,
    MessageRequest, MessageScene, SessionStartLimit, User,
};
pub use models::{MediaUploadRequest, StreamMessageRequest, StreamMessageResponse};
pub use models::{MessageCard, TuwenContent};
pub use segment::{MediaSegment, MediaSource, MessageBuilder, MessageSegment, Sendable};

// 将用户手册作为 doctest 输入，避免示例只在 Markdown 中更新却无法编译。
#[cfg(doctest)]
mod guides {
    #[doc = include_str!("../README.md")]
    mod readme {}
    #[doc = include_str!("../docs/01-getting-started.md")]
    mod getting_started {}
    #[doc = include_str!("../docs/02-messages.md")]
    mod messages {}
    #[doc = include_str!("../docs/03-user.md")]
    mod user {}
    #[doc = include_str!("../docs/04-group.md")]
    mod group {}
    #[doc = include_str!("../docs/05-guild-channel.md")]
    mod channel {}
    #[doc = include_str!("../docs/06-events.md")]
    mod events {}
    #[doc = include_str!("../docs/07-operations.md")]
    mod operations {}
    #[doc = include_str!("../docs/08-bot-tools.md")]
    mod bot_tools {}
    #[doc = include_str!("../docs/09-materials-and-cards.md")]
    mod materials {}
    #[doc = include_str!("../docs/DEVELOPMENT.md")]
    mod development {}
}
