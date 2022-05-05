#[derive(Debug, Default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
    pub telegram_chat_id: Option<i64>,
    pub pushover_user_key: Option<String>,
}
