use crate::app_state::AppState;
use crate::models::user::User;
use crate::utils::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub async fn get_users(State(state): State<Arc<AppState>>) -> Result<Json<Vec<User>>, AppError> {
    let users = state
        .users
        .read()
        .map_err(|_| AppError::InternalError)?
        .clone();
    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
) -> Result<Json<User>, AppError> {
    let users = state.users.read().map_err(|_| AppError::InternalError)?;
    let user = users
        .iter()
        .find(|u| u.id == id)
        .ok_or(AppError::UserNotFound)?
        .clone();

    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<User>,
) -> Result<(StatusCode, Json<User>), AppError> {
    // 验证输入
    if payload.name.is_empty() {
        return Err(AppError::InvalidInput("Name cannot be empty".into()));
    }

    if payload.email.is_empty() {
        return Err(AppError::InvalidInput("Email cannot be empty".into()));
    }

    // 检查邮箱是否已存在
    {
        let users = state.users.read().map_err(|_| AppError::InternalError)?;
        if users.iter().any(|u| u.email == payload.email) {
            return Err(AppError::InvalidInput("Email already exists".into()));
        }
    }

    let mut users = state.users.write().map_err(|_| AppError::InternalError)?;

    // 生成唯一的ID
    let new_id = users.iter().map(|user| user.id).max().unwrap_or(0) + 1;

    let user = User {
        id: new_id,
        name: payload.name,
        email: payload.email,
        password: payload.password,
        permission: payload.permission,
    };

    users.push(user.clone());
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Json(payload): Json<User>,
) -> Result<Json<User>, AppError> {
    // 先检查邮箱是否已存在于其他用户（在获取写锁之前进行检查）
    {
        let users = state.users.read().map_err(|_| AppError::InternalError)?;
        if users.iter().any(|u| u.id != id && u.email == payload.email) {
            return Err(AppError::InvalidInput("Email already exists".into()));
        }
    }
    
    let mut users = state.users.write().map_err(|_| AppError::InternalError)?;
    let user = users
        .iter_mut()
        .find(|u| u.id == id)
        .ok_or(AppError::UserNotFound)?;

    // 注意：User 结构体中的字段都是 String 类型，不是 Option<String>
    if !payload.name.is_empty() {
        user.name = payload.name;
    }

    if !payload.email.is_empty() {
        user.email = payload.email;
    }

    user.password = payload.password;
    user.permission = payload.permission;

    let user_clone = user.clone();
    Ok(Json(user_clone))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
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

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let users = state.users.read().map_err(|_| AppError::InternalError)?;
    
    // 查找匹配的用户
    let user = users
        .iter()
        .find(|u| u.email == payload.email && u.password == payload.password)
        .cloned()
        .ok_or(AppError::InvalidCredentials)?;

    // 生成简单token（实际项目中应该使用JWT或其他安全的token机制）
    let token = format!("token_{}_{}", user.id, chrono::Utc::now().timestamp());
    
    let response = LoginResponse {
        token,
        user,
    };
    
    Ok(Json(response))
}