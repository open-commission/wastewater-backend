mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod mqtt;
mod routes;
mod utils;
mod app_state;

use tracing_subscriber;
use std::sync::{Arc, RwLock};
use models::user::User;
use app_state::AppState;
use database::sea_orm_db::DbManager;
use database::sea_orm_example::run_sea_orm_example;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 初始化应用状态
    let initial_users = vec![
        User {
            id: 1,
            name: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
        },
        User {
            id: 2,
            name: "李四".to_string(),
            email: "lisi@example.com".to_string(),
        },
    ];

    let _app_state = AppState {
        users: Arc::new(RwLock::new(initial_users)),
    };

    // 测试SeaORM数据库连接
    println!("正在测试SeaORM数据库连接...");
    let db_manager = DbManager::new("sqlite::memory:").await?;
    println!("SeaORM数据库连接成功: {:?}", db_manager.get_connection().ping().await);
    
    // 运行SeaORM示例
    run_sea_orm_example().await?;

    Ok(())
}