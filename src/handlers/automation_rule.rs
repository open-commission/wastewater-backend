use crate::app_state::AppState;
use crate::models::automation_rule::{Entity as AutomationRuleEntity, Model as AutomationRule, ActiveModel as AutomationRuleActiveModel};
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
pub struct CreateAutomationRuleRequest {
    pub action: String,
    pub level: i32,
    pub trigger_time_range: String,
    pub sync_alarm: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateAutomationRuleRequest {
    pub action: Option<String>,
    pub level: Option<i32>,
    pub trigger_time_range: Option<String>,
    pub sync_alarm: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct Pagination {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// 获取自动化规则列表
#[utoipa::path(
    get,
    path = "/automation-rules",
    params(Pagination),
    responses(
        (status = 200, description = "获取自动化规则列表成功", body = [AutomationRule])
    ),
    tag = "Automation Rules"
)]
pub async fn get_automation_rules(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<AutomationRule>>, AppError> {
    let conn = state.db.get_connection();
    
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10).min(100); // 限制每页最多100条
    
    let automation_rules = AutomationRuleEntity::find()
        .all(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    // 简化的分页实现
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(automation_rules.len());
    let paginated_automation_rules = if start < automation_rules.len() {
        automation_rules[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(paginated_automation_rules))
}

/// 获取指定自动化规则
#[utoipa::path(
    get,
    path = "/automation-rules/{id}",
    params(
        ("id" = i32, Path, description = "自动化规则ID")
    ),
    responses(
        (status = 200, description = "获取自动化规则成功", body = AutomationRule),
        (status = 404, description = "自动化规则未找到")
    ),
    tag = "Automation Rules"
)]
pub async fn get_automation_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<AutomationRule>, AppError> {
    let conn = state.db.get_connection();
    
    let automation_rule = AutomationRuleEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(automation_rule))
}

/// 创建自动化规则
#[utoipa::path(
    post,
    path = "/automation-rules",
    request_body = CreateAutomationRuleRequest,
    responses(
        (status = 201, description = "创建自动化规则成功", body = AutomationRule),
        (status = 400, description = "请求参数错误")
    ),
    tag = "Automation Rules"
)]
pub async fn create_automation_rule(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAutomationRuleRequest>,
) -> Result<(StatusCode, Json<AutomationRule>), AppError> {
    let conn = state.db.get_connection();
    
    let new_automation_rule = AutomationRuleActiveModel {
        action: sea_orm::Set(payload.action),
        level: sea_orm::Set(payload.level),
        trigger_time_range: sea_orm::Set(payload.trigger_time_range),
        sync_alarm: sea_orm::Set(payload.sync_alarm),
        ..Default::default()
    };

    let automation_rule = AutomationRuleEntity::insert(new_automation_rule)
        .exec_with_returning(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok((StatusCode::CREATED, Json(automation_rule)))
}

/// 更新自动化规则
#[utoipa::path(
    put,
    path = "/automation-rules/{id}",
    params(
        ("id" = i32, Path, description = "自动化规则ID")
    ),
    request_body = UpdateAutomationRuleRequest,
    responses(
        (status = 200, description = "更新自动化规则成功", body = AutomationRule),
        (status = 404, description = "自动化规则未找到")
    ),
    tag = "Automation Rules"
)]
pub async fn update_automation_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateAutomationRuleRequest>,
) -> Result<Json<AutomationRule>, AppError> {
    let conn = state.db.get_connection();
    
    let existing_automation_rule = AutomationRuleEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;
        
    let mut automation_rule_active_model = existing_automation_rule.into_active_model();
    
    if let Some(action) = payload.action {
        automation_rule_active_model.action = sea_orm::Set(action);
    }
    
    if let Some(level) = payload.level {
        automation_rule_active_model.level = sea_orm::Set(level);
    }
    
    if let Some(trigger_time_range) = payload.trigger_time_range {
        automation_rule_active_model.trigger_time_range = sea_orm::Set(trigger_time_range);
    }
    
    if let Some(sync_alarm) = payload.sync_alarm {
        automation_rule_active_model.sync_alarm = sea_orm::Set(sync_alarm);
    }
    
    // 更新 updated_at 字段
    automation_rule_active_model.updated_at = sea_orm::Set(chrono::Utc::now());
    
    let updated_automation_rule = AutomationRuleEntity::update(automation_rule_active_model)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(Json(updated_automation_rule))
}

/// 删除自动化规则
#[utoipa::path(
    delete,
    path = "/automation-rules/{id}",
    params(
        ("id" = i32, Path, description = "自动化规则ID")
    ),
    responses(
        (status = 204, description = "删除自动化规则成功"),
        (status = 404, description = "自动化规则未找到")
    ),
    tag = "Automation Rules"
)]
pub async fn delete_automation_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let conn = state.db.get_connection();
    
    let automation_rule = AutomationRuleEntity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or_else(|| AppError::NotFound)?;

    let _ = AutomationRuleEntity::delete_by_id(automation_rule.id)
        .exec(conn)
        .await
        .map_err(|_| AppError::InternalError)?;

    Ok(StatusCode::NO_CONTENT)
}