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
        let prepare: UploadPrepareResponse = self
            .client
            .request_json(
                Method::POST,
                &prepare_path,
                Some(&json!({
                    "file_type": file_type,
                    "file_size": data.len().to_string(),
                    "file_name": file_name,
                    "md5": md5,
                    "sha1": sha1,
                    "md5_10m": md5_10m,
                })),
            )
            .await?;
        if prepare.parts.is_empty() {
            return Err(SdkError::InvalidInput(
                "官方分片上传未返回任何上传分片".into(),
            ));
        }
        let data_len = u64::try_from(data.len())
            .map_err(|_| SdkError::InvalidInput("上传文件大小超出平台可处理范围".into()))?;
        let mut offset = 0u64;
        for part in &prepare.parts {
            let start = offset;
            let end = start.saturating_add(part.block_size).min(data_len);
            if start >= data_len || end <= start {
                return Err(SdkError::InvalidInput(
                    "官方分片上传返回的分片范围无效".into(),
                ));
            }
            let start_offset = usize::try_from(start).map_err(|_| {
                SdkError::InvalidInput("官方分片上传返回的分片偏移超出平台可处理范围".into())
            })?;
            let end_offset = usize::try_from(end).map_err(|_| {
                SdkError::InvalidInput("官方分片上传返回的分片偏移超出平台可处理范围".into())
            })?;
            self.client
                .http
                .put(&part.presigned_url)
                .body(data[start_offset..end_offset].to_vec())
                .send()
                .await?
                .error_for_status()?;
            let finish_path = match target {
                MediaTarget::C2c(_) => format!(
                    "/v2/users/{target_id}/upload_part_finish",
                    target_id = segment(target_id)
                ),
                MediaTarget::Group(_) => format!(
                    "/v2/groups/{target_id}/upload_part_finish",
                    target_id = segment(target_id)
                ),
            };
            let part_md5 = hex::encode(Md5::digest(&data[start_offset..end_offset]));
            let _: Value = self
                .client
                .request_json(
                    Method::POST,
                    &finish_path,
                    Some(&json!({
                        "upload_id": &prepare.upload_id,
                        "part_index": part.index,
                        "block_size": (end - start).to_string(),
                        "md5": part_md5,
                    })),
                )
                .await?;
            offset = end;
        }
        if offset != data_len {
            return Err(SdkError::InvalidInput(
                "官方分片上传返回的总分片大小与文件大小不一致".into(),
            ));
        }
        let path = match target {
            MediaTarget::C2c(_) => format!("/v2/users/{}/files", segment(target_id)),
            MediaTarget::Group(_) => format!("/v2/groups/{}/files", segment(target_id)),
        };
        self.client
            .request_json(
                Method::POST,
                &path,
                Some(&json!({
                    "file_type": file_type,
                    "file_name": file_name,
                    "upload_id": &prepare.upload_id,
                    "srv_send_msg": srv_send_msg,
                })),
            )
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
struct UploadPrepareResponse {
    upload_id: String,
    #[serde(default)]
    parts: Vec<UploadPart>,
}

#[derive(Debug, Deserialize)]
struct UploadPart {
    index: u32,
    presigned_url: String,
    #[serde(deserialize_with = "deserialize_u64_string")]
    block_size: u64,
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
