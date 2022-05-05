use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::user::User;

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("query return nothing")]
    NotFound,
    #[error("database return error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait UserRepositories {
    async fn create_user(&self, user: &User) -> Result<i64, UserRepositoryError>;
    async fn update_password(&self, id: i64, password: &str) -> Result<(), UserRepositoryError>;
    async fn update_user_role(&self, id: i64, is_admin: bool) -> Result<(), UserRepositoryError>;
    async fn get_all_users(&self) -> Result<Vec<User>, UserRepositoryError>;
    async fn get_admins(&self) -> Result<Vec<User>, UserRepositoryError>;
    async fn get_user_by_id(&self, id: i64) -> Result<User, UserRepositoryError>;
    async fn get_user_by_username(&self, username: &str) -> Result<User, UserRepositoryError>;
    async fn update_user_telegram(
        &self,
        id: i64,
        chat_id: Option<i64>,
    ) -> Result<(), UserRepositoryError>;
    async fn update_user_pushover(
        &self,
        id: i64,
        pushover_key: Option<String>,
    ) -> Result<(), UserRepositoryError>;
}
