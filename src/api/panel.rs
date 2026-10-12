use super::{optional_query, segment};
use crate::{client::QQBotClient, error::Result};
use reqwest::Method;
use serde_json::Value;

/// 面板管理 API。
pub struct PanelApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> PanelApi<'a> {
    /// 按场景和分页参数查询面板；`scope` 支持 `c2c`、`group`、`channel`、`dm`。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn list_with_options(
        &self,
        scope: &str,
        cursor: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Value> {
        let query = optional_query([
            ("scope", Some(scope.to_owned())),
            ("cursor", cursor.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(Method::GET, "/v2/panels", Option::<&Value>::None, &query)
            .await
    }

    /// 获取指定场景的第一页面板。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn list(&self, scope: &str) -> Result<Value> {
        self.list_with_options(scope, None, None).await
    }
    /// 创建面板。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn create(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(Method::POST, "/v2/panels", Some(body))
            .await
    }
    /// 获取面板详情。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn get(&self, panel_id: &str) -> Result<Value> {
        self.client
            .request_json(
                Method::GET,
                &format!("/v2/panels/{}", segment(panel_id)),
                Option::<&Value>::None,
            )
            .await
    }
    /// 更新面板。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn update(&self, panel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!("/v2/panels/{}", segment(panel_id)),
                Some(body),
            )
            .await
    }
    /// 删除面板。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn delete(&self, panel_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/v2/panels/{}", segment(panel_id)),
                None,
            )
            .await
    }
    /// 更新面板投放目标。
    ///
    /// # Errors
    /// 请求失败、鉴权失败、响应解析失败或 QQ 平台返回业务错误时返回 `SdkError`；超出本地限频返回 `SdkError::RateLimited`。
    pub async fn update_target(&self, panel_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::PUT,
                &format!("/v2/panels/{}/target", segment(panel_id)),
                Some(body),
            )
            .await
    }
}
