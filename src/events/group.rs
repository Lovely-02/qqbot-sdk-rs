//! 群成员、入群与订阅类强类型事件。

use super::{Event, EventContext};
use crate::{
    entities::{GroupHandle, GroupMemberHandle, UserHandle},
    error::{Result, SdkError},
    models::Message,
    segment::Sendable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
/// 群成员加入事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMemberAdd {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub member_openid: Option<String>,
    pub user_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupMemberAdd {
    const NAME: &'static str = "GROUP_MEMBER_ADD";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupMemberAdd {
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }

    /// 返回群成员用户实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn user(&self) -> Result<UserHandle> {
        let id = self
            .user_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 user_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(id))
    }

    /// 返回发生变更的群成员实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn member(&self) -> Result<GroupMemberHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let member_openid = self
            .member_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 member_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group_member(group_openid, member_openid))
    }
}

/// 群成员退出事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMemberRemove {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub member_openid: Option<String>,
    pub user_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupMemberRemove {
    const NAME: &'static str = "GROUP_MEMBER_REMOVE";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupMemberRemove {
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }

    /// 返回群成员用户实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn user(&self) -> Result<UserHandle> {
        let id = self
            .user_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 user_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(id))
    }

    /// 返回发生变更的群成员实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn member(&self) -> Result<GroupMemberHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 group_openid".into()))?;
        let member_openid = self
            .member_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群成员事件缺少 member_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group_member(group_openid, member_openid))
    }
}

/// 机器人加入群聊事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupAddRobot {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub op_member_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupAddRobot {
    const NAME: &'static str = "GROUP_ADD_ROBOT";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupAddRobot {
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }

    /// 使用事件 ID 发送机器人入群欢迎消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let group_openid = self
            .group_openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        context
            .client()
            .api()
            .messages()
            .reply_group(group_openid, message, None, context.event_id.as_deref())
            .await
    }
}

/// 机器人退出群聊事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupDelRobot {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub op_member_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupDelRobot {
    const NAME: &'static str = "GROUP_DEL_ROBOT";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupDelRobot {
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let id = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(id))
    }
}

/// 群聊消息接收开关变更事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMessageSetting {
    pub timestamp: Option<i64>,
    pub group_openid: Option<String>,
    pub op_member_openid: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl GroupMessageSetting {
    fn context(&self) -> Result<&EventContext> {
        self.context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))
    }

    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        Ok(self.context()?.client().group(group_openid))
    }

    /// 群聊消息接收开启后，使用事件 ID 发送一条被动消息。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub async fn reply(&self, message: impl Into<Sendable>) -> Result<Message> {
        let context = self.context()?;
        if context.event_name != "GROUP_MSG_RECEIVE" {
            return Err(SdkError::InvalidInput(
                "GROUP_MSG_REJECT 事件不支持被动回复".into(),
            ));
        }
        let group_openid = self
            .group_openid
            .as_deref()
            .ok_or_else(|| SdkError::InvalidInput("群事件缺少 group_openid".into()))?;
        context
            .client()
            .api()
            .messages()
            .reply_group(group_openid, message, None, context.event_id.as_deref())
            .await
    }
}

impl Event for GroupMessageSetting {
    const NAME: &'static str = "GROUP_MSG_RECEIVE";
    const NAMES: &'static [&'static str] = &["GROUP_MSG_RECEIVE", "GROUP_MSG_REJECT"];

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

/// 用户申请加群事件中的问答项。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewQuestion {
    pub question: Option<String>,
    pub answer: Option<String>,
}

/// 用户申请加群事件中的验证信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JoinVerifyInfo {
    pub method: Option<String>,
    pub verify_message: Option<String>,
    #[serde(default)]
    pub review_qa_list: Vec<ReviewQuestion>,
}

/// 用户申请加群事件中的自动审批信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JoinAutoApproved {
    pub strategy_id: Option<String>,
}

/// 用户申请加群事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupJoinRequest {
    pub group_openid: Option<String>,
    pub join_request_id: Option<String>,
    pub risk_tips: Option<String>,
    pub union_openid: Option<String>,
    pub member_openid: Option<String>,
    pub username: Option<String>,
    pub apply_at: Option<String>,
    pub apply_source: Option<String>,
    pub invited_by: Option<String>,
    pub bot: Option<bool>,
    pub verify_info: Option<JoinVerifyInfo>,
    pub auto_approved: Option<JoinAutoApproved>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for GroupJoinRequest {
    const NAME: &'static str = "GROUP_JOIN_REQUEST";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl GroupJoinRequest {
    /// 返回群会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("入群申请缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(group_openid))
    }

    /// 返回申请入群的成员实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn member(&self) -> Result<GroupMemberHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("入群申请缺少 group_openid".into()))?;
        let member_openid = self
            .member_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("入群申请缺少 member_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group_member(group_openid, member_openid))
    }
}

/// 订阅消息模板的授权结果。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscribeMessageTemplateResult {
    pub template_id: Option<u32>,
    pub custom_template_id: Option<String>,
    pub op: Option<u8>,
    pub subscribe_id: Option<String>,
    pub subscribe_ts: Option<i64>,
    pub update_ts: Option<i64>,
}

/// 订阅消息授权状态变更事件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscribeMessageStatus {
    pub group_openid: Option<String>,
    pub openid: Option<String>,
    #[serde(default)]
    pub result: Vec<SubscribeMessageTemplateResult>,
    #[serde(flatten)]
    pub extra: Value,
    #[serde(skip)]
    context: Option<EventContext>,
}

impl Event for SubscribeMessageStatus {
    const NAME: &'static str = "SUBSCRIBE_MESSAGE_STATUS";

    fn attach_context(&mut self, context: EventContext) {
        self.context = Some(context);
    }
}

impl SubscribeMessageStatus {
    /// 返回订阅群实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn group(&self) -> Result<GroupHandle> {
        let group_openid = self
            .group_openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("订阅事件缺少 group_openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().group(group_openid))
    }

    /// 返回订阅用户会话实体。
    ///
    /// # Errors
    /// 事件未绑定运行时 Client 或缺少必要 ID 时返回 `SdkError::InvalidInput`；`reply` 类方法在发送失败时还会返回鉴权、序列化、限频或平台业务错误。
    pub fn user(&self) -> Result<UserHandle> {
        let openid = self
            .openid
            .clone()
            .ok_or_else(|| SdkError::InvalidInput("订阅事件缺少 openid".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| SdkError::InvalidInput("事件未绑定运行时 Client".into()))?;
        Ok(context.client().user(openid))
    }
}
