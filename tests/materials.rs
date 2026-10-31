use qqbot_sdk_rs::{Bot, ClientConfig, ImageUploadRequest, SdkError};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

struct Request {
    head: String,
    body: Vec<u8>,
}

fn mock_server(
    replies: impl FnOnce(&str) -> Vec<(u16, String, String)>,
) -> (Bot, thread::JoinHandle<Vec<Request>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let responses = replies(&base);
    let worker = thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, headers, body) in responses {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "expected another request");
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let (head, body_start, length) = loop {
                let mut buffer = [0; 4096];
                let count = stream.read(&mut buffer).unwrap();
                assert_ne!(count, 0);
                bytes.extend_from_slice(&buffer[..count]);
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    let head = String::from_utf8(bytes[..end].to_vec()).unwrap();
                    let length = head
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    break (head, end + 4, length);
                }
            };
            while bytes.len() < body_start + length {
                let mut buffer = [0; 4096];
                let count = stream.read(&mut buffer).unwrap();
                assert_ne!(count, 0);
                bytes.extend_from_slice(&buffer[..count]);
            }
            requests.push(Request {
                head,
                body: bytes[body_start..body_start + length].to_vec(),
            });
            write!(stream, "HTTP/1.1 {status} Response\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}", body.len()).unwrap();
        }
        requests
    });
    let bot = Bot::with_config(
        "app",
        "secret",
        ClientConfig {
            api_base_url: base,
            bot_qps: 100,
            ..Default::default()
        },
    )
    .unwrap();
    (bot, worker)
}

fn json_reply(body: Value) -> (u16, String, String) {
    (
        200,
        "Content-Type: application/json\r\n".into(),
        body.to_string(),
    )
}

fn token_reply() -> (u16, String, String) {
    json_reply(json!({"access_token":"test-token","expires_in":7200}))
}

#[tokio::test]
async fn group_reply_segments_and_typed_requests_share_sequences_but_event_replies_do_not_use_them()
{
    use qqbot_sdk_rs::{MessageRequest, segment};
    let (bot, worker) = mock_server(|_| {
        let mut replies = vec![token_reply()];
        replies.extend((0..4).map(|_| json_reply(json!({"id":"sent"}))));
        replies
    });
    for _ in 0..2 {
        bot.send_group_message(
            "group",
            vec![segment::text("reply"), segment::reply_to("original")],
        )
        .await
        .unwrap();
    }
    bot.send_group_message(
        "group",
        MessageRequest {
            content: Some("typed".into()),
            msg_id: Some("original".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    bot.api()
        .messages()
        .reply_group("group", "event reply", None, Some("event"))
        .await
        .unwrap();
    let requests = worker.join().unwrap();
    for (index, request) in requests[1..4].iter().enumerate() {
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        assert_eq!(
            body["msg_seq"].as_u64(),
            Some(u64::try_from(index + 1).unwrap())
        );
    }
    let event_reply: Value = serde_json::from_slice(&requests[4].body).unwrap();
    assert_eq!(event_reply["event_id"], "event");
    assert!(event_reply.get("msg_seq").is_none());
}

#[tokio::test]
async fn event_replies_allocate_unique_sequences_across_handlers_and_client_clones() {
    use qqbot_sdk_rs::{C2cMessageReceive, EventEnvelope, EventRouter, MessageRequest, segment};
    use std::sync::Arc;
    let (bot, worker) = mock_server(|_| {
        let mut replies = vec![token_reply()];
        replies.extend((0..16).map(|_| json_reply(json!({"id":"sent"}))));
        replies
    });
    let router = EventRouter::new();
    router
        .on::<C2cMessageReceive, _, _>(|event| async move {
            let mut replies = tokio::task::JoinSet::new();
            for index in 0..12 {
                let event = event.clone();
                replies.spawn(async move { event.reply(segment::text(index.to_string())).await });
            }
            while let Some(reply) = replies.join_next().await {
                reply.unwrap()?;
            }
            Ok(())
        })
        .await;
    router
        .on::<C2cMessageReceive, _, _>(|event| async move {
            event.reply(segment::text("second handler")).await?;
            Ok(())
        })
        .await;
    router
        .dispatch(
            EventEnvelope {
                id: Some("event".into()),
                name: "C2C_MESSAGE_CREATE".into(),
                sequence: None,
                data: json!({"id":"original", "author":{"user_openid":"user"}}),
            },
            Arc::new(bot.clone()),
        )
        .await
        .unwrap();
    for sequence in [Some(20), Some(20), None] {
        bot.api()
            .messages()
            .reply_c2c(
                "user",
                MessageRequest {
                    content: Some("retry or next".into()),
                    msg_seq: sequence,
                    ..Default::default()
                },
                Some("original"),
                None,
            )
            .await
            .unwrap();
    }
    let requests = worker.join().unwrap();
    let bodies = requests[1..]
        .iter()
        .map(|request| serde_json::from_slice::<Value>(&request.body).unwrap())
        .collect::<Vec<_>>();
    let mut sequences = bodies[..13]
        .iter()
        .map(|body| body["msg_seq"].as_u64().unwrap())
        .collect::<Vec<_>>();
    sequences.sort_unstable();
    assert_eq!(sequences, (1..=13).collect::<Vec<_>>());
    assert_eq!(bodies[13]["msg_seq"], 20);
    assert_eq!(bodies[14]["msg_seq"], 20);
    assert_eq!(bodies[15]["msg_seq"], 21);
    assert!(
        bodies
            .iter()
            .all(|body| body["msg_id"] == "original" && body.get("event_id").is_none())
    );
}

#[tokio::test]
async fn upload_list_and_delete_follow_the_official_wire_format() {
    let (bot, worker) = mock_server(|base| {
        vec![
            token_reply(),
            json_reply(
                json!({"upload_id":"task","presigned_url":format!("{base}/cos"),"expires_in":3600}),
            ),
            (
                200,
                "ETag: \"900150983cd24fb0d6963f7d28e17f72\"\r\n".into(),
                String::new(),
            ),
            json_reply(
                json!({"image_id":"image","url":"qqbot-media://image/v1/token","ttl":7776000,"preview_url":"https://preview.example/image"}),
            ),
            json_reply(
                json!({"images":[{"image_id":"image","url":"qqbot-media://image/v1/token","file_name":"plot.png","preview_url":"https://preview.example/image","created_at":1792051409_i64,"expires_at":1799827409_i64}],"next_cursor":"a+/=","total":1}),
            ),
            json_reply(json!({"images":[],"next_cursor":"","total":1})),
            json_reply(json!({})),
        ]
    });
    let api = bot.api().materials();
    let image = api.upload("plot.png", "image/png", b"abc").await.unwrap();
    assert_eq!(image.url, "qqbot-media://image/v1/token");
    let page = api.list(None).await.unwrap();
    assert_eq!(page.images[0].file_name, "plot.png");
    assert!(
        api.list(Some(&page.next_cursor))
            .await
            .unwrap()
            .next_cursor
            .is_empty()
    );
    api.delete("image/id").await.unwrap();
    let requests = worker.join().unwrap();
    assert!(
        requests[1]
            .head
            .starts_with("POST /v2/media-images/prepare ")
    );
    let prepare: Value = serde_json::from_slice(&requests[1].body).unwrap();
    assert_eq!(
        prepare,
        json!({"file_name":"plot.png","file_size":3,"content_type":"image/png","md5":"900150983cd24fb0d6963f7d28e17f72","sha1":"a9993e364706816aba3e25717850c26c9cd0d89d"})
    );
    assert!(requests[2].head.starts_with("PUT /cos "));
    assert!(
        requests[2]
            .head
            .to_ascii_lowercase()
            .contains("content-type: image/png")
    );
    assert!(
        !requests[2]
            .head
            .to_ascii_lowercase()
            .contains("authorization:")
    );
    assert_eq!(requests[2].body, b"abc");
    assert!(
        requests[3]
            .head
            .starts_with("POST /v2/media-images/complete ")
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[3].body).unwrap(),
        json!({"upload_id":"task","md5":"900150983cd24fb0d6963f7d28e17f72"})
    );
    assert!(requests[4].head.starts_with("GET /v2/media-images "));
    assert!(
        requests[5]
            .head
            .starts_with("GET /v2/media-images?cursor=a%2B%2F%3D ")
    );
    assert!(
        requests[6]
            .head
            .starts_with("DELETE /v2/media-images/image%2Fid/delete ")
    );
    for index in [1, 3, 4, 5, 6] {
        assert!(
            requests[index]
                .head
                .to_ascii_lowercase()
                .contains("authorization: qqbot test-token")
        );
    }
}

#[tokio::test]
async fn failed_cos_upload_does_not_complete() {
    let (bot, worker) = mock_server(|base| {
        vec![
            token_reply(),
            json_reply(
                json!({"upload_id":"task","presigned_url":format!("{base}/cos"),"expires_in":3600}),
            ),
            (403, String::new(), "denied".into()),
        ]
    });
    assert!(matches!(
        bot.api()
            .materials()
            .upload("plot.png", "image/png", b"abc")
            .await,
        Err(SdkError::Http(_))
    ));
    assert_eq!(worker.join().unwrap().len(), 3);
}

#[tokio::test]
async fn platform_errors_preserve_the_error_code() {
    let (bot, worker) = mock_server(|_| {
        vec![
            token_reply(),
            (
                400,
                String::new(),
                json!({"code":40035007,"message":"quota exceeded"}).to_string(),
            ),
        ]
    });
    assert!(matches!(
        bot.api().materials().list(None).await,
        Err(SdkError::Api { code: 40035007, .. })
    ));
    worker.join().unwrap();
}

#[test]
fn invalid_material_metadata_is_rejected() {
    assert!(ImageUploadRequest::from_bytes("x.png", "image/png", b"").is_err());
    assert!(ImageUploadRequest::from_bytes("x.gif", "image/png", b"abc").is_err());
    assert!(ImageUploadRequest::from_bytes("x.bmp", "image/bmp", b"abc").is_err());
    assert!(
        ImageUploadRequest::from_bytes("x.png", "image/png", &vec![0; 10 * 1024 * 1024 + 1])
            .is_err()
    );
    for (name, mime) in [
        ("x.jpg", "image/jpeg"),
        ("x.jpeg", "image/jpeg"),
        ("x.png", "image/png"),
        ("x.gif", "image/gif"),
        ("x.webp", "image/webp"),
    ] {
        assert!(ImageUploadRequest::from_bytes(name, mime, b"abc").is_ok());
    }
}

#[tokio::test]
async fn successful_http_status_does_not_hide_business_errors_or_audit_data() {
    for (status, error_key) in [(200, "err_code"), (201, "code"), (202, "code")] {
        let (bot, worker) = mock_server(|_| {
            vec![token_reply(), (
            status, "X-Tps-trace-ID: header-trace\r\n".into(),
                    json!({"err_code":0, error_key:304023, "message":"waiting for audit", "trace_id":"body-trace",
                "data":{"message_audit":{"audit_id":"audit-1"}}}).to_string(),
        )]
        });
        let error = bot.user("user").send("hello").await.unwrap_err();
        match error {
            SdkError::Api {
                status: actual,
                code,
                trace_id,
                data,
                ..
            } => {
                assert_eq!(actual, status);
                assert_eq!(code, 304023);
                assert_eq!(trace_id.as_deref(), Some("body-trace"));
                assert_eq!(data.unwrap()["message_audit"]["audit_id"], "audit-1");
            }
            other => panic!("unexpected error: {other}"),
        }
        worker.join().unwrap();
    }
}

#[tokio::test]
async fn authentication_errors_preserve_trace_id_from_headers() {
    let (bot, worker) = mock_server(|_| {
        vec![(
            200,
            "X-Tps-trace-ID: auth-trace\r\n".into(),
            json!({"err_code":100007, "message":"invalid app"}).to_string(),
        )]
    });
    assert!(matches!(bot.auth().token().await,
        Err(SdkError::Api {code:100007,trace_id:Some(trace),..}) if trace == "auth-trace"));
    worker.join().unwrap();
}

#[tokio::test]
async fn stream_messages_keep_all_official_fields_and_response_metadata() {
    use qqbot_sdk_rs::StreamMessageRequest;
    let response = json!({"id":"stream", "timestamp":"2026-10-31T10:00:00+08:00", "remain_msg_len":2000, "ext_info":{"ref_idx":"ref"}});
    let (bot, worker) = mock_server(|_| {
        vec![
            token_reply(),
            json_reply(response.clone()),
            json_reply(response),
        ]
    });
    let mut body = StreamMessageRequest {
        input_mode: Some("append".into()),
        input_state: Some(1),
        index: Some(0),
        content_type: Some("markdown".into()),
        content_raw: Some("first".into()),
        msg_id: Some("message".into()),
        msg_seq: Some(1),
        ..Default::default()
    };
    let first = bot
        .api()
        .users()
        .send_stream("user/id", &body)
        .await
        .unwrap();
    assert_eq!(first.remain_msg_len, Some(2000));
    assert_eq!(first.ext_info.unwrap().ref_idx.as_deref(), Some("ref"));
    body.stream_msg_id = first.id;
    body.index = Some(1);
    body.input_state = Some(10);
    body.msg_id = None;
    body.event_id = Some("event".into());
    body.is_wakeup = Some(true);
    bot.api()
        .users()
        .send_stream("user/id", &body)
        .await
        .unwrap();
    let requests = worker.join().unwrap();
    assert!(
        requests[1]
            .head
            .starts_with("POST /v2/users/user%2Fid/stream_messages ")
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[1].body).unwrap(),
        json!({
        "input_mode":"append","input_state":1,"index":0,"content_type":"markdown","content_raw":"first","msg_id":"message","msg_seq":1})
    );
    let last: Value = serde_json::from_slice(&requests[2].body).unwrap();
    assert_eq!(last["stream_msg_id"], "stream");
    assert_eq!(last["input_state"], 10);
    assert_eq!(last["event_id"], "event");
    assert_eq!(last["is_wakeup"], true);
    assert!(last.get("msg_id").is_none());
}

#[tokio::test]
async fn group_pagination_supports_query_and_documented_get_body_forms() {
    let (bot, worker) = mock_server(|_| {
        let mut replies = vec![token_reply()];
        replies.extend((0..5).map(|_| json_reply(json!({"next_cursor":"next"}))));
        replies
    });
    let api = bot.api().groups();
    api.list_join_approval_strategies_page(Some("a+/="), Some(50))
        .await
        .unwrap();
    let body = json!({"cursor":"next","limit":20});
    api.list_join_approval_strategies_request(&body)
        .await
        .unwrap();
    api.join_request_list_request("group/id", &body)
        .await
        .unwrap();
    api.member_blacklist_request("group/id", &body)
        .await
        .unwrap();
    api.members_request("group/id", &json!({"cursor":"next"}))
        .await
        .unwrap();
    let requests = worker.join().unwrap();
    assert!(
        requests[1]
            .head
            .starts_with("GET /v2/groups/join_approval_strategy?cursor=a%2B%2F%3D&limit=50 ")
    );
    for (index, path) in [
        (2, "/v2/groups/join_approval_strategy"),
        (3, "/v2/groups/group%2Fid/join_request_list"),
        (4, "/v2/groups/group%2Fid/member_blacklist"),
        (5, "/v2/groups/group%2Fid/members"),
    ] {
        assert!(requests[index].head.starts_with(&format!("GET {path} ")));
        let value: Value = serde_json::from_slice(&requests[index].body).unwrap();
        assert_eq!(value["cursor"], "next");
    }
}

#[tokio::test]
async fn typed_media_upload_preserves_file_data_and_file_name() {
    use qqbot_sdk_rs::{MediaTarget, MediaUploadRequest};
    let (bot, worker) = mock_server(|_| {
        vec![
            token_reply(),
            json_reply(json!({
        "file_uuid":"uuid","file_info":"file","ttl":0,"id":"sent","raw_url":"https://example.com/file"})),
            json_reply(json!({"file_info":"merged"})),
        ]
    });
    let body = MediaUploadRequest {
        file_type: Some(4),
        file_name: Some("note.txt".into()),
        file_data: Some("YWJj".into()),
        srv_send_msg: Some(true),
        ..Default::default()
    };
    let media = bot
        .api()
        .messages()
        .upload_media_with(MediaTarget::Group("group"), &body)
        .await
        .unwrap();
    assert_eq!(media.ttl, Some(0));
    assert_eq!(media.id.as_deref(), Some("sent"));
    assert_eq!(media.raw_url.as_deref(), Some("https://example.com/file"));
    let merge = MediaUploadRequest {
        file_type: Some(4),
        file_name: Some("note.txt".into()),
        upload_id: Some("upload".into()),
        ..Default::default()
    };
    bot.api()
        .messages()
        .upload_media_with(MediaTarget::C2c("user"), &merge)
        .await
        .unwrap();
    let requests = worker.join().unwrap();
    assert!(requests[1].head.starts_with("POST /v2/groups/group/files "));
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[1].body).unwrap(),
        json!({
        "file_type":4,"file_name":"note.txt","file_data":"YWJj","srv_send_msg":true})
    );
    assert!(requests[2].head.starts_with("POST /v2/users/user/files "));
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[2].body).unwrap()["upload_id"],
        "upload"
    );
}

#[tokio::test]
async fn missing_etag_stops_material_upload_before_completion() {
    let (bot, worker) = mock_server(|base| {
        vec![
            token_reply(),
            json_reply(
                json!({"upload_id":"task","presigned_url":format!("{base}/cos"),"expires_in":3600}),
            ),
            (200, String::new(), String::new()),
        ]
    });
    assert!(matches!(
        bot.api()
            .materials()
            .upload("image.png", "image/png", b"abc")
            .await,
        Err(SdkError::InvalidInput(_))
    ));
    worker.join().unwrap();
}

#[tokio::test]
async fn cards_and_channel_markdown_keyboards_reach_the_official_message_endpoints() {
    use qqbot_sdk_rs::{TuwenContent, segment};
    let (bot, worker) = mock_server(|_| {
        vec![
            token_reply(),
            json_reply(json!({"id":"c2c"})),
            json_reply(json!({"id":"group"})),
            json_reply(json!({"id":"channel"})),
            json_reply(json!({"id":"markdown"})),
        ]
    });
    let card = TuwenContent {
        title: "标题".into(),
        description: "说明".into(),
        url: "https://example.com/news".into(),
        pic_url: "https://example.com/image.png".into(),
    };
    bot.user("user")
        .send(segment::card(card.clone()))
        .await
        .unwrap();
    bot.group("group").send(segment::card(card)).await.unwrap();
    bot.channel("channel").send([segment::markdown("**通知**"), segment::keyboard_content(json!({
        "rows":[{"buttons":[{"id":"confirm", "group_id":"once", "render_data":{"label":"确认", "style":4},
        "action":{"type":1,"data":"confirm","permission":{"type":2},"modal":{"content":"确认执行？","confirm_text":"确认","cancel_text":"取消"}}}]}]
    }))]).await.unwrap();
    bot.group("group")
        .send(qqbot_sdk_rs::MessageRequest {
            markdown: Some(json!({"content":"![图表](qqbot-media://image/v1/token)"})),
            force_verify_image_resource: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = worker.join().unwrap();
    for (index, path) in [
        (1, "/v2/users/user/messages"),
        (2, "/v2/groups/group/messages"),
    ] {
        assert!(requests[index].head.starts_with(&format!("POST {path} ")));
        let body: Value = serde_json::from_slice(&requests[index].body).unwrap();
        assert_eq!(body["msg_type"], 8);
        assert_eq!(body["card"]["type"], "tuwen");
        assert_eq!(body["card"]["content"]["title"], "标题");
    }
    assert!(
        requests[3]
            .head
            .starts_with("POST /channels/channel/messages ")
    );
    let body: Value = serde_json::from_slice(&requests[3].body).unwrap();
    assert!(body.get("msg_type").is_none());
    let button = &body["keyboard"]["content"]["rows"][0]["buttons"][0];
    assert_eq!(button["action"]["modal"]["confirm_text"], "确认");
    assert_eq!(button["group_id"], "once");
    assert_eq!(button["render_data"]["style"], 4);
    let body: Value = serde_json::from_slice(&requests[4].body).unwrap();
    assert_eq!(body["markdown"]["force_verify_image_resource"], true);
    assert!(body.get("force_verify_image_resource").is_none());
}

#[tokio::test]
async fn raw_messages_preserve_native_fields_in_all_scenes_and_reply_helpers() {
    use qqbot_sdk_rs::segment;
    let (bot, worker) = mock_server(|_| {
        let mut replies = vec![token_reply()];
        replies.extend((0..9).map(|_| json_reply(json!({"id":"sent"}))));
        replies
    });
    // Deliberately outside the typed schema: raw must neither reject nor rewrite it.
    let body = json!({"msg_type":99, "content":"native", "markdown":null,
        "card":{"type":"future","content":{"unknown":true}},
        "msg_id":"provided", "event_id":"also-provided", "msg_seq":17,
        "force_verify_image_resource":true, "future_field":{"nested":[1,null,false]}});
    let raw = segment::raw(body.clone());
    bot.user("u/id").send(raw.clone()).await.unwrap();
    bot.group("g/id").send(raw.clone()).await.unwrap();
    bot.channel("c/id").send(raw.clone()).await.unwrap();
    bot.direct("d/id").send(raw.clone()).await.unwrap();
    let api = bot.api().messages();
    api.reply_c2c("u/id", raw.clone(), Some("helper-msg"), None)
        .await
        .unwrap();
    api.reply_group("g/id", raw.clone(), None, Some("helper-event"))
        .await
        .unwrap();
    api.reply_channel("c/id", raw.clone(), Some("helper-msg"), None)
        .await
        .unwrap();
    api.reply_dm("d/id", raw, None, Some("helper-event"))
        .await
        .unwrap();
    let minimal = json!({"content":"no inferred type", "extra":null});
    bot.group("g")
        .send(segment::raw(minimal.clone()))
        .await
        .unwrap();
    let requests = worker.join().unwrap();
    for (index, path) in [
        "/v2/users/u%2Fid/messages",
        "/v2/groups/g%2Fid/messages",
        "/channels/c%2Fid/messages",
        "/dms/d%2Fid/messages",
    ]
    .into_iter()
    .cycle()
    .take(8)
    .enumerate()
    {
        let request = &requests[index + 1];
        assert!(request.head.starts_with(&format!("POST {path} ")));
        assert_eq!(
            serde_json::from_slice::<Value>(&request.body).unwrap(),
            body
        );
        assert!(
            request
                .head
                .to_ascii_lowercase()
                .contains("authorization: qqbot test-token")
        );
    }
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[9].body).unwrap(),
        minimal
    );
}

#[tokio::test]
async fn raw_rejects_mixing_and_upload_wrappers_before_network_requests() {
    use qqbot_sdk_rs::{MediaTarget, MessageRequest, segment};
    let bot = Bot::with_config(
        "app",
        "secret",
        ClientConfig {
            api_base_url: "http://127.0.0.1:1".into(),
            ..Default::default()
        },
    )
    .unwrap();
    for segments in [
        vec![segment::raw(json!({})), segment::text("text")],
        vec![segment::text("text"), segment::raw(json!({}))],
        vec![segment::raw(json!({})), segment::raw(json!({}))],
        vec![segment::raw(json!([1, 2]))],
    ] {
        assert!(matches!(
            bot.group("g").send(segments).await,
            Err(SdkError::InvalidInput(_))
        ));
    }
    let api = bot.api().messages();
    assert!(matches!(
        api.send_media(MediaTarget::Group("g"), 1, b"data", segment::raw(json!({})))
            .await,
        Err(SdkError::InvalidInput(_))
    ));
    assert!(matches!(
        api.send_media_url(
            MediaTarget::Group("g"),
            1,
            "https://example.com/image",
            segment::raw(json!({}))
        )
        .await,
        Err(SdkError::InvalidInput(_))
    ));
    assert!(MessageRequest::from_segments([segment::raw(json!({"future":true}))]).is_err());
}
