use crate::{
    error::{Result, SdkError},
    models::{
        Ark, Embed, InputNotify, Keyboard, Media, MessageCard, MessageReference, MessageRequest,
    },
};
use serde_json::{Value, json};

/// 富媒体来源。
#[derive(Debug, Clone)]
pub enum MediaSource {
    /// 网络地址、本地路径或 Base64 数据。
    Location(String),
    /// 原始文件字节。
    Bytes(Vec<u8>),
}

/// 待上传的富媒体。
#[derive(Debug, Clone)]
pub struct MediaSegment {
    pub file_type: u8,
    pub source: MediaSource,
    pub name: Option<String>,
}

/// 可组合消息段。
#[derive(Debug, Clone)]
pub enum MessageSegment {
    /// 完整原生 JSON 请求体，独立发送，不做消息字段转换或校验。
    Raw(Value),
    Card(MessageCard),
    Text(String),
    At(String),
    /// 当前官方格式的用户提及；群聊请在 Markdown 中使用对应标签。
    AtUser(String),
    CommandEnter(String),
    CommandInput {
        text: String,
        show: Option<String>,
        reference: bool,
    },
    Face(u32),
    Image(MediaSegment),
    Video(MediaSegment),
    Audio(MediaSegment),
    File(MediaSegment),
    Markdown(Value),
    InputNotify(InputNotify),
    Keyboard(Keyboard),
    Button(Value),
    Link(String),
    /// 引用已有消息，与被动回复无关。
    Reply(String),
    /// 被动回复元数据。
    PassiveReply {
        msg_id: Option<String>,
        event_id: Option<String>,
    },
    Ark(Ark),
    Embed(Embed),
    Media(Media),
}

impl From<String> for MessageSegment {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for MessageSegment {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// 可发送的消息内容。
#[derive(Debug, Clone)]
pub enum Sendable {
    Request(Box<MessageRequest>),
    Segments(Vec<MessageSegment>),
}

impl Sendable {
    pub(crate) fn build(self) -> Result<BuiltMessage> {
        match self {
            Self::Request(request) => Ok(BuiltMessage {
                request: *request,
                media: None,
                channel_only_content: false,
                raw: None,
            }),
            Self::Segments(segments) => MessageBuilder::new().build_parts(segments),
        }
    }
}

impl From<MessageRequest> for Sendable {
    fn from(value: MessageRequest) -> Self {
        Self::Request(Box::new(value))
    }
}

impl From<&MessageRequest> for Sendable {
    fn from(value: &MessageRequest) -> Self {
        Self::Request(Box::new(value.clone()))
    }
}

impl From<MessageSegment> for Sendable {
    fn from(value: MessageSegment) -> Self {
        Self::Segments(vec![value])
    }
}

impl From<Vec<MessageSegment>> for Sendable {
    fn from(value: Vec<MessageSegment>) -> Self {
        Self::Segments(value)
    }
}

impl<const N: usize> From<[MessageSegment; N]> for Sendable {
    fn from(value: [MessageSegment; N]) -> Self {
        Self::Segments(Vec::from(value))
    }
}

impl From<String> for Sendable {
    fn from(value: String) -> Self {
        Self::Segments(vec![text(value)])
    }
}

impl From<&str> for Sendable {
    fn from(value: &str) -> Self {
        Self::Segments(vec![text(value)])
    }
}

pub(crate) struct BuiltMessage {
    pub raw: Option<Value>,
    pub request: MessageRequest,
    pub media: Option<MediaSegment>,
    pub channel_only_content: bool,
}

/// 消息段构建器。
#[derive(Debug, Default)]
pub struct MessageBuilder {
    request: MessageRequest,
    buttons: Vec<Value>,
    media: Option<MediaSegment>,
    channel_only_content: bool,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 构建无需上传的消息请求。
    ///
    /// # Errors
    /// 消息段包含待上传富媒体或 raw 请求体时返回 `SdkError::InvalidInput`，需直接通过 `send_*` 发送。
    pub fn build<I, S>(self, segments: I) -> Result<MessageRequest>
    where
        I: IntoIterator<Item = S>,
        S: Into<MessageSegment>,
    {
        let built = self.build_parts(segments.into_iter().map(Into::into))?;
        if built.raw.is_some() {
            return Err(SdkError::InvalidInput(
                "raw 不能转换为 MessageRequest，请直接交给 send/reply 发送".into(),
            ));
        }
        if built.media.is_some() {
            return Err(SdkError::InvalidInput(
                "图片、视频、音频或文件消息段需要通过 send_* 方法发送，以便 SDK 自动上传".into(),
            ));
        }
        Ok(built.request)
    }

    pub(crate) fn build_parts(
        mut self,
        segments: impl IntoIterator<Item = MessageSegment>,
    ) -> Result<BuiltMessage> {
        let mut segments = segments.into_iter();
        if let Some(first) = segments.next() {
            if let MessageSegment::Raw(body) = first {
                if segments.next().is_some() {
                    return Err(SdkError::InvalidInput(
                        "raw 是完整请求体，不能与其他消息段混用".into(),
                    ));
                }
                if !body.is_object() {
                    return Err(SdkError::InvalidInput("raw 必须接收 JSON 对象".into()));
                }
                return Ok(BuiltMessage {
                    raw: Some(body),
                    request: MessageRequest::default(),
                    media: None,
                    channel_only_content: false,
                });
            }
            self.push(first)?;
            for segment in segments {
                self.push(segment)?;
            }
        }
        self.finish_buttons();
        Ok(BuiltMessage {
            raw: None,
            request: self.request,
            media: self.media,
            channel_only_content: self.channel_only_content,
        })
    }

    fn push(&mut self, segment: MessageSegment) -> Result<()> {
        match segment {
            MessageSegment::Raw(_) => {
                return Err(SdkError::InvalidInput(
                    "raw 是完整请求体，不能与其他消息段混用".into(),
                ));
            }
            MessageSegment::Card(card) => {
                card.validate()?;
                self.request.card = Some(card);
                self.request.msg_type = Some(8);
            }
            MessageSegment::Text(text) => self.request.content_mut().push_str(&text),
            MessageSegment::At(user_id) => {
                self.channel_only_content = true;
                if user_id == "all" {
                    self.request.content_mut().push_str("@everyone");
                } else {
                    self.request
                        .content_mut()
                        .push_str(&format!("<@!{user_id}>"));
                }
            }
            MessageSegment::AtUser(user_id) => {
                if user_id.is_empty() || user_id.contains(['"', '<', '>', '&', '\'']) {
                    return Err(SdkError::InvalidInput(
                        "提及用户 ID 不能为空或包含标签特殊字符".into(),
                    ));
                }
                self.request
                    .content_mut()
                    .push_str(&format!("<qqbot-at-user id=\"{user_id}\" />"));
            }
            MessageSegment::CommandEnter(text) => {
                self.set_command_markdown(
                    &text,
                    None,
                    &format!(
                        "<qqbot-cmd-enter text=\"{}\" />",
                        urlencoding::encode(&text)
                    ),
                )?;
            }
            MessageSegment::CommandInput {
                text,
                show,
                reference,
            } => {
                let show_attribute = show
                    .as_ref()
                    .map(|show| format!(" show=\"{}\"", urlencoding::encode(show)))
                    .unwrap_or_default();
                self.set_command_markdown(
                    &text,
                    show.as_deref(),
                    &format!(
                        "<qqbot-cmd-input text=\"{}\"{show_attribute} reference=\"{reference}\" />",
                        urlencoding::encode(&text)
                    ),
                )?;
            }
            MessageSegment::Face(id) => self.push_channel_inline(&format!("<emoji:{id}>")),
            MessageSegment::Link(channel_id) => {
                self.push_channel_inline(&format!("<#{channel_id}>"))
            }
            MessageSegment::Reply(message_id) => {
                self.request.message_reference = Some(MessageReference {
                    message_id: Some(message_id),
                    ignore_get_message_error: None,
                });
            }
            MessageSegment::PassiveReply { msg_id, event_id } => {
                if msg_id.is_none() == event_id.is_none() {
                    return Err(SdkError::InvalidInput(
                        "被动回复必须且只能提供 msg_id 或 event_id".into(),
                    ));
                }
                self.request.msg_id = msg_id;
                self.request.event_id = event_id;
            }
            MessageSegment::Markdown(markdown) => {
                self.request.markdown = Some(markdown);
                self.request.msg_type = Some(2);
            }
            MessageSegment::InputNotify(input_notify) => {
                self.request.input_notify = Some(input_notify);
                self.request.msg_type = Some(6);
            }
            MessageSegment::Keyboard(keyboard) => {
                self.request.keyboard = Some(keyboard);
                self.request.msg_type = Some(2);
            }
            MessageSegment::Button(button) => self.buttons.push(button),
            MessageSegment::Ark(ark) => {
                self.request.ark = Some(ark);
            }
            MessageSegment::Embed(embed) => {
                self.request.embed = Some(embed);
            }
            MessageSegment::Media(media) => {
                self.request.media = Some(media);
                self.request.msg_type = Some(7);
            }
            MessageSegment::Image(media)
            | MessageSegment::Video(media)
            | MessageSegment::Audio(media)
            | MessageSegment::File(media) => {
                if self.media.is_some() || self.request.media.is_some() {
                    return Err(SdkError::InvalidInput(
                        "一条消息只能包含一个待上传的富媒体消息段".into(),
                    ));
                }
                self.media = Some(media);
                self.request.msg_type = Some(7);
            }
        }
        Ok(())
    }

    fn finish_buttons(&mut self) {
        if self.buttons.is_empty() {
            return;
        }

        let rows = self
            .buttons
            .chunks(5)
            .map(|buttons| json!({ "buttons": buttons }))
            .collect::<Vec<_>>();
        self.request.keyboard = Some(Keyboard {
            content: Some(json!({ "rows": rows })),
            ..Default::default()
        });
        self.request.msg_type = Some(2);
    }

    fn set_command_markdown(&mut self, text: &str, show: Option<&str>, tag: &str) -> Result<()> {
        if text.is_empty()
            || text.chars().count() > 100
            || show.is_some_and(|show| show.chars().count() > 100)
        {
            return Err(SdkError::InvalidInput(
                "指令 text 必须为 1–100 字符，show 最多 100 字符".into(),
            ));
        }
        if self.request.markdown.is_some() {
            return Err(SdkError::InvalidInput(
                "指令消息段不能覆盖已有 Markdown；组合内容请在 markdown 中写入标签".into(),
            ));
        }
        self.request.markdown = Some(json!({ "content": tag }));
        self.request.msg_type = Some(2);
        Ok(())
    }

    fn push_channel_inline(&mut self, value: &str) {
        self.channel_only_content = true;
        self.request.content_mut().push_str(value);
    }
}

impl MessageRequest {
    /// 创建纯文本请求。
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            ..Default::default()
        }
    }

    /// 从无需上传的消息段构建请求。
    ///
    /// # Errors
    /// 消息段包含待上传富媒体或 raw 请求体时返回 `SdkError::InvalidInput`，需直接通过 `send_*` 发送。
    pub fn from_segments<I, S>(segments: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<MessageSegment>,
    {
        MessageBuilder::new().build(segments)
    }

    fn content_mut(&mut self) -> &mut String {
        self.content.get_or_insert_with(String::new)
    }
}

pub fn text(value: impl Into<String>) -> MessageSegment {
    MessageSegment::Text(value.into())
}

/// 原样发送完整原生 JSON 对象，不推断类型、上传媒体或校验消息字段。
///
/// 必须独立发送；即使使用 reply，msg_id/event_id/msg_seq 也由调用方填写。
pub fn raw(body: Value) -> MessageSegment {
    MessageSegment::Raw(body)
}

pub fn at(user_id: impl Into<String>) -> MessageSegment {
    MessageSegment::At(user_id.into())
}

pub fn at_all() -> MessageSegment {
    MessageSegment::At("all".into())
}

/// 当前官方的文字子频道用户提及格式。群聊提及用户仅支持 Markdown。
pub fn at_user(user_openid: impl Into<String>) -> MessageSegment {
    MessageSegment::AtUser(user_openid.into())
}

/// 群聊／文字子频道 @全体成员，权限和次数由平台检查。
pub fn at_everyone() -> MessageSegment {
    text("<qqbot-at-everyone />")
}

/// 单聊 Markdown 回车指令，text 自动 URL 编码，最多 100 字符。
pub fn cmd_enter(text: impl Into<String>) -> MessageSegment {
    MessageSegment::CommandEnter(text.into())
}

/// Markdown 参数指令，text/show 自动 URL 编码，reference 控制输入框引用。
pub fn cmd_input(text: impl Into<String>, show: Option<String>, reference: bool) -> MessageSegment {
    MessageSegment::CommandInput {
        text: text.into(),
        show,
        reference,
    }
}

/// 群聊 @全体成员。机器人需为群管理员，每群每天仅可发送一次。
/// Markdown 中可直接使用 `<qqbot-at-everyone />`。
pub fn group_at_all() -> MessageSegment {
    at_everyone()
}

/// 单聊或群聊图文卡片，接收 `TuwenContent` 或 `MessageCard`。
/// 发送时校验标题、描述和 HTTPS 链接。
pub fn card(value: impl Into<MessageCard>) -> MessageSegment {
    MessageSegment::Card(value.into())
}

pub fn face(id: u32) -> MessageSegment {
    MessageSegment::Face(id)
}

pub fn image(file: impl Into<String>) -> MessageSegment {
    media_location(1, file)
}

pub fn image_bytes(data: impl AsRef<[u8]>) -> MessageSegment {
    media_bytes(1, data)
}

pub fn video(file: impl Into<String>) -> MessageSegment {
    media_location(2, file)
}

pub fn video_bytes(data: impl AsRef<[u8]>) -> MessageSegment {
    media_bytes(2, data)
}

pub fn audio(file: impl Into<String>) -> MessageSegment {
    media_location(3, file)
}

pub fn audio_bytes(data: impl AsRef<[u8]>) -> MessageSegment {
    media_bytes(3, data)
}

/// 创建文件消息段。
pub fn file(file: impl Into<String>) -> MessageSegment {
    media_location(4, file)
}

/// 从字节创建文件消息段。
pub fn file_bytes(data: impl AsRef<[u8]>) -> MessageSegment {
    media_bytes(4, data)
}

pub fn markdown(content: impl Into<String>) -> MessageSegment {
    MessageSegment::Markdown(json!({ "content": content.into() }))
}

pub fn markdown_template(
    template_id: u32,
    params: impl IntoIterator<Item = Value>,
) -> MessageSegment {
    MessageSegment::Markdown(json!({
        "template_id": template_id,
        "params": params.into_iter().collect::<Vec<_>>(),
    }))
}

pub fn markdown_value(value: Value) -> MessageSegment {
    MessageSegment::Markdown(value)
}

pub fn input_notify(input_type: u8, input_second: u32) -> MessageSegment {
    MessageSegment::InputNotify(InputNotify {
        input_type: Some(input_type),
        input_second: Some(input_second),
    })
}

pub fn keyboard(id: impl Into<String>) -> MessageSegment {
    MessageSegment::Keyboard(Keyboard {
        id: Some(id.into()),
        ..Default::default()
    })
}

pub fn keyboard_content(content: Value) -> MessageSegment {
    MessageSegment::Keyboard(Keyboard {
        content: Some(content),
        ..Default::default()
    })
}

pub fn button(data: Value) -> MessageSegment {
    MessageSegment::Button(data)
}

pub fn link(channel_id: impl Into<String>) -> MessageSegment {
    MessageSegment::Link(channel_id.into())
}

pub fn reply(message_id: impl Into<String>) -> MessageSegment {
    MessageSegment::Reply(message_id.into())
}

pub fn reply_to(message_id: impl Into<String>) -> MessageSegment {
    MessageSegment::PassiveReply {
        msg_id: Some(message_id.into()),
        event_id: None,
    }
}

pub fn reply_event(event_id: impl Into<String>) -> MessageSegment {
    MessageSegment::PassiveReply {
        msg_id: None,
        event_id: Some(event_id.into()),
    }
}

pub fn ark(template_id: u32, kv: Vec<Value>) -> MessageSegment {
    MessageSegment::Ark(Ark { template_id, kv })
}

pub fn embed(
    title: impl Into<String>,
    prompt: impl Into<String>,
    thumbnail: Value,
    fields: Vec<Value>,
) -> MessageSegment {
    MessageSegment::Embed(Embed {
        title: Some(title.into()),
        prompt: Some(prompt.into()),
        thumbnail: Some(thumbnail),
        fields: Some(fields),
        ..Default::default()
    })
}

pub fn embed_value(value: Embed) -> MessageSegment {
    MessageSegment::Embed(value)
}

pub fn media(value: Media) -> MessageSegment {
    MessageSegment::Media(value)
}

fn media_location(file_type: u8, file: impl Into<String>) -> MessageSegment {
    let media = MediaSegment {
        file_type,
        source: MediaSource::Location(file.into()),
        name: None,
    };
    match file_type {
        1 => MessageSegment::Image(media),
        2 => MessageSegment::Video(media),
        3 => MessageSegment::Audio(media),
        _ => MessageSegment::File(media),
    }
}

fn media_bytes(file_type: u8, data: impl AsRef<[u8]>) -> MessageSegment {
    let media = MediaSegment {
        file_type,
        source: MediaSource::Bytes(data.as_ref().to_vec()),
        name: None,
    };
    match file_type {
        1 => MessageSegment::Image(media),
        2 => MessageSegment::Video(media),
        3 => MessageSegment::Audio(media),
        _ => MessageSegment::File(media),
    }
}
