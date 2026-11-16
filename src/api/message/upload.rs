use super::UploadConfig;
use crate::{Result, SdkError};
use std::{future::Future, time::Duration};
use tokio::time::{Instant, sleep, timeout_at};

pub(super) struct UploadPolicy {
    pub concurrency: usize,
    timeout: Duration,
    delay: Duration,
}

impl UploadPolicy {
    pub fn from_config(config: Option<&UploadConfig>) -> Result<Self> {
        let concurrency = config.and_then(|config| config.concurrency).unwrap_or(1);
        if concurrency == 0 {
            return Err(SdkError::InvalidInput("上传并发数不能为 0".into()));
        }
        Ok(Self {
            // 限制在途文件副本，避免异常配置导致瞬时内存耗尽。
            concurrency: usize::try_from(concurrency.min(64))
                .map_err(|_| SdkError::InvalidInput("上传并发数超出范围".into()))?,
            timeout: Duration::from_secs(u64::from(
                config.and_then(|c| c.retry_timeout).unwrap_or(300),
            )),
            delay: Duration::from_secs(u64::from(config.and_then(|c| c.retry_delay).unwrap_or(1))),
        })
    }

    pub async fn run<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.run_with_retry(operation, Self::retryable).await
    }

    /// 只重试尚未发出 HTTP 请求的本地限频，避免重复触发合并／发消息副作用。
    pub async fn run_locally_limited<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.run_with_retry(operation, |error| matches!(error, SdkError::RateLimited(_)))
            .await
    }

    async fn run_with_retry<F, Fut, T>(
        &self,
        mut operation: F,
        retryable: fn(&SdkError) -> bool,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if self.timeout.is_zero() {
            return operation().await;
        }
        let deadline = Instant::now() + self.timeout;
        loop {
            // 退避结束时执行器可能已延迟到预算之外，不能再调用操作闭包。
            if Instant::now() >= deadline {
                return Err(SdkError::Timeout("富媒体上传调度".into()));
            }
            let error = match timeout_at(deadline, operation()).await {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(error)) => error,
                Err(_) => return Err(SdkError::Timeout("富媒体上传调度".into())),
            };
            if !retryable(&error) || Instant::now() + self.delay >= deadline {
                return Err(error);
            }
            sleep(self.delay).await;
        }
    }
    fn retryable(error: &SdkError) -> bool {
        match error {
            SdkError::Http(error) => {
                error.is_request()
                    || error.is_timeout()
                    || error.is_connect()
                    || error.is_body()
                    || error
                        .status()
                        .is_some_and(|s| s.is_server_error() || s.as_u16() == 429)
            }
            SdkError::Api { status, .. } => *status == 429 || *status >= 500,
            SdkError::RateLimited(_) => true,
            SdkError::Serialization(_)
            | SdkError::Auth(_)
            | SdkError::WebSocket(_)
            | SdkError::InvalidSignature
            | SdkError::InvalidInput(_)
            | SdkError::Logging(_)
            | SdkError::Timeout(_) => false,
        }
    }
}
