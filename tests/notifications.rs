use qqbot_sdk_rs::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

async fn check<T: Event + Serialize>(name: &str, body: Value) {
    let router = EventRouter::new();
    let count = Arc::new(AtomicUsize::new(0));
    let received = count.clone();
    router
        .on::<T, _, _>(move |event| {
            let count = received.clone();
            async move {
                assert_eq!(serde_json::to_value(event).unwrap()["future"], true);
                count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        })
        .await;
    assert_eq!(T::NAME, name);
    router
        .dispatch(
            EventEnvelope {
                id: Some("evt".into()),
                name: name.into(),
                sequence: Some(1),
                data: body,
            },
            Arc::new(Bot::new("app", "secret", BotMode::Webhook).unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn all_added_notifications_reach_typed_handlers_and_preserve_unknown_fields() {
    macro_rules! check {
        ($body:expr; $($ty:ident => $name:literal),+ $(,)?) => {
            $(check::<$ty>($name, $body).await;)+
        };
    }
    check!(json!({"message":{"id":"msg"},"op_user":{"id":"user"},"future":true});
        MessageDelete => "MESSAGE_DELETE",
        PublicMessageDelete => "PUBLIC_MESSAGE_DELETE",
        DirectMessageDelete => "DIRECT_MESSAGE_DELETE",
    );
    check!(json!({"user_id":"user","guild_id":"guild","channel_id":"channel","target":{"id":"msg","type":0},"emoji":{"id":"1","type":1},"future":true});
        MessageReactionAdd => "MESSAGE_REACTION_ADD",
        MessageReactionRemove => "MESSAGE_REACTION_REMOVE",
    );
    check!(json!({"guild_id":47129941624960822_u64,"channel_id":1661124,"author_id":144115218182563108_u64,"future":true,"thread_info":{"title":[{"type":1,"text_info":{"text":"title"}}],"content":"body","future_nested":1}});
        ForumThreadCreate => "FORUM_THREAD_CREATE",
        ForumThreadUpdate => "FORUM_THREAD_UPDATE",
        ForumThreadDelete => "FORUM_THREAD_DELETE",
        ForumPostCreate => "FORUM_POST_CREATE",
        ForumPostDelete => "FORUM_POST_DELETE",
        ForumReplyCreate => "FORUM_REPLY_CREATE",
        ForumReplyDelete => "FORUM_REPLY_DELETE",
        ForumPublishAuditResult => "FORUM_PUBLISH_AUDIT_RESULT",
        OpenForumThreadCreate => "OPEN_FORUM_THREAD_CREATE",
        OpenForumThreadUpdate => "OPEN_FORUM_THREAD_UPDATE",
        OpenForumThreadDelete => "OPEN_FORUM_THREAD_DELETE",
        OpenForumPostCreate => "OPEN_FORUM_POST_CREATE",
        OpenForumPostDelete => "OPEN_FORUM_POST_DELETE",
        OpenForumReplyCreate => "OPEN_FORUM_REPLY_CREATE",
        OpenForumReplyDelete => "OPEN_FORUM_REPLY_DELETE",
    );
    check!(json!({"guild_id":"guild","channel_id":"channel","audio_url":"https://example.com/audio","text":"title","future":true});
        AudioStart => "AUDIO_START",
        AudioFinish => "AUDIO_FINISH",
        AudioOnMic => "AUDIO_ON_MIC",
        AudioOffMic => "AUDIO_OFF_MIC",
    );
    check!(json!({"guild_id":"guild","channel_id":"channel","channel_type":2,"user_id":"user","future":true});
        AudioOrLiveChannelMemberEnter => "AUDIO_OR_LIVE_CHANNEL_MEMBER_ENTER",
        AudioOrLiveChannelMemberExit => "AUDIO_OR_LIVE_CHANNEL_MEMBER_EXIT",
    );
}

#[test]
fn official_forum_number_ids_and_rich_content_are_readable() {
    let event: ForumThreadCreate = serde_json::from_value(json!({
        "guild_id":47129941624960822_u64,"channel_id":"1661124","author_id":144115218182563108_u64,
        "thread_info":{"thread_id":"thread","title":[{"type":1,"text_info":{"text":"title"}}],"content":"body","future_nested":1}
    })).unwrap();
    assert_eq!(event.data.guild_id.as_deref(), Some("47129941624960822"));
    let thread = event.data.thread_info.unwrap();
    assert_eq!(thread.title.unwrap()[0]["text_info"]["text"], "title");
    assert_eq!(thread.extra["future_nested"], 1);
    let deleted: MessageDelete =
        serde_json::from_value(json!({"message":{"id":"msg"},"op_user":{"id":"user"}})).unwrap();
    assert_eq!(deleted.data.message.unwrap().id.as_deref(), Some("msg"));
    assert_eq!(deleted.data.op_user.unwrap().id.as_deref(), Some("user"));
}
