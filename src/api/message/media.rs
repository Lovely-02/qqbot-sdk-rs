//! 单聊/群聊富媒体上传实现。

use super::{MediaTarget, MessageApi, is_http_url};
use crate::{
    error::{Result, SdkError},
    models::{Media, Message},
    segment::Sendable,
};
use md5::{Digest, Md5};
use reqwest::Method;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::api::segment;

impl<'a> MessageApi<'a> {
    /// 类型化富媒体上传，保留 file_name、file_data 等官方字段。
    ///
    /// # Errors
    /// 鉴权、HTTP、限频、序列化或平台业务失败时返回 `SdkError`。
    pub async fn upload_media_with(
        &self,
        target: MediaTarget<'_>,
        body: &crate::models::MediaUploadRequest,
    ) -> Result<Media> {
        let path = match target {
            MediaTarget::C2c(id) => format!("/v2/users/{}/files", segment(id)),
            MediaTarget::Group(id) => format!("/v2/groups/{}/files", segment(id)),
        };
        self.client
            .request_json(Method::POST, &path, Some(body))
            .await
    }

    /// 按官方分片流程上传富媒体并返回 `file_info`。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
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
        if data.is_empty() {
            return Err(SdkError::InvalidInput("上传文件不能为空".into()));
        }
        let target_id = match target {
            MediaTarget::C2c(id) | MediaTarget::Group(id) => id,
        };
        let file_name = format!("upload.{}", media_extension(file_type));
        let md5 = hex::encode(Md5::digest(data));
        let sha1 = {
            use sha1::Digest as Sha1Digest;
            let mut hasher = sha1::Sha1::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        };
        let md5_10m = hex::encode(Md5::digest(&data[..data.len().min(10_002_432)]));
        let prepare_path = match target {
            MediaTarget::C2c(_) => format!(
                "/v2/users/{target_id}/upload_prepare",
                target_id = segment(target_id)
            ),
            MediaTarget::Group(_) => format!(
                "/v2/groups/{target_id}/upload_prepare",
                target_id = segment(target_id)
            ),
        };
        let prepare_body = json!({
            "file_type": file_type,
            "file_size": data.len().to_string(),
            "file_name": file_name,
            "md5": md5,
            "sha1": sha1,
            "md5_10m": md5_10m,
        });
        let prepare: UploadPrepareResponse = super::upload::UploadPolicy::from_config(None)?
            .run_locally_limited(|| {
                self.client
                    .request_json(Method::POST, &prepare_path, Some(&prepare_body))
            })
            .await?;
        let ranges = prepare.ranges(data.len())?;
        let policy = super::upload::UploadPolicy::from_config(prepare.upload_config.as_ref())?;
        let finish_path = match target {
            MediaTarget::C2c(_) => format!("/v2/users/{}/upload_part_finish", segment(target_id)),
            MediaTarget::Group(_) => {
                format!("/v2/groups/{}/upload_part_finish", segment(target_id))
            }
        };
        use futures_util::{StreamExt, TryStreamExt, stream};
        let uploads = ranges
            .into_iter()
            .map(|(part, range)| {
                let finish_path = &finish_path;
                let upload_id = &prepare.upload_id;
                let policy = &policy;
                async move {
                    let chunk = &data[range];
                    policy
                        .run(|| async {
                            self.client
                                .http
                                .put(&part.presigned_url)
                                .body(chunk.to_vec())
                                .send()
                                .await?
                                .error_for_status()?;
                            Ok(())
                        })
                        .await?;
                    let body = json!({
                        "upload_id": upload_id,
                        "part_index": part.index,
                        "block_size": chunk.len().to_string(),
                        "md5": hex::encode(Md5::digest(chunk)),
                    });
                    policy
                        .run(|| async {
                            self.client
                                .request_empty(Method::POST, finish_path, Some(&body))
                                .await
                        })
                        .await
                }
            })
            .collect::<Vec<_>>();
        stream::iter(uploads)
            .buffer_unordered(policy.concurrency)
            .try_collect::<Vec<_>>()
            .await?;
        let path = match target {
            MediaTarget::C2c(_) => format!("/v2/users/{}/files", segment(target_id)),
            MediaTarget::Group(_) => format!("/v2/groups/{}/files", segment(target_id)),
        };
        let body = json!({
            "file_type": file_type,
            "file_name": file_name,
            "upload_id": &prepare.upload_id,
            "srv_send_msg": srv_send_msg,
        });
        policy
            .run_locally_limited(|| self.client.request_json(Method::POST, &path, Some(&body)))
            .await
    }

    /// 使用官方 URL 直传格式上传富媒体。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
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
        if !is_http_url(url) {
            return Err(SdkError::InvalidInput(
                "富媒体 URL 必须以 http:// 或 https:// 开头".into(),
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
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn upload_media_request(
        &self,
        target: MediaTarget<'_>,
        body: &Value,
    ) -> Result<Media> {
        let path = match target {
            MediaTarget::C2c(openid) => format!("/v2/users/{}/files", segment(openid)),
            MediaTarget::Group(openid) => format!("/v2/groups/{}/files", segment(openid)),
        };
        self.client
            .request_json(Method::POST, &path, Some(body))
            .await
    }

    /// 上传富媒体并立即发送，适用于图片、视频、语音和文件。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn send_media(
        &self,
        target: MediaTarget<'_>,
        file_type: u8,
        data: &[u8],
        message: impl Into<Sendable>,
    ) -> Result<Message> {
        let built = message.into().build()?;
        if built.raw.is_some() {
            return Err(SdkError::InvalidInput(
                "raw 不支持 send_media，请直接发送原生请求体".into(),
            ));
        }
        if built.media.is_some() || built.request.media.is_some() {
            return Err(SdkError::InvalidInput(
                "send_media 已经接收上传数据，不要再传入图片、视频、音频或富媒体消息段".into(),
            ));
        }
        let media = self.upload_media(target, file_type, data, false).await?;
        let mut request = built.request;
        request.media = Some(media);
        request.msg_type = Some(7);
        match target {
            MediaTarget::C2c(id) => self.send_c2c(id, request).await,
            MediaTarget::Group(id) => self.send_group(id, request).await,
        }
    }

    /// 使用 URL 直传并立即发送富媒体消息。
    ///
    /// # Errors
    /// 消息段无效、请求字段组合不合法时返回 `SdkError::InvalidInput`；发送或上传失败时返回 HTTP、鉴权、序列化或平台业务错误。
    pub async fn send_media_url(
        &self,
        target: MediaTarget<'_>,
        file_type: u8,
        url: &str,
        message: impl Into<Sendable>,
    ) -> Result<Message> {
        let built = message.into().build()?;
        if built.raw.is_some() {
            return Err(SdkError::InvalidInput(
                "raw 不支持 send_media_url，请直接发送原生请求体".into(),
            ));
        }
        if built.media.is_some() || built.request.media.is_some() {
            return Err(SdkError::InvalidInput(
                "send_media_url 已经接收上传地址，不要再传入图片、视频、音频或富媒体消息段".into(),
            ));
        }
        let media = self.upload_media_url(target, file_type, url, false).await?;
        let mut request = built.request;
        request.media = Some(media);
        request.msg_type = Some(7);
        match target {
            MediaTarget::C2c(id) => self.send_c2c(id, request).await,
            MediaTarget::Group(id) => self.send_group(id, request).await,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadPrepareResponse {
    pub upload_id: String,
    #[serde(default, deserialize_with = "deserialize_u64_string")]
    pub block_size: u64,
    #[serde(default)]
    pub upload_config: Option<UploadConfig>,
    #[serde(default)]
    pub parts: Vec<UploadPart>,
}

#[derive(Debug, Deserialize)]
pub struct UploadPart {
    pub index: u32,
    pub presigned_url: String,
    #[serde(deserialize_with = "deserialize_u64_string")]
    pub block_size: u64,
}

/// 平台下发的分片上传配置；可通过 users/groups 的 upload_prepare 原始响应获取。
#[derive(Debug, Deserialize)]
pub struct UploadConfig {
    pub concurrency: Option<u32>,
    pub retry_timeout: Option<u32>,
    pub retry_delay: Option<u32>,
}

fn deserialize_u64_string<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| serde::de::Error::custom("必须为无符号整数")),
        Value::String(value) => value.parse::<u64>().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("必须为数字或数字字符串")),
    }
}

fn media_extension(file_type: u8) -> &'static str {
    match file_type {
        1 => "jpg",
        2 => "mp4",
        3 => "silk",
        _ => "bin",
    }
}

impl UploadPrepareResponse {
    fn ranges(&self, data_len: usize) -> Result<Vec<(&UploadPart, std::ops::Range<usize>)>> {
        let mut parts = self.parts.iter().collect::<Vec<_>>();
        parts.sort_unstable_by_key(|part| part.index);
        let mut offset = 0usize;
        let mut ranges = Vec::with_capacity(parts.len());
        for (index, part) in parts.into_iter().enumerate() {
            if usize::try_from(part.index).ok() != Some(index)
                || part.block_size == 0
                || offset >= data_len
            {
                return Err(SdkError::InvalidInput("预上传分片序号或大小无效".into()));
            }
            let size = usize::try_from(part.block_size)
                .map_err(|_| SdkError::InvalidInput("分片大小超出可表示范围".into()))?;
            let end = offset.saturating_add(size).min(data_len);
            ranges.push((part, offset..end));
            offset = end;
        }
        if ranges.is_empty() || offset != data_len {
            return Err(SdkError::InvalidInput(
                "预上传分片总大小与文件大小不匹配".into(),
            ));
        }
        Ok(ranges)
    }
}
