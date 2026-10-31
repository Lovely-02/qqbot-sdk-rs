//! Official field-name/type snapshots, using synthetic values and source URLs.
use qqbot_sdk_rs::models::GuildMember;
use qqbot_sdk_rs::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn assert_subset(expected: &Value, actual: &Value, path: &str) {
    match expected {
        Value::Object(fields) => {
            for (key, value) in fields {
                assert!(
                    actual.get(key).is_some(),
                    "missing documented field {path}.{key}"
                );
                assert_subset(value, &actual[key], &format!("{path}.{key}"));
            }
        }
        Value::Array(items) => {
            assert_eq!(actual.as_array().map(Vec::len), Some(items.len()), "{path}");
            for (index, item) in items.iter().enumerate() {
                assert_subset(item, &actual[index], &format!("{path}[{index}]"));
            }
        }
        _ => assert_eq!(expected, actual, "{path}"),
    }
}

fn roundtrip<T: DeserializeOwned + Serialize>(value: Value, name: &str) {
    let parsed: T =
        serde_json::from_value(value.clone()).unwrap_or_else(|error| panic!("{name}: {error}"));
    assert_subset(&value, &serde_json::to_value(parsed).unwrap(), name);
}

#[test]
fn documented_typed_fields_survive_deserialization_and_serialization() {
    let snapshot: Value =
        serde_json::from_str(include_str!("fixtures/official-fields.json")).unwrap();
    for case in snapshot["cases"].as_array().unwrap() {
        let name = case["type"].as_str().unwrap();
        let payload = case["payload"].clone();
        macro_rules! check_types {
            ($($ty:ident),+ $(,)?) => {
                match name {
                    $(stringify!($ty) => roundtrip::<$ty>(payload, name),)+
                    _ => panic!("snapshot type has no test: {name}"),
                }
            };
        }
        check_types!(
            Ark,
            GuildMember,
            MessageExtInfo,
            Keyboard,
            ImageUploadRequest,
            ImageUploadPreparation,
            ImageUploadCompletion,
            MaterialImage,
            MaterialImageEntry,
            MaterialImagePage,
            ArkData,
            C2cMessageReceive,
            C2cMsgReceive,
            C2cMsgReject,
            Channel,
            ChannelEvent,
            Embed,
            FriendAdd,
            FriendAuthor,
            FriendDelete,
            Group,
            GroupAddRobot,
            GroupAtMessageCreate,
            GroupDelRobot,
            GroupJoinRequest,
            GroupMemberAdd,
            GroupMemberRemove,
            GroupMessageCreate,
            GroupMessageSetting,
            Guild,
            GuildEvent,
            InteractionAuthorizeData,
            InteractionCreate,
            InteractionData,
            InteractionMessageScene,
            InteractionResolved,
            JoinAutoApproved,
            JoinVerifyInfo,
            Media,
            MediaUploadRequest,
            Message,
            MessageAttachment,
            MessageAudited,
            MessageCard,
            MessageElement,
            MessageReference,
            MessageRequest,
            MessageScene,
            ReviewQuestion,
            StreamMessageRequest,
            StreamMessageResponse,
            SubscribeMessageStatus,
            SubscribeMessageTemplateResult,
            TuwenContent,
            User,
        );
    }
}

#[test]
fn channel_messages_and_audit_events_expose_documented_fields() {
    let value = json!({"id":"message", "edited_timestamp":"2026-10-31T10:00:00+08:00",
        "ark":{"template_id":23,"kv":[{"key":"#DESC#","value":"summary"}]},
        "embeds":[{"description":"description"}]});
    roundtrip::<MessageCreateEvent>(value.clone(), "MessageCreateEvent");
    let event: MessageCreateEvent = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(event.ark.unwrap().template_id, 23);
    assert!(event.edited_timestamp.is_some());
    assert_eq!(event.embeds[0].description.as_deref(), Some("description"));
    roundtrip::<DirectMessageCreate>(value, "DirectMessageCreate");
    let event: MessageAuditPass =
        serde_json::from_value(json!({"audit_id":"audit", "seq_in_channel":"42"})).unwrap();
    assert_eq!(event.data.seq_in_channel.as_deref(), Some("42"));
}

#[test]
fn preupload_configuration_and_part_sizes_remain_available() {
    use qqbot_sdk_rs::api::message::UploadPrepareResponse;
    let preparation: UploadPrepareResponse = serde_json::from_value(json!({
        "upload_id":"upload", "block_size":"5242880",
        "parts":[{"index":0,"presigned_url":"https://example.com/part","block_size":123}],
        "upload_config":{"concurrency":2,"retry_timeout":300,"retry_delay":1},
    }))
    .unwrap();
    assert_eq!(preparation.block_size, 5_242_880);
    assert_eq!(preparation.parts[0].block_size, 123);
    let config = preparation.upload_config.unwrap();
    assert_eq!(config.concurrency, Some(2));
    assert_eq!(config.retry_timeout, Some(300));
    assert_eq!(config.retry_delay, Some(1));
}
