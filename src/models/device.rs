use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub location: String,
    pub status: i32,
    pub device_type: String,        // 设备类型
    pub manufacturer: String,       // 制造商
    pub model: String,              // 型号
    pub installation_date: DateTime<Utc>, // 安装日期
    pub last_maintenance: DateTime<Utc>,  // 上次维护时间
    pub operational_hours: f64,     // 运行小时数
    pub temperature: f64,           // 当前温度
    pub pressure: f64,              // 当前压力
    pub flow_rate: f64,             // 流量
    pub power_consumption: f64,     // 功耗
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}