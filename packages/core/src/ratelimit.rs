use crate::error::{Result, SdkError};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};

#[derive(Debug, Clone)]
struct Window {
    started: Instant,
    count: u32,
}

#[derive(Debug, Default)]
struct State {
    windows: HashMap<String, Window>,
    last_cleanup: Option<Instant>,
}

/// 按键共享配额的本地固定窗口限频器。
#[derive(Debug, Clone)]
pub struct RateLimiter {
    windows: Arc<Mutex<State>>,
    limit: u32,
    period: Duration,
}

impl RateLimiter {
    /// 创建固定窗口限频器。
    pub fn new(limit: u32, period: Duration) -> Self {
        Self {
            windows: Arc::new(Mutex::new(State::default())),
            limit,
            period,
        }
    }

    /// 消耗配额，超额返回 [`SdkError::RateLimited`]。
    ///
    /// # 错误
    /// 超过当前窗口配额时返回 `SdkError::RateLimited`。
    pub async fn acquire(&self, key: impl Into<String>) -> Result<()> {
        let key = key.into();
        let mut state = self.windows.lock().await;
        self.check(&mut state, &key, Instant::now())?;
        state.windows.get_mut(&key).expect("检查后窗口存在").count += 1;
        Ok(())
    }

    // 双层申请统一先锁全局、再锁接口，两层通过后才扣减。
    pub(crate) async fn acquire_pair(&self, other: &Self) -> Result<()> {
        let mut global = self.windows.lock().await;
        let mut endpoint = other.windows.lock().await;
        let now = Instant::now();
        self.check(&mut global, "bot", now)?;
        other.check(&mut endpoint, "endpoint", now)?;
        global.windows.get_mut("bot").expect("检查后窗口存在").count += 1;
        endpoint
            .windows
            .get_mut("endpoint")
            .expect("检查后窗口存在")
            .count += 1;
        Ok(())
    }

    fn check(&self, state: &mut State, key: &str, now: Instant) -> Result<()> {
        if state
            .last_cleanup
            .is_none_or(|last| now.duration_since(last) >= self.period)
        {
            state
                .windows
                .retain(|_, window| now.duration_since(window.started) < self.period);
            state.last_cleanup = Some(now);
        }
        if let Some(window) = state.windows.get_mut(key) {
            if now.duration_since(window.started) >= self.period {
                window.started = now;
                window.count = 0;
            }
            if window.count >= self.limit {
                return Err(SdkError::RateLimited(format!(
                    "{key}: {}/{:?}",
                    self.limit, self.period
                )));
            }
        } else {
            if self.limit == 0 {
                return Err(SdkError::RateLimited(format!("{key}: 0/{:?}", self.period)));
            }
            state.windows.insert(
                key.to_owned(),
                Window {
                    started: now,
                    count: 0,
                },
            );
        }
        Ok(())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejected_pair_does_not_consume_either_available_quota() {
        let global = RateLimiter::new(2, Duration::from_secs(60));
        let endpoint = RateLimiter::new(1, Duration::from_secs(60));
        global.acquire_pair(&endpoint).await.unwrap();
        assert!(global.acquire_pair(&endpoint).await.is_err());
        global.acquire("bot").await.unwrap();
        assert!(global.acquire("bot").await.is_err());
        let fresh = RateLimiter::new(1, Duration::from_secs(60));
        assert!(global.acquire_pair(&fresh).await.is_err());
        fresh.acquire("endpoint").await.unwrap();
    }

    #[tokio::test]
    async fn expired_keys_are_removed_and_active_windows_preserved() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        limiter.acquire("old").await.unwrap();
        {
            let mut state = limiter.windows.lock().await;
            state.windows.get_mut("old").unwrap().started -= Duration::from_secs(120);
            state.last_cleanup = None;
        }
        limiter.acquire("new").await.unwrap();
        assert!(!limiter.windows.lock().await.windows.contains_key("old"));
        assert!(limiter.acquire("new").await.is_err());
    }

    #[test]
    fn window_resets_at_boundary_without_resetting_active_keys() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        let now = Instant::now();
        let mut state = State::default();
        limiter.check(&mut state, "old", now).unwrap();
        state.windows.get_mut("old").unwrap().count = 1;
        let later = now + Duration::from_secs(30);
        limiter.check(&mut state, "active", later).unwrap();
        state.windows.get_mut("active").unwrap().count = 1;
        let boundary = now + Duration::from_secs(60);
        limiter.check(&mut state, "old", boundary).unwrap();
        assert_eq!(state.windows["old"].count, 0);
        assert!(limiter.check(&mut state, "active", boundary).is_err());
        assert!(
            RateLimiter::new(0, Duration::from_secs(60))
                .check(&mut State::default(), "blocked", now)
                .is_err()
        );
    }

    #[tokio::test]
    async fn concurrent_pairs_respect_both_limits() {
        let global = RateLimiter::new(1, Duration::from_secs(60));
        let a = RateLimiter::new(1, Duration::from_secs(60));
        let b = RateLimiter::new(1, Duration::from_secs(60));
        let (a, b) = tokio::join!(global.acquire_pair(&a), global.acquire_pair(&b));
        assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    }
}
