use crate::{handlers::user, app_state::AppState};
use axum::{routing::get, Router};
use std::sync::Arc;

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