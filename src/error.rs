use thiserror::Error;

/// SDK 统一错误类型。
#[derive(Debug, Error)]
pub enum SdkError {
    /// HTTP 客户端错误。
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    /// JSON 序列化或反序列化错误。
    #[error("数据序列化失败: {0}")]
    Serialization(#[from] serde_json::Error),
    /// QQ API 返回的业务错误。
    #[error("QQ API 错误 (HTTP {status}, code {code}): {message}")]
    Api {
        status: u16,
        code: i64,
        message: String,
    },
    /// 鉴权失败或 Token 不可用。
    #[error("鉴权失败: {0}")]
    Auth(String),
    /// 网关连接或协议错误。
    #[error("WebSocket 错误: {0}")]
    WebSocket(String),
    /// Webhook 签名不合法。
    #[error("Webhook 签名校验失败")]
    InvalidSignature,
    /// 参数不满足 QQ API 要求。
    #[error("参数错误: {0}")]
    InvalidInput(String),
    /// 触发了本地限频器。
    #[error("请求频率受限: {0}")]
    RateLimited(String),
    /// 日志初始化失败。
    #[error("日志初始化失败: {0}")]
    Logging(String),
}

/// SDK 公开方法使用的结果别名。
pub type Result<T> = std::result::Result<T, SdkError>;
