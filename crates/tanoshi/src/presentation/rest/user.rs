use axum::{extract::Extension, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    config::GLOBAL_CONFIG,
    domain::{
        repositories::user::UserRepositories,
        services::user::{UserError, UserService},
    },
    infrastructure::auth::Claims,
};

use super::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

impl From<UserError> for ErrorResponse {
    fn from(e: UserError) -> Self {
        let (status, message) = match e {
            UserError::UserNotFound => (StatusCode::NOT_FOUND, "user not found".to_string()),
            UserError::WrongPassword => (StatusCode::UNAUTHORIZED, "wrong password".to_string()),
            UserError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            UserError::InsufficientPasswordLength => (
                StatusCode::BAD_REQUEST,
                "password length insufficient".to_string(),
            ),
            UserError::Other(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        Self {
            http_code: status.as_u16(),
            message,
        }
    }
}

pub async fn login<R>(
    Json(req): Json<LoginRequest>,
    Extension(svc): Extension<UserService<R>>,
) -> Result<Json<LoginResponse>, ErrorResponse>
where
    R: UserRepositories,
{
    let user = svc.login(&req.username, &req.password).await?;

    let secret = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: format!("config not set"),
        })?
        .secret
        .to_owned();

    let claim = Claims {
        sub: user.id,
        username: user.username,
        is_admin: user.is_admin,
        exp: (chrono::Utc::now().timestamp() + 2678400) as _, // 31 days
    };

    let token = claim.into_token(&secret).map_err(|e| ErrorResponse {
        http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        message: format!("{e}"),
    })?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

pub async fn create_user<R>(
    claim: Claims,
    Json(req): Json<CreateUserRequest>,
    Extension(svc): Extension<UserService<R>>,
) -> Result<StatusCode, ErrorResponse>
where
    R: UserRepositories,
{
    if !claim.is_admin {
        return Err(ErrorResponse {
            http_code: StatusCode::FORBIDDEN.as_u16(),
            message: "only admin can create user".to_string(),
        });
    }

    svc.create_user(&req.username, &req.password, req.is_admin)
        .await?;

    Ok(StatusCode::CREATED)
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

pub async fn register<R>(
    Json(req): Json<RegisterRequest>,
    Extension(svc): Extension<UserService<R>>,
) -> Result<StatusCode, ErrorResponse>
where
    R: UserRepositories,
{
    svc.register(&req.username, &req.password, req.is_admin)
        .await?;

    Ok(StatusCode::CREATED)
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password<R>(
    claim: Claims,
    Json(req): Json<ChangePasswordRequest>,
    Extension(svc): Extension<UserService<R>>,
) -> Result<StatusCode, ErrorResponse>
where
    R: UserRepositories,
{
    svc.change_password(claim.sub, &req.old_password, &req.new_password)
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct UpdateTelegramChatIdRequest {
    pub chat_id: Option<i64>,
}

pub async fn update_telegram_chat_id<R>(
    claim: Claims,
    Json(req): Json<UpdateTelegramChatIdRequest>,
    Extension(svc): Extension<UserService<R>>,
) -> Result<StatusCode, ErrorResponse>
where
    R: UserRepositories,
{
    svc.update_telegram_chat_id(claim.sub, req.chat_id).await?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct UpdatePushoverUserKeyRequest {
    pub user_key: Option<String>,
}

pub async fn update_pushover_user_key<R>(
    claim: Claims,
    Json(req): Json<UpdatePushoverUserKeyRequest>,
    Extension(svc): Extension<UserService<R>>,
) -> Result<StatusCode, ErrorResponse>
where
    R: UserRepositories,
{
    svc.update_pushover_user_key(claim.sub, req.user_key)
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub telegram_chat_id: Option<i64>,
    pub pushover_user_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FetchUsersResponse {
    pub users: Vec<User>,
}

pub async fn fetch_users<R>(
    claim: Claims,
    Extension(svc): Extension<UserService<R>>,
) -> Result<Json<FetchUsersResponse>, ErrorResponse>
where
    R: UserRepositories,
{
    if !claim.is_admin {
        return Err(ErrorResponse {
            http_code: StatusCode::FORBIDDEN.as_u16(),
            message: "forbidden".to_string(),
        });
    }

    let users = svc
        .fetch_all_users()
        .await?
        .into_iter()
        .map(|user| User {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
            telegram_chat_id: user.telegram_chat_id,
            pushover_user_key: user.pushover_user_key,
        })
        .collect();

    Ok(Json(FetchUsersResponse { users }))
}

#[derive(Debug, Serialize)]
pub struct FetchUserResponse {
    pub user: User,
}

pub async fn fetch_user<R>(
    claim: Claims,
    Extension(svc): Extension<UserService<R>>,
) -> Result<Json<FetchUserResponse>, ErrorResponse>
where
    R: UserRepositories,
{
    let user = svc.fetch_user(claim.sub).await?;

    Ok(Json(FetchUserResponse {
        user: User {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
            telegram_chat_id: user.telegram_chat_id,
            pushover_user_key: user.pushover_user_key,
        },
    }))
}
