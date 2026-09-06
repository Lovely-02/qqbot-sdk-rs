//! qqbot-sdk-rs：异步 QQ 机器人 SDK。

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
pub use client::{Bot, BotMode, Client, ClientConfig, EventTransport, QQBotClient};
pub use entities::{
    ChannelHandle, DirectHandle, GroupHandle, GroupMemberHandle, GuildHandle, GuildMemberHandle,
    UserHandle,
};
pub use error::{Result, SdkError};
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
pub use intents::{GuildMode, Intents};
pub use logging::{Format as LogFormat, LogTarget, SakuraLogger, WorkerGuard};
pub use models::{
    Ark, ArkData, Channel, Embed, FriendAuthor, GatewayBotResponse, Group, Guild, InputNotify,
    Keyboard, Media, Message, MessageAttachment, MessageElement, MessageExtInfo, MessageReference,
    MessageRequest, MessageScene, SessionStartLimit, User,
};
pub use segment::{MediaSegment, MediaSource, MessageBuilder, MessageSegment, Sendable};
