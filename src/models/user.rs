#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub permission: String,
}
