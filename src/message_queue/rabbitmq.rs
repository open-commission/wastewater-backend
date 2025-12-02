use anyhow::Result;
use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    publisher_confirm::Confirmation,
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// 消息内容结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// RabbitMQ 管理器
#[derive(Clone)]
pub struct RabbitMQManager {
    connection: Arc<Mutex<Option<Connection>>>,
    uri: String,
}

impl RabbitMQManager {
    pub fn new(uri: &str) -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            uri: uri.to_string(),
        }
    }

    /// 建立连接
    pub async fn connect(&self) -> Result<()> {
        let mut guard = self.connection.lock().await;
        let conn = Connection::connect(&self.uri, ConnectionProperties::default()).await?;
        info!("Connected to RabbitMQ: {}", &self.uri);
        *guard = Some(conn);
        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&self) -> Result<()> {
        let mut guard = self.connection.lock().await;
        if let Some(conn) = guard.take() {
            conn.close(0, "").await?;
            info!("Disconnected from RabbitMQ");
        }
        Ok(())
    }

    /// 获取一个 channel（内部使用）
    async fn get_channel(&self) -> Result<Channel> {
        let guard = self.connection.lock().await;
        if let Some(conn) = guard.as_ref() {
            let ch = conn.create_channel().await?;
            Ok(ch)
        } else {
            Err(anyhow::anyhow!("RabbitMQ connection not established"))
        }
    }

    /// 发布消息
    pub async fn publish_message(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &Message,
    ) -> Result<Confirmation> {
        let channel = self.get_channel().await?;

        // 先声明 exchange（如果需要）
        channel
            .exchange_declare(
                exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let payload = serde_json::to_vec(message)?;
        let confirm = channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?
            .await?; // 这里需 await 两次：publish + confirmation

        info!(
            "Published message to exchange '{}', routing_key '{}'",
            exchange, routing_key
        );
        Ok(confirm)
    }

    /// 绑定队列到 exchange
    pub async fn bind_queue(
        &self,
        queue_name: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<()> {
        let channel = self.get_channel().await?;

        channel
            .exchange_declare(
                exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_bind(
                queue_name,
                exchange,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!(
            "Queue '{}' bound to exchange '{}', routing_key '{}'",
            queue_name, exchange, routing_key
        );
        Ok(())
    }

    /// 订阅（消费）队列
    ///
    /// 返回一个 Consumer。调用者应该 spawn tokio 任务负责 .next() + ack/nack
    pub async fn subscribe(&self, queue_name: &str) -> Result<Consumer> {
        let channel = self.get_channel().await?;

        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let consumer = channel
            .basic_consume(
                queue_name,
                "", // 空 tag，让 server 自动生成
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Subscribed to queue '{}'", queue_name);
        Ok(consumer)
    }
}
