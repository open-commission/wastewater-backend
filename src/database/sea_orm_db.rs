use sea_orm::{
    Database, DatabaseConnection, DbErr
};
use std::fmt::Debug;
use std::result::Result as StdResult;

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

pub type Result<T> = StdResult<T, DbError>;

/// 泛型数据库工具类
pub struct DbManager {
    db: DatabaseConnection,
}

impl DbManager {
    /// 创建或打开数据库连接
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Database::connect(uri).await?;
        Ok(DbManager { db })
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Condition, Set};
    
    // 创建一个简单的测试实体
    mod test_entity {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_items")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub name: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }
}