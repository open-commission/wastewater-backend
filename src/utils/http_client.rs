use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::codec::{FramedRead, LinesCodec};
use url::Url;

// --- 错误定义 ---

#[derive(thiserror::Error, Debug)]
pub enum NetError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type NetResult<T> = Result<T, NetError>;

// --- 模型定义 ---

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
    pub id: Option<String>,
}

// --- 核心客户端 ---

/// 统一网络客户端，内部维护连接池
#[derive(Clone)]
pub struct NetClient {
    inner: Client,
}

impl NetClient {
    pub fn new() -> NetResult<Self> {
        let client = ClientBuilder::new()
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()?;
        Ok(Self { inner: client })
    }

    // --- HTTP 部分 ---

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NetResult<T> {
        let resp = self.inner.get(url).send().await?.error_for_status()?;
        Ok(resp.json::<T>().await?)
    }

    pub async fn post<S: Serialize, T: for<'de> Deserialize<'de>>(&self, url: &str, body: &S) -> NetResult<T> {
        let resp = self.inner.post(url).json(body).send().await?.error_for_status()?;
        Ok(resp.json::<T>().await?)
    }

    pub async fn put<S: Serialize, T: for<'de> Deserialize<'de>>(&self, url: &str, body: &S) -> NetResult<T> {
        let resp = self.inner.put(url).json(body).send().await?.error_for_status()?;
        Ok(resp.json::<T>().await?)
    }

    pub async fn delete<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NetResult<T> {
        let resp = self.inner.delete(url).send().await?.error_for_status()?;
        Ok(resp.json::<T>().await?)
    }

    // --- SSE 部分 ---

    pub async fn sse_stream(&self, url: &str) -> NetResult<mpsc::Receiver<SseEvent>> {
        let response = self.inner.get(url)
            .header("Accept", "text/event-stream")
            .send().await?.error_for_status()?;

        let (tx, rx) = mpsc::channel(100);
        let stream = response.bytes_stream();

        // 使用 tokio_util 处理字节流转行
        let mut lines = FramedRead::new(
            tokio_util::io::StreamReader::new(stream.map(|res| {
                res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })),
            LinesCodec::new(),
        );

        tokio::spawn(async move {
            let mut current_event = "message".to_string();
            let mut current_data = String::new();
            let mut current_id = None;

            while let Some(Ok(line)) = lines.next().await {
                if line.is_empty() {
                    if !current_data.is_empty() {
                        let _ = tx.send(SseEvent {
                            event: current_event.clone(),
                            data: current_data.trim().to_string(),
                            id: current_id.clone(),
                        }).await;
                        current_data.clear();
                        current_event = "message".to_string();
                    }
                    continue;
                }

                if let Some((field, value)) = line.split_once(':') {
                    let value = value.trim_start();
                    match field {
                        "event" => current_event = value.to_string(),
                        "data" => current_data.push_str(value),
                        "id" => current_id = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        });

        Ok(rx)
    }

    // --- WebSocket 部分 ---

    pub async fn ws_connect(&self, url: &str) -> NetResult<WsSession> {
        // 1. 先验证 URL 是否合法
        let parsed_url = Url::parse(url)?;

        // 2. 将 parsed_url 转换为 String 传入，或者直接传入原始 str
        // connect_async 明确支持 String 和 &str 实现 IntoClientRequest
        let (ws_stream, _) = connect_async(parsed_url.to_string()).await?;

        let (write, read) = ws_stream.split();
        Ok(WsSession { write, read })
    }
}

// --- WebSocket 会话句柄 ---

pub struct WsSession {
    pub write: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    pub read: futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
}

impl WsSession {
    pub async fn send_text(&mut self, text: String) -> NetResult<()> {
        // 使用 .into() 自动转换为 Utf8Bytes
        self.write.send(Message::Text(text.into())).await.map_err(NetError::Ws)
    }

    // 如果你想支持更通用的类型（如 &str 或 String）
    pub async fn send<S: Into<String>>(&mut self, msg: S) -> NetResult<()> {
        self.write.send(Message::Text(msg.into().into())).await.map_err(NetError::Ws)
    }

    pub async fn next_message(&mut self) -> Option<NetResult<Message>> {
        self.read.next().await.map(|res| res.map_err(NetError::Ws))
    }
}