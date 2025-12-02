use crate::app_state::AppState;
use crate::models::user::Model as User;
use crate::utils::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub permission: Option<String>,
}

/// 获取用户列表
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "获取用户列表成功", body = [User])
    ),
    tag = "Users"
)]
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = state.users.read().unwrap().clone();
    Ok(Json(users))
}

/// 获取指定用户
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = i32, Path, description = "用户ID")
    ),
    responses(
        (status = 200, description = "获取用户成功", body = User),
        (status = 404, description = "用户未找到")
    ),
    tag = "Users"
)]
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
) -> Result<Json<User>, AppError> {
    let users = state.users.read().unwrap();
    let user = users.iter().find(|u| u.id == id).cloned();

    match user {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound),
    }
}

/// 创建用户
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "创建用户成功", body = User),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Users"
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let mut users = state.users.write().unwrap();
    
    // 确定新用户的ID
    let new_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;

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

/// 更新用户
#[utoipa::path(
    put,
    path = "/users/{id}",
    params(
        ("id" = i32, Path, description = "用户ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "更新用户成功", body = User),
        (status = 404, description = "用户未找到")
    ),
    tag = "Users"
)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    let mut users = state.users.write().unwrap();
    let user = users.iter_mut().find(|u| u.id == id);

    match user {
        Some(u) => {
            if let Some(name) = payload.name {
                u.name = name;
            }
            if let Some(email) = payload.email {
                u.email = email;
            }
            if let Some(password) = payload.password {
                u.password = password;
            }
            if let Some(permission) = payload.permission {
                u.permission = permission;
            }
            
            let user_clone = u.clone();
            Ok(Json(user_clone))
        }
        None => Err(AppError::NotFound),
    }
}

/// 删除用户
#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = i32, Path, description = "用户ID")
    ),
    responses(
        (status = 204, description = "删除用户成功"),
        (status = 404, description = "用户未找到")
    ),
    tag = "Users"
)]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
) -> Result<StatusCode, AppError> {
    let mut users = state.users.write().unwrap();
    let len_before = users.len();
    users.retain(|u| u.id != id);
    
    if users.len() < len_before {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}