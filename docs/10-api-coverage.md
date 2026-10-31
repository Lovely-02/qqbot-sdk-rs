# 10 官方接口覆盖表

核对日期：2026-10-31。来源：[官方 API v2](https://bot.q.qq.com/wiki/develop/api-v2/)（文档站 v1.46.0），并核对 [变更记录](https://bot.q.qq.com/wiki/develop/api-v2/changelog.html)。

本轮读取 API v2 目录下 206 个官方页面（含鉴权、网关、自动生成接口和事件、旧版专题模型及说明页），按 HTTP 方法和路径去重后核对 100 个服务端接口组合，均存在 SDK 实现。下表包括上一轮遗漏在清单之外的 AccessToken 和带分片网关接口。此清单表示 SDK 接口覆盖，不表示全部接口已完成线上联调。

字段与事件的核对结果见 [11 字段覆盖与验证](11-field-coverage.md)。部分嵌套请求／响应保留为 `serde_json::Value`，可完整传递官方字段；类型化结构另有带来源的字段快照测试。

官方文档存在说明不一致之处：消息类型概览未完整列出图文卡片，SDK 按单聊／群聊发送接口及 2026-10-28 变更记录支持 `msg_type=8`；文字交互页明确允许文字子频道使用新版全体提及标签。GET 分页字段在部分页面列为请求体，SDK 同时保留查询参数入口和独立请求体入口，详见群聊手册。

| 方法 | 路径（点击查看官方文档） | SDK 实现 |
| --- | --- | --- |
| DELETE | [`/channels/{channel_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/channels/{channel_id}/announces/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/announces/delete_channel_announces.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/channels/{channel_id}/messages/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/recall.html) | [src/api/channel.rs](../src/api/channel.rs), [src/api/message/mod.rs](../src/api/message/mod.rs) |
| DELETE | [`/channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/trans/emoji.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/channels/{channel_id}/mic`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/audio/delete_mic.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/channels/{channel_id}/pins/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/pins/delete_pins_message.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/channels/{channel_id}/schedules/{schedule_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/schedule/delete_schedule.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/channels/{channel_id}/threads/{thread_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/forum/delete_thread.html) | [src/api/channel.rs](../src/api/channel.rs) |
| DELETE | [`/dms/{guild_id}/messages/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/dms.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| DELETE | [`/guilds/{guild_id}/announces/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/announces/delete_guild_announces.html) | [src/api/guild.rs](../src/api/guild.rs) |
| DELETE | [`/guilds/{guild_id}/members/{user_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role/member/delete_member.html) | [src/api/guild.rs](../src/api/guild.rs) |
| DELETE | [`/guilds/{guild_id}/members/{user_id}/roles/{role_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/delete_guild_member_role.html) | [src/api/guild.rs](../src/api/guild.rs) |
| DELETE | [`/guilds/{guild_id}/roles/{role_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/delete_guild_role.html) | [src/api/guild.rs](../src/api/guild.rs) |
| DELETE | [`/v2/groups/join_approval_strategy/{strategy_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| DELETE | [`/v2/groups/{group_openid}/messages/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| DELETE | [`/v2/media-images/{image_id}/delete`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) | [src/api/material.rs](../src/api/material.rs) |
| DELETE | [`/v2/panels/{panel_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/panel.rs](../src/api/panel.rs) |
| DELETE | [`/v2/users/{user_openid}/messages/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| GET | [`/channels/{channel_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/members/{user_id}/permissions`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/channel_permissions/get_channel_permissions.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/trans/emoji.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/online_nums`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role/get_online_nums.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/pins`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/pins/get_pins_message.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/roles/{role_id}/permissions`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/channel_permissions/get_channel_roles_permissions.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/schedules`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/schedule/get_schedules.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/schedules/{schedule_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/schedule/get_schedule.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/threads`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/forum/get_threads_list.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/channels/{channel_id}/threads/{thread_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/forum/get_thread.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/gateway`](https://bot.q.qq.com/wiki/develop/api-v2/openapi/wss/url_get.html) | [src/client.rs](../src/client.rs) |
| GET | [`/gateway/bot`](https://bot.q.qq.com/wiki/develop/api-v2/openapi/wss/shard_url_get.html) | [src/client.rs](../src/client.rs) |
| GET | [`/guilds/{guild_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/guilds/{guild_id}/api_permission`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/api_permissions/get_guild_api_permission.html) | [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/guilds/{guild_id}/channels`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/channel.rs](../src/api/channel.rs), [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/guilds/{guild_id}/members`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role/member/get_members.html) | [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/guilds/{guild_id}/members/{user_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role/member/get_member.html) | [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/guilds/{guild_id}/message/setting`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/speak/setting/message_setting.html) | [src/api/channel.rs](../src/api/channel.rs) |
| GET | [`/guilds/{guild_id}/roles`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/get_guild_roles.html) | [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/guilds/{guild_id}/roles/{role_id}/members`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role/member/get_role_members.html) | [src/api/guild.rs](../src/api/guild.rs) |
| GET | [`/users/@me`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/users_me.get.html) | [src/api/bot.rs](../src/api/bot.rs), [src/api/user.rs](../src/api/user.rs) |
| GET | [`/users/@me/guilds`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/users_me_guilds.get.html) | [src/api/bot.rs](../src/api/bot.rs), [src/api/user.rs](../src/api/user.rs) |
| GET | [`/v2/groups/join_approval_strategy`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy.get.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/bot_state`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/info`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/join_request_list`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/member_blacklist`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/members`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/members/{member_openid}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/groups/{group_openid}/restrict_chat_setting`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| GET | [`/v2/media-images`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) | [src/api/material.rs](../src/api/material.rs) |
| GET | [`/v2/menu`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_menu.get.html) | [src/api/menu.rs](../src/api/menu.rs) |
| GET | [`/v2/panels`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels.get.html) | [src/api/panel.rs](../src/api/panel.rs) |
| GET | [`/v2/panels/{panel_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/panel.rs](../src/api/panel.rs) |
| PATCH | [`/channels/{channel_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PATCH | [`/channels/{channel_id}/schedules/{schedule_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/schedule/patch_schedule.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PATCH | [`/guilds/{guild_id}/members/{user_id}/mute`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/speak/patch_guild_member_mute.html) | [src/api/guild.rs](../src/api/guild.rs) |
| PATCH | [`/guilds/{guild_id}/mute`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/speak/patch_guild_mute_multi_member.html) | [src/api/guild.rs](../src/api/guild.rs) |
| PATCH | [`/guilds/{guild_id}/roles/{role_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/patch_guild_role.html) | [src/api/guild.rs](../src/api/guild.rs) |
| PATCH | [`/v2/groups/join_approval_strategy/{strategy_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/app/getAppAccessToken`](https://bot.q.qq.com/wiki/develop/api-v2/dev-prepare/access-token.html) | [src/auth.rs](../src/auth.rs) |
| POST | [`/channels/{channel_id}/announces`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/announces/post_channel_announces.html) | [src/api/channel.rs](../src/api/channel.rs) |
| POST | [`/channels/{channel_id}/audio`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/audio/audio_control.html) | [src/api/channel.rs](../src/api/channel.rs) |
| POST | [`/channels/{channel_id}/messages`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/send.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| POST | [`/channels/{channel_id}/schedules`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/schedule/post_schedule.html) | [src/api/channel.rs](../src/api/channel.rs) |
| POST | [`/dms/{guild_id}/messages`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/dms.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| POST | [`/guilds/{guild_id}/announces`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/announces/post_guild_announces.html) | [src/api/guild.rs](../src/api/guild.rs) |
| POST | [`/guilds/{guild_id}/api_permission/demand`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/api_permissions/post_api_permission_demand.html) | [src/api/guild.rs](../src/api/guild.rs) |
| POST | [`/guilds/{guild_id}/channels`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/channel.rs](../src/api/channel.rs), [src/api/guild.rs](../src/api/guild.rs) |
| POST | [`/guilds/{guild_id}/roles`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/post_guild_role.html) | [src/api/guild.rs](../src/api/guild.rs) |
| POST | [`/users/@me/dms`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/dms.html) | [src/api/bot.rs](../src/api/bot.rs) |
| POST | [`/v2/generate_url_link`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_generate_url_link.post.html) | [src/api/utility.rs](../src/api/utility.rs) |
| POST | [`/v2/groups/join_approval_strategy`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_join_approval_strategy.post.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/join_approval_strategy/{strategy_id}/execute`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/join_approval_strategy/{strategy_id}/whitelist_users`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/{group_openid}/approval_join_request/{member_openid}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/{group_openid}/batch_remove_members`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/{group_openid}/files`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs), [src/api/message/media.rs](../src/api/message/media.rs) |
| POST | [`/v2/groups/{group_openid}/member_blacklist`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/{group_openid}/messages`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| POST | [`/v2/groups/{group_openid}/restrict_chat_setting`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs) |
| POST | [`/v2/groups/{group_id}/upload_part_finish`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs), [src/api/message/media.rs](../src/api/message/media.rs) |
| POST | [`/v2/groups/{group_id}/upload_prepare`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/group.rs](../src/api/group.rs), [src/api/message/media.rs](../src/api/message/media.rs) |
| POST | [`/v2/media-images/complete`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) | [src/api/material.rs](../src/api/material.rs) |
| POST | [`/v2/media-images/prepare`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html) | [src/api/material.rs](../src/api/material.rs) |
| POST | [`/v2/panels`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_panels.post.html) | [src/api/panel.rs](../src/api/panel.rs) |
| POST | [`/v2/users/{user_openid}/files`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/rich-media.html) | [src/api/message/media.rs](../src/api/message/media.rs), [src/api/user.rs](../src/api/user.rs) |
| POST | [`/v2/users/{user_openid}/messages`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/message/mod.rs](../src/api/message/mod.rs) |
| POST | [`/v2/users/{user_openid}/stream_messages`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/user.rs](../src/api/user.rs) |
| POST | [`/v2/users/{user_id}/upload_part_finish`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/message/media.rs](../src/api/message/media.rs), [src/api/user.rs](../src/api/user.rs) |
| POST | [`/v2/users/{user_id}/upload_prepare`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/message/media.rs](../src/api/message/media.rs), [src/api/user.rs](../src/api/user.rs) |
| PUT | [`/channels/{channel_id}/members/{user_id}/permissions`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/channel_permissions/put_channel_permissions.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PUT | [`/channels/{channel_id}/messages/{message_id}/reactions/{type}/{id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/trans/emoji.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PUT | [`/channels/{channel_id}/mic`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/audio/put_mic.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PUT | [`/channels/{channel_id}/pins/{message_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/pins/put_pins_message.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PUT | [`/channels/{channel_id}/roles/{role_id}/permissions`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/channel_permissions/put_channel_roles_permissions.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PUT | [`/channels/{channel_id}/threads`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/content/forum/put_thread.html) | [src/api/channel.rs](../src/api/channel.rs) |
| PUT | [`/guilds/{guild_id}/members/{user_id}/roles/{role_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/role-group/put_guild_member_role.html) | [src/api/guild.rs](../src/api/guild.rs) |
| PUT | [`/interactions/{interaction_id}`](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/trans/overview.html) | [src/api/interaction.rs](../src/api/interaction.rs) |
| PUT | [`/v2/menu`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_menu.put.html) | [src/api/menu.rs](../src/api/menu.rs) |
| PUT | [`/v2/panels/{panel_id}`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/panel.rs](../src/api/panel.rs) |
| PUT | [`/v2/panels/{panel_id}/target`](https://bot.q.qq.com/wiki/develop/api-v2/autogen/index.html) | [src/api/panel.rs](../src/api/panel.rs) |

小程序的 `qq.getGuildInfo` 等客户端 API 不属于 Rust 服务端 HTTP SDK；请按官方小程序文档在对应运行时调用。
