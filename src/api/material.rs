//! Markdown 图片素材管理，与会话富媒体 `file_info` 上传相互独立。

use crate::{
    client::QQBotClient,
    error::{Result, SdkError},
};
use md5::{Digest, Md5};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 申请图片素材上传的参数。扩展名必须与 MIME 类型一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUploadRequest {
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub md5: String,
    pub sha1: String,
}

impl ImageUploadRequest {
    /// 从文件字节计算平台需要的校验值。
    ///
    /// # Errors
    /// 文件为空、超过 10 MiB、扩展名或 MIME 类型不受支持时返回 `SdkError::InvalidInput`。
    pub fn from_bytes(file_name: &str, content_type: &str, data: &[u8]) -> Result<Self> {
        let request = Self {
            file_name: file_name.into(),
            file_size: u64::try_from(data.len())
                .map_err(|_| SdkError::InvalidInput("图片大小超出范围".into()))?,
            content_type: content_type.into(),
            md5: hex::encode(Md5::digest(data)),
            sha1: hex::encode(sha1::Sha1::digest(data)),
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> Result<()> {
        let extension = self
            .file_name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase());
        let valid_format = matches!(
            (self.content_type.as_str(), extension.as_deref()),
            ("image/png", Some("png"))
                | ("image/jpeg", Some("jpg" | "jpeg"))
                | ("image/gif", Some("gif"))
                | ("image/webp", Some("webp"))
        );
        if !valid_format || self.file_size == 0 || self.file_size > 10 * 1024 * 1024 {
            return Err(SdkError::InvalidInput("素材必须为非空且不超过 10 MiB 的 png/jpeg/gif/webp 图片，扩展名与 Content-Type 必须一致".into()));
        }
        if self.md5.len() != 32
            || !self.md5.bytes().all(|b| b.is_ascii_hexdigit())
            || self.sha1.len() != 40
            || !self.sha1.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(SdkError::InvalidInput(
                "素材 MD5/SHA1 必须为 32/40 位十六进制字符串".into(),
            ));
        }
        Ok(())
    }
}

/// 上传任务有效期为 4 小时；预签名 URL 有效期由 `expires_in` 指定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUploadPreparation {
    pub upload_id: String,
    pub presigned_url: String,
    pub expires_in: u32,
}

/// 完成图片上传的参数；`md5` 使用 COS PUT 响应 ETag 去掉引号后的值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUploadCompletion {
    pub upload_id: String,
    pub md5: String,
}

/// 上传完成的素材。持久化 `url` 用于 Markdown，`preview_url` 仅用于临时预览。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialImage {
    pub image_id: String,
    pub url: String,
    /// 每次被消息引用后重新计算的有效期窗口，单位秒。
    pub ttl: u32,
    pub preview_url: String,
}

/// 查询返回的有效图片。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialImageEntry {
    pub image_id: String,
    pub url: String,
    pub file_name: String,
    pub preview_url: String,
    pub created_at: i64,
    pub expires_at: i64,
}

/// 素材按上传时间倒序返回；`next_cursor` 为空表示末页。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialImagePage {
    pub images: Vec<MaterialImageEntry>,
    pub next_cursor: String,
    pub total: u64,
}

/// 机器人级 Markdown 图片素材 API。
pub struct MaterialApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl MaterialApi<'_> {
    /// 申请图片上传任务。
    ///
    /// # Errors
    /// 参数不合法、鉴权、限频、HTTP 请求或平台业务失败时返回 `SdkError`。
    pub async fn prepare(&self, request: &ImageUploadRequest) -> Result<ImageUploadPreparation> {
        request.validate()?;
        self.client
            .request_json(Method::POST, "/v2/media-images/prepare", Some(request))
            .await
    }

    /// 完成上传，获取 `qqbot-media://` 引用。
    ///
    /// # Errors
    /// 鉴权、限频、HTTP 请求、响应解析或平台业务失败时返回 `SdkError`。
    pub async fn complete(&self, request: &ImageUploadCompletion) -> Result<MaterialImage> {
        self.client
            .request_json(Method::POST, "/v2/media-images/complete", Some(request))
            .await
    }

    /// 查询有效素材；首页传 `None`，之后使用 `next_cursor` 翻页。
    ///
    /// # Errors
    /// 鉴权、限频、HTTP 请求、响应解析或平台业务失败时返回 `SdkError`。
    pub async fn list(&self, cursor: Option<&str>) -> Result<MaterialImagePage> {
        let query = super::optional_query([("cursor", cursor.map(str::to_owned))]);
        self.client
            .request_json_query(
                Method::GET,
                "/v2/media-images",
                Option::<&Value>::None,
                &query,
            )
            .await
    }

    /// 删除素材并释放配额，引用立即失效，已发送消息不受影响。
    ///
    /// # Errors
    /// 鉴权、限频、HTTP 请求、响应解析或平台业务失败时返回 `SdkError`。
    pub async fn delete(&self, image_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/v2/media-images/{}/delete", super::segment(image_id)),
                None,
            )
            .await
    }

    /// 申请、直传并完成图片上传，返回可用于 Markdown 的素材引用。
    ///
    /// # Errors
    /// 文件参数无效、PUT 未返回 ETag、鉴权、HTTP、限频或平台业务失败时返回 `SdkError`。
    pub async fn upload(
        &self,
        file_name: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<MaterialImage> {
        let request = ImageUploadRequest::from_bytes(file_name, content_type, data)?;
        let preparation = self.prepare(&request).await?;
        // COS 的签名绑定 Content-Type；此请求不附带 QQBot 鉴权头。
        let response = self
            .client
            .http
            .put(&preparation.presigned_url)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .body(data.to_vec())
            .send()
            .await?
            .error_for_status()?;
        let etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| SdkError::InvalidInput("素材上传响应缺少有效 ETag".into()))?;
        self.complete(&ImageUploadCompletion {
            upload_id: preparation.upload_id,
            md5: etag.trim_matches('"').to_owned(),
        })
        .await
    }
}
