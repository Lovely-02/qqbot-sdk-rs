use super::{optional_query, segment};
use crate::{client::QQBotClient, error::Result, models::User};
use reqwest::Method;
use serde_json::Value;

/// 用户相关 API。
pub struct UserApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> UserApi<'a> {
    /// 获取单聊用户信息。
    pub async fn get(&self, openid: &str) -> Result<User> {
        self.client
            .request_json(
                Method::GET,
                &format!("/v2/users/{}", segment(openid)),
                Option::<&serde_json::Value>::None,
            )
            .await
    }

    /// 获取机器人自身信息。
    pub async fn me(&self) -> Result<User> {
        self.client
            .request_json(Method::GET, "/users/@me", Option::<&Value>::None)
            .await
    }
    /// 获取机器人加入的频道列表。
    pub async fn guilds(
        &self,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<crate::models::Guild>> {
        let query = optional_query([
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
