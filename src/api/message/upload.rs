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

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(status: u16) -> SdkError {
        SdkError::Api {
            status,
            code: 1,
            message: "test".into(),
            trace_id: None,
            data: None,
        }
    }

    #[tokio::test]
    async fn retries_transient_errors_but_not_permanent_errors() {
        let policy = UploadPolicy {
            concurrency: 2,
            timeout: Duration::from_secs(1),
            delay: Duration::ZERO,
        };
        let mut attempts = 0;
        policy
            .run(|| {
                attempts += 1;
                std::future::ready(if attempts < 3 {
                    Err(failure(503))
                } else {
                    Ok(())
                })
            })
            .await
            .unwrap();
        assert_eq!(attempts, 3);
        attempts = 0;
        assert!(
            policy
                .run(|| {
                    attempts += 1;
                    std::future::ready(Err::<(), _>(failure(400)))
                })
                .await
                .is_err()
        );
        assert_eq!(attempts, 1);
    }

    #[tokio::test]
    async fn timeout_cancels_a_hung_attempt_and_zero_budget_disables_retry() {
        let mut policy = UploadPolicy {
            concurrency: 1,
            timeout: Duration::from_millis(10),
            delay: Duration::ZERO,
        };
        assert!(matches!(
            policy.run(std::future::pending::<Result<()>>).await,
            Err(SdkError::Timeout(_))
        ));
        policy.timeout = Duration::ZERO;
        let mut attempts = 0;
        assert!(
            policy
                .run(|| {
                    attempts += 1;
                    std::future::ready(Err::<(), _>(failure(503)))
                })
                .await
                .is_err()
        );
        assert_eq!(attempts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn retries_respect_before_at_and_after_the_deadline() {
        use std::cell::Cell;
        for local_only in [false, true] {
            for elapsed in [29, 30, 31] {
                let policy = UploadPolicy {
                    concurrency: 1,
                    timeout: Duration::from_secs(30),
                    delay: Duration::from_secs(10),
                };
                let attempts = Cell::new(0);
                let operation = || {
                    attempts.set(attempts.get() + 1);
                    std::future::ready(if attempts.get() == 1 {
                        Err(if local_only {
                            SdkError::RateLimited("test".into())
                        } else {
                            failure(503)
                        })
                    } else {
                        Ok(())
                    })
                };
                let mut retry = Box::pin(async {
                    if local_only {
                        policy.run_locally_limited(operation).await
                    } else {
                        policy.run(operation).await
                    }
                });
                assert!(futures_util::poll!(&mut retry).is_pending());
                assert_eq!(attempts.get(), 1);
                tokio::time::advance(Duration::from_secs(elapsed)).await;
                let result = retry.await;
                if elapsed < 30 {
                    assert!(result.is_ok());
                    assert_eq!(attempts.get(), 2);
                } else {
                    assert!(matches!(result, Err(SdkError::Timeout(_))));
                    assert_eq!(attempts.get(), 1);
                }
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn cancelling_backoff_does_not_leave_background_retries() {
        use std::cell::Cell;
        let policy = UploadPolicy {
            concurrency: 1,
            timeout: Duration::from_secs(30),
            delay: Duration::from_secs(1),
        };
        let attempts = Cell::new(0);
        let mut retry = Box::pin(policy.run(|| {
            attempts.set(attempts.get() + 1);
            std::future::ready(Err::<(), _>(failure(503)))
        }));
        assert!(futures_util::poll!(&mut retry).is_pending());
        drop(retry);
        tokio::time::advance(Duration::from_secs(60)).await;
        assert_eq!(attempts.get(), 1);
    }
}
