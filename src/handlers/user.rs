use axum::{
    http::StatusCode,
    response::Json,
    extract::{Path, State},
};
use std::sync::{Arc};
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::AppState;

pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let users = state.users.read().unwrap().clone();
    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<User>, StatusCode> {
    let users = state.users.read().unwrap();
    let user = users.iter().find(|u| u.id == id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();
    
    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let mut users = state.users.write().unwrap();
    let user = User {
        id: users.len() as u64 + 1,
        name: payload.name,
        email: payload.email,
    };

    users.push(user.clone());
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let mut users = state.users.write().unwrap();
    let user = users.iter_mut().find(|u| u.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = payload.name {
        user.name = name;
    }
    
    if let Some(email) = payload.email {
        user.email = email;
    }

    let user_clone = user.clone();
    Ok(Json(user_clone))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, StatusCode> {
    let mut users = state.users.write().unwrap();
    let len_before = users.len();
    users.retain(|u| u.id != id);
    
    if users.len() == len_before {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}