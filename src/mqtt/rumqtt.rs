use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::{collections::HashSet, error::Error, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Mutex},
    task, time,
};
use tracing::{error, info};

/// 消息队列条目
#[derive(Clone, Debug)]
struct PendingMessage {
    topic: String,
    payload: Vec<u8>,
    qos: QoS,
    retries: u8,
    id: u64, // 消息唯一 ID，用于幂等或日志
}

/// 异步 MQTT 工具类
#[derive(Clone)]
pub struct MqttManager {
    client: AsyncClient,
    eventloop: Arc<Mutex<EventLoop>>,
    tx: mpsc::Sender<PendingMessage>,
    rx: Arc<Mutex<mpsc::Receiver<PendingMessage>>>,
    subscribed_topics: Arc<Mutex<HashSet<String>>>, // 自动重连用
    msg_counter: Arc<Mutex<u64>>,                   // 消息 ID
}

impl MqttManager {
    /// 创建 MQTT 客户端
    pub async fn new(
        client_id: &str,
        broker: &str,
        port: u16,
        keep_alive_secs: u64,
    ) -> Result<Self, Box<dyn Error>> {
        let mut mqttoptions = MqttOptions::new(client_id, broker, port);
        mqttoptions.set_keep_alive(Duration::from_secs(keep_alive_secs));
        mqttoptions.set_clean_session(false);

        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        let (tx, rx) = mpsc::channel(100);

        Ok(MqttManager {
            client,
            eventloop: Arc::new(Mutex::new(eventloop)),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            subscribed_topics: Arc::new(Mutex::new(HashSet::new())),
            msg_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// 订阅主题
    pub async fn subscribe(&self, topic: &str, qos: QoS) -> Result<(), Box<dyn Error>> {
        self.client.subscribe(topic, qos).await?;
        let mut subs = self.subscribed_topics.lock().await;
        subs.insert(topic.to_string());
        info!("Subscribed to topic: {}", topic);
        Ok(())
    }

    /// 将消息加入发送队列
    pub async fn enqueue_publish(&self, topic: &str, payload: Vec<u8>, qos: QoS) {
        let mut counter = self.msg_counter.lock().await;
        *counter += 1;
        let msg = PendingMessage {
            topic: topic.to_string(),
            payload,
            qos,
            retries: 0,
            id: *counter,
        };
        let _ = self.tx.send(msg).await;
    }

    /// 循环处理消息队列，自动重发
    async fn process_queue(&self) {
        let rx = self.rx.clone();
        let client = self.client.clone();

        loop {
            let mut rx_lock = rx.lock().await;
            if let Some(mut msg) = rx_lock.recv().await {
                drop(rx_lock);

                let result = client
                    .publish(&msg.topic, msg.qos, false, msg.payload.clone())
                    .await;

                if let Err(e) = result {
                    error!(
                        "Publish error: {:?}, msg_id: {}, retries: {}",
                        e, msg.id, msg.retries
                    );
                    if msg.retries < 5 {
                        msg.retries += 1;
                        time::sleep(Duration::from_secs(1)).await;
                        let _ = self.tx.send(msg).await;
                    } else {
                        error!("Message dropped after 5 retries: {:?}", msg);
                    }
                } else {
                    info!("Published message to {} (id={})", msg.topic, msg.id);
                    time::sleep(Duration::from_millis(50)).await; // 节流
                }
            } else {
                time::sleep(Duration::from_millis(10)).await;
            }
        }
    }

    /// 自动重新订阅所有主题
    async fn resubscribe_all(&self) {
        let topics: Vec<String> = {
            let subs = self.subscribed_topics.lock().await;
            subs.iter().cloned().collect()
        };

        for topic in topics {
            if let Err(e) = self.client.subscribe(&topic, QoS::AtLeastOnce).await {
                error!("Failed to resubscribe {}: {:?}", topic, e);
            } else {
                info!("Resubscribed to topic: {}", topic);
            }
        }
    }

    /// 启动事件循环
    pub async fn start_event_loop<F>(&self, mut callback: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        let eventloop = self.eventloop.clone();

        // 后台任务：处理消息队列
        let manager_for_queue = self.clone();
        task::spawn(async move {
            manager_for_queue.process_queue().await;
        });

        // 事件循环任务
        let manager_for_loop = self.clone();
        task::spawn(async move {
            loop {
                let event_result = {
                    let mut lock = eventloop.lock().await;
                    lock.poll().await
                };

                match event_result {
                    Ok(event) => {
                        // callback 在锁外执行
                        match &event {
                            Event::Incoming(Packet::ConnAck(connack)) => {
                                if connack.session_present {
                                    info!("MQTT session resumed, resubscribing topics...");
                                    manager_for_loop.resubscribe_all().await;
                                } else {
                                    info!("New MQTT session established, skipping resubscribe");
                                }
                            }
                            _ => {}
                        }
                        callback(event);
                    }
                    Err(e) => {
                        error!("MQTT event loop error: {:?}, retrying in 5s...", e);
                        time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });
    }
}

/// 测试函数
pub async fn mqtt_test() -> Result<(), Box<dyn Error>> {
    // 初始化 tracing 日志
    tracing_subscriber::fmt::init();

    let mqtt = MqttManager::new("rust-client", "192.168.100.100", 1883, 30).await?;

    // 启动事件循环
    mqtt.start_event_loop(|event| match event {
        Event::Incoming(packet) => match packet {
            Packet::ConnAck(conn) => info!("Connected: {:?}", conn),
            Packet::Publish(publish) => info!(
                "Received: Topic={}, Payload={:?}, QoS={:?}, Payload Size={}",
                publish.topic,
                publish.payload,
                publish.qos,
                publish.payload.len()
            ),
            _ => info!("Incoming packet: {:?}", packet),
        },
        Event::Outgoing(outgoing) => info!("Sent: {:?}", outgoing),
    })
    .await;

    // 订阅主题
    mqtt.subscribe("hello/world", QoS::AtMostOnce).await?;
    mqtt.subscribe("hello/world2", QoS::AtMostOnce).await?;

    // 将消息加入发送队列
    for i in 1..=10 {
        mqtt.enqueue_publish("hello/world", vec![1; i], QoS::ExactlyOnce)
            .await;
        time::sleep(Duration::from_secs(1)).await;
    }

    // 保持程序运行
    loop {
        time::sleep(Duration::from_secs(60)).await;
    }
}
