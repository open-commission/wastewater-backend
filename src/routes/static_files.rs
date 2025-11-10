use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse},
    http::{StatusCode, Uri},
};
use std::sync::Arc;
use crate::app_state::AppState;

pub fn create_static_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(serve_index))
        .route("/{*file}", get(serve_static_file))
}

async fn serve_index() -> impl IntoResponse {
    serve_file("dist/index.html").await
}

async fn serve_static_file(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    let path = path.strip_prefix("/").unwrap_or(path);
    
    // 确保路径安全，防止目录遍历攻击
    if path.contains("..") {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let file_path = format!("dist/{}", path);
    serve_file(&file_path).await
}

async fn serve_file(file_path: &str) -> Result<Html<String>, StatusCode> {
    // 尝试读取请求的文件
    let content = match tokio::fs::read_to_string(file_path).await {
        Ok(content) => content,
        Err(_) => tokio::fs::read_to_string("dist/index.html").await
            .map_err(|_| StatusCode::NOT_FOUND)?,
    };
    
    Ok(Html(content))
}