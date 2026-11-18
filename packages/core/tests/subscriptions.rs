use qqbot_sdk_core::{Bot, BotMode, EVENT_TOPICS, EventEnvelope, EventRouter, FriendAdd, SdkError};
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

fn bot() -> Arc<Bot> {
    Arc::new(Bot::new("test", "secret", BotMode::PublicWebSocket).unwrap())
}

fn envelope(name: &str) -> EventEnvelope {
    EventEnvelope {
        id: Some("event-id".into()),
        name: name.into(),
        sequence: Some(42),
        data: json!({"openid": "friend-id"}),
    }
}

#[tokio::test]
async fn bot_topics_bubble_with_context_on_an_independent_router() {
    let bot = bot();
    let router = EventRouter::new();
    let observed = Arc::new(Mutex::new(Vec::new()));
    for name in [
        "event",
        "FRIEND_ADD",
        "friendAdd",
        "notice.friend.increase",
        "notice.friend",
        "notice",
    ] {
        let observed = observed.clone();
        bot.on(name, move |event| {
            let observed = observed.clone();
            async move {
                assert_eq!(event.name, "FRIEND_ADD");
                assert_eq!(event.sequence, Some(42));
                assert_eq!(event.data["openid"], "friend-id");
                event.parse::<FriendAdd>()?.user()?;
                assert!(event.parse::<qqbot_sdk_core::GroupMemberAdd>().is_err());
                assert!(matches!(
                    event.reply("no").await,
                    Err(SdkError::InvalidInput(_))
                ));
                observed.lock().unwrap().push(name);
                Ok(())
            }
        })
        .await;
    }
    router.dispatch(envelope("FRIEND_ADD"), bot).await.unwrap();
    assert_eq!(
        *observed.lock().unwrap(),
        [
            "event",
            "FRIEND_ADD",
            "friendAdd",
            "notice.friend.increase",
            "notice.friend",
            "notice"
        ]
    );
}

#[tokio::test]
async fn once_is_shared_by_clones_and_concurrent_dispatches_without_duplicates() {
    let bot = bot();
    let count = Arc::new(AtomicUsize::new(0));
    let calls = count.clone();
    bot.once("notice", move |_| {
        let calls = calls.clone();
        async move {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    })
    .await;
    let router = bot.event_router();
    let (a, b) = tokio::join!(
        router.dispatch(envelope("FRIEND_ADD"), bot.clone()),
        router.dispatch(envelope("FRIEND_ADD"), bot.clone()),
    );
    a.unwrap();
    b.unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 1);
    let calls = count.clone();
    let id = bot
        .on("notice", move |_| {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        })
        .await;
    assert!(bot.as_ref().clone().off(id).await);
    assert!(!bot.off(id).await);
    router.dispatch(envelope("FRIEND_ADD"), bot).await.unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn typed_raw_and_named_handlers_coexist_and_errors_propagate() {
    let bot = bot();
    let router = EventRouter::new();
    let count = Arc::new(AtomicUsize::new(0));
    let calls = count.clone();
    router
        .on::<FriendAdd, _, _>(move |_| {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        })
        .await;
    let calls = count.clone();
    router
        .on_raw(move |_, _| {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        })
        .await;
    let calls = count.clone();
    router
        .on_named("notice", move |_| {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        })
        .await;
    router
        .dispatch(envelope("FRIEND_ADD"), bot.clone())
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 3);
    bot.on("notice", |_| async {
        Err(SdkError::InvalidInput("handler failed".into()))
    })
    .await;
    assert!(matches!(router.dispatch(envelope("FRIEND_ADD"), bot).await,
        Err(SdkError::InvalidInput(message)) if message == "handler failed"));
}

#[tokio::test]
async fn every_official_topic_matches_the_node_mapping_and_dispatches_once() {
    let node_source =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../npm/src/event-topics.ts");
    let source = node_source
        .exists()
        .then(|| std::fs::read_to_string(node_source).unwrap());
    assert_eq!(EVENT_TOPICS.len(), 58);
    for name in ["GROUP_AT_MESSAGE_CREATE", "GROUP_MESSAGE_CREATE"] {
        assert!(EVENT_TOPICS.contains(&(name, "message.group")));
    }
    for (official, topic) in EVENT_TOPICS {
        if let Some(source) = &source {
            assert!(source.contains(&format!("{official}: '{topic}'")));
        }
        let bot = bot();
        let count = Arc::new(AtomicUsize::new(0));
        let mut current = *topic;
        let mut expected = 0;
        loop {
            let calls = count.clone();
            bot.on(current, move |_| {
                let calls = calls.clone();
                async move {
                    calls.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            })
            .await;
            expected += 1;
            let Some((parent, _)) = current.rsplit_once('.') else {
                break;
            };
            current = parent;
        }
        bot.event_router()
            .dispatch(envelope(official), bot.clone())
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::Relaxed), expected, "{official}");
        bot.event_router()
            .dispatch(envelope("UNKNOWN_EVENT"), bot.clone())
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::Relaxed), expected);
    }
}

#[tokio::test]
async fn continue_policy_delivers_after_errors_and_preserves_once() {
    use qqbot_sdk_core::events::DispatchErrorPolicy;
    let bot = bot();
    let router = EventRouter::new();
    router
        .on::<FriendAdd, _, _>(|_| async { Err(SdkError::InvalidInput("first".into())) })
        .await;
    let count = Arc::new(AtomicUsize::new(0));
    let calls = count.clone();
    bot.once("notice", move |_| {
        let calls = calls.clone();
        async move {
            calls.fetch_add(1, Ordering::Relaxed);
            Err(SdkError::InvalidInput("second".into()))
        }
    })
    .await;
    assert!(
        router
            .dispatch(envelope("FRIEND_ADD"), bot.clone())
            .await
            .is_err()
    );
    assert_eq!(count.load(Ordering::Relaxed), 0);
    let router = router.with_error_policy(DispatchErrorPolicy::Continue);
    for _ in 0..2 {
        assert!(
            matches!(router.dispatch(envelope("FRIEND_ADD"), bot.clone()).await,
            Err(SdkError::InvalidInput(message)) if message == "first")
        );
    }
    assert_eq!(count.load(Ordering::Relaxed), 1);
}
