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
