use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub permission: String,
}