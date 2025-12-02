use crate::app_state::AppState;
use crate::models::flow_value::{Entity as FlowValueEntity, Model as FlowValue, ActiveModel as FlowValueActiveModel};
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
pub struct CreateFlowValueRequest {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub device_id: Option<i32>,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateFlowValueRequest {
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

/// 获取流量值列表
#[utoipa::path(
    get,
    path = "/flow-values",
    params(Pagination),
    responses(
        (status = 200, description = "获取流量值列表成功", body = [FlowValue])
    ),
    tag = "Flow Values"
)]
pub async fn get_flow_values(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<FlowValue>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let flow_values = FlowValueEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(flow_values.len());
    let paginated_flow_values = if start < flow_values.len() {
        flow_values[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_flow_values))
}

/// 获取指定流量值
#[utoipa::path(
    get,
    path = "/flow-values/{id}",
    params(
        ("id" = i32, Path, description = "流量值ID")
    ),
    responses(
        (status = 200, description = "获取流量值成功", body = FlowValue),
        (status = 404, description = "流量值未找到")
    ),
    tag = "Flow Values"
)]
pub async fn get_flow_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<FlowValue>, AppError> {
    let conn = state.db.get_connection();
    
    let flow_value = FlowValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(flow_value))
}

/// 创建流量值
#[utoipa::path(
    post,
    path = "/flow-values",
    request_body = CreateFlowValueRequest,
    responses(
        (status = 201, description = "创建流量值成功", body = FlowValue),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Flow Values"
)]
pub async fn create_flow_value(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateFlowValueRequest>,
) -> Result<(StatusCode, Json<FlowValue>), AppError> {
    let conn = state.db.get_connection();
    
    let new_flow_value = FlowValueActiveModel {
        timestamp: sea_orm::Set(payload.timestamp),
        value: sea_orm::Set(payload.value),
        device_id: sea_orm::Set(payload.device_id),
        unit: sea_orm::Set(payload.unit),
        ..Default::default()
    };

    let flow_value = FlowValueEntity::insert(new_flow_value)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(flow_value)))
}

/// 更新流量值
#[utoipa::path(
    put,
    path = "/flow-values/{id}",
    params(
        ("id" = i32, Path, description = "流量值ID")
    ),
    request_body = UpdateFlowValueRequest,
    responses(
        (status = 200, description = "更新流量值成功", body = FlowValue),
        (status = 404, description = "流量值未找到")
    ),
    tag = "Flow Values"
)]
pub async fn update_flow_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateFlowValueRequest>,
) -> Result<Json<FlowValue>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_flow_value = FlowValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut flow_value_active_model = existing_flow_value.into_active_model();
    
    if let Some(timestamp) = payload.timestamp {
        flow_value_active_model.timestamp = sea_orm::Set(timestamp);
    }
    
    if let Some(value) = payload.value {
        flow_value_active_model.value = sea_orm::Set(value);
    }
    
    if let Some(device_id) = payload.device_id {
        flow_value_active_model.device_id = sea_orm::Set(device_id);
    }
    
    if let Some(unit) = payload.unit {
        flow_value_active_model.unit = sea_orm::Set(unit);
    }
    
    // 更新 updated_at 字段
    flow_value_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_flow_value = FlowValueEntity::update(flow_value_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_flow_value))
}

/// 删除流量值
#[utoipa::path(
    delete,
    path = "/flow-values/{id}",
    params(
        ("id" = i32, Path, description = "流量值ID")
    ),
    responses(
        (status = 204, description = "删除流量值成功"),
        (status = 404, description = "流量值未找到")
    ),
    tag = "Flow Values"
)]
pub async fn delete_flow_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let flow_value = FlowValueEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = FlowValueEntity::delete_by_id(flow_value.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}