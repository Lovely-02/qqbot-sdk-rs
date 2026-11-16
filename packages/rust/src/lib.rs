//! qqbot-sdk-rs：异步 QQ 机器人 SDK。

pub use qqbot_sdk_core::api;
pub use qqbot_sdk_core::auth;
pub use qqbot_sdk_core::client;
pub use qqbot_sdk_core::entities;
pub use qqbot_sdk_core::error;
pub use qqbot_sdk_core::events;
pub use qqbot_sdk_core::intents;
pub use qqbot_sdk_core::logging;
pub use qqbot_sdk_core::models;
pub use qqbot_sdk_core::ratelimit;
pub use qqbot_sdk_core::segment;

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
