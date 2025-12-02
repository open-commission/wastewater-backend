use crate::app_state::AppState;
use crate::models::alarm_log::{Entity as AlarmLogEntity, Model as AlarmLog, ActiveModel as AlarmLogActiveModel};
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
pub struct CreateAlarmLogRequest {
    pub rule_name: String,
    pub trigger_value: f64,
    pub is_processed: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateAlarmLogRequest {
    pub rule_name: Option<String>,
    pub trigger_value: Option<f64>,
    pub is_processed: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct Pagination {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// 获取报警日志列表
#[utoipa::path(
    get,
    path = "/alarm-logs",
    params(Pagination),
    responses(
        (status = 200, description = "获取报警日志列表成功", body = [AlarmLog])
    ),
    tag = "Alarm Logs"
)]
pub async fn get_alarm_logs(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<AlarmLog>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let alarm_logs = AlarmLogEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(alarm_logs.len());
    let paginated_logs = if start < alarm_logs.len() {
        alarm_logs[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_logs))
}

/// 获取指定报警日志
#[utoipa::path(
    get,
    path = "/alarm-logs/{id}",
    params(
        ("id" = i32, Path, description = "报警日志ID")
    ),
    responses(
        (status = 200, description = "获取报警日志成功", body = AlarmLog),
        (status = 404, description = "报警日志未找到")
    ),
    tag = "Alarm Logs"
)]
pub async fn get_alarm_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<AlarmLog>, AppError> {
    let conn = state.db.get_connection();
    
    let alarm_log = AlarmLogEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(alarm_log))
}

/// 创建报警日志
#[utoipa::path(
    post,
    path = "/alarm-logs",
    request_body = CreateAlarmLogRequest,
    responses(
        (status = 201, description = "创建报警日志成功", body = AlarmLog),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Alarm Logs"
)]
pub async fn create_alarm_log(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAlarmLogRequest>,
) -> Result<(StatusCode, Json<AlarmLog>), AppError> {
    let conn = state.db.get_connection();
    
    let new_alarm_log = AlarmLogActiveModel {
        rule_name: sea_orm::Set(payload.rule_name),
        trigger_time: sea_orm::Set(chrono::Utc::now()),
        trigger_value: sea_orm::Set(payload.trigger_value),
        is_processed: sea_orm::Set(payload.is_processed),
        ..Default::default()
    };

    let alarm_log = AlarmLogEntity::insert(new_alarm_log)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(alarm_log)))
}

/// 更新报警日志
#[utoipa::path(
    put,
    path = "/alarm-logs/{id}",
    params(
        ("id" = i32, Path, description = "报警日志ID")
    ),
    request_body = UpdateAlarmLogRequest,
    responses(
        (status = 200, description = "更新报警日志成功", body = AlarmLog),
        (status = 404, description = "报警日志未找到")
    ),
    tag = "Alarm Logs"
)]
pub async fn update_alarm_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateAlarmLogRequest>,
) -> Result<Json<AlarmLog>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_alarm_log = AlarmLogEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut alarm_log_active_model = existing_alarm_log.into_active_model();
    
    if let Some(rule_name) = payload.rule_name {
        alarm_log_active_model.rule_name = sea_orm::Set(rule_name);
    }
    
    if let Some(trigger_value) = payload.trigger_value {
        alarm_log_active_model.trigger_value = sea_orm::Set(trigger_value);
    }
    
    if let Some(is_processed) = payload.is_processed {
        alarm_log_active_model.is_processed = sea_orm::Set(is_processed);
    }
    
    // 更新 updated_at 字段
    alarm_log_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_alarm_log = AlarmLogEntity::update(alarm_log_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_alarm_log))
}

/// 删除报警日志
#[utoipa::path(
    delete,
    path = "/alarm-logs/{id}",
    params(
        ("id" = i32, Path, description = "报警日志ID")
    ),
    responses(
        (status = 204, description = "删除报警日志成功"),
        (status = 404, description = "报警日志未找到")
    ),
    tag = "Alarm Logs"
)]
pub async fn delete_alarm_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let alarm_log = AlarmLogEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = AlarmLogEntity::delete_by_id(alarm_log.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}