use crate::{Result, SdkError};
use std::{collections::HashMap, time::Duration};
use tokio::{sync::Mutex, time::Instant};

#[derive(Default)]
pub(crate) struct ReplySequences {
    state: Mutex<SequenceState>,
}

#[derive(Default)]
struct SequenceState {
    entries: HashMap<(String, String), Entry>,
    last_cleanup: Option<Instant>,
}

struct Entry {
    highest: u32,
    touched: Instant,
}

impl ReplySequences {
    pub(crate) async fn reserve(
        &self,
        path: &str,
        message_id: &str,
        explicit: Option<u32>,
    ) -> Result<u32> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        // 被动回复窗口为五分钟；额外保留五分钟，覆盖请求排队和断线补发。
        if state
            .last_cleanup
            .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(60))
        {
            state
                .entries
                .retain(|_, entry| now.duration_since(entry.touched) < Duration::from_secs(600));
            state.last_cleanup = Some(now);
        }
        let entry = state
            .entries
            .entry((path.into(), message_id.into()))
            .or_insert(Entry {
                highest: 0,
                touched: now,
            });
        let sequence = match explicit {
            Some(sequence) => sequence,
            None => entry.highest.checked_add(1).ok_or_else(|| {
                SdkError::InvalidInput("回复序号已耗尽，请显式指定重试序号".into())
            })?,
        };
        // 显式序号允许重试，自动序号只向前推进。
        entry.highest = entry.highest.max(sequence);
        entry.touched = now;
        Ok(sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn concurrent_replies_share_a_counter_and_explicit_retries_stay_stable() {
        let sequences = Arc::new(ReplySequences::default());
        let mut tasks = tokio::task::JoinSet::new();
        for _ in 0..32 {
            let sequences = sequences.clone();
            tasks.spawn(async move { sequences.reserve("group", "msg", None).await.unwrap() });
        }
        let mut values = Vec::new();
        while let Some(value) = tasks.join_next().await {
            values.push(value.unwrap());
        }
        values.sort_unstable();
        assert_eq!(values, (1..=32).collect::<Vec<_>>());
        assert_eq!(
            sequences.reserve("group", "msg", Some(40)).await.unwrap(),
            40
        );
        assert_eq!(
            sequences.reserve("group", "msg", Some(40)).await.unwrap(),
            40
        );
        assert_eq!(sequences.reserve("group", "msg", None).await.unwrap(), 41);
        assert_eq!(sequences.reserve("user", "msg", None).await.unwrap(), 1);
        assert_eq!(sequences.reserve("group", "other", None).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn exhausted_sequences_do_not_wrap_and_old_entries_are_removed() {
        let sequences = ReplySequences::default();
        sequences
            .reserve("group", "msg", Some(u32::MAX))
            .await
            .unwrap();
        assert!(sequences.reserve("group", "msg", None).await.is_err());
        {
            let mut state = sequences.state.lock().await;
            state.entries.values_mut().next().unwrap().touched =
                Instant::now() - Duration::from_secs(601);
            state.last_cleanup = None;
        }
        assert_eq!(sequences.reserve("group", "msg", None).await.unwrap(), 1);
    }
}
