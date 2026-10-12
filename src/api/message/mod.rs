//! 单聊、群聊、子频道消息 API 与目标会话定义。

use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
    models::{Message, MessageRequest},
    segment::{MediaSegment, MediaSource, Sendable},
};
use base64::Engine;
use reqwest::Method;
use serde_json::Value;
use std::fs;

use super::segment;

mod media;

/// 单聊、群聊、子频道和富媒体消息 API。
pub struct MessageApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> MessageApi<'a> {
    /// 向单聊用户发送主动消息。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn send_c2c(
        &self,
        user_openid: &str,
        message: impl Into<Sendable>,
    ) -> Result<Message> {
        self.send_to(MessageTarget::C2c(user_openid), message.into(), None, None)
            .await
    }

    /// 回复单聊事件，要求携带 `msg_id` 或 `event_id`。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn reply_c2c(
        &self,
        user_openid: &str,
        message: impl Into<Sendable>,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        self.send_to(
            MessageTarget::C2c(user_openid),
            message.into(),
            msg_id,
            event_id,
        )
        .await
    }

    /// 向群发送主动消息。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn send_group(
        &self,
        group_openid: &str,
        message: impl Into<Sendable>,
    ) -> Result<Message> {
        self.send_to(
            MessageTarget::Group(group_openid),
            message.into(),
            None,
            None,
        )
        .await
    }

    /// 回复群事件，要求携带 `msg_id` 或 `event_id`。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn reply_group(
        &self,
        group_openid: &str,
        message: impl Into<Sendable>,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        self.send_to(
            MessageTarget::Group(group_openid),
            message.into(),
            msg_id,
            event_id,
        )
        .await
    }

    /// 向子频道发送消息。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn send_channel(
        &self,
        channel_id: &str,
        message: impl Into<Sendable>,
    ) -> Result<Message> {
        self.send_to(
            MessageTarget::Channel(channel_id),
            message.into(),
            None,
            None,
        )
        .await
    }

    /// 回复子频道事件，要求携带 `msg_id` 或 `event_id`。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn reply_channel(
        &self,
        channel_id: &str,
        message: impl Into<Sendable>,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        self.send_to(
            MessageTarget::Channel(channel_id),
            message.into(),
            msg_id,
            event_id,
        )
        .await
    }

    /// 向频道私信发送消息。`guild_id` 是频道私信会话标识。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn send_dm(&self, guild_id: &str, message: impl Into<Sendable>) -> Result<Message> {
        self.send_to(MessageTarget::Dm(guild_id), message.into(), None, None)
            .await
    }

    /// 回复频道私信事件，要求携带 `msg_id` 或 `event_id`。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn reply_dm(
        &self,
        guild_id: &str,
        message: impl Into<Sendable>,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        self.send_to(
            MessageTarget::Dm(guild_id),
            message.into(),
            msg_id,
            event_id,
        )
        .await
    }
}

impl<'a> MessageApi<'a> {
    /// 撤回子频道中的一条消息。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn delete_channel(&self, channel_id: &str, message_id: &str) -> Result<()> {
        if self.client.guild_mode() == Some(crate::intents::GuildMode::Public) {
            return Err(SdkError::InvalidInput(
                "公域 Bot 不支持频道消息撤回，请使用私域 Bot".into(),
            ));
        }
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
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
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
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
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
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn delete_dm(&self, guild_id: &str, message_id: &str, hide_tip: bool) -> Result<()> {
        if self.client.guild_mode() == Some(crate::intents::GuildMode::Public) {
            return Err(SdkError::InvalidInput(
                "公域 Bot 不支持频道私信撤回，请使用私域 Bot".into(),
            ));
        }
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

    async fn send_to(
        &self,
        target: MessageTarget<'_>,
        message: Sendable,
        msg_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<Message> {
        let mut built = message.build()?;
        let mut channel_file = None;
        if let Some(media) = built.media.take() {
            if matches!(target, MessageTarget::Channel(_))
                && media.file_type == 1
                && !matches!(&media.source, MediaSource::Location(location) if is_http_url(location))
            {
                channel_file = Some(media);
            } else {
                self.apply_media(target, media, &mut built.request).await?;
            }
        }
        if msg_id.is_some() || event_id.is_some() {
            set_reply_id(&mut built.request, msg_id, event_id)?;
        }

        normalize_request(
            target,
            built.channel_only_content,
            channel_file.is_some(),
            &mut built.request,
        )?;
        if let (MessageTarget::Channel(_), Some(media)) = (target, channel_file) {
            return self
                .send_channel_image_file(target, media, built.request)
                .await;
        }
        self.send_request(target, built.request).await
    }

    async fn send_channel_image_file(
        &self,
        target: MessageTarget<'_>,
        media: MediaSegment,
        request: MessageRequest,
    ) -> Result<Message> {
        let data = match media.source {
            MediaSource::Location(location) => decode_or_read_media(&location)?,
            MediaSource::Bytes(data) => data,
        };
        let file_name = media.name.unwrap_or_else(|| "image.jpg".into());
        let mut form = reqwest::multipart::Form::new().part(
            "file_image",
            reqwest::multipart::Part::bytes(data).file_name(file_name),
        );
        if let Some(content) = request.content {
            form = form.text("content", content);
        }
        if let Some(embed) = request.embed {
            form = form.text("embed", serde_json::to_string(&embed)?);
        }
        if let Some(ark) = request.ark {
            form = form.text("ark", serde_json::to_string(&ark)?);
        }
        if let Some(markdown) = request.markdown {
            form = form.text("markdown", serde_json::to_string(&markdown)?);
        }
        if let Some(reference) = request.message_reference {
            form = form.text("message_reference", serde_json::to_string(&reference)?);
        }
        if let Some(msg_id) = request.msg_id {
            form = form.text("msg_id", msg_id);
        }
        if let Some(msg_seq) = request.msg_seq {
            form = form.text("msg_seq", msg_seq.to_string());
        }
        if let Some(event_id) = request.event_id {
            form = form.text("event_id", event_id);
        }
        self.client
            .request_multipart(Method::POST, &target.path(), form)
            .await
    }

    async fn apply_media(
        &self,
        target: MessageTarget<'_>,
        media: MediaSegment,
        request: &mut MessageRequest,
    ) -> Result<()> {
        if matches!(target, MessageTarget::Dm(_) | MessageTarget::Channel(_))
            && media.file_type == 1
            && let MediaSource::Location(ref location) = media.source
            && is_http_url(location)
        {
            request.image = Some(location.clone());
            return Ok(());
        }

        if matches!(target, MessageTarget::Dm(_) | MessageTarget::Channel(_)) {
            return Err(SdkError::InvalidInput(
                "频道/频道私信只支持图片消息；频道本地图片使用 file_image，频道私信请使用网络图片 URL"
                    .into(),
            ));
        }

        let upload_target = target
            .media_target_borrowed()
            .ok_or_else(|| SdkError::InvalidInput("当前会话不支持富媒体上传".into()))?;
        let uploaded = match media.source {
            MediaSource::Location(location) if is_http_url(&location) => {
                self.upload_media_url(upload_target, media.file_type, &location, false)
                    .await?
            }
            MediaSource::Location(location) => {
                let data = decode_or_read_media(&location)?;
                self.upload_media(upload_target, media.file_type, &data, false)
                    .await?
            }
            MediaSource::Bytes(data) => {
                self.upload_media(upload_target, media.file_type, &data, false)
                    .await?
            }
        };
        request.media = Some(uploaded);
        request.msg_type = Some(7);
        Ok(())
    }

    async fn send_request(
        &self,
        target: MessageTarget<'_>,
        mut request: MessageRequest,
    ) -> Result<Message> {
        if matches!(target, MessageTarget::C2c(_) | MessageTarget::Group(_))
            && request.msg_type.is_none()
        {
            request.msg_type = Some(infer_msg_type(&request));
        }
        apply_force_verify_image_resource(&mut request)?;
        self.client
            .request_json(Method::POST, &target.path(), Some(&request))
            .await
    }
}

fn infer_msg_type(request: &MessageRequest) -> u8 {
    if request.media.is_some() {
        7
    } else if request.markdown.is_some() {
        2
    } else if request.input_notify.is_some() {
        6
    } else {
        0
    }
}

/// 将便捷字段写入 `markdown` 对象。
fn apply_force_verify_image_resource(request: &mut MessageRequest) -> Result<()> {
    let Some(force_verify) = request.force_verify_image_resource else {
        return Ok(());
    };
    let markdown = request.markdown.as_mut().ok_or_else(|| {
        SdkError::InvalidInput(
            "force_verify_image_resource 只能与单聊或群聊 Markdown 消息一起使用".into(),
        )
    })?;
    let object = markdown.as_object_mut().ok_or_else(|| {
        SdkError::InvalidInput(
            "markdown 必须是官方对象，才能设置 force_verify_image_resource".into(),
        )
    })?;
    object.insert(
        "force_verify_image_resource".into(),
        Value::Bool(force_verify),
    );
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum MessageTarget<'a> {
    C2c(&'a str),
    Group(&'a str),
    Channel(&'a str),
    Dm(&'a str),
}

impl MessageTarget<'_> {
    fn path(self) -> String {
        match self {
            Self::C2c(id) => format!("/v2/users/{}/messages", segment(id)),
            Self::Group(id) => format!("/v2/groups/{}/messages", segment(id)),
            Self::Channel(id) => format!("/channels/{}/messages", segment(id)),
            Self::Dm(id) => format!("/dms/{}/messages", segment(id)),
        }
    }
}

impl<'a> MessageTarget<'a> {
    fn media_target_borrowed(self) -> Option<MediaTarget<'a>> {
        match self {
            Self::C2c(id) => Some(MediaTarget::C2c(id)),
            Self::Group(id) => Some(MediaTarget::Group(id)),
            Self::Channel(_) => None,
            Self::Dm(_) => None,
        }
    }
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn decode_or_read_media(value: &str) -> Result<Vec<u8>> {
    if let Some(encoded) = value.strip_prefix("base64://") {
        return base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|error| SdkError::InvalidInput(format!("Base64 文件数据无效: {error}")));
    }
    if value.starts_with("data:")
        && let Some((_, encoded)) = value.split_once(",")
    {
        return base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|error| SdkError::InvalidInput(format!("Data URL 文件数据无效: {error}")));
    }
    let path = value.strip_prefix("file://").unwrap_or(value);
    fs::read(path)
        .map_err(|error| SdkError::InvalidInput(format!("无法读取媒体文件 {path}: {error}")))
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
    // 官方 API 将 msg_id 和 event_id 视为二选一参数。事件辅助方法可能同时拿到两者，
    // 因此在消息 ID 存在时优先使用消息 ID。
    request.msg_id = msg_id.map(str::to_owned);
    request.event_id = if msg_id.is_some() {
        None
    } else {
        event_id.map(str::to_owned)
    };
    if request.msg_id.is_some() && request.msg_seq.is_none() {
        request.msg_seq = Some(1);
    }
    Ok(())
}

fn normalize_request(
    target: MessageTarget<'_>,
    channel_only_content: bool,
    has_file_image: bool,
    request: &mut MessageRequest,
) -> Result<()> {
    if request.msg_id.is_some() && request.event_id.is_some() {
        return Err(SdkError::InvalidInput(
            "msg_id 与 event_id 必须二选一".into(),
        ));
    }
    if request.msg_id.is_none() && request.msg_seq.is_some() {
        return Err(SdkError::InvalidInput(
            "msg_seq 只能与 msg_id 一起使用".into(),
        ));
    }
    match target {
        MessageTarget::C2c(_) => {
            if channel_only_content {
                return Err(SdkError::InvalidInput(
                    "@用户、@全体、表情和子频道链接是频道内嵌格式，不能用于单聊".into(),
                ));
            }
            if request.image.is_some() || request.embed.is_some() || request.ark.is_some() {
                return Err(SdkError::InvalidInput(
                    "单聊官方仅支持文本、Markdown、输入状态和富媒体消息".into(),
                ));
            }
            if request.markdown.is_some() && request.content.is_some() {
                return Err(SdkError::InvalidInput(
                    "发送 Markdown 时 content 必须为空".into(),
                ));
            }
            if request.force_verify_image_resource.is_some() && request.markdown.is_none() {
                return Err(SdkError::InvalidInput(
                    "force_verify_image_resource 只能与 Markdown 消息一起使用".into(),
                ));
            }
            let primary = [
                request.content.as_ref().map(|_| 0),
                request.markdown.as_ref().map(|_| 2),
                request.input_notify.as_ref().map(|_| 6),
                request.media.as_ref().map(|_| 7),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            if primary.len() != 1 {
                return Err(SdkError::InvalidInput(
                    "单聊消息必须且只能包含 content、markdown、input_notify、media 之一".into(),
                ));
            }
            if let Some(msg_type) = request.msg_type
                && !matches!(msg_type, 0 | 2 | 6 | 7)
            {
                return Err(SdkError::InvalidInput(
                    "单聊 msg_type 只能是 0、2、6 或 7".into(),
                ));
            }
            if request
                .msg_type
                .is_some_and(|msg_type| msg_type != primary[0])
            {
                return Err(SdkError::InvalidInput(
                    "单聊 msg_type 与消息内容字段不匹配".into(),
                ));
            }
            if request.is_wakeup.is_some()
                && (request.msg_id.is_some() || request.event_id.is_some())
            {
                return Err(SdkError::InvalidInput(
                    "is_wakeup 不能与 msg_id/event_id 同时使用".into(),
                ));
            }
            if let Some(input) = request.input_notify.as_ref()
                && (input.input_type != Some(1)
                    || input
                        .input_second
                        .is_none_or(|seconds| !(1..=60).contains(&seconds)))
            {
                return Err(SdkError::InvalidInput(
                    "input_notify 必须使用 input_type=1，input_second 范围为 1..=60".into(),
                ));
            }
            if request.input_notify.is_some()
                && (request.content.is_some()
                    || request.markdown.is_some()
                    || request.media.is_some()
                    || request.keyboard.is_some())
            {
                return Err(SdkError::InvalidInput(
                    "输入状态消息不能同时携带普通消息内容".into(),
                ));
            }
            if request.keyboard.is_some() && request.markdown.is_none() {
                return Err(SdkError::InvalidInput(
                    "单聊 keyboard 必须与 Markdown 消息一起发送".into(),
                ));
            }
        }
        MessageTarget::Group(_) => {
            if channel_only_content {
                return Err(SdkError::InvalidInput(
                    "@用户、@全体、表情和子频道链接是频道内嵌格式，不能用于群聊".into(),
                ));
            }
            if request.image.is_some()
                || request.embed.is_some()
                || request.ark.is_some()
                || request.input_notify.is_some()
                || request.is_wakeup.is_some()
            {
                return Err(SdkError::InvalidInput(
                    "群聊官方仅支持文本、Markdown 和富媒体消息".into(),
                ));
            }
            if request.markdown.is_some() && request.content.is_some() {
                return Err(SdkError::InvalidInput(
                    "发送 Markdown 时 content 必须为空".into(),
                ));
            }
            if request.force_verify_image_resource.is_some() && request.markdown.is_none() {
                return Err(SdkError::InvalidInput(
                    "force_verify_image_resource 只能与 Markdown 消息一起使用".into(),
                ));
            }
            let primary = [
                request.content.as_ref().map(|_| 0),
                request.markdown.as_ref().map(|_| 2),
                request.media.as_ref().map(|_| 7),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            if primary.len() != 1 {
                return Err(SdkError::InvalidInput(
                    "群聊消息必须且只能包含 content、markdown、media 之一".into(),
                ));
            }
            if let Some(msg_type) = request.msg_type
                && !matches!(msg_type, 0 | 2 | 7)
            {
                return Err(SdkError::InvalidInput(
                    "群聊 msg_type 只能是 0、2 或 7".into(),
                ));
            }
            if request
                .msg_type
                .is_some_and(|msg_type| msg_type != primary[0])
            {
                return Err(SdkError::InvalidInput(
                    "群聊 msg_type 与消息内容字段不匹配".into(),
                ));
            }
            if request.keyboard.is_some() && request.markdown.is_none() {
                return Err(SdkError::InvalidInput(
                    "群聊 keyboard 必须与 Markdown 消息一起发送".into(),
                ));
            }
        }
        MessageTarget::Channel(_) | MessageTarget::Dm(_) => {
            if request.force_verify_image_resource.is_some() {
                return Err(SdkError::InvalidInput(
                    "force_verify_image_resource 仅适用于单聊和群聊 Markdown 消息".into(),
                ));
            }
            if request.keyboard.is_some()
                || request.media.is_some()
                || request.input_notify.is_some()
                || request.is_wakeup.is_some()
            {
                return Err(SdkError::InvalidInput(
                    "频道消息官方不支持 keyboard、media、input_notify 或 is_wakeup".into(),
                ));
            }
            if !has_file_image
                && request.content.is_none()
                && request.embed.is_none()
                && request.ark.is_none()
                && request.image.is_none()
                && request.markdown.is_none()
            {
                return Err(SdkError::InvalidInput(
                    "频道消息必须包含 content、embed、ark、image 或 markdown".into(),
                ));
            }
            // 频道请求不使用 msg_type。
            request.msg_type = None;
            request.msg_seq = None;
        }
    }
    Ok(())
}

/// 富媒体上传的目标会话。
#[derive(Debug, Clone, Copy)]
pub enum MediaTarget<'a> {
    C2c(&'a str),
    Group(&'a str),
}
