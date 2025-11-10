    use bincode::{config, Decode, Encode};
use redb::{
    CommitError, Database, DatabaseError, ReadableDatabase, ReadableTable, StorageError,
    TableDefinition, TableError, TableStats, TransactionError,
};
use std::fmt::Debug;
use std::path::Path;
use std::result::Result as StdResult;

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    #[error("Commit error: {0}")]
    Commit(#[from] CommitError),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Table error: {0}")]
    Table(#[from] TableError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

pub type Result<T> = StdResult<T, DbError>;

/// 泛型数据库工具类
pub struct DbManager {
    db: Database,
}

impl DbManager {
    /// 创建或打开数据库
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;
        Ok(DbManager { db })
    }

    /// 插入或更新键值对
    pub fn put<V>(&self, table_name: &str, key: &str, value: &V) -> Result<()>
    where
        V: Encode + Debug,
    {
        let table = TableDefinition::<String, Vec<u8>>::new(table_name);

        let serialized_data = bincode::encode_to_vec(value, config::standard())
            .map_err(|e| DbError::Serialization(format!("Serialization error: {}", e)))?;

        let write_txn = self.db.begin_write()?;
        {
            let mut t = write_txn.open_table(table)?;
            t.insert(key.to_string(), serialized_data)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 获取指定键的值
    pub fn get<V>(&self, table_name: &str, key: &str) -> Result<Option<V>>
    where
        V: Decode<()> + Debug,
    {
        let table = TableDefinition::<String, Vec<u8>>::new(table_name);
        let read_txn = self.db.begin_read()?;
        let t = match read_txn.open_table(table) {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };

        match t.get(key.to_string())? {
            Some(value) => {
                let decoded =
                    bincode::decode_from_slice(value.value().as_slice(), config::standard())
                        .map_err(|e| {
                            DbError::Serialization(format!("Deserialization error: {}", e))
                        })?
                        .0;
                Ok(Some(decoded))
            }
            None => Ok(None),
        }
    }

    /// 删除指定键的值
    pub fn delete(&self, table_name: &str, key: &str) -> Result<bool> {
        let table = TableDefinition::<String, Vec<u8>>::new(table_name);
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut t = write_txn.open_table(table)?;
            let val = t.remove(key.to_string())?;
            val.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    /// 检查键是否存在
    pub fn exists(&self, table_name: &str, key: &str) -> Result<bool> {
        let table = TableDefinition::<String, Vec<u8>>::new(table_name);
        let read_txn = self.db.begin_read()?;
        let t = match read_txn.open_table(table) {
            Ok(t) => t,
            Err(_) => return Ok(false),
        };
        Ok(t.get(key.to_string())?.is_some())
    }

    /// 清空表中所有数据
    pub fn clear(&self, table_name: &str) -> Result<()> {
        let table = TableDefinition::<String, Vec<u8>>::new(table_name);
        let write_txn = self.db.begin_write()?;
        write_txn.delete_table(table)?;
        write_txn.commit()?;
        Ok(())
    }
}

pub fn test_redb_basic() -> Result<()> {
    let db = DbManager::new("../../test_redb.db")?;

    db.put("config", "app_name", &1280)?;
    db.put("config", "version", &1)?;

    let app_name: Option<i32> = db.get("config", "app_name")?;
    println!("App Name = {:?}", app_name);

    let exists = db.exists("config", "version")?;
    println!("Exists = {:?}", exists);

    let deleted = db.delete("config", "version")?;
    println!("Deleted = {:?}", deleted);

    let version: Option<i32> = db.get("config", "version")?;
    println!("Version = {:?}", version);

    db.clear("config")?;

    let exists = db.exists("config", "app_name")?;
    println!("Exists = {:?}", exists);

    Ok(())
}
