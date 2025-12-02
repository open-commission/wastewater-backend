use crate::{handlers::{user, device, ph_value, tds_value, turbidity_value, flow_value, alarm_rule, alarm_log, automation_rule}, app_state::AppState};
use axum::{routing::get, Router};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        user::get_users,
        user::get_user,
        user::create_user,
        user::update_user,
        user::delete_user,
        device::get_devices,
        device::get_device,
        device::create_device,
        device::update_device,
        device::delete_device,
        ph_value::get_ph_values,
        ph_value::get_ph_value,
        ph_value::create_ph_value,
        ph_value::update_ph_value,
        ph_value::delete_ph_value,
        tds_value::get_tds_values,
        tds_value::get_tds_value,
        tds_value::create_tds_value,
        tds_value::update_tds_value,
        tds_value::delete_tds_value,
        turbidity_value::get_turbidity_values,
        turbidity_value::get_turbidity_value,
        turbidity_value::create_turbidity_value,
        turbidity_value::update_turbidity_value,
        turbidity_value::delete_turbidity_value,
        flow_value::get_flow_values,
        flow_value::get_flow_value,
        flow_value::create_flow_value,
        flow_value::update_flow_value,
        flow_value::delete_flow_value,
        alarm_rule::get_alarm_rules,
        alarm_rule::get_alarm_rule,
        alarm_rule::create_alarm_rule,
        alarm_rule::update_alarm_rule,
        alarm_rule::delete_alarm_rule,
        alarm_log::get_alarm_logs,
        alarm_log::get_alarm_log,
        alarm_log::create_alarm_log,
        alarm_log::update_alarm_log,
        alarm_log::delete_alarm_log,
        automation_rule::get_automation_rules,
        automation_rule::get_automation_rule,
        automation_rule::create_automation_rule,
        automation_rule::update_automation_rule,
        automation_rule::delete_automation_rule,
    ),
    components(
        schemas(
            crate::models::user::Model,
            crate::models::device::Model,
            crate::models::ph_value::Model,
            crate::models::tds_value::Model,
            crate::models::turbidity_value::Model,
            crate::models::flow_value::Model,
            crate::models::alarm_rule::Model,
            crate::models::alarm_log::Model,
            crate::models::automation_rule::Model,
            user::CreateUserRequest,
            user::UpdateUserRequest,
            device::CreateDeviceRequest,
            device::UpdateDeviceRequest,
            ph_value::CreatePhValueRequest,
            ph_value::UpdatePhValueRequest,
            tds_value::CreateTdsValueRequest,
            tds_value::UpdateTdsValueRequest,
            turbidity_value::CreateTurbidityValueRequest,
            turbidity_value::UpdateTurbidityValueRequest,
            flow_value::CreateFlowValueRequest,
            flow_value::UpdateFlowValueRequest,
            alarm_rule::CreateAlarmRuleRequest,
            alarm_rule::UpdateAlarmRuleRequest,
            alarm_log::CreateAlarmLogRequest,
            alarm_log::UpdateAlarmLogRequest,
            automation_rule::CreateAutomationRuleRequest,
            automation_rule::UpdateAutomationRuleRequest,
        )
    ),
    tags(
        (name = "Users", description = "用户管理接口"),
        (name = "Devices", description = "设备管理接口"),
        (name = "PH Values", description = "PH值数据接口"),
        (name = "TDS Values", description = "TDS值数据接口"),
        (name = "Turbidity Values", description = "浊度值数据接口"),
        (name = "Flow Values", description = "流量值数据接口"),
        (name = "Alarm Rules", description = "报警规则接口"),
        (name = "Alarm Logs", description = "报警日志接口"),
        (name = "Automation Rules", description = "自动化规则接口"),
    )
)]
struct ApiDoc;

pub fn create_api_router() -> Router<Arc<AppState>> {
    Router::new()
        // 用户管理路由
        .route("/users", get(user::get_users).post(user::create_user))
        .route(
            "/users/{id}",
            get(user::get_user)
                .put(user::update_user)
                .delete(user::delete_user),
        )
        // 设备管理路由
        .route("/devices", get(device::get_devices).post(device::create_device))
        .route(
            "/devices/{id}",
            get(device::get_device)
                .put(device::update_device)
                .delete(device::delete_device),
        )
        // PH值管理路由
        .route("/ph-values", get(ph_value::get_ph_values).post(ph_value::create_ph_value))
        .route(
            "/ph-values/{id}",
            get(ph_value::get_ph_value)
                .put(ph_value::update_ph_value)
                .delete(ph_value::delete_ph_value),
        )
        // TDS值管理路由
        .route("/tds-values", get(tds_value::get_tds_values).post(tds_value::create_tds_value))
        .route(
            "/tds-values/{id}",
            get(tds_value::get_tds_value)
                .put(tds_value::update_tds_value)
                .delete(tds_value::delete_tds_value),
        )
        // 浊度值管理路由
        .route("/turbidity-values", get(turbidity_value::get_turbidity_values).post(turbidity_value::create_turbidity_value))
        .route(
            "/turbidity-values/{id}",
            get(turbidity_value::get_turbidity_value)
                .put(turbidity_value::update_turbidity_value)
                .delete(turbidity_value::delete_turbidity_value),
        )
        // 流量值管理路由
        .route("/flow-values", get(flow_value::get_flow_values).post(flow_value::create_flow_value))
        .route(
            "/flow-values/{id}",
            get(flow_value::get_flow_value)
                .put(flow_value::update_flow_value)
                .delete(flow_value::delete_flow_value),
        )
        // 报警规则管理路由
        .route("/alarm-rules", get(alarm_rule::get_alarm_rules).post(alarm_rule::create_alarm_rule))
        .route(
            "/alarm-rules/{id}",
            get(alarm_rule::get_alarm_rule)
                .put(alarm_rule::update_alarm_rule)
                .delete(alarm_rule::delete_alarm_rule),
        )
        // 报警日志管理路由
        .route("/alarm-logs", get(alarm_log::get_alarm_logs).post(alarm_log::create_alarm_log))
        .route(
            "/alarm-logs/{id}",
            get(alarm_log::get_alarm_log)
                .put(alarm_log::update_alarm_log)
                .delete(alarm_log::delete_alarm_log),
        )
        // 自动化规则管理路由
        .route("/automation-rules", get(automation_rule::get_automation_rules).post(automation_rule::create_automation_rule))
        .route(
            "/automation-rules/{id}",
            get(automation_rule::get_automation_rule)
                .put(automation_rule::update_automation_rule)
                .delete(automation_rule::delete_automation_rule),
        )
        .merge(
            SwaggerUi::new("/swagger") // 用於 UI 的 endpoint
                .url("/api-doc/openapi.json", ApiDoc::openapi()) // 提供 openapi.json
        )
}