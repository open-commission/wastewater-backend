use std::sync::{Arc, RwLock};
use crate::models::user::Model as User;
use crate::database::sea_orm_db::DbManager;

#[derive(Debug, Clone)]
pub struct AppState {
    pub users: Arc<RwLock<Vec<User>>>,
    pub db: DbManager,
}