use crate::{
    error::{Result, SdkError},
    models::{Ark, Embed, InputNotify, Keyboard, Media, MessageReference, MessageRequest},
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
    Text(String),
    At(String),
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
    pub fn build<I, S>(self, segments: I) -> Result<MessageRequest>
    where
        I: IntoIterator<Item = S>,
        S: Into<MessageSegment>,
    {
        let built = self.build_parts(segments.into_iter().map(Into::into))?;
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
        for segment in segments {
            self.push(segment)?;
        }
        self.finish_buttons();
        Ok(BuiltMessage {
            request: self.request,
            media: self.media,
            channel_only_content: self.channel_only_content,
        })
    }

    fn push(&mut self, segment: MessageSegment) -> Result<()> {
        match segment {
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
                if self.request.msg_id.is_some() {
                    self.request.msg_seq.get_or_insert(1);
                }
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

pub fn at(user_id: impl Into<String>) -> MessageSegment {
    MessageSegment::At(user_id.into())
}

pub fn at_all() -> MessageSegment {
    MessageSegment::At("all".into())
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
