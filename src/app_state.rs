use std::sync::{Arc, RwLock};
use crate::models::user::User;

#[derive(Debug, Clone)]
pub struct AppState {
    pub users: Arc<RwLock<Vec<User>>>,
}