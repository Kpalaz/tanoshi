use async_trait::async_trait;

use crate::{
    domain::{
        entities::user::User,
        repositories::user::{UserRepositories, UserRepositoryError},
    },
    infrastructure::database::Pool,
};
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct UserRepository {
    db: Pool,
}

impl UserRepository {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepositories for UserRepository {
    async fn create_user(&self, user: &User) -> Result<i64, UserRepositoryError> {
        let row_id = sqlx::query(
            r#"INSERT INTO user(
                username,
                password,
                is_admin
            ) VALUES (?, ?, ?)"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.is_admin)
        .execute(&self.db as &SqlitePool)
        .await?
        .last_insert_rowid();

        Ok(row_id)
    }

    async fn update_password(&self, id: i64, password: &str) -> Result<(), UserRepositoryError> {
        sqlx::query(
            r#"UPDATE user
                SET password = ?
                WHERE id = ?"#,
        )
        .bind(&password)
        .bind(id)
        .execute(&self.db as &SqlitePool)
        .await?;

        Ok(())
    }

    async fn update_user_role(&self, id: i64, is_admin: bool) -> Result<(), UserRepositoryError> {
        sqlx::query(
            r#"UPDATE user
                SET is_admin = ?
                WHERE id = ?"#,
        )
        .bind(&is_admin)
        .bind(id)
        .execute(&self.db as &SqlitePool)
        .await?;

        Ok(())
    }

    async fn get_all_users(&self) -> Result<Vec<User>, UserRepositoryError> {
        let users = sqlx::query(
            r#"SELECT 
                id, 
                username, 
                password, 
                is_admin, 
                telegram_chat_id, 
                pushover_user_key 
                FROM user"#,
        )
        .fetch_all(&self.db as &SqlitePool)
        .await?
        .iter()
        .map(|row| User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            is_admin: row.get(3),
            telegram_chat_id: row.get(4),
            pushover_user_key: row.get(5),
        })
        .collect::<Vec<User>>();

        Ok(users)
    }

    async fn get_admins(&self) -> Result<Vec<User>, UserRepositoryError> {
        let users = sqlx::query(
            r#"SELECT 
                id, 
                username, 
                password, 
                is_admin, 
                telegram_chat_id, 
                pushover_user_key 
                FROM user
                WHERE is_admin = true"#,
        )
        .fetch_all(&self.db as &SqlitePool)
        .await?
        .iter()
        .map(|row| User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            is_admin: row.get(3),
            telegram_chat_id: row.get(4),
            pushover_user_key: row.get(5),
        })
        .collect::<Vec<User>>();

        Ok(users)
    }

    async fn get_user_by_id(&self, id: i64) -> Result<User, UserRepositoryError> {
        let user = sqlx::query(
            r#"SELECT 
                id, 
                username, 
                password, 
                is_admin, 
                telegram_chat_id, 
                pushover_user_key 
                FROM user
                WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.db as &SqlitePool)
        .await?
        .map(|row| User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            is_admin: row.get(3),
            telegram_chat_id: row.get(4),
            pushover_user_key: row.get(5),
        })
        .ok_or_else(|| UserRepositoryError::NotFound)?;

        Ok(user)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, UserRepositoryError> {
        let user = sqlx::query(
            r#"SELECT 
                id, 
                username, 
                password, 
                is_admin, 
                telegram_chat_id, 
                pushover_user_key 
                FROM user
                WHERE username = ?"#,
        )
        .bind(username)
        .fetch_one(&self.db as &SqlitePool)
        .await
        .map(|row| User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            is_admin: row.get(3),
            telegram_chat_id: row.get(4),
            pushover_user_key: row.get(5),
        })?;

        Ok(user)
    }

    async fn update_user_telegram(
        &self,
        id: i64,
        chat_id: Option<i64>,
    ) -> Result<(), UserRepositoryError> {
        sqlx::query(
            r#"UPDATE user
                SET telegram_chat_id = ?
                WHERE id = ?"#,
        )
        .bind(chat_id)
        .bind(id)
        .execute(&self.db as &SqlitePool)
        .await?;

        Ok(())
    }

    async fn update_user_pushover(
        &self,
        id: i64,
        pushover_key: Option<String>,
    ) -> Result<(), UserRepositoryError> {
        sqlx::query(
            r#"UPDATE user
                SET pushover_key = ?
                WHERE id = ?"#,
        )
        .bind(pushover_key)
        .bind(id)
        .execute(&self.db as &SqlitePool)
        .await?;

        Ok(())
    }
}
