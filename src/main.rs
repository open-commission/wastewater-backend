use axum::{
    Router,
};
use std::sync::{Arc, RwLock};
use tracing_subscriber;

mod config;
mod models;
mod handlers;
mod routes;
mod middleware;
mod utils;

use config::server::ServerConfig;
use models::user::User;

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<Vec<User>>>,
    pub config: ServerConfig,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = ServerConfig::default();
    
    // Create initial state
    let state = Arc::new(AppState {
        users: Arc::new(RwLock::new(vec![])),
        config: config.clone(),
    });

    // Create router
    let app = Router::new()
        .nest("/api/v1", routes::api::create_api_router())
        .layer(axum::middleware::from_fn(middleware::logging::logging_middleware))
        .with_state(state);

    // Run server
    let listener = tokio::net::TcpListener::bind(config.address()).await.unwrap();
    tracing::info!("Server listening on {}", config.address());
    
    axum::serve(listener, app).await.unwrap();
}