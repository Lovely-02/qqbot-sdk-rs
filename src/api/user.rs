use super::{optional_query, segment};
use crate::{client::QQBotClient, error::Result, models::User};
use reqwest::Method;
use serde_json::Value;

/// 用户相关 API。
pub struct UserApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> UserApi<'a> {
    /// 使用类型化请求发送单聊流式消息；原始字段入口为 `send_stream_message`。
    ///
    /// # Errors
    /// 字段组合无效时返回 `InvalidInput`，其他请求错误返回对应 `SdkError`。
    pub async fn send_stream(
        &self,
        user_openid: &str,
        body: &crate::models::StreamMessageRequest,
    ) -> Result<crate::models::StreamMessageResponse> {
        if body.msg_id.is_some() && body.event_id.is_some()
            || body
                .input_mode
                .as_deref()
                .is_some_and(|mode| !matches!(mode, "append" | "replace"))
            || body
                .input_state
                .is_some_and(|state| !matches!(state, 1 | 10))
            || body
                .content_type
                .as_deref()
                .is_some_and(|kind| !matches!(kind, "text" | "markdown"))
        {
            return Err(crate::error::SdkError::InvalidInput("流式消息需使用 append/replace、状态 1/10、text/markdown；msg_id 与 event_id 二选一".into()));
        }
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/users/{}/stream_messages", segment(user_openid)),
                Some(body),
            )
            .await
    }

    /// 获取机器人自身信息。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn me(&self) -> Result<User> {
        self.client
            .request_json(Method::GET, "/users/@me", Option::<&Value>::None)
            .await
    }
    /// 获取机器人加入的频道列表。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn guilds(
        &self,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<crate::models::Guild>> {
        let query = optional_query([
            ("before", before.map(str::to_owned)),
            ("after", after.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                "/users/@me/guilds",
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 用户富媒体上传。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn upload_file(&self, user_openid: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/users/{}/files", segment(user_openid)),
                Some(body),
            )
            .await
    }
    /// 用户分片上传准备。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn upload_prepare(&self, user_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/users/{}/upload_prepare", segment(user_id)),
                Some(body),
            )
            .await
    }
    /// 用户分片上传完成。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn upload_part_finish(&self, user_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/users/{}/upload_part_finish", segment(user_id)),
                Some(body),
            )
            .await
    }
    /// 向用户发送流式消息片段。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn send_stream_message(&self, user_openid: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/users/{}/stream_messages", segment(user_openid)),
                Some(body),
            )
            .await
    }
}
