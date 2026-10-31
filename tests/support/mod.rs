use qqbot_sdk_rs::{Bot, ClientConfig};
use serde_json::Value;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinSet,
};

pub struct Request {
    pub path: String,
    pub body: Vec<u8>,
    pub head: String,
    pub base_url: String,
}

pub enum Reply {
    Http { status: u16, body: String },
    Disconnect,
}
impl Reply {
    pub fn json(body: Value) -> Self {
        Self::Http {
            status: 200,
            body: body.to_string(),
        }
    }
    pub fn status(status: u16) -> Self {
        Self::Http {
            status,
            body: "{}".into(),
        }
    }
}

pub struct MockHttp {
    pub base_url: String,
    task: tokio::task::JoinHandle<()>,
}
impl MockHttp {
    pub async fn start<F, Fut>(handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Reply> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let base = base_url.clone();
        let handler = Arc::new(handler);
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let (mut socket, _) = accepted.unwrap();
                        let handler = handler.clone();
                        let base_url = base.clone();
                        connections.spawn(async move {
                            let mut bytes = Vec::new();
                            let end = loop {
                                let mut buffer = [0u8; 4096];
                                let count = socket.read(&mut buffer).await.unwrap();
                                if count == 0 { return; }
                                bytes.extend_from_slice(&buffer[..count]);
                                if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") { break end; }
                            };
                            let head = String::from_utf8(bytes[..end].to_vec()).unwrap();
                            let length = head.lines().find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().unwrap())
                            }).unwrap_or(0);
                            while bytes.len() < end + 4 + length {
                                let mut buffer = [0u8; 4096];
                                let count = socket.read(&mut buffer).await.unwrap();
                                assert_ne!(count, 0);
                                bytes.extend_from_slice(&buffer[..count]);
                            }
                            let path = head.split_whitespace().nth(1).unwrap().to_owned();
                            let reply = handler(Request { path, head, body: bytes[end+4..end+4+length].to_vec(), base_url }).await;
                            let Reply::Http { status, body } = reply else { return; };
                            let response = format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}", body.len());
                            // 网关组取消或上传失败会主动关闭其他在途请求。
                            let _ = socket.write_all(response.as_bytes()).await;
                        });
                    }
                    result = connections.join_next(), if !connections.is_empty() => { result.unwrap().unwrap(); }
                }
            }
        });
        Self { base_url, task }
    }

    pub fn bot(&self) -> Bot {
        Bot::with_config(
            "app",
            "secret",
            ClientConfig {
                api_base_url: self.base_url.clone(),
                bot_qps: 1000,
                request_timeout: Duration::from_secs(2),
                ..Default::default()
            },
        )
        .unwrap()
    }
}
impl Drop for MockHttp {
    fn drop(&mut self) {
        self.task.abort();
    }
}
