use super::{optional_query, segment};
use crate::{client::QQBotClient, error::Result, models::Group};
use reqwest::Method;
use serde_json::Value;

/// 群相关 API。
pub struct GroupApi<'a> {
    pub(crate) client: &'a QQBotClient,
}

impl<'a> GroupApi<'a> {
    /// 获取群信息。
    pub async fn get(&self, openid: &str) -> Result<Group> {
        self.client
            .request_json(
                Method::GET,
                &format!("/v2/groups/{}/info", segment(openid)),
                Option::<&serde_json::Value>::None,
            )
            .await
    }

    /// 获取机器人在群内的状态。
    pub async fn bot_state(&self, group_openid: &str) -> Result<Value> {
        self.get_value(&format!("/v2/groups/{}/bot_state", segment(group_openid)))
            .await
    }

    /// 上传群聊富媒体文件。请求体可使用官方的 URL 直传或分片上传字段。
    pub async fn upload_file(&self, group_openid: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/groups/{}/files", segment(group_openid)),
                Some(body),
            )
            .await
    }
    /// 获取群成员列表。
    pub async fn members(&self, group_openid: &str) -> Result<Value> {
        self.get_value(&format!("/v2/groups/{}/members", segment(group_openid)))
            .await
    }

    /// 获取群成员列表并传入官方分页游标。
    pub async fn members_page(&self, group_openid: &str, cursor: Option<&str>) -> Result<Value> {
        let query = optional_query([("cursor", cursor.map(str::to_owned))]);
        self.client
            .request_json_query(
                Method::GET,
                &format!("/v2/groups/{}/members", segment(group_openid)),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 获取单个群成员。
    pub async fn member(&self, group_openid: &str, member_openid: &str) -> Result<Value> {
        self.get_value(&format!(
            "/v2/groups/{}/members/{}",
            segment(group_openid),
            segment(member_openid)
        ))
        .await
    }
    /// 获取加群申请列表。
    pub async fn join_request_list(&self, group_openid: &str) -> Result<Value> {
        self.get_value(&format!(
            "/v2/groups/{}/join_request_list",
            segment(group_openid)
        ))
        .await
    }
    /// 获取群禁言/限制聊天设置。
    pub async fn restrict_chat_setting(&self, group_openid: &str) -> Result<Value> {
        self.get_value(&format!(
            "/v2/groups/{}/restrict_chat_setting",
            segment(group_openid)
        ))
        .await
    }
    /// 修改群禁言/限制聊天设置。
    pub async fn update_restrict_chat_setting(
        &self,
        group_openid: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/groups/{}/restrict_chat_setting", segment(group_openid)),
                Some(body),
            )
            .await
    }

    /// 获取加群申请列表并传入分页参数。
    pub async fn join_request_list_page(
        &self,
        group_openid: &str,
        cursor: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Value> {
        let query = optional_query([
            ("cursor", cursor.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                &format!("/v2/groups/{}/join_request_list", segment(group_openid)),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 批量移除群成员。
    pub async fn batch_remove_members(&self, group_openid: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/groups/{}/batch_remove_members", segment(group_openid)),
                Some(body),
            )
            .await
    }
    /// 处理指定成员的加群申请。
    pub async fn approve_join_request(
        &self,
        group_openid: &str,
        member_openid: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!(
                    "/v2/groups/{}/approval_join_request/{}",
                    segment(group_openid),
                    segment(member_openid)
                ),
                Some(body),
            )
            .await
    }
    /// 获取群成员黑名单。
    pub async fn member_blacklist(&self, group_openid: &str) -> Result<Value> {
        self.get_value(&format!(
            "/v2/groups/{}/member_blacklist",
            segment(group_openid)
        ))
        .await
    }

    /// 获取群成员黑名单并传入官方分页参数。
    pub async fn member_blacklist_page(
        &self,
        group_openid: &str,
        cursor: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Value> {
        let query = optional_query([
            ("cursor", cursor.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.client
            .request_json_query(
                Method::GET,
                &format!("/v2/groups/{}/member_blacklist", segment(group_openid)),
                Option::<&Value>::None,
                &query,
            )
            .await
    }
    /// 更新群成员黑名单。
    pub async fn update_member_blacklist(&self, group_openid: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/groups/{}/member_blacklist", segment(group_openid)),
                Some(body),
            )
            .await
    }
    /// 创建入群审批策略。
    pub async fn create_join_approval_strategy(&self, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                "/v2/groups/join_approval_strategy",
                Some(body),
            )
            .await
    }
    /// 查询入群审批策略。
    pub async fn list_join_approval_strategies(&self) -> Result<Value> {
        self.get_value("/v2/groups/join_approval_strategy").await
    }
    /// 更新入群审批策略。
    pub async fn update_join_approval_strategy(
        &self,
        strategy_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::PATCH,
                &format!("/v2/groups/join_approval_strategy/{}", segment(strategy_id)),
                Some(body),
            )
            .await
    }
    /// 删除入群审批策略。
    pub async fn delete_join_approval_strategy(&self, strategy_id: &str) -> Result<()> {
        self.client
            .request_empty::<Value>(
                Method::DELETE,
                &format!("/v2/groups/join_approval_strategy/{}", segment(strategy_id)),
                None,
            )
            .await
    }
    /// 执行入群审批策略。
    pub async fn execute_join_approval_strategy(
        &self,
        strategy_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!(
                    "/v2/groups/join_approval_strategy/{}/execute",
                    segment(strategy_id)
                ),
                Some(body),
            )
            .await
    }
    /// 更新入群审批策略白名单。
    pub async fn whitelist_join_approval_strategy(
        &self,
        strategy_id: &str,
        body: &Value,
    ) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!(
                    "/v2/groups/join_approval_strategy/{}/whitelist_users",
                    segment(strategy_id)
                ),
                Some(body),
            )
            .await
    }

    /// 分片上传准备。
    pub async fn upload_prepare(&self, group_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/groups/{}/upload_prepare", segment(group_id)),
                Some(body),
            )
            .await
    }
    /// 分片上传完成。
    pub async fn upload_part_finish(&self, group_id: &str, body: &Value) -> Result<Value> {
        self.client
            .request_json(
                Method::POST,
                &format!("/v2/groups/{}/upload_part_finish", segment(group_id)),
                Some(body),
            )
            .await
    }

    async fn get_value(&self, path: &str) -> Result<Value> {
        self.client
            .request_json(Method::GET, path, Option::<&Value>::None)
            .await
    }
}
