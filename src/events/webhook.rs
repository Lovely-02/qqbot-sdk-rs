use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
    events::{EventEnvelope, EventRouter, Payload},
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Webhook 请求签名校验器。
#[derive(Clone)]
pub struct WebhookVerifier {
    key: VerifyingKey,
}

impl WebhookVerifier {
    /// 从开发者平台的 Bot Secret 推导 Ed25519 公钥。
    pub fn from_secret(bot_secret: &str) -> Result<Self> {
        if bot_secret.is_empty() {
            return Err(SdkError::Auth("Bot Secret 不能为空".into()));
        }
        Ok(Self {
            key: SigningKey::from_bytes(&secret_seed(bot_secret)).verifying_key(),
        })
    }

    /// 从 64 位十六进制公钥创建校验器。
    pub fn from_hex(public_key: &str) -> Result<Self> {
        let bytes = hex::decode(public_key)
            .map_err(|e| SdkError::Auth(format!("公钥不是合法十六进制: {e}")))?;
        let key = VerifyingKey::from_bytes(
            bytes
                .as_slice()
                .try_into()
                .map_err(|_| SdkError::Auth("公钥必须为 32 字节".into()))?,
        )
        .map_err(|e| SdkError::Auth(format!("公钥无效: {e}")))?;
        Ok(Self { key })
    }

    /// 校验 `timestamp + body` 的 Ed25519 签名。
    pub fn verify(&self, timestamp: &str, signature_hex: &str, body: &[u8]) -> Result<()> {
        let signature_bytes = hex::decode(signature_hex).map_err(|_| SdkError::InvalidSignature)?;
        let signature =
            Signature::from_slice(&signature_bytes).map_err(|_| SdkError::InvalidSignature)?;
        let mut message = timestamp.as_bytes().to_vec();
        message.extend_from_slice(body);
        self.key
            .verify(&message, &signature)
            .map_err(|_| SdkError::InvalidSignature)
    }
}

/// Webhook 解析器，先验签再解析为通用 Payload。
#[derive(Clone)]
pub struct Webhook {
    verifier: WebhookVerifier,
}

/// 回调地址验证请求的响应体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackValidationResponse {
    pub plain_token: String,
    pub signature: String,
}

impl Webhook {
    /// 创建 Webhook 解析器。
    pub fn new(verifier: WebhookVerifier) -> Self {
        Self { verifier }
    }

    /// 使用 AppSecret 为平台的 `op=13` 回调验证生成响应。
    ///
    /// 平台约定以 AppSecret 重复/截断为 32 字节 Ed25519 seed，并对
    /// `event_ts + plain_token` 签名。
    pub fn validation_response(
        body: &[u8],
        app_secret: &str,
    ) -> Result<CallbackValidationResponse> {
        let payload: Payload<Value> = serde_json::from_slice(body)?;
        if payload.op != crate::events::OpCode::CallbackVerify as u8 {
            return Err(SdkError::InvalidInput(
                "回调地址验证请求的 op 必须为 13".into(),
            ));
        }
        let plain_token = payload
            .d
            .get("plain_token")
            .and_then(Value::as_str)
            .ok_or_else(|| SdkError::InvalidInput("验证请求缺少 plain_token".into()))?
            .to_owned();
        let event_ts = payload
            .d
            .get("event_ts")
            .and_then(Value::as_str)
            .ok_or_else(|| SdkError::InvalidInput("验证请求缺少 event_ts".into()))?;
        if app_secret.is_empty() {
            return Err(SdkError::Auth("AppSecret 不能为空".into()));
        }
        let seed = secret_seed(app_secret);
        let signing_key = SigningKey::from_bytes(&seed);
        let mut message = event_ts.as_bytes().to_vec();
        message.extend_from_slice(plain_token.as_bytes());
        let signature = hex::encode(signing_key.sign(&message).to_bytes());
        Ok(CallbackValidationResponse {
            plain_token,
            signature,
        })
    }

    /// 校验请求头并解析事件。
    pub fn parse(&self, timestamp: &str, signature: &str, body: &[u8]) -> Result<Payload<Value>> {
        self.verifier.verify(timestamp, signature, body)?;
        Ok(serde_json::from_slice(body)?)
    }

    /// 校验并直接返回 SDK 事件信封。
    pub fn parse_envelope(
        &self,
        timestamp: &str,
        signature: &str,
        body: &[u8],
    ) -> Result<EventEnvelope> {
        let payload = self.parse(timestamp, signature, body)?;
        Ok(EventEnvelope {
            id: payload.id,
            name: payload.t.unwrap_or_default(),
            sequence: payload.s,
            data: payload.d,
        })
    }

    /// 验签、解析并交给事件路由器处理，适合接入任意 HTTP 框架。
    pub async fn dispatch(
        &self,
        timestamp: &str,
        signature: &str,
        body: &[u8],
        router: &EventRouter,
        client: Arc<QQBotClient>,
    ) -> Result<()> {
        if !client.mode().is_webhook() {
            return Err(SdkError::InvalidInput(
                "当前 Bot 模式不是 Webhook，不能分发 Webhook 事件".into(),
            ));
        }
        router
            .dispatch(self.parse_envelope(timestamp, signature, body)?, client)
            .await
    }
}

fn secret_seed(secret: &str) -> [u8; 32] {
    let secret = secret.as_bytes();
    let mut seed = [0u8; 32];
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = secret[index % secret.len()];
    }
    seed
}
