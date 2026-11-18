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
    retention: Duration,
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
        // 单聊回复序号保留 60 分钟，群聊保留 5 分钟。
        if state
            .last_cleanup
            .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(60))
        {
            state
                .entries
                .retain(|_, entry| now.duration_since(entry.touched) < entry.retention);
            state.last_cleanup = Some(now);
        }
        let entry = state
            .entries
            .entry((path.into(), message_id.into()))
            .or_insert(Entry {
                highest: 0,
                touched: now,
                retention: Duration::from_secs(if path.starts_with("/v2/users/") {
                    60 * 60
                } else {
                    5 * 60
                }),
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

    #[tokio::test]
    async fn c2c_and_group_sequences_expire_with_their_respective_windows() {
        let sequences = ReplySequences::default();
        let c2c = "/v2/users/user/messages";
        let group = "/v2/groups/group/messages";
        assert_eq!(sequences.reserve(c2c, "message", None).await.unwrap(), 1);
        assert_eq!(sequences.reserve(group, "message", None).await.unwrap(), 1);
        {
            let mut state = sequences.state.lock().await;
            let c2c_entry = state
                .entries
                .get_mut(&(c2c.into(), "message".into()))
                .unwrap();
            assert_eq!(c2c_entry.retention, Duration::from_secs(60 * 60));
            c2c_entry.touched -= Duration::from_secs(59 * 60);
            let group_entry = state
                .entries
                .get_mut(&(group.into(), "message".into()))
                .unwrap();
            assert_eq!(group_entry.retention, Duration::from_secs(5 * 60));
            group_entry.touched -= Duration::from_secs(5 * 60);
            state.last_cleanup = None;
        }
        assert_eq!(sequences.reserve(c2c, "message", None).await.unwrap(), 2);
        assert_eq!(sequences.reserve(group, "message", None).await.unwrap(), 1);
        assert_eq!(sequences.reserve(c2c, "message", Some(4)).await.unwrap(), 4);
        assert_eq!(sequences.reserve(c2c, "message", None).await.unwrap(), 5);
        {
            let mut state = sequences.state.lock().await;
            state
                .entries
                .get_mut(&(c2c.into(), "message".into()))
                .unwrap()
                .touched -= Duration::from_secs(60 * 60);
            state.last_cleanup = None;
        }
        assert_eq!(sequences.reserve(c2c, "message", None).await.unwrap(), 1);
    }
}
