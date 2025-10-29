use axum::{
    http::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;

pub async fn logging_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    info!("Processing request: {} {}", method, uri);
    
    let response = next.run(request).await;
    
    info!("Response status: {}", response.status());
    
    response
}