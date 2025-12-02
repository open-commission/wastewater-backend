use crate::app_state::AppState;
use crate::models::ph_value::{Entity as PhValueEntity, Model as PhValue, ActiveModel as PhValueActiveModel};
use crate::utils::error::AppError;
use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use sea_orm::{EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa::IntoParams;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePhValueRequest {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub device_id: Option<i32>,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatePhValueRequest {
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub value: Option<f64>,
    pub device_id: Option<Option<i32>>,
    pub unit: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct Pagination {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// 获取PH值列表
#[utoipa::path(
    get,
    path = "/ph-values",
    params(Pagination),
    responses(
        (status = 200, description = "获取PH值列表成功", body = [PhValue])
    ),
    tag = "PH Values"
)]
pub async fn get_ph_values(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<PhValue>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let ph_values = PhValueEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(ph_values.len());
    let paginated_ph_values = if start < ph_values.len() {
        ph_values[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_ph_values))
}

/// 获取指定PH值
#[utoipa::path(
    get,
    path = "/ph-values/{id}",
    params(
        ("id" = i32, Path, description = "PH值ID")
    ),
    responses(
        (status = 200, description = "获取PH值成功", body = PhValue),
        (status = 404, description = "PH值未找到")
    ),
    tag = "PH Values"
)]
pub async fn get_ph_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<PhValue>, AppError> {
    let conn = state.db.get_connection();
    
    let ph_value = PhValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(ph_value))
}

/// 创建PH值
#[utoipa::path(
    post,
    path = "/ph-values",
    request_body = CreatePhValueRequest,
    responses(
        (status = 201, description = "创建PH值成功", body = PhValue),
        (status = 400, description = "请求参数错误")
    ),
    tag = "PH Values"
)]
pub async fn create_ph_value(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePhValueRequest>,
) -> Result<(StatusCode, Json<PhValue>), AppError> {
    let conn = state.db.get_connection();
    
    let new_ph_value = PhValueActiveModel {
        timestamp: sea_orm::Set(payload.timestamp),
        value: sea_orm::Set(payload.value),
        device_id: sea_orm::Set(payload.device_id),
        unit: sea_orm::Set(payload.unit),
        ..Default::default()
    };

    let ph_value = PhValueEntity::insert(new_ph_value)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(ph_value)))
}

/// 更新PH值
#[utoipa::path(
    put,
    path = "/ph-values/{id}",
    params(
        ("id" = i32, Path, description = "PH值ID")
    ),
    request_body = UpdatePhValueRequest,
    responses(
        (status = 200, description = "更新PH值成功", body = PhValue),
        (status = 404, description = "PH值未找到")
    ),
    tag = "PH Values"
)]
pub async fn update_ph_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePhValueRequest>,
) -> Result<Json<PhValue>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_ph_value = PhValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut ph_value_active_model = existing_ph_value.into_active_model();
    
    if let Some(timestamp) = payload.timestamp {
        ph_value_active_model.timestamp = sea_orm::Set(timestamp);
    }
    
    if let Some(value) = payload.value {
        ph_value_active_model.value = sea_orm::Set(value);
    }
    
    if let Some(device_id) = payload.device_id {
        ph_value_active_model.device_id = sea_orm::Set(device_id);
    }
    
    if let Some(unit) = payload.unit {
        ph_value_active_model.unit = sea_orm::Set(unit);
    }
    
    // 更新 updated_at 字段
    ph_value_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_ph_value = PhValueEntity::update(ph_value_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_ph_value))
}

/// 删除PH值
#[utoipa::path(
    delete,
    path = "/ph-values/{id}",
    params(
        ("id" = i32, Path, description = "PH值ID")
    ),
    responses(
        (status = 204, description = "删除PH值成功"),
        (status = 404, description = "PH值未找到")
    ),
    tag = "PH Values"
)]
pub async fn delete_ph_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let ph_value = PhValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = PhValueEntity::delete_by_id(ph_value.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}