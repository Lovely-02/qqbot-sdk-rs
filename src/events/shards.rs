use super::{EventRouter, GatewayClient, GatewayConfig};
use crate::{QQBotClient, Result, SdkError, SessionStartLimit};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, Notify, mpsc},
    time::{Instant, sleep},
};

/// 在一个进程中启动全部分片，并协调首次鉴权及重连的启动配额。
pub struct GatewayShardManager {
    shards: Vec<GatewayClient>,
    running: Arc<Mutex<()>>,
}

impl GatewayShardManager {
    /// 获取 `/gateway/bot`，按服务端推荐数量创建分片。
    ///
    /// # Errors
    /// 网关配置、配额无效或获取网关信息失败时返回 `SdkError`。
    pub async fn new(
        client: Arc<QQBotClient>,
        router: EventRouter,
        config: GatewayConfig,
    ) -> Result<Self> {
        Self::with_shard_count(client, router, config, None).await
    }

    /// 显式覆盖推荐分片数；配置中的单分片编号由管理器分配。
    ///
    /// # Errors
    /// 分片数量为零、网关配置或配额无效、获取网关信息失败时返回 `SdkError`。
    pub async fn with_shard_count(
        client: Arc<QQBotClient>,
        router: EventRouter,
        config: GatewayConfig,
        count: Option<u32>,
    ) -> Result<Self> {
        // 在 HTTP 请求前验证 Bot 模式、Intents 等本地配置。
        let mut config = config;
        config.shard = (0, 1);
        GatewayClient::new(client.clone(), router.clone(), config.clone())?;
        let info = client.gateway_bot().await?;
        let count = count.unwrap_or(info.shards);
        if count == 0 || info.url.is_empty() {
            return Err(SdkError::InvalidInput(
                "网关分片数必须大于 0，URL 不能为空".into(),
            ));
        }
        let coordinator = Arc::new(SessionStartCoordinator::new(
            client.clone(),
            info.session_start_limit,
        )?);
        let shards = (0..count)
            .map(|id| {
                let mut config = config.clone();
                config.shard = (id, count);
                let mut gateway = GatewayClient::new(client.clone(), router.clone(), config)?;
                gateway.coordinate_starts(coordinator.clone(), info.url.clone());
                Ok(gateway)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            shards,
            running: Arc::default(),
        })
    }

    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    /// 运行所有分片；任一分片终止时取消其余连接及事件分发任务。
    ///
    /// # Errors
    /// 已在运行或清理时返回 `SdkError::InvalidInput`；否则返回首个终止分片的错误。
    pub async fn run(&self) -> Result<()> {
        let running = Arc::new(
            self.running
                .clone()
                .try_lock_owned()
                .map_err(|_| SdkError::InvalidInput("分片管理器正在运行或清理中".into()))?,
        );
        futures_util::future::try_join_all(
            self.shards
                .iter()
                .map(|shard| shard.run_with_manager(Some(running.clone()))),
        )
        .await?;
        Ok(())
    }
}

struct StartState {
    limit: SessionStartLimit,
    reset_at: Instant,
    buckets: HashMap<u32, Arc<Mutex<Instant>>>,
    epoch: Arc<()>,
    refunds: mpsc::UnboundedReceiver<Arc<()>>,
}

pub(super) struct SessionStartCoordinator {
    client: Arc<QQBotClient>,
    state: Mutex<StartState>,
    interval: Duration,
    refund_tx: mpsc::UnboundedSender<Arc<()>>,
    refund_notify: Arc<Notify>,
}

impl SessionStartCoordinator {
    fn new(client: Arc<QQBotClient>, limit: SessionStartLimit) -> Result<Self> {
        Self::validate(&limit)?;
        let reset_at = Instant::now()
            .checked_add(Duration::from_millis(limit.reset_after))
            .ok_or_else(|| SdkError::InvalidInput("网关配额重置时间超出范围".into()))?;
        let (refund_tx, refunds) = mpsc::unbounded_channel();
        Ok(Self {
            client,
            state: Mutex::new(StartState {
                limit,
                reset_at,
                buckets: HashMap::new(),
                epoch: Arc::new(()),
                refunds,
            }),
            interval: Duration::from_secs(5),
            refund_tx,
            refund_notify: Arc::default(),
        })
    }

    fn validate(limit: &SessionStartLimit) -> Result<()> {
        if limit.total == 0
            || limit.remaining > limit.total
            || limit.max_concurrency == 0
            || limit.reset_after == 0
        {
            return Err(SdkError::InvalidInput(
                "网关 session_start_limit 无效".into(),
            ));
        }
        Ok(())
    }

    pub(super) async fn acquire(&self, shard: u32) -> Result<StartPermit> {
        loop {
            // 先注册再检查余额，避免退款发生在检查和进入等待之间时丢失唤醒。
            let refunded = self.refund_notify.notified();
            tokio::pin!(refunded);
            refunded.as_mut().enable();
            let mut state = self.state.lock().await;
            while let Ok(epoch) = state.refunds.try_recv() {
                if Arc::ptr_eq(&epoch, &state.epoch) {
                    state.limit.remaining = state
                        .limit
                        .remaining
                        .saturating_add(1)
                        .min(state.limit.total);
                }
            }
            let now = Instant::now();
            if now >= state.reset_at {
                let limit = self.client.gateway_bot().await?.session_start_limit;
                Self::validate(&limit)?;
                // 并发度变化会改变桶映射，要求调用方重建管理器，避免新旧桶同时放行。
                if limit.max_concurrency != state.limit.max_concurrency {
                    return Err(SdkError::InvalidInput(
                        "网关最大并发数已变化，请重建分片管理器".into(),
                    ));
                }
                state.reset_at = Instant::now()
                    .checked_add(Duration::from_millis(limit.reset_after))
                    .ok_or_else(|| SdkError::InvalidInput("网关配额重置时间超出范围".into()))?;
                state.limit = limit;
                state.epoch = Arc::new(());
                continue;
            }
            if state.limit.remaining == 0 {
                let reset_at = state.reset_at;
                drop(state);
                tokio::select! {
                    _ = sleep(reset_at.saturating_duration_since(Instant::now())) => {},
                    _ = refunded => {},
                }
                continue;
            }
            let bucket_id = shard % state.limit.max_concurrency;
            let bucket = state
                .buckets
                .entry(bucket_id)
                .or_insert_with(|| Arc::new(Mutex::new(now)))
                .clone();
            drop(state);
            let permit = bucket.lock_owned().await;
            sleep(permit.saturating_duration_since(Instant::now())).await;
            let mut state = self.state.lock().await;
            if state.limit.remaining == 0 || Instant::now() >= state.reset_at {
                drop(permit);
                continue;
            }
            state.limit.remaining -= 1;
            return Ok(StartPermit {
                next: permit,
                interval: self.interval,
                refund_tx: self.refund_tx.clone(),
                epoch: state.epoch.clone(),
                reset_at: state.reset_at,
                committed: false,
                refund_notify: self.refund_notify.clone(),
            });
        }
    }
}

pub(super) struct StartPermit {
    next: tokio::sync::OwnedMutexGuard<Instant>,
    interval: Duration,
    refund_tx: mpsc::UnboundedSender<Arc<()>>,
    epoch: Arc<()>,
    reset_at: Instant,
    committed: bool,
    refund_notify: Arc<Notify>,
}

impl StartPermit {
    pub(super) fn commit(&mut self) -> Result<()> {
        if Instant::now() >= self.reset_at {
            return Err(SdkError::WebSocket(
                "网关启动预约已跨过配额重置时间，请重新连接".into(),
            ));
        }
        self.committed = true;
        Ok(())
    }
}

impl Drop for StartPermit {
    fn drop(&mut self) {
        if self.committed {
            // 发送已尝试，即使响应丢失也不能退款；从发送结束开始冷却。
            *self.next = Instant::now() + self.interval;
        } else {
            // Drop 不能 await；下次预约时由协调器回收同一周期内未使用的配额。
            let _ = self.refund_tx.send(self.epoch.clone());
            self.refund_notify.notify_waiters();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BotMode;

    #[tokio::test]
    async fn starts_use_shard_buckets_and_consume_quota() {
        let client = Arc::new(QQBotClient::new("app", "secret", BotMode::PublicWebSocket).unwrap());
        let mut coordinator = SessionStartCoordinator::new(
            client,
            SessionStartLimit {
                total: 4,
                remaining: 4,
                reset_after: 60_000,
                max_concurrency: 2,
            },
        )
        .unwrap();
        coordinator.interval = Duration::from_millis(20);
        let mut first = coordinator.acquire(0).await.unwrap();
        let mut second = coordinator.acquire(1).await.unwrap();
        first.commit().unwrap();
        second.commit().unwrap();
        assert_eq!(coordinator.state.lock().await.limit.remaining, 2);
        drop(first);
        drop(second);
        let deadline = *coordinator.state.lock().await.buckets[&0].lock().await;
        coordinator.acquire(2).await.unwrap();
        assert!(Instant::now() >= deadline);
        assert_eq!(coordinator.state.lock().await.limit.remaining, 1);
    }

    #[test]
    fn invalid_start_limits_are_rejected() {
        assert!(SessionStartCoordinator::validate(&SessionStartLimit::default()).is_err());
    }

    #[tokio::test]
    async fn unused_reservations_are_refunded_but_attempted_identifies_are_not() {
        let client = Arc::new(QQBotClient::new("app", "secret", BotMode::PublicWebSocket).unwrap());
        let coordinator = SessionStartCoordinator::new(
            client,
            SessionStartLimit {
                total: 1,
                remaining: 1,
                reset_after: 60_000,
                max_concurrency: 1,
            },
        )
        .unwrap();
        let unused = coordinator.acquire(0).await.unwrap();
        let waiter = coordinator.acquire(0);
        tokio::pin!(waiter);
        tokio::select! {
            _ = &mut waiter => panic!("未退款前不应放行"),
            _ = sleep(Duration::from_millis(10)) => {},
        }
        drop(unused);
        let mut permit = tokio::time::timeout(Duration::from_millis(100), &mut waiter)
            .await
            .expect("取消未发送的鉴权必须归还启动配额")
            .unwrap();
        permit.commit().unwrap();
        drop(permit);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), coordinator.acquire(0))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn refunds_from_an_old_quota_period_do_not_increase_the_new_balance() {
        let client = Arc::new(QQBotClient::new("app", "secret", BotMode::PublicWebSocket).unwrap());
        let coordinator = SessionStartCoordinator::new(
            client,
            SessionStartLimit {
                total: 1,
                remaining: 1,
                reset_after: 60_000,
                max_concurrency: 1,
            },
        )
        .unwrap();
        let old = coordinator.acquire(0).await.unwrap();
        coordinator.state.lock().await.epoch = Arc::new(());
        drop(old);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), coordinator.acquire(0))
                .await
                .is_err()
        );
        assert_eq!(coordinator.state.lock().await.limit.remaining, 0);
    }

    #[tokio::test]
    async fn a_reservation_cannot_send_identify_after_its_quota_period_expires() {
        let client = Arc::new(QQBotClient::new("app", "secret", BotMode::PublicWebSocket).unwrap());
        let coordinator = SessionStartCoordinator::new(
            client,
            SessionStartLimit {
                total: 1,
                remaining: 1,
                reset_after: 60_000,
                max_concurrency: 1,
            },
        )
        .unwrap();
        let mut permit = coordinator.acquire(0).await.unwrap();
        permit.reset_at = Instant::now();
        assert!(matches!(permit.commit(), Err(SdkError::WebSocket(_))));
    }
}
