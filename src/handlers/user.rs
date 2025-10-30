use axum::{
    http::StatusCode,
    response::Json,
    extract::{Path, State},
};
use std::sync::{Arc};
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::AppState;
use crate::utils::error::AppError;


pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = state.users.read().map_err(|_| AppError::InternalError)?.clone();
    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<User>, AppError> {
    let users = state.users.read().map_err(|_| AppError::InternalError)?;
    let user = users.iter().find(|u| u.id == id)
        .ok_or(AppError::UserNotFound)?
        .clone();
    
    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    // 验证输入
    if payload.name.is_empty() {
        return Err(AppError::InvalidInput("Name cannot be empty".into()));
    }
    
    if payload.email.is_empty() {
        return Err(AppError::InvalidInput("Email cannot be empty".into()));
    }
    
    let mut users = state.users.write().map_err(|_| AppError::InternalError)?;
    
    // 生成唯一的ID
    let new_id = users.iter().map(|user| user.id).max().unwrap_or(0) + 1;
    
    let user = User {
        id: new_id,
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
) -> Result<Json<User>, AppError> {
    let mut users = state.users.write().map_err(|_| AppError::InternalError)?;
    let user = users.iter_mut().find(|u| u.id == id)
        .ok_or(AppError::UserNotFound)?;
    
    if let Some(name) = payload.name {
        if name.is_empty() {
            return Err(AppError::InvalidInput("Name cannot be empty".into()));
        }
        user.name = name;
    }
    
    if let Some(email) = payload.email {
        if email.is_empty() {
            return Err(AppError::InvalidInput("Email cannot be empty".into()));
        }
        user.email = email;
    }

    let user_clone = user.clone();
    Ok(Json(user_clone))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    let mut users = state.users.write().map_err(|_| AppError::InternalError)?;
    let len_before = users.len();
    users.retain(|u| u.id != id);
    
    if users.len() == len_before {
        Err(AppError::UserNotFound)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}