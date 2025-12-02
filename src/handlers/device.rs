use crate::app_state::AppState;
use crate::models::device::{Entity as DeviceEntity, Model as Device, ActiveModel as DeviceActiveModel};
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
pub struct CreateDeviceRequest {
    pub name: String,
    pub location: String,
    pub status: i32,
    pub device_type: String,
    pub manufacturer: String,
    pub model: String,
    pub installation_date: chrono::DateTime<chrono::Utc>,
    pub last_maintenance: chrono::DateTime<chrono::Utc>,
    pub operational_hours: f64,
    pub temperature: f64,
    pub pressure: f64,
    pub flow_rate: f64,
    pub power_consumption: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub location: Option<String>,
    pub status: Option<i32>,
    pub device_type: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub installation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_maintenance: Option<chrono::DateTime<chrono::Utc>>,
    pub operational_hours: Option<f64>,
    pub temperature: Option<f64>,
    pub pressure: Option<f64>,
    pub flow_rate: Option<f64>,
    pub power_consumption: Option<f64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct Pagination {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// 获取设备列表
#[utoipa::path(
    get,
    path = "/devices",
    params(Pagination),
    responses(
        (status = 200, description = "获取设备列表成功", body = [Device])
    ),
    tag = "Devices"
)]
pub async fn get_devices(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<Device>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let devices = DeviceEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(devices.len());
    let paginated_devices = if start < devices.len() {
        devices[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_devices))
}

/// 获取指定设备
#[utoipa::path(
    get,
    path = "/devices/{id}",
    params(
        ("id" = i32, Path, description = "设备ID")
    ),
    responses(
        (status = 200, description = "获取设备成功", body = Device),
        (status = 404, description = "设备未找到")
    ),
    tag = "Devices"
)]
pub async fn get_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Device>, AppError> {
    let conn = state.db.get_connection();
    
    let device = DeviceEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(device))
}

/// 创建设备
#[utoipa::path(
    post,
    path = "/devices",
    request_body = CreateDeviceRequest,
    responses(
        (status = 201, description = "创建设备成功", body = Device),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Devices"
)]
pub async fn create_device(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDeviceRequest>,
) -> Result<(StatusCode, Json<Device>), AppError> {
    let conn = state.db.get_connection();
    
    let new_device = DeviceActiveModel {
        name: sea_orm::Set(payload.name),
        location: sea_orm::Set(payload.location),
        status: sea_orm::Set(payload.status),
        device_type: sea_orm::Set(payload.device_type),
        manufacturer: sea_orm::Set(payload.manufacturer),
        model: sea_orm::Set(payload.model),
        installation_date: sea_orm::Set(payload.installation_date),
        last_maintenance: sea_orm::Set(payload.last_maintenance),
        operational_hours: sea_orm::Set(payload.operational_hours),
        temperature: sea_orm::Set(payload.temperature),
        pressure: sea_orm::Set(payload.pressure),
        flow_rate: sea_orm::Set(payload.flow_rate),
        power_consumption: sea_orm::Set(payload.power_consumption),
        ..Default::default()
    };

    let device = DeviceEntity::insert(new_device)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(device)))
}

/// 更新设备
#[utoipa::path(
    put,
    path = "/devices/{id}",
    params(
        ("id" = i32, Path, description = "设备ID")
    ),
    request_body = UpdateDeviceRequest,
    responses(
        (status = 200, description = "更新设备成功", body = Device),
        (status = 404, description = "设备未找到")
    ),
    tag = "Devices"
)]
pub async fn update_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateDeviceRequest>,
) -> Result<Json<Device>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_device = DeviceEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut device_active_model = existing_device.into_active_model();
    
    if let Some(name) = payload.name {
        device_active_model.name = sea_orm::Set(name);
    }
    
    if let Some(location) = payload.location {
        device_active_model.location = sea_orm::Set(location);
    }
    
    if let Some(status) = payload.status {
        device_active_model.status = sea_orm::Set(status);
    }
    
    if let Some(device_type) = payload.device_type {
        device_active_model.device_type = sea_orm::Set(device_type);
    }
    
    if let Some(manufacturer) = payload.manufacturer {
        device_active_model.manufacturer = sea_orm::Set(manufacturer);
    }
    
    if let Some(model) = payload.model {
        device_active_model.model = sea_orm::Set(model);
    }
    
    if let Some(installation_date) = payload.installation_date {
        device_active_model.installation_date = sea_orm::Set(installation_date);
    }
    
    if let Some(last_maintenance) = payload.last_maintenance {
        device_active_model.last_maintenance = sea_orm::Set(last_maintenance);
    }
    
    if let Some(operational_hours) = payload.operational_hours {
        device_active_model.operational_hours = sea_orm::Set(operational_hours);
    }
    
    if let Some(temperature) = payload.temperature {
        device_active_model.temperature = sea_orm::Set(temperature);
    }
    
    if let Some(pressure) = payload.pressure {
        device_active_model.pressure = sea_orm::Set(pressure);
    }
    
    if let Some(flow_rate) = payload.flow_rate {
        device_active_model.flow_rate = sea_orm::Set(flow_rate);
    }
    
    if let Some(power_consumption) = payload.power_consumption {
        device_active_model.power_consumption = sea_orm::Set(power_consumption);
    }
    
    // 更新 updated_at 字段
    device_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_device = DeviceEntity::update(device_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_device))
}

/// 删除设备
#[utoipa::path(
    delete,
    path = "/devices/{id}",
    params(
        ("id" = i32, Path, description = "设备ID")
    ),
    responses(
        (status = 204, description = "删除设备成功"),
        (status = 404, description = "设备未找到")
    ),
    tag = "Devices"
)]
pub async fn delete_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let device = DeviceEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = DeviceEntity::delete_by_id(device.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}