//! RabbitMQ 消息消费者示例
//!
//! 展示如何订阅消息队列并处理接收到的消息

use crate::message_queue::rabbitmq::{Message, RabbitMQManager};
use anyhow::Result;
use futures_util::StreamExt;
use tracing::{error, info};

/// 启动消息消费者任务
///
/// # 参数
/// * `rabbitmq_manager` - RabbitMQ 管理器实例
/// * `queue_name` - 要订阅的队列名称
///
/// # 返回
/// 返回一个任务句柄，可用于等待任务完成
pub async fn start_consumer_task(
    rabbitmq_manager: RabbitMQManager,
    queue_name: &str,
) -> Result<tokio::task::JoinHandle<()>> {
    // 克隆管理器以在异步任务中使用
    let manager = rabbitmq_manager.clone();
    let queue = queue_name.to_string();

    // 创建异步任务处理消息
    let handle = tokio::spawn(async move {
        // 订阅队列
        match manager.subscribe(&queue).await {
            Ok(mut consumer) => {
                info!("成功订阅队列: {}", queue);

                // 持续接收消息
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            // 解析消息内容
                            match serde_json::from_slice::<Message>(&delivery.data) {
                                Ok(message) => {
                                    // 处理消息
                                    handle_message(&message).await;

                                    // 确认消息已被处理
                                    if let Err(e) = delivery.ack(Default::default()).await {
                                        error!("确认消息失败: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("解析消息失败: {}", e);
                                    // 拒绝消息并重新入队
                                    if let Err(e) = delivery
                                        .nack(lapin::options::BasicNackOptions {
                                            requeue: true,
                                            ..Default::default()
                                        })
                                        .await
                                    {
                                        error!("拒绝消息失败: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("接收消息时发生错误: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("订阅队列失败: {}", e);
            }
        }
    });

    Ok(handle)
}

/// 处理接收到的消息
///
/// 这是一个示例处理函数，您可以根据实际需求修改此函数
///
/// # 参数
/// * `message` - 接收到的消息
async fn handle_message(message: &Message) {
    info!("接收到消息:");
    info!("  主题: {}", message.topic);
    info!("  内容: {}", message.payload);
    info!("  时间戳: {}", message.timestamp);

    // 根据消息主题执行不同操作
    match message.topic.as_str() {
        "device.status" => {
            handle_device_status_update(&message.payload).await;
        }
        "sensor.data" => {
            handle_sensor_data(&message.payload).await;
        }
        "alarm.trigger" => {
            handle_alarm_trigger(&message.payload).await;
        }
        _ => {
            info!("未知消息主题: {}", message.topic);
        }
    }
}

/// 处理设备状态更新消息
async fn handle_device_status_update(payload: &str) {
    info!("处理设备状态更新: {}", payload);
    // 在这里添加具体的设备状态更新逻辑
    // 例如：更新数据库中的设备状态
}

/// 处理传感器数据消息
async fn handle_sensor_data(payload: &str) {
    info!("处理传感器数据: {}", payload);
    // 在这里添加具体的传感器数据处理逻辑
    // 例如：将数据存储到数据库或触发某些操作
}

/// 处理报警触发消息
async fn handle_alarm_trigger(payload: &str) {
    info!("处理报警触发: {}", payload);
    // 在这里添加具体的报警处理逻辑
    // 例如：发送通知、记录日志等
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_message_handling() {
        let message = Message {
            topic: "test.message".to_string(),
            payload: "Hello, RabbitMQ!".to_string(),
            timestamp: chrono::Utc::now(),
        };

        handle_message(&message).await;
    }
}
