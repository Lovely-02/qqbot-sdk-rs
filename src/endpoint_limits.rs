use crate::{Result, ratelimit::RateLimiter};
use reqwest::Method;
use std::{collections::HashMap, time::Duration};
use tokio::sync::Mutex;

// 来源：官方 API v2 v1.46.0 的“接口频率限制”；未知接口仍由全局限频约束。
const RULES: &[(&str, &str, u32, u64)] = &[
    ("DELETE", "/channels/{channel_id}", 50, 1),
    ("GET", "/channels/{channel_id}", 50, 1),
    ("PATCH", "/channels/{channel_id}", 50, 1),
    ("GET", "/gateway", 2, 60),
    ("GET", "/guilds/{guild_id}", 50, 1),
    ("GET", "/guilds/{guild_id}/channels", 50, 1),
    ("POST", "/guilds/{guild_id}/channels", 50, 1),
    ("PUT", "/interactions/{interaction_id}", 50, 1),
    ("GET", "/users/@me", 50, 1),
    ("GET", "/users/@me/guilds", 50, 1),
    ("POST", "/v2/generate_url_link", 50, 1),
    ("POST", "/v2/groups/{group_id}/upload_part_finish", 10, 1),
    ("POST", "/v2/groups/{group_id}/upload_prepare", 10, 1),
    (
        "POST",
        "/v2/groups/{group_openid}/approval_join_request/{member_openid}",
        60,
        60,
    ),
    (
        "POST",
        "/v2/groups/{group_openid}/batch_remove_members",
        30,
        60,
    ),
    ("GET", "/v2/groups/{group_openid}/bot_state", 30, 60),
    ("POST", "/v2/groups/{group_openid}/files", 50, 1),
    ("GET", "/v2/groups/{group_openid}/info", 30, 60),
    ("GET", "/v2/groups/{group_openid}/join_request_list", 30, 60),
    ("GET", "/v2/groups/{group_openid}/member_blacklist", 30, 60),
    ("POST", "/v2/groups/{group_openid}/member_blacklist", 60, 60),
    ("GET", "/v2/groups/{group_openid}/members", 60, 60),
    (
        "GET",
        "/v2/groups/{group_openid}/members/{member_openid}",
        30,
        60,
    ),
    ("POST", "/v2/groups/{group_openid}/messages", 100, 1),
    (
        "DELETE",
        "/v2/groups/{group_openid}/messages/{message_id}",
        10,
        1,
    ),
    (
        "GET",
        "/v2/groups/{group_openid}/restrict_chat_setting",
        30,
        60,
    ),
    (
        "POST",
        "/v2/groups/{group_openid}/restrict_chat_setting",
        60,
        60,
    ),
    ("GET", "/v2/groups/join_approval_strategy", 60, 60),
    ("POST", "/v2/groups/join_approval_strategy", 60, 60),
    (
        "DELETE",
        "/v2/groups/join_approval_strategy/{strategy_id}",
        60,
        60,
    ),
    (
        "PATCH",
        "/v2/groups/join_approval_strategy/{strategy_id}",
        60,
        60,
    ),
    (
        "POST",
        "/v2/groups/join_approval_strategy/{strategy_id}/execute",
        60,
        60,
    ),
    (
        "POST",
        "/v2/groups/join_approval_strategy/{strategy_id}/whitelist_users",
        60,
        60,
    ),
    ("GET", "/v2/menu", 30, 60),
    ("PUT", "/v2/menu", 5, 60),
    ("GET", "/v2/panels", 30, 60),
    ("POST", "/v2/panels", 10, 60),
    ("DELETE", "/v2/panels/{panel_id}", 10, 60),
    ("GET", "/v2/panels/{panel_id}", 30, 60),
    ("PUT", "/v2/panels/{panel_id}", 10, 60),
    ("PUT", "/v2/panels/{panel_id}/target", 60, 60),
    ("POST", "/v2/users/{user_id}/upload_part_finish", 10, 1),
    ("POST", "/v2/users/{user_id}/upload_prepare", 10, 1),
    ("POST", "/v2/users/{user_openid}/files", 50, 1),
    ("POST", "/v2/users/{user_openid}/messages", 100, 1),
    (
        "DELETE",
        "/v2/users/{user_openid}/messages/{message_id}",
        10,
        1,
    ),
    ("POST", "/v2/users/{user_openid}/stream_messages", 50, 1),
];

#[derive(Default)]
pub(crate) struct EndpointLimits {
    limiters: Mutex<HashMap<usize, RateLimiter>>,
}

impl EndpointLimits {
    pub(crate) async fn acquire(&self, method: &Method, path: &str) -> Result<()> {
        let Some((index, &(_, _, limit, seconds))) = RULES
            .iter()
            .enumerate()
            .find(|(_, rule)| method.as_str() == rule.0 && Self::matches(rule.1, path))
        else {
            return Ok(());
        };
        let limiter = self
            .limiters
            .lock()
            .await
            .entry(index)
            .or_insert_with(|| RateLimiter::new(limit, Duration::from_secs(seconds)))
            .clone();
        limiter.acquire("endpoint").await
    }

    fn matches(template: &str, path: &str) -> bool {
        let mut template = template.split('/');
        let mut path = path.split('/');
        loop {
            match (template.next(), path.next()) {
                (None, None) => return true,
                (Some(expected), Some(actual))
                    if expected == actual || expected.starts_with('{') && !actual.is_empty() => {}
                _ => return false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn minute_limits_are_shared_across_ids_but_not_endpoints_or_methods() {
        let limits = EndpointLimits::default();
        for index in 0..30 {
            limits
                .acquire(&Method::GET, &format!("/v2/groups/{index}/info"))
                .await
                .unwrap();
        }
        assert!(
            limits
                .acquire(&Method::GET, "/v2/groups/other/info")
                .await
                .is_err()
        );
        limits
            .acquire(&Method::GET, "/v2/groups/group/bot_state")
            .await
            .unwrap();
        for _ in 0..5 {
            limits.acquire(&Method::PUT, "/v2/menu").await.unwrap();
        }
        assert!(limits.acquire(&Method::PUT, "/v2/menu").await.is_err());
        limits.acquire(&Method::GET, "/v2/menu").await.unwrap();
        limits.acquire(&Method::GET, "/future/api").await.unwrap();
    }
}
