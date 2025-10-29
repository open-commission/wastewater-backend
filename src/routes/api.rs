#[allow(unused_imports)]
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use crate::{handlers::user, AppState};

pub fn create_api_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(user::get_users).post(user::create_user))
        .route(
            "/users/{id}",
            get(user::get_user)
                .put(user::update_user)
                .delete(user::delete_user),
        )
}