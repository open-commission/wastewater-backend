use crate::app_state::AppState;
use crate::models::turbidity_value::{Entity as TurbidityValueEntity, Model as TurbidityValue, ActiveModel as TurbidityValueActiveModel};
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
pub struct CreateTurbidityValueRequest {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub device_id: Option<i32>,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTurbidityValueRequest {
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

/// 获取浊度值列表
#[utoipa::path(
    get,
    path = "/turbidity-values",
    params(Pagination),
    responses(
        (status = 200, description = "获取浊度值列表成功", body = [TurbidityValue])
    ),
    tag = "Turbidity Values"
)]
pub async fn get_turbidity_values(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<TurbidityValue>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let turbidity_values = TurbidityValueEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(turbidity_values.len());
    let paginated_turbidity_values = if start < turbidity_values.len() {
        turbidity_values[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_turbidity_values))
}

/// 获取指定浊度值
#[utoipa::path(
    get,
    path = "/turbidity-values/{id}",
    params(
        ("id" = i32, Path, description = "浊度值ID")
    ),
    responses(
        (status = 200, description = "获取浊度值成功", body = TurbidityValue),
        (status = 404, description = "浊度值未找到")
    ),
    tag = "Turbidity Values"
)]
pub async fn get_turbidity_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TurbidityValue>, AppError> {
    let conn = state.db.get_connection();
    
    let turbidity_value = TurbidityValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(turbidity_value))
}

/// 创建浊度值
#[utoipa::path(
    post,
    path = "/turbidity-values",
    request_body = CreateTurbidityValueRequest,
    responses(
        (status = 201, description = "创建浊度值成功", body = TurbidityValue),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Turbidity Values"
)]
pub async fn create_turbidity_value(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTurbidityValueRequest>,
) -> Result<(StatusCode, Json<TurbidityValue>), AppError> {
    let conn = state.db.get_connection();
    
    let new_turbidity_value = TurbidityValueActiveModel {
        timestamp: sea_orm::Set(payload.timestamp),
        value: sea_orm::Set(payload.value),
        device_id: sea_orm::Set(payload.device_id),
        unit: sea_orm::Set(payload.unit),
        ..Default::default()
    };

    let turbidity_value = TurbidityValueEntity::insert(new_turbidity_value)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(turbidity_value)))
}

/// 更新浊度值
#[utoipa::path(
    put,
    path = "/turbidity-values/{id}",
    params(
        ("id" = i32, Path, description = "浊度值ID")
    ),
    request_body = UpdateTurbidityValueRequest,
    responses(
        (status = 200, description = "更新浊度值成功", body = TurbidityValue),
        (status = 404, description = "浊度值未找到")
    ),
    tag = "Turbidity Values"
)]
pub async fn update_turbidity_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTurbidityValueRequest>,
) -> Result<Json<TurbidityValue>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_turbidity_value = TurbidityValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut turbidity_value_active_model = existing_turbidity_value.into_active_model();
    
    if let Some(timestamp) = payload.timestamp {
        turbidity_value_active_model.timestamp = sea_orm::Set(timestamp);
    }
    
    if let Some(value) = payload.value {
        turbidity_value_active_model.value = sea_orm::Set(value);
    }
    
    if let Some(device_id) = payload.device_id {
        turbidity_value_active_model.device_id = sea_orm::Set(device_id);
    }
    
    if let Some(unit) = payload.unit {
        turbidity_value_active_model.unit = sea_orm::Set(unit);
    }
    
    // 更新 updated_at 字段
    turbidity_value_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_turbidity_value = TurbidityValueEntity::update(turbidity_value_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_turbidity_value))
}

/// 删除浊度值
#[utoipa::path(
    delete,
    path = "/turbidity-values/{id}",
    params(
        ("id" = i32, Path, description = "浊度值ID")
    ),
    responses(
        (status = 204, description = "删除浊度值成功"),
        (status = 404, description = "浊度值未找到")
    ),
    tag = "Turbidity Values"
)]
pub async fn delete_turbidity_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let turbidity_value = TurbidityValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = TurbidityValueEntity::delete_by_id(turbidity_value.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}