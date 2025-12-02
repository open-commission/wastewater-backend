use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "alarm_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,         // 报警规则名称
    pub condition: String,    // 条件
    pub parameter: String,    // 参数
    pub value: f64,           // 值
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}