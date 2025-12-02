use crate::app_state::AppState;
use crate::models::tds_value::{Entity as TdsValueEntity, Model as TdsValue, ActiveModel as TdsValueActiveModel};
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
pub struct CreateTdsValueRequest {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub device_id: Option<i32>,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTdsValueRequest {
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

/// 获取TDS值列表
#[utoipa::path(
    get,
    path = "/tds-values",
    params(Pagination),
    responses(
        (status = 200, description = "获取TDS值列表成功", body = [TdsValue])
    ),
    tag = "TDS Values"
)]
pub async fn get_tds_values(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<TdsValue>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let tds_values = TdsValueEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(tds_values.len());
    let paginated_tds_values = if start < tds_values.len() {
        tds_values[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_tds_values))
}

/// 获取指定TDS值
#[utoipa::path(
    get,
    path = "/tds-values/{id}",
    params(
        ("id" = i32, Path, description = "TDS值ID")
    ),
    responses(
        (status = 200, description = "获取TDS值成功", body = TdsValue),
        (status = 404, description = "TDS值未找到")
    ),
    tag = "TDS Values"
)]
pub async fn get_tds_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TdsValue>, AppError> {
    let conn = state.db.get_connection();
    
    let tds_value = TdsValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(tds_value))
}

/// 创建TDS值
#[utoipa::path(
    post,
    path = "/tds-values",
    request_body = CreateTdsValueRequest,
    responses(
        (status = 201, description = "创建TDS值成功", body = TdsValue),
        (status = 400, description = "请求参数错误")
    ),
    tag = "TDS Values"
)]
pub async fn create_tds_value(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTdsValueRequest>,
) -> Result<(StatusCode, Json<TdsValue>), AppError> {
    let conn = state.db.get_connection();
    
    let new_tds_value = TdsValueActiveModel {
        timestamp: sea_orm::Set(payload.timestamp),
        value: sea_orm::Set(payload.value),
        device_id: sea_orm::Set(payload.device_id),
        unit: sea_orm::Set(payload.unit),
        ..Default::default()
    };

    let tds_value = TdsValueEntity::insert(new_tds_value)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(tds_value)))
}

/// 更新TDS值
#[utoipa::path(
    put,
    path = "/tds-values/{id}",
    params(
        ("id" = i32, Path, description = "TDS值ID")
    ),
    request_body = UpdateTdsValueRequest,
    responses(
        (status = 200, description = "更新TDS值成功", body = TdsValue),
        (status = 404, description = "TDS值未找到")
    ),
    tag = "TDS Values"
)]
pub async fn update_tds_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTdsValueRequest>,
) -> Result<Json<TdsValue>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_tds_value = TdsValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut tds_value_active_model = existing_tds_value.into_active_model();
    
    if let Some(timestamp) = payload.timestamp {
        tds_value_active_model.timestamp = sea_orm::Set(timestamp);
    }
    
    if let Some(value) = payload.value {
        tds_value_active_model.value = sea_orm::Set(value);
    }
    
    if let Some(device_id) = payload.device_id {
        tds_value_active_model.device_id = sea_orm::Set(device_id);
    }
    
    if let Some(unit) = payload.unit {
        tds_value_active_model.unit = sea_orm::Set(unit);
    }
    
    // 更新 updated_at 字段
    tds_value_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_tds_value = TdsValueEntity::update(tds_value_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_tds_value))
}

/// 删除TDS值
#[utoipa::path(
    delete,
    path = "/tds-values/{id}",
    params(
        ("id" = i32, Path, description = "TDS值ID")
    ),
    responses(
        (status = 204, description = "删除TDS值成功"),
        (status = 404, description = "TDS值未找到")
    ),
    tag = "TDS Values"
)]
pub async fn delete_tds_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let tds_value = TdsValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = TdsValueEntity::delete_by_id(tds_value.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}