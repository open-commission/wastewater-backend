use std::sync::{Arc, RwLock};

mod config;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;
mod database;
mod mqtt;

use config::server::ServerConfig;
use models::user::User;

use database::redb::test_redb_basic;

use mqtt::rumqtt::mqtt_test;

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<Vec<User>>>,
    pub config: ServerConfig,
}

#[tokio::main]
async fn main() {
    // // Initialize tracing
    // tracing_subscriber::fmt::init();
    //
    // // Load configuration
    // let config = ServerConfig::default();
    //
    // // Create initial state
    // let state = Arc::new(AppState {
    //     users: Arc::new(RwLock::new(vec![])),
    //     config: config.clone(),
    // });
    //
    // // Create router
    // let api_router = Router::new().nest("/api/v1", routes::api::create_api_router());
    // let static_router = routes::static_files::create_static_router();
    //
    // let app = static_router
    //     .merge(api_router)
    //     .layer(axum::middleware::from_fn(middleware::logging::logging_middleware))
    //     .with_state(state);
    //
    // // Run server
    // let listener = tokio::net::TcpListener::bind(config.address()).await.unwrap();
    // tracing::info!("Server listening on {}", config.address());
    //
    // axum::serve(listener, app).await.unwrap();

    // modbus_example::run_tcp_server().await.expect("Failed to run TCP server")

    // test_redb_basic().expect("Failed to run redb test")

    mqtt_test().await.expect("Failed to run mqtt test")
}