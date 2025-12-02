mod app_state;
mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod mqtt;
mod message_queue;
mod routes;
mod utils;

use app_state::AppState;
use database::sea_orm_db::DbManager;
use message_queue::consumer_example;
use message_queue::rabbitmq::{Message, RabbitMQManager};
use models::user::Model as User;
use routes::api::create_api_router;
use std::sync::{Arc, RwLock};
use tracing_subscriber;
use axum::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 测试SeaORM数据库连接
    println!("正在测试SeaORM数据库连接...");
    let db_manager = DbManager::new("sqlite://guolu.db?mode=rwc").await?;
    println!(
        "SeaORM数据库连接成功: {:?}",
        db_manager.get_connection().ping().await
    );

    // 初始化 RabbitMQ 连接
    println!("正在初始化 RabbitMQ 连接...");
    let rabbitmq_manager = RabbitMQManager::new("amqp://guest:guest@localhost:5672/%2f");
    
    // 尝试连接到 RabbitMQ 服务器
    match rabbitmq_manager.connect().await {
        Ok(()) => {
            println!("RabbitMQ 连接成功");
            
            // 示例：发送一条测试消息
            let message = Message {
                topic: "test.message".to_string(),
                payload: "Hello from boiler system!".to_string(),
                timestamp: chrono::Utc::now(),
            };
            
            match rabbitmq_manager
                .publish_message("boiler_exchange", "test.message", &message)
                .await
            {
                Ok(_) => println!("测试消息发送成功"),
                Err(e) => println!("测试消息发送失败: {}", e),
            }
            
            // 启动消息消费者任务
            match consumer_example::start_consumer_task(rabbitmq_manager.clone(), "boiler_queue").await {
                Ok(handle) => {
                    println!("消息消费者任务已启动");
                    // 可以保存 handle 以便后续管理任务
                    // 这里为了简单起见，我们不等待任务完成
                    tokio::spawn(handle);
                }
                Err(e) => {
                    println!("启动消息消费者任务失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("RabbitMQ 连接失败: {}", e);
        }
    }

    // 初始化应用状态
    let initial_users = vec![
        User {
            id: 1,
            name: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
            password: "123456".to_string(),
            permission: "123".to_string(),
        },
        User {
            id: 2,
            name: "李四".to_string(),
            email: "lisi@example.com".to_string(),
            password: "123456".to_string(),
            permission: "123".to_string(),
        },
    ];

    let app_state = AppState {
        users: Arc::new(RwLock::new(initial_users)),
        db: db_manager,
    };

    // 创建应用路由
    let app = Router::new()
        .merge(create_api_router())
        .with_state(Arc::new(app_state));

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("服务器运行在 http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}