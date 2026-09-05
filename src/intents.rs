use serde::{Deserialize, Serialize};

/// 公域或私域频道消息订阅模式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum GuildMode {
    /// 只接收 @ 机器人的公域频道消息。
    Public,
    /// 接收全部私域频道消息。
    Private,
}

/// QQ 网关 Intents 位掩码。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Intents(pub u64);

impl Intents {
    /// 公域频道消息（`1 << 30`）。
    pub const PUBLIC_GUILD_MESSAGES: u64 = 1 << 30;
    /// 私域频道消息（`1 << 9`）。
    pub const GUILD_MESSAGES: u64 = 1 << 9;
    /// 频道私信消息（`1 << 12`）。
    pub const DIRECT_MESSAGE: u64 = 1 << 12;
    /// 群聊与单聊事件（`1 << 25`）。
    pub const GROUP_AND_C2C_EVENT: u64 = 1 << 25;

    /// 创建空订阅集合。
    pub const fn empty() -> Self {
        Self(0)
    }

    /// 创建包含给定位的订阅集合。
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// 返回底层位掩码。
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// 添加一个或多个订阅位。
    pub fn insert(&mut self, bits: u64) {
        self.0 |= bits;
    }

    /// 判断是否包含所有指定订阅位。
    pub const fn contains(self, bits: u64) -> bool {
        self.0 & bits == bits
    }

    /// 根据频道模式和私聊开关生成常用订阅配置。
    pub fn for_mode(mode: GuildMode, direct_message: bool, group_and_c2c: bool) -> Self {
        let mut intents = Self::empty();
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
