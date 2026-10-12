//! 网关和 Webhook 事件系统。

pub mod audit;
pub mod friend;
pub mod group;
pub mod guild;
pub mod handler;
pub mod interaction;
pub mod message;
pub mod opcode;
pub mod payload;
pub mod webhook;
pub mod websocket;

pub use audit::{MessageAuditPass, MessageAuditReject, MessageAudited, RawEvent};
pub use friend::{C2cMsgReceive, C2cMsgReject, FriendAdd, FriendDelete};
pub use group::{
    GroupAddRobot, GroupDelRobot, GroupJoinRequest, GroupMemberAdd, GroupMemberRemove,
    GroupMessageSetting, JoinAutoApproved, JoinVerifyInfo, ReviewQuestion, SubscribeMessageStatus,
    SubscribeMessageTemplateResult,
};
pub use guild::{
    ChannelCreate, ChannelDelete, ChannelEvent, ChannelUpdate, GuildCreate, GuildDelete,
    GuildEvent, GuildMemberAdd, GuildMemberEvent, GuildMemberRemove, GuildMemberUpdate,
    GuildUpdate,
};
pub use handler::{Event, EventContext, EventEnvelope, EventHandler, EventRouter};
pub use interaction::{
    InteractionAuthorizeData, InteractionCreate, InteractionData, InteractionMessageScene,
    InteractionResolved,
};
pub use message::{
    C2cMessageReceive, DirectMessageCreate, GroupAtMessageCreate, GroupMessageCreate,
    MessageCreateEvent, ReadyEvent,
};
pub use opcode::OpCode;
pub use payload::Payload;
pub use webhook::{CallbackValidationResponse, Webhook, WebhookVerifier};
pub use websocket::{GatewayClient, GatewayConfig};
/// 返回网关事件的中文名称。
pub fn event_display_name(event: &str) -> &'static str {
    match event {
        "GUILD_CREATE" => "加入频道",
        "GUILD_UPDATE" => "频道更新",
        "GUILD_DELETE" => "退出频道",
        "CHANNEL_CREATE" => "子频道创建",
        "CHANNEL_UPDATE" => "子频道更新",
        "CHANNEL_DELETE" => "子频道删除",
        "GUILD_MEMBER_ADD" => "成员加入",
        "GUILD_MEMBER_UPDATE" => "成员更新",
        "GUILD_MEMBER_REMOVE" => "成员移除",
        "MESSAGE_CREATE" => "频道消息",
        "MESSAGE_DELETE" => "消息撤回",
        "MESSAGE_REACTION_ADD" => "添加表情表态",
        "MESSAGE_REACTION_REMOVE" => "删除表情表态",
        "DIRECT_MESSAGE_CREATE" => "私信消息",
        "DIRECT_MESSAGE_DELETE" => "私信撤回",
        "GROUP_MEMBER_ADD" => "群成员加入",
        "GROUP_MEMBER_REMOVE" => "群成员移除",
        "GROUP_JOIN_REQUEST" => "用户申请加群",
        "C2C_MESSAGE_CREATE" => "私聊消息",
        "FRIEND_ADD" => "添加好友",
        "FRIEND_DEL" => "删除好友",
        "C2C_MSG_REJECT" => "关闭主动消息",
        "C2C_MSG_RECEIVE" => "开启主动消息",
        "GROUP_AT_MESSAGE_CREATE" => "群@消息",
        "GROUP_MESSAGE_CREATE" => "群消息",
        "GROUP_ADD_ROBOT" => "加入群聊",
        "GROUP_DEL_ROBOT" => "移出群聊",
        "GROUP_MSG_REJECT" => "群关闭通知",
        "GROUP_MSG_RECEIVE" => "群开启通知",
        "SUBSCRIBE_MESSAGE_STATUS" => "订阅消息授权状态变更",
        "INTERACTION_CREATE" => "互动事件",
        "MESSAGE_AUDIT_PASS" => "审核通过",
        "MESSAGE_AUDIT_REJECT" => "审核拒绝",
        "FORUM_THREAD_CREATE" => "创建主题",
        "FORUM_THREAD_UPDATE" => "更新主题",
        "FORUM_THREAD_DELETE" => "删除主题",
        "FORUM_POST_CREATE" => "创建帖子",
        "FORUM_POST_DELETE" => "删除帖子",
        "FORUM_REPLY_CREATE" => "创建回复",
        "FORUM_REPLY_DELETE" => "删除回复",
        "FORUM_PUBLISH_AUDIT_RESULT" => "审核结果",
        "AUDIO_START" => "音频开始",
        "AUDIO_FINISH" => "音频结束",
        "AUDIO_ON_MIC" => "上麦",
        "AUDIO_OFF_MIC" => "下麦",
        "AT_MESSAGE_CREATE" => "@机器人消息",
        "PUBLIC_MESSAGE_DELETE" => "公开消息删除",
        "READY" => "就绪",
        "RESUMED" => "恢复连接",
        _ => "未知事件",
    }
}
