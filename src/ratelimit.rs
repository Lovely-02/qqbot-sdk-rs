use crate::error::{Result, SdkError};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct Window {
    started: Instant,
    count: u32,
}

/// 本地主动消息限频器。
#[derive(Debug, Clone)]
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<String, Window>>>,
    limit: u32,
    period: Duration,
}

impl RateLimiter {
    /// 创建固定窗口限频器。
    pub fn new(limit: u32, period: Duration) -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            limit,
            period,
        }
    }

    /// 消耗配额，超额返回 [`SdkError::RateLimited`]。
    pub async fn acquire(&self, key: impl Into<String>) -> Result<()> {
        let key = key.into();
        let mut windows = self.windows.lock().await;
        let now = Instant::now();
        let window = windows.entry(key.clone()).or_insert(Window {
            started: now,
            count: 0,
        });
        if now.duration_since(window.started) >= self.period {
            window.started = now;
            window.count = 0;
        }
        if window.count >= self.limit {
            return Err(SdkError::RateLimited(format!(
                "{key}: {}/{}",
                self.limit,
                self.period.as_secs()
            )));
        }
        window.count += 1;
        Ok(())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(1))
    }
}
