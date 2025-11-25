use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "alarm_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub rule_name: String,      // 规则名称
    pub trigger_time: DateTime<Utc>, // 触发时间
    pub trigger_value: f64,      // 触发值
    pub is_processed: bool,      // 是否处理
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}