use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
    models::{Media, Message, MessageRequest},
};
use base64::Engine;
use reqwest::Method;
use serde_json::{Value, json};

use super::segment;

/// 单聊、群聊、子频道和富媒体消息 API。
pub struct MessageApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> MessageApi<'a> {
    /// 向单聊用户发送主动消息。
    pub async fn send_c2c(&self, user_openid: &str, request: &MessageRequest) -> Result<Message> {
        self.send(
            &format!("/v2/users/{}/messages", segment(user_openid)),
            request,
        )
        .await
    }

    /// 回复单聊事件，要求携带 `msg_id` 或 `event_id`。
    pub async fn reply_c2c(
        &self,
        user_openid: &str,
        mut request: MessageRequest,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        set_reply_id(&mut request, msg_id, event_id)?;
        self.send_c2c(user_openid, &request).await
    }

    /// 向群发送主动消息。
    pub async fn send_group(
        &self,
        group_openid: &str,
        request: &MessageRequest,
    ) -> Result<Message> {
        self.send(
            &format!("/v2/groups/{}/messages", segment(group_openid)),
            request,
        )
        .await
    }

    /// 回复群事件，要求携带 `msg_id` 或 `event_id`。
    pub async fn reply_group(
        &self,
        group_openid: &str,
        mut request: MessageRequest,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        set_reply_id(&mut request, msg_id, event_id)?;
        self.send_group(group_openid, &request).await
    }

    /// 向子频道发送消息。
    pub async fn send_channel(
        &self,
        channel_id: &str,
        request: &MessageRequest,
    ) -> Result<Message> {
        self.send(
            &format!("/channels/{}/messages", segment(channel_id)),
            request,
        )
        .await
    }

    /// 回复子频道事件，要求携带 `msg_id` 或 `event_id`。
    pub async fn reply_channel(
        &self,
        channel_id: &str,
        mut request: MessageRequest,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        set_reply_id(&mut request, msg_id, event_id)?;
        self.send_channel(channel_id, &request).await
    }

    /// 向频道私信发送消息。`guild_id` 是频道私信会话标识。
    pub async fn send_dm(&self, guild_id: &str, request: &MessageRequest) -> Result<Message> {
        self.send(&format!("/dms/{}/messages", segment(guild_id)), request)
            .await
    }

    /// 回复频道私信事件，要求携带 `msg_id` 或 `event_id`。
    pub async fn reply_dm(
        &self,
        guild_id: &str,
        mut request: MessageRequest,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        set_reply_id(&mut request, msg_id, event_id)?;
        self.send_dm(guild_id, &request).await
    }

    /// 上传单聊/群聊/频道富媒体文件并返回 `file_info`。
    ///
    /// `file_type` 使用 QQ 定义：1 图片、2 视频、3 语音、4 文件。
    pub async fn upload_media(
        &self,
        target: MediaTarget<'_>,
        file_type: u8,
        data: &[u8],
        srv_send_msg: bool,
    ) -> Result<Media> {
        if !(1..=4).contains(&file_type) {
            return Err(SdkError::InvalidInput(
                "file_type 必须为 1、2、3 或 4".into(),
            ));
        }
        let path = match target {
            MediaTarget::C2c(openid) => format!("/v2/users/{}/files", segment(openid)),
            MediaTarget::Group(openid) => format!("/v2/groups/{}/files", segment(openid)),
            MediaTarget::Channel(id) => format!("/channels/{}/files", segment(id)),
        };
        let body = json!({ "file_type": file_type, "file_data": base64::engine::general_purpose::STANDARD.encode(data), "srv_send_msg": srv_send_msg });
        self.client
            .request_json(Method::POST, &path, Some(&body))
            .await
    }

    /// 使用官方 URL 直传格式上传富媒体。
    pub async fn upload_media_url(
        &self,
        target: MediaTarget<'_>,
        file_type: u8,
        url: &str,
        srv_send_msg: bool,
    ) -> Result<Media> {
        if !(1..=4).contains(&file_type) {
            return Err(SdkError::InvalidInput(
                "file_type 必须为 1、2、3 或 4".into(),
            ));
        }
        let body = json!({
            "file_type": file_type,
            "url": url,
            "srv_send_msg": srv_send_msg,
        });
        self.upload_media_request(target, &body).await
    }

    /// 按官方请求体上传富媒体，支持 URL 直传和分片合并字段。
    pub async fn upload_media_request(
        &self,
        target: MediaTarget<'_>,
        body: &Value,
    ) -> Result<Media> {
        let path = match target {
            MediaTarget::C2c(openid) => format!("/v2/users/{}/files", segment(openid)),
            MediaTarget::Group(openid) => format!("/v2/groups/{}/files", segment(openid)),
            MediaTarget::Channel(id) => format!("/channels/{}/files", segment(id)),
        };
        self.client
            .request_json(Method::POST, &path, Some(body))
            .await
    }

    /// 上传富媒体并立即发送，适用于图片、视频、语音和文件。
    pub async fn send_media(
        &self,
        target: MediaTarget<'_>,
        file_type: u8,
        data: &[u8],
        mut request: MessageRequest,
    ) -> Result<Message> {
        let media = self
            .upload_media(
                match target {
                    MediaTarget::C2c(id) => MediaTarget::C2c(id),
                    MediaTarget::Group(id) => MediaTarget::Group(id),
                    MediaTarget::Channel(id) => MediaTarget::Channel(id),
                },
                file_type,
                data,
                false,
            )
            .await?;
        request.media = Some(media);
        request.msg_type = Some(7);
        match target {
            MediaTarget::C2c(id) => self.send_c2c(id, &request).await,
            MediaTarget::Group(id) => self.send_group(id, &request).await,
            MediaTarget::Channel(id) => self.send_channel(id, &request).await,
        }
    }

    /// 使用 URL 直传并立即发送富媒体消息。
    pub async fn send_media_url(
        &self,
        target: MediaTarget<'_>,
        file_type: u8,
        url: &str,
        mut request: MessageRequest,
    ) -> Result<Message> {
        let media = self.upload_media_url(target, file_type, url, false).await?;
        request.media = Some(media);
        request.msg_type = Some(7);
        match target {
            MediaTarget::C2c(id) => self.send_c2c(id, &request).await,
            MediaTarget::Group(id) => self.send_group(id, &request).await,
            MediaTarget::Channel(id) => self.send_channel(id, &request).await,
        }
    }

    /// 撤回子频道中的一条消息。
    pub async fn delete_channel(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let path = format!(
            "/channels/{}/messages/{}",
            segment(channel_id),
            segment(message_id)
        );
        self.client
            .request_empty::<serde_json::Value>(Method::DELETE, &path, None)
            .await
    }

    /// 撤回单聊中的一条消息（若机器人权限和平台能力允许）。
    pub async fn delete_c2c(&self, user_openid: &str, message_id: &str) -> Result<()> {
        let path = format!(
            "/v2/users/{}/messages/{}",
            segment(user_openid),
            segment(message_id)
        );
        self.client
            .request_empty::<serde_json::Value>(Method::DELETE, &path, None)
            .await
    }

    /// 撤回群中的一条消息（若机器人权限和平台能力允许）。
    pub async fn delete_group(&self, group_openid: &str, message_id: &str) -> Result<()> {
        let path = format!(
            "/v2/groups/{}/messages/{}",
            segment(group_openid),
            segment(message_id)
        );
        self.client
            .request_empty::<serde_json::Value>(Method::DELETE, &path, None)
            .await
    }

    /// 撤回频道私信中的一条消息。
    pub async fn delete_dm(&self, guild_id: &str, message_id: &str, hide_tip: bool) -> Result<()> {
        let value = if hide_tip { "true" } else { "false" };
        self.client
            .request_json_query::<serde_json::Value, serde_json::Value, _>(
                Method::DELETE,
                &format!(
                    "/dms/{}/messages/{}",
                    segment(guild_id),
                    segment(message_id)
                ),
                None,
                &[("hidetip", value)],
            )
            .await
            .map(|_| ())
    }

    async fn send(&self, path: &str, request: &MessageRequest) -> Result<Message> {
        let mut request = request.clone();
        if request.msg_type.is_none() {
            request.msg_type = Some(if request.media.is_some() {
                7
            } else if request.markdown.is_some() {
                2
            } else {
                0
            });
        }
        self.client
            .request_json(Method::POST, path, Some(&request))
            .await
    }
}

fn set_reply_id(
    request: &mut MessageRequest,
    msg_id: Option<&str>,
    event_id: Option<&str>,
) -> Result<()> {
    if msg_id.is_none() && event_id.is_none() {
        return Err(SdkError::InvalidInput(
            "被动回复必须提供 msg_id 或 event_id".into(),
        ));
    }
    request.msg_id = msg_id.map(str::to_owned);
    request.event_id = event_id.map(str::to_owned);
    if msg_id.is_some() && request.msg_seq.is_none() {
        request.msg_seq = Some(1);
    }
    Ok(())
}

/// 富媒体上传的目标会话。
#[derive(Debug, Clone, Copy)]
pub enum MediaTarget<'a> {
    C2c(&'a str),
    Group(&'a str),
    Channel(&'a str),
}
