use crate::app_state::AppState;
use crate::models::alarm_rule::{Entity as AlarmRuleEntity, Model as AlarmRule, ActiveModel as AlarmRuleActiveModel};
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
pub struct CreateAlarmRuleRequest {
    pub name: String,
    pub condition: String,
    pub parameter: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateAlarmRuleRequest {
    pub name: Option<String>,
    pub condition: Option<String>,
    pub parameter: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct Pagination {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// 获取报警规则列表
#[utoipa::path(
    get,
    path = "/alarm-rules",
    params(Pagination),
    responses(
        (status = 200, description = "获取报警规则列表成功", body = [AlarmRule])
    ),
    tag = "Alarm Rules"
)]
pub async fn get_alarm_rules(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<AlarmRule>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let alarm_rules = AlarmRuleEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(alarm_rules.len());
    let paginated_alarm_rules = if start < alarm_rules.len() {
        alarm_rules[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_alarm_rules))
}

/// 获取指定报警规则
#[utoipa::path(
    get,
    path = "/alarm-rules/{id}",
    params(
        ("id" = i32, Path, description = "报警规则ID")
    ),
    responses(
        (status = 200, description = "获取报警规则成功", body = AlarmRule),
        (status = 404, description = "报警规则未找到")
    ),
    tag = "Alarm Rules"
)]
pub async fn get_alarm_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<AlarmRule>, AppError> {
    let conn = state.db.get_connection();
    
    let alarm_rule = AlarmRuleEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(alarm_rule))
}

/// 创建报警规则
#[utoipa::path(
    post,
    path = "/alarm-rules",
    request_body = CreateAlarmRuleRequest,
    responses(
        (status = 201, description = "创建报警规则成功", body = AlarmRule),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Alarm Rules"
)]
pub async fn create_alarm_rule(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAlarmRuleRequest>,
) -> Result<(StatusCode, Json<AlarmRule>), AppError> {
    let conn = state.db.get_connection();
    
    let new_alarm_rule = AlarmRuleActiveModel {
        name: sea_orm::Set(payload.name),
        condition: sea_orm::Set(payload.condition),
        parameter: sea_orm::Set(payload.parameter),
        value: sea_orm::Set(payload.value),
        ..Default::default()
    };

    let alarm_rule = AlarmRuleEntity::insert(new_alarm_rule)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(alarm_rule)))
}

/// 更新报警规则
#[utoipa::path(
    put,
    path = "/alarm-rules/{id}",
    params(
        ("id" = i32, Path, description = "报警规则ID")
    ),
    request_body = UpdateAlarmRuleRequest,
    responses(
        (status = 200, description = "更新报警规则成功", body = AlarmRule),
        (status = 404, description = "报警规则未找到")
    ),
    tag = "Alarm Rules"
)]
pub async fn update_alarm_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateAlarmRuleRequest>,
) -> Result<Json<AlarmRule>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_alarm_rule = AlarmRuleEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut alarm_rule_active_model = existing_alarm_rule.into_active_model();
    
    if let Some(name) = payload.name {
        alarm_rule_active_model.name = sea_orm::Set(name);
    }
    
    if let Some(condition) = payload.condition {
        alarm_rule_active_model.condition = sea_orm::Set(condition);
    }
    
    if let Some(parameter) = payload.parameter {
        alarm_rule_active_model.parameter = sea_orm::Set(parameter);
    }
    
    if let Some(value) = payload.value {
        alarm_rule_active_model.value = sea_orm::Set(value);
    }
    
    // 更新 updated_at 字段
    alarm_rule_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_alarm_rule = AlarmRuleEntity::update(alarm_rule_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_alarm_rule))
}

/// 删除报警规则
#[utoipa::path(
    delete,
    path = "/alarm-rules/{id}",
    params(
        ("id" = i32, Path, description = "报警规则ID")
    ),
    responses(
        (status = 204, description = "删除报警规则成功"),
        (status = 404, description = "报警规则未找到")
    ),
    tag = "Alarm Rules"
)]
pub async fn delete_alarm_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let alarm_rule = AlarmRuleEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = AlarmRuleEntity::delete_by_id(alarm_rule.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}