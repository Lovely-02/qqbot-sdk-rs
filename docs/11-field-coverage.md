# 11 字段覆盖与验证

核对日期：2026-10-31，官方文档站 v1.46.0。本轮覆盖 206 个 API v2 页面，其中包含 47 个自动生成 HTTP 接口页面、22 个自动生成事件页面，以及频道专题接口、模型、素材、鉴权和网关文档。完整 HTTP 路径清单见 [接口覆盖表](10-api-coverage.md)。

## 本轮补充与修正

- 审批策略列表新增 cursor/limit 分页入口；群成员、申请、黑名单和策略提供独立 GET JSON 请求体入口，保留现有查询参数形式。
- 流式单聊新增 `StreamMessageRequest` 的 10 个字段和 `StreamMessageResponse` 的 `remain_msg_len` 等字段，修正旧示例。
- 富媒体新增 `MediaUploadRequest` 六个字段的类型化入口，公开预上传整体 block_size、parts 和 upload_config 三个配置字段。
- 补充 `MessageAudited.seq_in_channel`、频道／频道私信消息事件的 edited_timestamp/ark，以及 Embed.description。
- 修正新版 @全体标签的文字子频道支持，补充 @用户、回车指令和参数指令帮助函数及场景校验。
- 允许频道 Markdown 携带内嵌键盘，保留 group_id、样式 3/4、anchor、modal 等嵌套字段。
- API／鉴权错误保留 trace_id 和 data，识别 200/201/202 中的非零业务码；异步审核信息不会再被当作已发送消息丢弃。
- 修复 Webhook 根级导出，将 README 和全部包含 Rust 示例的开发手册纳入 cargo test 的 doctest。

## 类型化字段契约

[测试快照](../tests/fixtures/official-fields.json)为官方字段名和类型构造合成值，并逐条记录来源 URL，不含真实用户、消息或凭据。[字段测试](../tests/official_fields.rs)对下列 55 个结构检查反序列化后再次序列化不会丢失已记录字段；频道消息和审核新增字段另有直接字段访问测试。

| SDK 结构 | 文档字段 | 来源 |
| --- | --- | --- |
| `Ark` | `template_id`, `kv` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/template/model.html) |
| `ArkData` | `prompt`, `ark_type`, `ark_name`, `fields` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_message_create.html) |
| `C2cMessageReceive` | `id`, `author`, `content`, `timestamp`, `message_type`, `message_scene`, `attachments`, `ark_data`, `msg_elements` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html) |
| `C2cMsgReceive` | `timestamp`, `openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_msg_receive.html) |
| `C2cMsgReject` | `timestamp`, `openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_msg_reject.html) |
| `Channel` | `id`, `guild_id`, `name`, `type`, `sub_type`, `position`, `parent_id`, `owner_id`, `private_type`, `speak_permission`, `application_id`, `permissions` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/channels_channel_id.get.html) |
| `ChannelEvent` | `id`, `guild_id`, `name`, `type`, `sub_type`, `owner_id`, `op_user_id` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/channel_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/channel_delete.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/channel_update.html) |
| `Embed` | `title`, `prompt`, `thumbnail`, `fields` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/template/model.html) |
| `FriendAdd` | `timestamp`, `openid`, `scene`, `scene_param`, `author`, `short_code` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/friend_add.html) |
| `FriendAuthor` | `union_openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/friend_add.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/friend_del.html) |
| `FriendDelete` | `timestamp`, `openid`, `author` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/friend_del.html) |
| `Group` | `group_openid`, `group_name`, `group_finger_memo`, `group_class_text`, `group_tags`, `group_member_num` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_info.get.html) |
| `GroupAddRobot` | `timestamp`, `group_openid`, `op_member_openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_add_robot.html) |
| `GroupAtMessageCreate` | `id`, `author`, `content`, `group_openid`, `timestamp`, `message_type`, `message_scene`, `attachments`, `mentions`, `ark_data`, `msg_elements` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html) |
| `GroupDelRobot` | `timestamp`, `group_openid`, `op_member_openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_del_robot.html) |
| `GroupJoinRequest` | `group_openid`, `join_request_id`, `risk_tips`, `union_openid`, `member_openid`, `username`, `apply_at`, `apply_source`, `invited_by`, `bot`, `verify_info`, `auto_approved` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_join_request.html) |
| `GroupMemberAdd` | `timestamp`, `group_openid`, `member_openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_member_add.html) |
| `GroupMemberRemove` | `timestamp`, `group_openid`, `member_openid`, `user_openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_member_remove.html) |
| `GroupMessageCreate` | `id`, `author`, `content`, `group_openid`, `timestamp`, `message_type`, `message_scene`, `attachments`, `mentions`, `ark_data`, `msg_elements` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_message_create.html) |
| `GroupMessageSetting` | `timestamp`, `group_openid`, `op_member_openid` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_msg_receive.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_msg_reject.html) |
| `Guild` | `id`, `name`, `icon`, `owner_id`, `owner`, `member_count`, `max_members`, `description`, `joined_at` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/guilds_guild_id.get.html) |
| `GuildEvent` | `id`, `name`, `icon`, `owner_id`, `member_count`, `max_members`, `description`, `joined_at`, `op_user_id` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/guild_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/guild_delete.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/guild_update.html) |
| `GuildMember` | `user`, `nick`, `roles`, `joined_at` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/openapi/member/model.html) |
| `ImageUploadCompletion` | `upload_id`, `md5` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) |
| `ImageUploadPreparation` | `upload_id`, `presigned_url`, `expires_in` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) |
| `ImageUploadRequest` | `file_name`, `file_size`, `content_type`, `md5`, `sha1` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) |
| `InteractionAuthorizeData` | `opt_scene`, `scope` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/interaction_create.html) |
| `InteractionCreate` | `id`, `type`, `scene`, `chat_type`, `timestamp`, `guild_id`, `channel_id`, `user_openid`, `group_openid`, `group_member_openid`, `data`, `version`, `application_id` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/interaction_create.html) |
| `InteractionData` | `type`, `resolved` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/interaction_create.html) |
| `InteractionMessageScene` | `ext` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/interaction_create.html) |
| `InteractionResolved` | `button_data`, `button_id`, `user_id`, `feature_id`, `message_id`, `feedback_opt`, `checked`, `action`, `message_scene`, `authorize_data` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/interaction_create.html) |
| `JoinAutoApproved` | `strategy_id` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_join_request.html) |
| `JoinVerifyInfo` | `method`, `verify_message`, `review_qa_list` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_join_request.html) |
| `Keyboard` | `id`, `content` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) |
| `MaterialImage` | `image_id`, `url`, `ttl`, `preview_url` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) |
| `MaterialImageEntry` | `image_id`, `url`, `file_name`, `preview_url`, `created_at`, `expires_at` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) |
| `MaterialImagePage` | `images`, `next_cursor`, `total` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) |
| `Media` | `file_uuid`, `file_info`, `ttl`, `id`, `raw_url` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_files.post.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_files.post.html) |
| `MediaUploadRequest` | `file_type`, `url`, `srv_send_msg`, `upload_id`, `file_name`, `file_data` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_files.post.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_files.post.html) |
| `Message` | `id`, `channel_id`, `guild_id`, `content`, `timestamp`, `edited_timestamp`, `mention_everyone`, `author`, `attachments`, `embeds`, `mentions`, `member`, `ark`, `seq`, `seq_in_channel`, `message_reference` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/template/model.html) |
| `MessageAttachment` | `url`, `filename`, `width`, `height`, `size`, `content_type`, `voice_wav_url`, `asr_refer_text` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_message_create.html) |
| `MessageAudited` | `audit_id`, `message_id`, `guild_id`, `channel_id`, `audit_time`, `create_time`, `seq_in_channel` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/template/model.html) |
| `MessageCard` | `type`, `content` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) |
| `MessageElement` | `msg_idx`, `author`, `message_type`, `content`, `attachments`, `ark_data`, `msg_elements` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_message_create.html) |
| `MessageExtInfo` | `ref_idx` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) |
| `MessageReference` | `message_id`, `ignore_get_message_error` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/template/model.html) |
| `MessageRequest` | `msg_type`, `content`, `markdown`, `keyboard`, `msg_id`, `event_id`, `msg_seq`, `media`, `message_reference`, `card`, `is_wakeup`, `input_notify` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_messages.post.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) |
| `MessageScene` | `source`, `ext` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_message_create.html) |
| `ReviewQuestion` | `question`, `answer` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_join_request.html) |
| `StreamMessageRequest` | `input_mode`, `input_state`, `index`, `content_type`, `content_raw`, `event_id`, `msg_id`, `stream_msg_id`, `msg_seq`, `is_wakeup` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_stream_messages.post.html) |
| `StreamMessageResponse` | `id`, `timestamp`, `ext_info`, `remain_msg_len` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_stream_messages.post.html) |
| `SubscribeMessageStatus` | `group_openid`, `openid`, `result` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/subscribe_message_status.html) |
| `SubscribeMessageTemplateResult` | `template_id`, `custom_template_id`, `op`, `subscribe_id`, `subscribe_ts`, `update_ts` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/subscribe_message_status.html) |
| `TuwenContent` | `title`, `description`, `url`, `pic_url` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) |
| `User` | `id`, `username`, `bot`, `union_openid`, `union_user_account`, `user_openid`, `member_openid`, `member_role`, `avatar` | [文档1](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html), [文档2](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html), [文档3](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_message_create.html), [文档4](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/users_me.get.html) |

快照中的 MessageCard/TuwenContent 字段来自发送接口；Embed.description 来自官方响应示例，另有回归测试。MaterialImage 等素材结构均来自素材管理页。`force_verify_image_resource` 的官方位置是 markdown 对象；MessageRequest 同名便捷字段会在发送前写入该对象，不作为顶层 JSON 字段发送。

## 可扩展请求／响应字段

下表记录自动生成接口页的顶层字段及嵌套表字段。SDK 以类型化结构或原始 `serde_json::Value` 接受／返回这些字段：Markdown、键盘内容、群管理、角色、日程、菜单和面板等可扩展对象不采用字段白名单裁剪。重复字段名按所在官方嵌套对象理解；具体层级请点击对应官方页面。

| 官方接口 | 请求字段（不含路径参数） | 嵌套请求字段 | 响应字段（含嵌套） |
| --- | --- | --- | --- |
| [删除子频道](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/channels_channel_id.delete.html) | — | — | — |
| [获取子频道详情](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/channels_channel_id.get.html) | — | — | `id`, `guild_id`, `name`, `type`, `sub_type`, `position`, `parent_id`, `owner_id`, `private_type`, `speak_permission`, `application_id`, `permissions` |
| [修改子频道](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/channels_channel_id.patch.html) | `name`, `position`, `parent_id`, `private_type`, `speak_permission` | — | `id`, `guild_id`, `name`, `type`, `sub_type`, `position`, `parent_id`, `owner_id`, `private_type`, `speak_permission`, `application_id`, `permissions` |
| [获取通用 WSS 接入点](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/gateway.get.html) | — | — | `url` |
| [获取频道详情](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/guilds_guild_id.get.html) | — | — | `id`, `name`, `icon`, `owner_id`, `owner`, `member_count`, `max_members`, `description`, `joined_at` |
| [获取子频道列表](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/guilds_guild_id_channels.get.html) | — | — | `channels`, `id`, `guild_id`, `name`, `type`, `sub_type`, `position`, `parent_id`, `owner_id`, `private_type`, `speak_permission`, `application_id`, `permissions` |
| [创建子频道](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/guilds_guild_id_channels.post.html) | `name`, `type`, `sub_type`, `position`, `parent_id`, `private_type`, `private_user_ids`, `speak_permission`, `application_id` | — | `id`, `guild_id`, `name`, `type`, `sub_type`, `position`, `parent_id`, `owner_id`, `private_type`, `speak_permission`, `application_id`, `permissions` |
| [互动事件响应](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/interactions_interaction_id.put.html) | `code` | — | — |
| [获取机器人详情](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/users_me.get.html) | — | — | `id`, `username`, `avatar`, `bot`, `union_openid`, `union_user_account` |
| [获取机器人频道列表](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/users_me_guilds.get.html) | `before`, `after`, `limit` | — | `guilds`, `id`, `name`, `icon`, `owner_id`, `owner`, `joined_at`, `member_count`, `max_members`, `description` |
| [生成分享链接](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_generate_url_link.post.html) | `callback_data` | — | `data`, `url` |
| [群聊分片上传完成](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_id_upload_part_finish.post.html) | `upload_id`, `part_index`, `block_size`, `md5` | — | — |
| [群聊富媒体预上传](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_id_upload_prepare.post.html) | `file_type`, `file_size`, `file_name`, `md5`, `sha1`, `md5_10m` | — | `upload_id`, `block_size`, `parts`, `upload_config`, `index`, `presigned_url`, `concurrency`, `retry_timeout`, `retry_delay` |
| [入群申请审批](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_approval_join_request_member_openid.post.html) | `op`, `join_request_id`, `reject_reason`, `add_to_member_blacklist` | — | — |
| [群成员批量移除](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_batch_remove_members.post.html) | `member_openids`, `add_to_member_blacklist` | — | `remove_members_result`, `add_to_member_blacklist_fail_openids` |
| [获取机器人群内状态](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_bot_state.get.html) | — | — | `member_openid`, `joined_at`, `allow_proactive_msg`, `recv_msg_setting`, `member_role` |
| [群聊富媒体上传](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_files.post.html) | `file_type`, `url`, `srv_send_msg`, `upload_id`, `file_name`, `file_data` | — | `file_uuid`, `file_info`, `ttl`, `id`, `raw_url` |
| [获取群基本信息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_info.get.html) | — | — | `group_openid`, `group_name`, `group_finger_memo`, `group_class_text`, `group_tags`, `group_member_num` |
| [入群申请列表拉取](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_join_request_list.get.html) | `cursor`, `limit` | — | `list`, `next_cursor`, `join_request_id`, `risk_tips`, `union_openid`, `member_openid`, `username`, `apply_at`, `apply_source`, `invited_by`, `bot`, `verify_info`, `method`, `verify_message`, `review_qa_list`, `question`, `answer` |
| [群黑名单查询](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_member_blacklist.get.html) | `cursor`, `limit` | — | `users`, `next_cursor`, `union_openid`, `member_openid`, `username`, `banned_at`, `bot` |
| [群黑名单操作](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_member_blacklist.post.html) | `op`, `member_openids` | — | `fail_openids` |
| [获取群成员列表](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_members.get.html) | `cursor` | — | `members`, `next_cursor`, `member_openid`, `username`, `member_role`, `bot`, `joined_at`, `union_openid` |
| [获取群成员信息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_members_member_openid.get.html) | — | — | `member_openid`, `username`, `member_role`, `bot`, `joined_at`, `union_openid` |
| [发送群聊消息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_messages.post.html) | `msg_type`, `content`, `markdown`, `keyboard`, `msg_id`, `event_id`, `msg_seq`, `media`, `message_reference`, `card` | `template_id`, `content`, `custom_template_id`, `force_verify_image_resource`, `id`, `rows`, `buttons`, `render_data`, `action`, `group_id`, `label`, `visited_label`, `style`, `type`, `permission`, `data`, `click_limit`, `unsupport_tips`, `enter`, `reply`, `anchor`, `modal`, `specify_user_ids`, `specify_role_ids`, `confirm_text`, `cancel_text`, `file_info`, `message_id` | `id`, `timestamp`, `ext_info`, `ref_idx` |
| [撤回群聊消息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_messages_message_id.delete.html) | — | — | — |
| [查询群禁言状态](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_restrict_chat_setting.get.html) | — | — | `global_rule`, `members`, `mode`, `schedule_rules`, `recurring_rules`, `task_id`, `start_at`, `end_at`, `enabled`, `weekdays`, `start_time`, `end_time`, `member_openid`, `mute_expire_at`, `username`, `union_openid` |
| [设置群成员禁言](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_restrict_chat_setting.post.html) | `members` | `op`, `member_openid`, `mute_expire_at` | — |
| [查询入群自动审批策略列表](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy.get.html) | `cursor`, `limit` | — | `strategies`, `next_cursor`, `strategy_id`, `group_openids`, `group_ids`, `whitelist_user_count`, `is_enable`, `expire_at`, `created_at`, `updated_at`, `remark` |
| [创建入群自动审批策略](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy.post.html) | `group_openids`, `group_ids`, `is_enable`, `expire_at`, `remark` | — | `strategy_id`, `is_enable`, `expire_at` |
| [删除入群自动审批策略](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy_strategy_id.delete.html) | — | — | — |
| [修改入群自动审批策略](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy_strategy_id.patch.html) | `is_enable`, `expire_at`, `group_action`, `remark` | `op`, `group_openids`, `group_ids` | `is_enable`, `expire_at` |
| [执行入群自动审批策略](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy_strategy_id_execute.post.html) | — | — | — |
| [修改入群自动审批策略的白名单号码](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy_strategy_id_whitelist_users.post.html) | `op`, `whitelist_users` | — | `strategy_id`, `whitelist_user_count`, `updated_at` |
| [查询全局自定义菜单](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_menu.get.html) | — | — | `version`, `menu`, `items`, `name`, `type`, `sub_menu_items`, `send_message`, `link`, `switch`, `switch_id`, `default` |
| [修改全局自定义菜单](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_menu.put.html) | `menu` | `items`, `name`, `type`, `sub_menu_items`, `send_message`, `link`, `switch`, `switch_id`, `default` | `version` |
| [查询指令面板列表](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels.get.html) | `scope`, `cursor`, `limit` | — | `records`, `next_cursor`, `is_end`, `panel_id`, `scope`, `target_type`, `panel`, `created_at`, `updated_at`, `version`, `items`, `remark`, `name`, `desc`, `type`, `only_admin`, `link` |
| [创建指令面板](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels.post.html) | `scope`, `target_type`, `user_openids`, `group_openids`, `panel` | `items`, `remark`, `version`, `name`, `desc`, `type`, `only_admin`, `link` | `panel_id` |
| [删除指令面板](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels_panel_id.delete.html) | — | — | — |
| [查询指令面板详情](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels_panel_id.get.html) | — | — | `panel_id`, `scope`, `target_type`, `panel`, `created_at`, `updated_at`, `version`, `user_openids`, `group_openids`, `items`, `remark`, `name`, `desc`, `type`, `only_admin`, `link` |
| [修改指令面板](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels_panel_id.put.html) | `panel` | `items`, `remark`, `version`, `name`, `desc`, `type`, `only_admin`, `link` | `version` |
| [修改指令面板关联对象](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels_panel_id_target.put.html) | `op`, `user_openids`, `group_openids` | — | — |
| [单聊分片上传完成](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_id_upload_part_finish.post.html) | `upload_id`, `part_index`, `block_size`, `md5` | — | — |
| [单聊富媒体预上传](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_id_upload_prepare.post.html) | `file_type`, `file_size`, `file_name`, `md5`, `sha1`, `md5_10m` | — | `upload_id`, `block_size`, `parts`, `upload_config`, `index`, `presigned_url`, `concurrency`, `retry_timeout`, `retry_delay` |
| [单聊富媒体上传](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_files.post.html) | `file_type`, `url`, `srv_send_msg`, `upload_id`, `file_name`, `file_data` | — | `file_uuid`, `file_info`, `ttl`, `id`, `raw_url` |
| [发送单聊消息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) | `msg_type`, `content`, `markdown`, `keyboard`, `msg_id`, `event_id`, `msg_seq`, `media`, `message_reference`, `is_wakeup`, `card`, `input_notify` | `template_id`, `content`, `custom_template_id`, `force_verify_image_resource`, `id`, `rows`, `buttons`, `render_data`, `action`, `group_id`, `label`, `visited_label`, `style`, `type`, `permission`, `data`, `click_limit`, `unsupport_tips`, `enter`, `reply`, `anchor`, `modal`, `specify_user_ids`, `specify_role_ids`, `confirm_text`, `cancel_text`, `file_info`, `message_id`, `input_type`, `input_second` | `id`, `timestamp`, `ext_info`, `ref_idx` |
| [撤回单聊消息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages_message_id.delete.html) | — | — | — |
| [流式发送单聊消息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_stream_messages.post.html) | `input_mode`, `input_state`, `index`, `content_type`, `content_raw`, `event_id`, `msg_id`, `stream_msg_id`, `msg_seq`, `is_wakeup` | — | `id`, `timestamp`, `ext_info`, `remain_msg_len`, `ref_idx` |

## 验证边界

本地 HTTP 契约测试覆盖素材申请／PUT／ETag／完成／分页／删除、上传失败和 ETag 缺失、业务错误和链路信息、类型化流式消息、file_data/file_name、GET 分页两种形式、图文卡片和频道 Markdown 键盘。消息测试覆盖新版标签和命令的适用场景与编码。

没有使用真实机器人凭据调用线上服务；接口权限、平台白名单、自然日配额、文件真实格式校验以及部分 GET 分页参数传输形式仍需在线上确认。此处的完整性结论限定为已读取版本中的服务端接口和字段，后续官方更新仍需重新核对。

## 官方文档差异

- 消息类型概览的枚举表落后于 2026-10-28 更新；按单聊／群聊发送接口支持 msg_type=8。
- 子频道列表／机器人频道列表的字段表有 channels/guilds 包装名称，但响应示例是顶层数组；现有 Vec 返回按数组示例实现。
- 部分群 GET 分页页将 cursor/limit 标为请求体，成员示例则使用 URL 查询。SDK 为两种形式分别提供入口，不混发。
- 文本交互页声明新版 qqbot-at-everyone 标签也适用于文字子频道；全体提及权限和配额按各场景分别处理。

较早频道专题的撤回、表态、论坛、音频及成员进出通知已补充专用 Rust 事件类型，未知字段保留在 extra；当前自动生成文档中的 22 个事件页面也均有对应类型化入口。新增通知的路由与字段兼容性见 tests/notifications.rs，调度与回复序号的 HTTP／WebSocket 验证见 tests/runtime_scheduling.rs 和 tests/materials.rs。
