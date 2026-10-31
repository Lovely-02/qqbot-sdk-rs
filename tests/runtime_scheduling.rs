mod support;
use futures_util::{SinkExt, StreamExt};
use qqbot_sdk_rs::{EventRouter, GatewayConfig, GatewayShardManager, MediaTarget};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use support::{MockHttp, Reply};
use tokio::{
    sync::{Barrier, Mutex},
    time::timeout,
};

#[tokio::test]
async fn uploads_are_concurrent_and_retry_put_and_confirmation_without_reuploading_success() {
    let barrier = Arc::new(Barrier::new(2));
    let records = Arc::new(Mutex::new(Vec::new()));
    let attempt = Arc::new(AtomicUsize::new(0));
    let confirmations = Arc::new(AtomicUsize::new(0));
    let server = MockHttp::start({
        let records = records.clone(); let barrier = barrier.clone();
        let attempt = attempt.clone(); let confirmations = confirmations.clone();
        move |request| {
            let records = records.clone(); let barrier = barrier.clone();
            let attempt = attempt.clone(); let confirmations = confirmations.clone();
            async move {
                records.lock().await.push((request.path.clone(), request.body.clone()));
                match request.path.as_str() {
                    "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                    "/v2/groups/group/upload_prepare" => Reply::json(json!({"upload_id":"upload","parts":[
                        {"index":1,"block_size":"2","presigned_url":format!("{}/put/1",request.base_url)},
                        {"index":0,"block_size":"2","presigned_url":format!("{}/put/0",request.base_url)}],
                        "upload_config":{"concurrency":2,"retry_timeout":2,"retry_delay":0}})),
                    "/put/0" => {
                        assert!(!request.head.to_ascii_lowercase().contains("authorization:"));
                        if attempt.fetch_add(1, Ordering::SeqCst) == 0 {
                            barrier.wait().await;
                            Reply::status(503)
                        } else { Reply::status(200) }
                    },
                    "/put/1" => { barrier.wait().await; Reply::status(200) },
                    "/v2/groups/group/upload_part_finish" => {
                        if confirmations.fetch_add(1, Ordering::SeqCst) == 0 { Reply::status(503) } else { Reply::json(json!({})) }
                    },
                    "/v2/groups/group/files" => {
                        assert!(confirmations.load(Ordering::SeqCst) >= 3);
                        Reply::json(json!({"file_info":"file","file_uuid":"uuid","ttl":60}))
                    },
                    other => panic!("unexpected path {other}"),
                }
            }
        }
    }).await;
    let bot = server.bot();
    let uploaded = timeout(
        Duration::from_secs(3),
        bot.api()
            .messages()
            .upload_media(MediaTarget::Group("group"), 4, b"abcd", false),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(uploaded.file_info, "file");
    let records = records.lock().await;
    let first = records
        .iter()
        .filter(|(path, _)| path == "/put/0")
        .collect::<Vec<_>>();
    assert_eq!(first.len(), 2);
    assert!(first.iter().all(|(_, body)| body == b"ab"));
    let second = records
        .iter()
        .filter(|(path, _)| path == "/put/1")
        .collect::<Vec<_>>();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].1, b"cd");
    let finishes = records
        .iter()
        .filter(|(path, _)| path.ends_with("upload_part_finish"))
        .map(|(_, b)| serde_json::from_slice::<Value>(b).unwrap())
        .collect::<Vec<_>>();
    for body in finishes {
        assert_eq!(body["upload_id"], "upload");
        assert_eq!(body["block_size"], "2");
    }
    assert_eq!(records.last().unwrap().0, "/v2/groups/group/files");
}

#[tokio::test]
async fn malformed_parts_and_permanent_upload_failures_never_merge() {
    for malformed in [true, false] {
        let puts = Arc::new(AtomicUsize::new(0));
        let merges = Arc::new(AtomicUsize::new(0));
        let server = MockHttp::start({ let puts = puts.clone(); let merges = merges.clone(); move |request| {
            let puts = puts.clone(); let merges = merges.clone(); async move {
                match request.path.as_str() {
                    "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                    "/v2/users/user/upload_prepare" => Reply::json(json!({"upload_id":"upload","parts":[
                        {"index":if malformed { 1 } else { 0 },"block_size":"2","presigned_url":format!("{}/put",request.base_url)}],
                        "upload_config":{"concurrency":1,"retry_timeout":2,"retry_delay":0}})),
                    "/put" => { puts.fetch_add(1, Ordering::SeqCst); Reply::status(403) },
                    "/v2/users/user/files" => { merges.fetch_add(1, Ordering::SeqCst); Reply::json(json!({})) },
                    other => panic!("unexpected path {other}"),
                }
            }
        }}).await;
        let bot = server.bot();
        assert!(
            bot.api()
                .messages()
                .upload_media(MediaTarget::C2c("user"), 4, b"ab", false)
                .await
                .is_err()
        );
        assert_eq!(puts.load(Ordering::SeqCst), usize::from(!malformed));
        assert_eq!(merges.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn all_shards_start_after_quota_refresh_and_cancel_when_one_fails() {
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let identified = Arc::new(Mutex::new(Vec::new()));
    let websocket = tokio::spawn({
        let identified = identified.clone();
        async move {
            let barrier = Arc::new(Barrier::new(2));
            let mut connections = tokio::task::JoinSet::new();
            for _ in 0..2 {
                let (socket, _) = listener.accept().await.unwrap();
                let barrier = barrier.clone();
                let identified = identified.clone();
                connections.spawn(async move {
                    let mut socket = accept_async(socket).await.unwrap();
                    socket
                        .send(Message::Text(
                            json!({"op":10,"d":{"heartbeat_interval":45000}})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                    let payload: Value = serde_json::from_str(
                        &socket.next().await.unwrap().unwrap().into_text().unwrap(),
                    )
                    .unwrap();
                    assert_eq!(payload["op"], 2);
                    assert_eq!(payload["d"]["shard"][1], 2);
                    let shard = payload["d"]["shard"][0].as_u64().unwrap();
                    identified.lock().await.push(shard);
                    barrier.wait().await;
                    if shard == 0 {
                        socket
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::from(4001),
                                reason: "fatal".into(),
                            })))
                            .await
                            .unwrap();
                    } else {
                        // 管理器必须取消另一条仍存活的连接。
                        let _ = socket.next().await;
                    }
                });
            }
            while let Some(result) = connections.join_next().await {
                result.unwrap();
            }
        }
    });
    let lookups = Arc::new(AtomicUsize::new(0));
    let server = MockHttp::start({ let lookups = lookups.clone(); move |request| {
        let url = url.clone(); let lookups = lookups.clone(); async move {
            match request.path.as_str() {
                "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                "/gateway/bot" => {
                    let first = lookups.fetch_add(1, Ordering::SeqCst) == 0;
                    Reply::json(json!({"url":url,"shards":2,"session_start_limit":{"total":10,"remaining":if first {0} else {2},"reset_after":if first {20} else {60000},"max_concurrency":2}}))
                },
                other => panic!("unexpected path {other}"),
            }
        }
    }}).await;
    let manager = GatewayShardManager::new(
        Arc::new(server.bot()),
        EventRouter::new(),
        GatewayConfig::default(),
    )
    .await
    .unwrap();
    assert_eq!(manager.shard_count(), 2);
    assert!(
        timeout(Duration::from_secs(3), manager.run())
            .await
            .unwrap()
            .is_err()
    );
    timeout(Duration::from_secs(1), websocket)
        .await
        .unwrap()
        .unwrap();
    let mut shards = identified.lock().await.clone();
    shards.sort_unstable();
    assert_eq!(shards, [0, 1]);
    assert_eq!(lookups.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn resume_bypasses_identify_quota_and_uses_the_processed_sequence() {
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let websocket = tokio::spawn(async move {
        for connection in 0..2 {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(socket).await.unwrap();
            socket
                .send(Message::Text(
                    json!({"op":10,"d":{"heartbeat_interval":45000}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let payload: Value =
                serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap())
                    .unwrap();
            if connection == 0 {
                assert_eq!(payload["op"], 2);
                socket
                    .send(Message::Text(
                        json!({"op":0,"s":1,"t":"READY","d":{"session_id":"session"}})
                            .to_string()
                            .into(),
                    ))
                    .await
                    .unwrap();
                socket
                    .send(Message::Close(Some(CloseFrame {
                        code: CloseCode::from(4009),
                        reason: "resume".into(),
                    })))
                    .await
                    .unwrap();
            } else {
                assert_eq!(payload["op"], 6);
                assert_eq!(payload["d"]["session_id"], "session");
                assert_eq!(payload["d"]["seq"], 1);
                socket
                    .send(Message::Close(Some(CloseFrame {
                        code: CloseCode::from(4001),
                        reason: "fatal".into(),
                    })))
                    .await
                    .unwrap();
            }
        }
    });
    let lookups = Arc::new(AtomicUsize::new(0));
    let server = MockHttp::start({ let lookups = lookups.clone(); move |request| {
        let url = url.clone(); let lookups = lookups.clone(); async move {
            match request.path.as_str() {
                "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                "/gateway/bot" => {
                    lookups.fetch_add(1, Ordering::SeqCst);
                    Reply::json(json!({"url":url,"shards":1,"session_start_limit":{"total":1,"remaining":1,"reset_after":60000,"max_concurrency":1}}))
                },
                other => panic!("unexpected path {other}"),
            }
        }
    }}).await;
    let manager = GatewayShardManager::new(
        Arc::new(server.bot()),
        EventRouter::new(),
        GatewayConfig {
            reconnect_delay: Duration::from_millis(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(
        timeout(Duration::from_secs(2), manager.run())
            .await
            .unwrap()
            .is_err()
    );
    websocket.await.unwrap();
    assert_eq!(lookups.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn endpoint_minute_quota_is_enforced_before_network_and_shared_by_clones() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = MockHttp::start({
        let calls = calls.clone();
        move |request| {
            let calls = calls.clone();
            async move {
                if request.path == "/app/getAppAccessToken" {
                    Reply::json(json!({"access_token":"token","expires_in":7200}))
                } else {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Reply::json(json!({"group_openid":"group"}))
                }
            }
        }
    })
    .await;
    let bot = server.bot();
    for _ in 0..30 {
        bot.api().groups().get("group").await.unwrap();
    }
    assert!(matches!(
        bot.clone().api().groups().get("other").await.unwrap_err(),
        qqbot_sdk_rs::SdkError::RateLimited(_)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 30);
}

#[tokio::test]
async fn quota_wait_does_not_open_a_websocket_before_identify_is_allowed() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let server = MockHttp::start(move |request| {
        let url = url.clone();
        async move {
            if request.path == "/app/getAppAccessToken" {
                Reply::json(json!({"access_token":"token","expires_in":7200}))
            } else {
                Reply::json(json!({"url":url,"shards":1,"session_start_limit":{"total":1,"remaining":0,"reset_after":60000,"max_concurrency":1}}))
            }
        }
    }).await;
    let manager = GatewayShardManager::new(
        Arc::new(server.bot()),
        EventRouter::new(),
        GatewayConfig::default(),
    )
    .await
    .unwrap();
    let run = manager.run();
    tokio::pin!(run);
    tokio::select! {
        _ = listener.accept() => panic!("启动配额不可用时不应建立待鉴权连接"),
        result = &mut run => panic!("管理器应等待配额，实际返回 {result:?}"),
        _ = tokio::time::sleep(Duration::from_millis(50)) => {},
    }
}

#[tokio::test]
async fn upload_retries_a_connection_closed_before_response_headers() {
    let puts = Arc::new(AtomicUsize::new(0));
    let server = MockHttp::start({ let puts = puts.clone(); move |request| {
        let puts = puts.clone(); async move {
            match request.path.as_str() {
                "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                "/v2/users/user/upload_prepare" => Reply::json(json!({"upload_id":"upload","parts":[
                    {"index":0,"block_size":"2","presigned_url":format!("{}/put",request.base_url)}],
                    "upload_config":{"concurrency":1,"retry_timeout":2,"retry_delay":0}})),
                "/put" => if puts.fetch_add(1, Ordering::SeqCst) == 0 { Reply::Disconnect } else { Reply::status(200) },
                "/v2/users/user/upload_part_finish" => Reply::json(json!({})),
                "/v2/users/user/files" => Reply::json(json!({"file_info":"file","file_uuid":"uuid","ttl":60})),
                other => panic!("unexpected path {other}"),
            }
        }
    }}).await;
    let bot = server.bot();
    let media = bot
        .api()
        .messages()
        .upload_media(MediaTarget::C2c("user"), 4, b"ab", false)
        .await
        .unwrap();
    assert_eq!(media.file_info, "file");
    assert_eq!(puts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn default_bot_qps_does_not_prevent_merge_after_four_parts() {
    let merges = Arc::new(AtomicUsize::new(0));
    let server = MockHttp::start({ let merges = merges.clone(); move |request| {
        let merges = merges.clone(); async move {
            match request.path.as_str() {
                "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                "/v2/users/user/upload_prepare" => Reply::json(json!({"upload_id":"upload","parts":
                    (0..4).map(|index| json!({"index":index,"block_size":"1","presigned_url":format!("{}/put/{index}",request.base_url)})).collect::<Vec<_>>(),
                    "upload_config":{"concurrency":4,"retry_timeout":2,"retry_delay":0}})),
                path if path.starts_with("/put/") => Reply::status(200),
                "/v2/users/user/upload_part_finish" => Reply::json(json!({})),
                "/v2/users/user/files" => { merges.fetch_add(1, Ordering::SeqCst); Reply::json(json!({"file_info":"file","file_uuid":"uuid","ttl":60})) },
                other => panic!("unexpected path {other}"),
            }
        }
    }}).await;
    let bot = qqbot_sdk_rs::Bot::with_config(
        "app",
        "secret",
        qqbot_sdk_rs::ClientConfig {
            api_base_url: server.base_url.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    let media = timeout(
        Duration::from_secs(3),
        bot.api()
            .messages()
            .upload_media(MediaTarget::C2c("user"), 4, b"abcd", true),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(media.file_info, "file");
    assert_eq!(merges.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn merge_is_not_retried_after_a_server_error_or_lost_response() {
    for disconnect in [false, true] {
        let merges = Arc::new(AtomicUsize::new(0));
        let server = MockHttp::start({ let merges = merges.clone(); move |request| {
            let merges = merges.clone(); async move {
                match request.path.as_str() {
                    "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                    "/v2/users/user/upload_prepare" => Reply::json(json!({"upload_id":"upload","parts":[
                        {"index":0,"block_size":"2","presigned_url":format!("{}/put",request.base_url)}],
                        "upload_config":{"concurrency":1,"retry_timeout":2,"retry_delay":0}})),
                    "/put" => Reply::status(200),
                    "/v2/users/user/upload_part_finish" => Reply::json(json!({})),
                    "/v2/users/user/files" => {
                        let body: Value = serde_json::from_slice(&request.body).unwrap();
                        assert_eq!(body["srv_send_msg"], true);
                        merges.fetch_add(1, Ordering::SeqCst);
                        if disconnect { Reply::Disconnect } else { Reply::status(503) }
                    },
                    other => panic!("unexpected path {other}"),
                }
            }
        }}).await;
        let bot = server.bot();
        let error = bot
            .api()
            .messages()
            .upload_media(MediaTarget::C2c("user"), 4, b"ab", true)
            .await
            .unwrap_err();
        if disconnect {
            assert!(matches!(error, qqbot_sdk_rs::SdkError::Http(_)));
        } else {
            assert!(matches!(
                error,
                qqbot_sdk_rs::SdkError::Api { status: 503, .. }
            ));
        }
        assert_eq!(merges.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn stalled_gateway_handshake_hello_and_ready_are_bounded() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, SdkError};
    use tokio_tungstenite::{accept_async, tungstenite::Message};
    for stage in ["handshake", "hello", "ready"] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let websocket = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            if stage == "handshake" {
                let _socket = socket;
                std::future::pending::<()>().await;
            } else {
                let mut socket = accept_async(socket).await.unwrap();
                if stage == "ready" {
                    socket
                        .send(Message::Text(
                            json!({"op":10,"d":{"heartbeat_interval":45000}})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                    assert!(socket.next().await.unwrap().is_ok());
                }
                let _socket = socket;
                std::future::pending::<()>().await;
            }
        });
        let http = MockHttp::start(|_| async {
            Reply::json(json!({"access_token":"token","expires_in":7200}))
        })
        .await;
        let bot = Bot::with_config(
            "app",
            "secret",
            ClientConfig {
                gateway_url: Some(url),
                api_base_url: http.base_url.clone(),
                request_timeout: Duration::from_millis(100),
                ..Default::default()
            },
        )
        .unwrap();
        let gateway = GatewayClient::new(
            Arc::new(bot),
            EventRouter::new(),
            GatewayConfig {
                auto_reconnect: false,
                ..Default::default()
            },
        )
        .unwrap();
        let error = timeout(Duration::from_secs(2), gateway.run())
            .await
            .expect(stage)
            .unwrap_err();
        assert!(matches!(error, SdkError::Timeout(_)), "{stage}: {error}");
        websocket.abort();
        assert!(websocket.await.unwrap_err().is_cancelled());
    }
}

#[tokio::test]
async fn cancelling_gateway_run_stops_handlers_before_allowing_restart() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, ReadyEvent};
    use tokio::{net::TcpListener, sync::Notify};
    use tokio_tungstenite::{accept_async, tungstenite::Message};
    struct DropSignal(Arc<Notify>);
    impl Drop for DropSignal {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let websocket = tokio::spawn(async move {
        for connection in 0..2 {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(socket).await.unwrap();
            socket
                .send(Message::Text(
                    json!({"op":10,"d":{"heartbeat_interval":45000}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let auth: Value =
                serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap())
                    .unwrap();
            if connection == 0 {
                assert_eq!(auth["op"], 2);
                socket
                    .send(Message::Text(
                        json!({"op":0,"s":1,"t":"READY","d":{"session_id":"session"}})
                            .to_string()
                            .into(),
                    ))
                    .await
                    .unwrap();
                let _ = socket.next().await;
            } else {
                assert_eq!(
                    auth["op"], 2,
                    "未处理完 READY、没有检查点的会话应重新 Identify"
                );
                use tokio_tungstenite::tungstenite::protocol::{
                    CloseFrame, frame::coding::CloseCode,
                };
                socket
                    .send(Message::Close(Some(CloseFrame {
                        code: CloseCode::from(4001),
                        reason: "stop".into(),
                    })))
                    .await
                    .unwrap();
            }
        }
    });
    let http = MockHttp::start(|_| async {
        Reply::json(json!({"access_token":"token","expires_in":7200}))
    })
    .await;
    let bot = Bot::with_config(
        "app",
        "secret",
        ClientConfig {
            gateway_url: Some(url),
            api_base_url: http.base_url.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    let started = Arc::new(Notify::new());
    let dropped = Arc::new(Notify::new());
    let router = EventRouter::new();
    router
        .on::<ReadyEvent, _, _>({
            let started = started.clone();
            let dropped = dropped.clone();
            move |_| {
                let started = started.clone();
                let dropped = dropped.clone();
                async move {
                    let _guard = DropSignal(dropped);
                    started.notify_one();
                    std::future::pending::<qqbot_sdk_rs::Result<()>>().await
                }
            }
        })
        .await;
    let gateway = GatewayClient::new(Arc::new(bot), router, GatewayConfig::default()).unwrap();
    let mut run = Box::pin(gateway.run());
    tokio::select! {
        _ = started.notified() => {},
        result = &mut run => panic!("事件处理器开始前网关退出: {result:?}"),
        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("处理器未启动"),
    }
    drop(run);
    let cleaning = timeout(Duration::from_millis(100), gateway.run())
        .await
        .expect("清理中的网关不应启动另一个会话");
    assert!(matches!(
        cleaning,
        Err(qqbot_sdk_rs::SdkError::InvalidInput(_))
    ));
    timeout(Duration::from_secs(1), dropped.notified())
        .await
        .expect("取消 run 必须取消正在等待的处理器");
    let restarted = timeout(Duration::from_secs(2), gateway.run())
        .await
        .expect("清理结束后应该可以再次运行");
    assert!(matches!(
        restarted,
        Err(qqbot_sdk_rs::SdkError::WebSocket(_))
    ));
    timeout(Duration::from_secs(1), websocket)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn single_gateway_reconnects_reuse_discovery_without_hitting_two_per_minute_limit() {
    use qqbot_sdk_rs::GatewayClient;
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let websocket = tokio::spawn(async move {
        for connection in 0..4 {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(socket).await.unwrap();
            socket
                .send(Message::Text(
                    json!({"op":10,"d":{"heartbeat_interval":45000}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let payload: Value =
                serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap())
                    .unwrap();
            assert_eq!(payload["op"], if connection == 0 { 2 } else { 6 });
            if connection == 0 {
                socket
                    .send(Message::Text(
                        json!({"op":0,"s":1,"t":"READY","d":{"session_id":"session"}})
                            .to_string()
                            .into(),
                    ))
                    .await
                    .unwrap();
            }
            socket
                .send(Message::Close(Some(CloseFrame {
                    code: CloseCode::from(if connection == 3 { 4001 } else { 4009 }),
                    reason: "reconnect".into(),
                })))
                .await
                .unwrap();
        }
    });
    let discoveries = Arc::new(AtomicUsize::new(0));
    let http = MockHttp::start({
        let discoveries = discoveries.clone();
        move |request| {
            let discoveries = discoveries.clone();
            let url = url.clone();
            async move {
                match request.path.as_str() {
                    "/app/getAppAccessToken" => {
                        Reply::json(json!({"access_token":"token","expires_in":7200}))
                    }
                    "/gateway" => {
                        discoveries.fetch_add(1, Ordering::SeqCst);
                        Reply::json(json!({"url":url}))
                    }
                    other => panic!("unexpected path {other}"),
                }
            }
        }
    })
    .await;
    let gateway = GatewayClient::new(
        Arc::new(http.bot()),
        EventRouter::new(),
        GatewayConfig {
            reconnect_delay: Duration::from_millis(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        timeout(Duration::from_secs(2), gateway.run())
            .await
            .expect("快速断线重连不应受发现接口配额阻塞")
            .is_err()
    );
    websocket.await.unwrap();
    assert_eq!(discoveries.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_gateway_rejects_a_second_run_while_the_first_is_connecting() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, SdkError};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let gateway = GatewayClient::new(
        Arc::new(
            Bot::with_config(
                "app",
                "secret",
                ClientConfig {
                    gateway_url: Some(url),
                    ..Default::default()
                },
            )
            .unwrap(),
        ),
        EventRouter::new(),
        GatewayConfig::default(),
    )
    .unwrap();
    let mut first = Box::pin(gateway.run());
    let _connection = tokio::select! {
        accepted = listener.accept() => accepted.unwrap().0,
        result = &mut first => panic!("第一次运行提前结束: {result:?}"),
        _ = tokio::time::sleep(Duration::from_secs(1)) => panic!("第一次运行没有连接"),
    };
    let second = timeout(Duration::from_millis(100), gateway.run())
        .await
        .expect("重复运行应立即拒绝");
    assert!(matches!(second, Err(SdkError::InvalidInput(_))));
    drop(first);
}

#[tokio::test]
async fn a_shard_manager_rejects_a_second_run_without_starting_any_more_connections() {
    use qqbot_sdk_rs::SdkError;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let http = MockHttp::start(move |request| {
        let url = url.clone(); async move {
            if request.path == "/app/getAppAccessToken" {
                Reply::json(json!({"access_token":"token","expires_in":7200}))
            } else {
                Reply::json(json!({"url":url,"shards":1,"session_start_limit":{"total":2,"remaining":2,"reset_after":60000,"max_concurrency":1}}))
            }
        }
    }).await;
    let manager = GatewayShardManager::new(
        Arc::new(http.bot()),
        EventRouter::new(),
        GatewayConfig::default(),
    )
    .await
    .unwrap();
    let mut first = Box::pin(manager.run());
    let _connection = tokio::select! {
        accepted = listener.accept() => accepted.unwrap().0,
        result = &mut first => panic!("第一次运行提前结束: {result:?}"),
        _ = tokio::time::sleep(Duration::from_secs(1)) => panic!("第一次运行没有连接"),
    };
    let second = timeout(Duration::from_millis(100), manager.run())
        .await
        .expect("分片管理器重复运行应立即拒绝");
    assert!(matches!(second, Err(SdkError::InvalidInput(_))));
    assert!(
        timeout(Duration::from_millis(20), listener.accept())
            .await
            .is_err()
    );
    drop(first);
}

#[tokio::test]
async fn cancelling_a_parallel_upload_never_confirms_or_merges_after_put_completes() {
    use tokio::sync::{Notify, Semaphore};
    let puts_started = Arc::new(Semaphore::new(0));
    let puts_completed = Arc::new(Semaphore::new(0));
    let release_puts = Arc::new(Notify::new());
    let publish = Arc::new(Notify::new());
    let server = MockHttp::start({
        let puts_started = puts_started.clone(); let puts_completed = puts_completed.clone();
        let release_puts = release_puts.clone(); let publish = publish.clone();
        move |request| {
            let puts_started = puts_started.clone(); let puts_completed = puts_completed.clone();
            let release_puts = release_puts.clone(); let publish = publish.clone();
            async move {
                match request.path.as_str() {
                    "/app/getAppAccessToken" => Reply::json(json!({"access_token":"token","expires_in":7200})),
                    "/v2/groups/group/upload_prepare" => Reply::json(json!({"upload_id":"upload","parts":[
                        {"index":0,"block_size":"1","presigned_url":format!("{}/put/0",request.base_url)},
                        {"index":1,"block_size":"1","presigned_url":format!("{}/put/1",request.base_url)}],
                        "upload_config":{"concurrency":2,"retry_timeout":2,"retry_delay":0}})),
                    path if path.starts_with("/put/") => {
                        let release = release_puts.notified();
                        tokio::pin!(release);
                        release.as_mut().enable();
                        puts_started.add_permits(1);
                        release.await;
                        puts_completed.add_permits(1);
                        Reply::status(200)
                    },
                    "/v2/groups/group/upload_part_finish" | "/v2/groups/group/files" => {
                        publish.notify_one();
                        Reply::json(json!({"file_info":"file","file_uuid":"uuid","ttl":60}))
                    },
                    other => panic!("unexpected path {other}"),
                }
            }
        }
    }).await;
    let bot = server.bot();
    let api = bot.api().messages();
    let mut upload = Box::pin(api.upload_media(MediaTarget::Group("group"), 4, b"ab", false));
    tokio::select! {
        permits = puts_started.acquire_many(2) => permits.unwrap().forget(),
        result = &mut upload => panic!("两个 PUT 都应等待，实际提前结束 {result:?}"),
        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("PUT 未并发启动"),
    }
    drop(upload);
    release_puts.notify_waiters();
    timeout(Duration::from_secs(1), puts_completed.acquire_many(2))
        .await
        .unwrap()
        .unwrap()
        .forget();
    assert!(
        timeout(Duration::from_millis(30), publish.notified())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn cancelling_cleanup_after_session_invalidation_restarts_with_identify() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient};
    use tokio::sync::Notify;
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    struct DropSignal(Arc<Notify>);
    impl Drop for DropSignal {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }
    for close_frame in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let started = Arc::new(Notify::new());
        let disconnected = Arc::new(Notify::new());
        let dropped = Arc::new(Notify::new());
        let websocket = tokio::spawn({
            let started = started.clone();
            let disconnected = disconnected.clone();
            async move {
                for connection in 0..2 {
                    let (socket, _) = listener.accept().await.unwrap();
                    let mut socket = accept_async(socket).await.unwrap();
                    socket
                        .send(Message::Text(
                            json!({"op":10,"d":{"heartbeat_interval":45000}})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                    let auth: Value = serde_json::from_str(
                        &socket.next().await.unwrap().unwrap().into_text().unwrap(),
                    )
                    .unwrap();
                    assert_eq!(auth["op"], 2, "失效的旧会话不能用于 Resume");
                    if connection == 0 {
                        socket
                            .send(Message::Text(
                                json!({"op":0,"s":1,"t":"READY","d":{"session_id":"old"}})
                                    .to_string()
                                    .into(),
                            ))
                            .await
                            .unwrap();
                        socket
                            .send(Message::Text(
                                json!({"op":0,"s":2,"t":"BLOCK","d":{}}).to_string().into(),
                            ))
                            .await
                            .unwrap();
                        started.notified().await;
                        if close_frame {
                            socket
                                .send(Message::Close(Some(CloseFrame {
                                    code: CloseCode::from(4006),
                                    reason: "invalid session".into(),
                                })))
                                .await
                                .unwrap();
                        } else {
                            socket
                                .send(Message::Text(json!({"op":9}).to_string().into()))
                                .await
                                .unwrap();
                        }
                        let _ = socket.next().await;
                        disconnected.notify_one();
                    } else {
                        assert!(auth["d"].get("seq").is_none());
                        socket
                            .send(Message::Text(json!({"op":1}).to_string().into()))
                            .await
                            .unwrap();
                        let heartbeat: Value = serde_json::from_str(
                            &socket.next().await.unwrap().unwrap().into_text().unwrap(),
                        )
                        .unwrap();
                        assert_eq!(heartbeat["op"], 1);
                        assert!(heartbeat["d"].is_null(), "新会话不能携带旧会话的已处理序号");
                        socket
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::from(4001),
                                reason: "stop".into(),
                            })))
                            .await
                            .unwrap();
                    }
                }
            }
        });
        let http = MockHttp::start(|_| async {
            Reply::json(json!({"access_token":"token","expires_in":7200}))
        })
        .await;
        let bot = Bot::with_config(
            "app",
            "secret",
            ClientConfig {
                gateway_url: Some(url),
                api_base_url: http.base_url.clone(),
                ..Default::default()
            },
        )
        .unwrap();
        let router = EventRouter::new();
        router
            .on_raw({
                let started = started.clone();
                let dropped = dropped.clone();
                move |event, _| {
                    let started = started.clone();
                    let dropped = dropped.clone();
                    async move {
                        if event.name == "BLOCK" {
                            let _guard = DropSignal(dropped);
                            started.notify_one();
                            std::future::pending::<qqbot_sdk_rs::Result<()>>().await
                        } else {
                            Ok(())
                        }
                    }
                }
            })
            .await;
        let gateway = GatewayClient::new(
            Arc::new(bot),
            router,
            GatewayConfig {
                auto_reconnect: false,
                ..Default::default()
            },
        )
        .unwrap();
        let mut first = Box::pin(gateway.run());
        tokio::select! {
            _ = disconnected.notified() => {},
            result = &mut first => panic!("旧处理器尚未排空: {result:?}"),
            _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("失效会话通知未处理"),
        }
        drop(first);
        timeout(Duration::from_secs(1), dropped.notified())
            .await
            .unwrap();
        assert!(
            timeout(Duration::from_secs(2), gateway.run())
                .await
                .unwrap()
                .is_err()
        );
        websocket.await.unwrap();
    }
}

#[tokio::test]
async fn a_dispatch_failure_during_cleanup_cannot_override_a_fatal_gateway_close() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, ReadyEvent, SdkError};
    use tokio::sync::Notify;
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let started = Arc::new(Notify::new());
    let fail_handler = Arc::new(Notify::new());
    let websocket = tokio::spawn({
        let started = started.clone();
        let fail_handler = fail_handler.clone();
        async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(socket).await.unwrap();
            socket
                .send(Message::Text(
                    json!({"op":10,"d":{"heartbeat_interval":45000}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            assert!(socket.next().await.unwrap().is_ok());
            socket
                .send(Message::Text(
                    json!({"op":0,"s":1,"t":"READY","d":{"session_id":"session"}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            started.notified().await;
            socket
                .send(Message::Close(Some(CloseFrame {
                    code: CloseCode::from(4915),
                    reason: "fatal".into(),
                })))
                .await
                .unwrap();
            let _ = socket.next().await;
            fail_handler.notify_one();
        }
    });
    let http = MockHttp::start(|_| async {
        Reply::json(json!({"access_token":"token","expires_in":7200}))
    })
    .await;
    let bot = Bot::with_config(
        "app",
        "secret",
        ClientConfig {
            gateway_url: Some(url),
            api_base_url: http.base_url.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    let router = EventRouter::new();
    router
        .on::<ReadyEvent, _, _>(move |_| {
            let started = started.clone();
            let fail_handler = fail_handler.clone();
            async move {
                started.notify_one();
                fail_handler.notified().await;
                Err(SdkError::InvalidInput("handler failed".into()))
            }
        })
        .await;
    let gateway = GatewayClient::new(
        Arc::new(bot),
        router,
        GatewayConfig {
            reconnect_delay: Duration::from_millis(1),
            ..Default::default()
        },
    )
    .unwrap();
    let error = timeout(Duration::from_secs(1), gateway.run())
        .await
        .expect("不可重试的关闭不能变成自动重连")
        .unwrap_err();
    assert!(matches!(error, SdkError::WebSocket(ref message) if message.contains("4915")));
    websocket.await.unwrap();
}

#[tokio::test]
async fn completed_run_futures_do_not_retain_gateway_or_manager_guards() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, SdkError};
    let gateway = GatewayClient::new(
        Arc::new(
            Bot::with_config(
                "app",
                "secret",
                ClientConfig {
                    gateway_url: Some("invalid://gateway".into()),
                    ..Default::default()
                },
            )
            .unwrap(),
        ),
        EventRouter::new(),
        GatewayConfig {
            auto_reconnect: false,
            ..Default::default()
        },
    )
    .unwrap();
    let mut completed_gateway = Box::pin(gateway.run());
    assert!(matches!(
        completed_gateway.as_mut().await,
        Err(SdkError::WebSocket(_))
    ));
    // 已完成的 future 仍然存在；守卫应在返回结果时释放，而不是等待 future 被 drop。
    assert!(matches!(gateway.run().await, Err(SdkError::WebSocket(_))));
    let http = MockHttp::start(|request| async move {
        if request.path == "/app/getAppAccessToken" {
            Reply::json(json!({"access_token":"token","expires_in":7200}))
        } else {
            Reply::json(json!({"url":"invalid://gateway","shards":1,"session_start_limit":{"total":1,"remaining":1,"reset_after":60000,"max_concurrency":1}}))
        }
    }).await;
    let manager = GatewayShardManager::new(
        Arc::new(http.bot()),
        EventRouter::new(),
        GatewayConfig {
            auto_reconnect: false,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let mut completed_manager = Box::pin(manager.run());
    assert!(matches!(
        completed_manager.as_mut().await,
        Err(SdkError::WebSocket(_))
    ));
    let restarted = timeout(Duration::from_secs(1), manager.run())
        .await
        .expect("已结束的运行不能继续占用守卫或未使用的启动配额");
    assert!(matches!(restarted, Err(SdkError::WebSocket(_))));
}

#[tokio::test]
async fn reauthenticated_session_survives_a_later_cancellation_and_resumes_the_new_checkpoint() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, SdkError};
    use tokio::sync::Notify;
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    struct DropSignal(Arc<Notify>);
    impl Drop for DropSignal {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let started = Arc::new(Notify::new());
    let dropped = Arc::new(Notify::new());
    let websocket = tokio::spawn(async move {
        for connection in 0..3 {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(socket).await.unwrap();
            socket
                .send(Message::Text(
                    json!({"op":10,"d":{"heartbeat_interval":45000}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let auth: Value =
                serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap())
                    .unwrap();
            match connection {
                0 => {
                    assert_eq!(auth["op"], 2);
                    socket
                        .send(Message::Text(
                            json!({"op":0,"s":1,"t":"READY","d":{"session_id":"old"}})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                    socket
                        .send(Message::Text(json!({"op":9}).to_string().into()))
                        .await
                        .unwrap();
                    let _ = socket.next().await;
                }
                1 => {
                    assert_eq!(auth["op"], 2);
                    socket
                        .send(Message::Text(
                            json!({"op":0,"s":7,"t":"READY","d":{"session_id":"new"}})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                    socket
                        .send(Message::Text(
                            json!({"op":0,"s":8,"t":"BLOCK","d":{}}).to_string().into(),
                        ))
                        .await
                        .unwrap();
                    let _ = socket.next().await;
                }
                2 => {
                    assert_eq!(auth["op"], 6, "已重新建立的有效会话应该 Resume");
                    assert_eq!(auth["d"]["session_id"], "new");
                    assert_eq!(auth["d"]["seq"], 7, "只能恢复新会话已经处理的序号");
                    socket
                        .send(Message::Close(Some(CloseFrame {
                            code: CloseCode::from(4001),
                            reason: "stop".into(),
                        })))
                        .await
                        .unwrap();
                }
                _ => unreachable!(),
            }
        }
    });
    let http = MockHttp::start(|_| async {
        Reply::json(json!({"access_token":"token","expires_in":7200}))
    })
    .await;
    let bot = Bot::with_config(
        "app",
        "secret",
        ClientConfig {
            gateway_url: Some(url),
            api_base_url: http.base_url.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    let router = EventRouter::new();
    router
        .on_raw({
            let started = started.clone();
            let dropped = dropped.clone();
            move |event, _| {
                let started = started.clone();
                let dropped = dropped.clone();
                async move {
                    if event.name == "BLOCK" {
                        let _guard = DropSignal(dropped);
                        started.notify_one();
                        std::future::pending::<qqbot_sdk_rs::Result<()>>().await
                    } else {
                        Ok(())
                    }
                }
            }
        })
        .await;
    let gateway = GatewayClient::new(
        Arc::new(bot),
        router,
        GatewayConfig {
            auto_reconnect: false,
            ..Default::default()
        },
    )
    .unwrap();
    let first_error = timeout(Duration::from_secs(2), gateway.run())
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(first_error, SdkError::Auth(_)));
    let mut second = Box::pin(gateway.run());
    tokio::select! {
        _ = started.notified() => {},
        result = &mut second => panic!("重新鉴权后的处理器尚未开始: {result:?}"),
        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("新会话未建立"),
    }
    drop(second);
    timeout(Duration::from_secs(1), dropped.notified())
        .await
        .unwrap();
    let third_error = timeout(Duration::from_secs(2), gateway.run())
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(third_error, SdkError::WebSocket(_)));
    websocket.await.unwrap();
}

#[tokio::test]
async fn invalid_session_and_a_dispatch_failure_still_allow_automatic_identify() {
    use qqbot_sdk_rs::{Bot, ClientConfig, GatewayClient, ReadyEvent, SdkError};
    use tokio::sync::Notify;
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            Message,
            protocol::{CloseFrame, frame::coding::CloseCode},
        },
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let started = Arc::new(Notify::new());
    let fail_handler = Arc::new(Notify::new());
    let websocket = tokio::spawn({
        let started = started.clone();
        let fail_handler = fail_handler.clone();
        async move {
            for connection in 0..2 {
                let (socket, _) = listener.accept().await.unwrap();
                let mut socket = accept_async(socket).await.unwrap();
                socket
                    .send(Message::Text(
                        json!({"op":10,"d":{"heartbeat_interval":45000}})
                            .to_string()
                            .into(),
                    ))
                    .await
                    .unwrap();
                let auth: Value = serde_json::from_str(
                    &socket.next().await.unwrap().unwrap().into_text().unwrap(),
                )
                .unwrap();
                assert_eq!(auth["op"], 2, "处理器错误不能丢失重新鉴权要求");
                if connection == 0 {
                    socket
                        .send(Message::Text(
                            json!({"op":0,"s":1,"t":"READY","d":{"session_id":"old"}})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                    started.notified().await;
                    socket
                        .send(Message::Text(json!({"op":9}).to_string().into()))
                        .await
                        .unwrap();
                    let _ = socket.next().await;
                    fail_handler.notify_one();
                } else {
                    socket
                        .send(Message::Close(Some(CloseFrame {
                            code: CloseCode::from(4915),
                            reason: "stop".into(),
                        })))
                        .await
                        .unwrap();
                }
            }
        }
    });
    let http = MockHttp::start(|_| async {
        Reply::json(json!({"access_token":"token","expires_in":7200}))
    })
    .await;
    let bot = Bot::with_config(
        "app",
        "secret",
        ClientConfig {
            gateway_url: Some(url),
            api_base_url: http.base_url.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    let router = EventRouter::new();
    router
        .on::<ReadyEvent, _, _>(move |_| {
            let started = started.clone();
            let fail_handler = fail_handler.clone();
            async move {
                started.notify_one();
                fail_handler.notified().await;
                Err(SdkError::InvalidInput("handler failed".into()))
            }
        })
        .await;
    let gateway = GatewayClient::new(
        Arc::new(bot),
        router,
        GatewayConfig {
            reconnect_delay: Duration::from_millis(1),
            ..Default::default()
        },
    )
    .unwrap();
    let error = timeout(Duration::from_secs(2), gateway.run())
        .await
        .expect("会话失效后仍应重新鉴权，再遵守不可重试关闭")
        .unwrap_err();
    assert!(matches!(error, SdkError::WebSocket(ref message) if message.contains("4915")));
    websocket.await.unwrap();
}

#[tokio::test]
async fn cancelled_ready_without_checkpoint_cannot_bypass_identify_quota() {
    use qqbot_sdk_rs::ReadyEvent;
    use tokio::{net::TcpListener, sync::Notify};
    use tokio_tungstenite::{accept_async, tungstenite::Message};
    struct DropSignal(Arc<Notify>);
    impl Drop for DropSignal {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let websocket = tokio::spawn({
        let listener = listener.clone();
        async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(socket).await.unwrap();
            socket
                .send(Message::Text(
                    json!({"op":10,"d":{"heartbeat_interval":45000}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let auth: Value =
                serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap())
                    .unwrap();
            assert_eq!(auth["op"], 2);
            socket
                .send(Message::Text(
                    json!({"op":0,"s":1,"t":"READY","d":{"session_id":"pending"}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let _ = socket.next().await;
        }
    });
    let http = MockHttp::start(move |request| { let url = url.clone(); async move {
        if request.path == "/app/getAppAccessToken" {
            Reply::json(json!({"access_token":"token","expires_in":7200}))
        } else {
            Reply::json(json!({"url":url,"shards":1,"session_start_limit":{"total":1,"remaining":1,"reset_after":60000,"max_concurrency":1}}))
        }
    }}).await;
    let started = Arc::new(Notify::new());
    let dropped = Arc::new(Notify::new());
    let router = EventRouter::new();
    router
        .on::<ReadyEvent, _, _>({
            let started = started.clone();
            let dropped = dropped.clone();
            move |_| {
                let started = started.clone();
                let dropped = dropped.clone();
                async move {
                    let _guard = DropSignal(dropped);
                    started.notify_one();
                    std::future::pending::<qqbot_sdk_rs::Result<()>>().await
                }
            }
        })
        .await;
    let manager = GatewayShardManager::new(Arc::new(http.bot()), router, GatewayConfig::default())
        .await
        .unwrap();
    let mut first = Box::pin(manager.run());
    tokio::select! {
        _ = started.notified() => {},
        result = &mut first => panic!("READY 处理器未开始前退出: {result:?}"),
        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("READY 未到达"),
    }
    drop(first);
    timeout(Duration::from_secs(1), dropped.notified())
        .await
        .unwrap();
    websocket.await.unwrap();
    let mut restarted = Box::pin(manager.run());
    tokio::select! {
        _ = listener.accept() => panic!("没有检查点应走 Identify，配额耗尽时不能再次连接"),
        result = &mut restarted => panic!("应该等待 Identify 配额: {result:?}"),
        _ = tokio::time::sleep(Duration::from_millis(50)) => {},
    }
}
