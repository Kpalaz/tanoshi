use rand::RngCore;

use crate::domain::{
    entities::user::User,
    repositories::user::{UserRepositories, UserRepositoryError},
};

pub enum UserError {
    UserNotFound,
    WrongPassword,
    Forbidden,
    InsufficientPasswordLength,
    Other(String),
}

impl From<UserRepositoryError> for UserError {
    fn from(e: UserRepositoryError) -> Self {
        match e {
            UserRepositoryError::NotFound => Self::Other(format!("db error: row not found")),
            UserRepositoryError::DbError(e) => Self::Other(format!("db error: {e}")),
        }
    }
}

#[derive(Clone)]
pub struct UserService<R>
where
    R: UserRepositories,
{
    repo: R,
}

impl<R> UserService<R>
where
    R: UserRepositories,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<User, UserError> {
        let user = self.repo.get_user_by_username(&username).await?;

        if !argon2::verify_encoded(&user.password, password.as_bytes())
            .map_err(|e| UserError::Other(format!("{e}")))?
        {
            return Err(UserError::WrongPassword);
        }

        Ok(user)
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> Result<(), UserError> {
        if password.len() < 8 {
            return Err(UserError::InsufficientPasswordLength);
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let hash = {
            let config = argon2::Config::default();
            argon2::hash_encoded(password.as_bytes(), &salt, &config)
                .map_err(|e| UserError::Other(format!("{e}")))?
        };

        let user = User {
            username: username.to_string(),
            password: hash,
            is_admin,
            ..Default::default()
        };

        self.repo.create_user(&user).await?;

        Ok(())
    }

    pub async fn register(
        &self,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> Result<(), UserError> {
        let user_count = self.repo.get_all_users().await?.len();
        if user_count > 0 {
            // only admin can create user
            return Err(UserError::Forbidden);
        }

        if password.len() < 8 {
            return Err(UserError::InsufficientPasswordLength);
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let hash = {
            let config = argon2::Config::default();
            argon2::hash_encoded(password.as_bytes(), &salt, &config)
                .map_err(|e| UserError::Other(format!("{e}")))?
        };

        // If first user, make it admin else make it reader by default
        let is_admin = if user_count == 0 { true } else { is_admin };

        let user = User {
            username: username.to_string(),
            password: hash,
            is_admin,
            ..Default::default()
        };

        self.repo.create_user(&user).await?;

        Ok(())
    }

    pub async fn change_password(
        &self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), UserError> {
        let user = self.repo.get_user_by_id(user_id).await?;

        if !argon2::verify_encoded(&user.password, old_password.as_bytes())
            .map_err(|e| UserError::Other(format!("{e}")))?
        {
            return Err(UserError::Other("Wrong old password".to_string()));
        }

        if new_password.len() < 8 {
            return Err(UserError::InsufficientPasswordLength);
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let hash = {
            let config = argon2::Config::default();
            argon2::hash_encoded(new_password.as_bytes(), &salt, &config)
                .map_err(|e| UserError::Other(format!("{e}")))?
        };

        self.repo.update_password(user.id, &hash).await?;

        Ok(())
    }

    pub async fn update_telegram_chat_id(
        &self,
        user_id: i64,
        chat_id: Option<i64>,
    ) -> Result<(), UserError> {
        self.repo.update_user_telegram(user_id, chat_id).await?;

        Ok(())
    }

    pub async fn update_pushover_user_key(
        &self,
        user_id: i64,
        key: Option<String>,
    ) -> Result<(), UserError> {
        self.repo.update_user_pushover(user_id, key).await?;

        Ok(())
    }

    pub async fn fetch_all_users(&self) -> Result<Vec<User>, UserError> {
        let users = self.repo.get_all_users().await?;

        Ok(users)
    }

    pub async fn fetch_user(&self, user_id: i64) -> Result<User, UserError> {
        let user = self.repo.get_user_by_id(user_id).await?;

        Ok(user)
    }
}
