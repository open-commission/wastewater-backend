use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "automation_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub action: String,              // 行为
    pub level: i32,                  // 等级
    pub trigger_time_range: String,  // 触发时间端
    pub sync_alarm: bool,            // 是否同步报警
    pub created_at: DateTime<Utc>,   // 创建时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}