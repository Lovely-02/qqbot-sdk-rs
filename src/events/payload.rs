use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// 网关/Webhook 通用 Payload。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload<T = serde_json::Value> {
    #[serde(default)]
    pub id: Option<String>,
    pub op: u8,
    /// 某些网关控制帧（例如心跳确认）不会携带 `d` 字段。
    #[serde(default)]
    pub d: T,
    #[serde(default)]
    pub s: Option<i64>,
    #[serde(default)]
    pub t: Option<String>,
}

impl<T: DeserializeOwned + Default> Payload<T> {
    /// 从 JSON 文本解析 Payload。
    pub fn parse(input: &str) -> crate::Result<Self> {
        Ok(serde_json::from_str(input)?)
    }
}
