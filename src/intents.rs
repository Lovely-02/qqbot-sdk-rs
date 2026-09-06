use serde::{Deserialize, Serialize};

/// 频道消息订阅模式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum GuildMode {
    /// 公域频道 @ 消息。
    Public,
    /// 私域频道全部消息。
    Private,
}

/// 网关 Intents 位掩码。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Intents(pub u64);

impl Intents {
    /// 频道基础事件。
    pub const GUILDS: u64 = 1 << 0;
    /// 频道成员事件。
    pub const GUILD_MEMBERS: u64 = 1 << 1;
    /// 公域频道消息（`1 << 30`）。
    pub const PUBLIC_GUILD_MESSAGES: u64 = 1 << 30;
    /// 私域频道消息（`1 << 9`）。
    pub const GUILD_MESSAGES: u64 = 1 << 9;
    /// 频道消息表情事件。
    pub const GUILD_MESSAGE_REACTIONS: u64 = 1 << 10;
    /// 频道私信消息（`1 << 12`）。
    pub const DIRECT_MESSAGE: u64 = 1 << 12;
    /// 群聊与单聊事件（`1 << 25`）。
    pub const GROUP_AND_C2C_EVENT: u64 = 1 << 25;
    /// 群成员与入群申请事件。
    pub const GROUP_MEMBER_EVENT: u64 = 1 << 24;
    /// 互动事件。
    pub const INTERACTION: u64 = 1 << 26;
    /// 消息审核结果事件。
    pub const MESSAGE_AUDIT: u64 = 1 << 27;
    /// 私域论坛事件。
    pub const FORUMS_EVENT: u64 = 1 << 28;
    /// 音频动作事件。
    pub const AUDIO_ACTION: u64 = 1 << 29;

    /// 创建空订阅集合。
    pub const fn empty() -> Self {
        Self(0)
    }

    /// 从位掩码创建订阅。
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// 返回底层位掩码。
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// 添加订阅位。
    pub fn insert(&mut self, bits: u64) {
        self.0 |= bits;
    }

    /// 判断是否包含指定订阅位。
    pub const fn contains(self, bits: u64) -> bool {
        self.0 & bits == bits
    }

    /// 生成常用订阅配置。
    pub fn for_mode(mode: GuildMode, direct_message: bool, group_and_c2c: bool) -> Self {
        let mut intents = Self::from_bits(Self::GUILDS);
        intents.insert(match mode {
            GuildMode::Public => Self::PUBLIC_GUILD_MESSAGES,
            GuildMode::Private => Self::GUILD_MESSAGES,
        });
        if direct_message {
            intents.insert(Self::DIRECT_MESSAGE);
        }
        if group_and_c2c {
            intents.insert(Self::GROUP_AND_C2C_EVENT);
        }
        intents
    }
}
