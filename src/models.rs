use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use serde_json::Value;

/// QQ 用户信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: Option<String>,
    pub username: Option<String>,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub public_flags: Option<u64>,
    pub system: Option<bool>,
    pub union_openid: Option<String>,
    pub union_user_account: Option<String>,
    pub share_url: Option<String>,
    pub welcome_msg: Option<String>,
    pub user_openid: Option<String>,
    pub member_openid: Option<String>,
    pub member_role: Option<String>,
}

/// 好友关系事件中返回的用户摘要。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FriendAuthor {
    pub union_openid: Option<String>,
}

/// QQ 群信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    pub group_openid: Option<String>,
    pub group_name: Option<String>,
    pub group_finger_memo: Option<String>,
    pub group_class_text: Option<String>,
    #[serde(default)]
    pub group_tags: Vec<String>,
    pub group_member_num: Option<u64>,
}

/// QQ 子频道信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Channel {
    pub id: Option<String>,
    pub guild_id: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<u32>,
    pub sub_type: Option<u32>,
    pub position: Option<u32>,
    pub parent_id: Option<String>,
    pub owner_id: Option<String>,
    pub private_type: Option<u32>,
    pub speak_permission: Option<u32>,
    pub application_id: Option<String>,
    pub permissions: Option<String>,
}

/// QQ 频道信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Guild {
    pub id: Option<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub owner_id: Option<String>,
    pub owner: Option<bool>,
    pub joined_at: Option<String>,
    pub description: Option<String>,
    pub member_count: Option<u64>,
    pub max_members: Option<u64>,
}

/// 频道消息中附带的成员信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuildMember {
    pub user: Option<User>,
    pub joined_at: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    pub nick: Option<String>,
    pub deaf: Option<bool>,
    pub mute: Option<bool>,
    pub pending: Option<bool>,
}

/// 消息中的嵌入卡片。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Embed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<Value>>,
}

/// Ark 模板消息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ark {
    pub template_id: u32,
    #[serde(default)]
    pub kv: Vec<Value>,
}

/// 引用一条已有消息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_get_message_error: Option<bool>,
}

/// C2C/群聊输入状态通知。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputNotify {
    pub input_type: Option<u8>,
    pub input_second: Option<u32>,
}

/// 消息响应中用于引用消息的扩展索引。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageExtInfo {
    pub ref_idx: Option<String>,
}

/// QQ 消息附件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageAttachment {
    pub url: Option<String>,
    pub filename: Option<String>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub size: Option<u64>,
    pub content_type: Option<String>,
    pub voice_wav_url: Option<String>,
    pub asr_refer_text: Option<String>,
}

/// 单聊和群聊消息场景上下文。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageScene {
    pub source: Option<String>,
    #[serde(default)]
    pub ext: Vec<String>,
}

impl MessageScene {
    /// 读取 `key=value` 扩展字段。
    pub fn extension(&self, key: &str) -> Option<&str> {
        self.ext.iter().find_map(|item| {
            let (item_key, value) = item.split_once('=')?;
            (item_key == key).then_some(value)
        })
    }

    /// 返回当前消息索引。
    pub fn msg_idx(&self) -> Option<&str> {
        self.extension("msg_idx")
    }

    /// 返回引用消息索引。
    pub fn ref_msg_idx(&self) -> Option<&str> {
        self.extension("ref_msg_idx")
    }

    /// 返回场景鉴权令牌。
    pub fn auth_token(&self) -> Option<&str> {
        self.extension("auth_token")
    }
}

/// 接收消息中的结构化卡片数据。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArkData {
    pub prompt: Option<String>,
    pub ark_type: Option<String>,
    pub ark_name: Option<String>,
    pub fields: Option<Value>,
}

/// 引用消息或并行消息中的嵌套元素。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageElement {
    pub msg_idx: Option<String>,
    pub author: Option<User>,
    pub message_type: Option<u16>,
    pub content: Option<String>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
}

/// 富媒体上传后得到的文件信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Media {
    #[serde(default)]
    pub file_info: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_url: Option<String>,
}

/// 富媒体上传请求，支持 URL、upload_id 合并或 file_data 直传。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaUploadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub srv_send_msg: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
}

/// 流式单聊请求；首片 index=0，后续携带上一片响应 id。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_state: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_msg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_seq: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_wakeup: Option<bool>,
}

/// 流式响应，包含后续分片标识和剩余字符数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamMessageResponse {
    pub id: Option<String>,
    pub timestamp: Option<String>,
    pub ext_info: Option<MessageExtInfo>,
    pub remain_msg_len: Option<i64>,
}

/// 内嵌按钮键盘定义。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Keyboard {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
}

/// 统一消息请求结构，字段可按消息类型组合使用。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<MessageCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// 频道消息中可直接引用的图片 URL。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<Media>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<Embed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark: Option<Ark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<Keyboard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<String>,
    /// 被动回复序号；与 `msg_id` 搭配。单聊／群聊普通发送未填写时自动分配，显式值可用于重试。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_seq: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_wakeup: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_notify: Option<InputNotify>,
    /// 图片转存失败时是否终止 Markdown 发送。
    #[serde(skip)]
    pub force_verify_image_resource: Option<bool>,
}

/// 单聊和群聊图文卡片。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCard {
    #[serde(rename = "type")]
    pub card_type: String,
    pub content: TuwenContent,
}

impl MessageCard {
    pub fn tuwen(content: TuwenContent) -> Self {
        Self {
            card_type: "tuwen".into(),
            content,
        }
    }

    pub(crate) fn validate(&self) -> crate::error::Result<()> {
        let content = &self.content;
        if self.card_type != "tuwen"
            || !(1..=50).contains(&content.title.chars().count())
            || !(1..=200).contains(&content.description.chars().count())
        {
            return Err(crate::error::SdkError::InvalidInput(
                "图文卡片必须使用 tuwen，标题为 1–50 字符，描述为 1–200 字符".into(),
            ));
        }
        for value in [&content.url, &content.pic_url] {
            if value.chars().count() > 1024
                || reqwest::Url::parse(value)
                    .ok()
                    .is_none_or(|url| url.scheme() != "https" || url.host_str().is_none())
            {
                return Err(crate::error::SdkError::InvalidInput(
                    "图文卡片链接必须为有效 HTTPS URL，且不超过 1024 字符".into(),
                ));
            }
        }
        Ok(())
    }
}

/// 图文卡片内容；`url` 和 `pic_url` 必须为 HTTPS 链接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuwenContent {
    pub title: String,
    pub description: String,
    pub url: String,
    pub pic_url: String,
}

impl From<TuwenContent> for MessageCard {
    fn from(content: TuwenContent) -> Self {
        Self::tuwen(content)
    }
}

/// QQ API 返回的消息对象。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub content: Option<String>,
    pub timestamp: Option<String>,
    pub edited_timestamp: Option<String>,
    pub author: Option<User>,
    pub member: Option<GuildMember>,
    #[serde(rename = "type")]
    pub msg_type: Option<u8>,
    pub message_type: Option<u16>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    pub pinned: Option<bool>,
    pub flags: Option<u64>,
    pub seq: Option<u64>,
    pub seq_in_channel: Option<String>,
    pub message_scene: Option<MessageScene>,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    pub mentions: Vec<User>,
    pub ark: Option<Ark>,
    pub ark_data: Option<ArkData>,
    #[serde(default)]
    pub msg_elements: Vec<MessageElement>,
    pub message_reference: Option<MessageReference>,
    pub ext_info: Option<MessageExtInfo>,
}

/// 网关会话启动配额。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStartLimit {
    /// 总启动次数。
    pub total: u32,
    /// 剩余启动次数。
    pub remaining: u32,
    /// 配额重置时间，单位毫秒。
    pub reset_after: u64,
    /// 最大并发数。
    pub max_concurrency: u32,
}

/// `/gateway/bot` 响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayBotResponse {
    /// WebSocket 网关地址。
    pub url: String,
    /// 建议分片数。
    pub shards: u32,
    /// 会话启动配额。
    pub session_start_limit: SessionStartLimit,
}

/// QQ API 统一分页结果。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub next: Option<String>,
}

/// 网关 Hello 数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloData {
    pub heartbeat_interval: u64,
    #[serde(default)]
    pub trace: Vec<String>,
}

/// AccessToken 接口的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    #[serde(default, deserialize_with = "deserialize_u64")]
    pub expires_in: u64,
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| D::Error::custom("必须为无符号整数")),
        Value::String(value) => value.parse::<u64>().map_err(D::Error::custom),
        Value::Null => Ok(0),
        _ => Err(D::Error::custom("必须为数字或数字字符串")),
    }
}

/// QQ API 业务错误响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiErrorBody {
    pub trace_id: Option<String>,
    pub data: Option<Value>,
    #[serde(deserialize_with = "deserialize_opt_i64", default)]
    pub code: Option<i64>,
    pub message: Option<String>,
    #[serde(deserialize_with = "deserialize_opt_i64", default)]
    pub err_code: Option<i64>,
}

impl ApiErrorBody {
    pub(crate) fn nonzero_code(&self) -> Option<i64> {
        self.err_code
            .filter(|code| *code != 0)
            .or_else(|| self.code.filter(|code| *code != 0))
    }
}

fn deserialize_opt_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_i64()
            .map(Some)
            .ok_or_else(|| D::Error::custom("必须为有符号整数")),
        Some(Value::String(value)) => value.parse::<i64>().map(Some).map_err(D::Error::custom),
        Some(_) => Err(D::Error::custom("必须为数字或数字字符串")),
    }
}
