use sea_orm::{
    Database, DatabaseConnection, DbErr
};
use std::fmt::Debug;
use std::result::Result as StdResult;
use std::sync::Arc;

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
#[derive(Debug, Clone)]
pub struct DbManager {
    db: Arc<DatabaseConnection>,
}

impl DbManager {
    /// 创建或打开数据库连接
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Database::connect(uri).await?;
        Ok(DbManager { db: Arc::new(db) })
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.db
    }
}