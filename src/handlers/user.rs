use axum::{
    http::StatusCode,
    response::Json,
    extract::{Path, State},
};
use std::sync::Arc;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::AppState;

pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, StatusCode> {
    Ok(Json(state.users.clone()))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<User>, StatusCode> {
    // In a real app, you would fetch from a database
    let user = state.users.iter().find(|u| u.id == id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();
    
    Ok(Json(user))
}

pub async fn create_user(
    State(mut state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let user = User {
        id: state.users.len() as u64 + 1,
        name: payload.name,
        email: payload.email,
    };

    // In a real app, you would insert into a database
    Arc::make_mut(&mut state).users.push(user.clone());
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<u64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    // In a real app, you would update in a database
    let user = User {
        id: 1,
        name: payload.name.unwrap_or_else(|| "John Doe".to_string()),
        email: payload.email.unwrap_or_else(|| "john.doe@example.com".to_string()),
    };

    Ok(Json(user))
}

pub async fn delete_user(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<u64>,
) -> Result<StatusCode, StatusCode> {
    // In a real app, you would delete from a database
    Ok(StatusCode::NO_CONTENT)
}