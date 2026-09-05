use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use serde_json::Value;

/// QQ 用户信息的常用字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: Option<String>,
    pub username: Option<String>,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub union_openid: Option<String>,
    pub openid: Option<String>,
}

/// QQ 群信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    pub id: Option<String>,
    pub name: Option<String>,
    pub member_count: Option<u64>,
    pub max_member_count: Option<u64>,
}

/// QQ 子频道信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Channel {
    pub id: Option<String>,
    pub guild_id: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<u8>,
    #[serde(rename = "subtype")]
    pub sub_type: Option<u8>,
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
    pub max_members: Option<u64>,
}

/// 消息中的嵌入卡片。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Embed {
    pub title: Option<String>,
    pub prompt: Option<String>,
    pub thumbnail: Option<Value>,
    pub fields: Option<Vec<Value>>,
}

/// Ark 模板消息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ark {
    pub template_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kv: Option<Vec<Value>>,
}

/// 引用一条已有消息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageReference {
    pub message_id: Option<String>,
    pub ignore_get_message_error: Option<bool>,
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

/// 内嵌按钮键盘定义。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Keyboard {
    pub id: Option<String>,
    pub content: Option<Value>,
}

/// 统一消息请求结构，字段可按消息类型组合使用。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageRequest {
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
    /// 被动回复同一消息时使用的序号；与 `msg_id` 搭配使用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_seq: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_wakeup: Option<bool>,
}

/// QQ API 返回的消息对象。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub content: Option<String>,
    pub timestamp: Option<String>,
    pub author: Option<User>,
    pub msg_type: Option<u8>,
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
    #[serde(deserialize_with = "deserialize_opt_i64", default)]
    pub code: Option<i64>,
    pub message: Option<String>,
    #[serde(deserialize_with = "deserialize_opt_i64", default)]
    pub err_code: Option<i64>,
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
