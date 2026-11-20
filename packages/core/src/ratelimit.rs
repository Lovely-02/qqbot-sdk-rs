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
