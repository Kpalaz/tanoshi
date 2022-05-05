use crate::{config::GLOBAL_CONFIG, infrastructure::auth::Claims};
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts, TypedHeader},
};
use headers::{authorization::Bearer, Authorization};
use http::StatusCode;

use super::ErrorResponse;

#[async_trait]
impl<B> FromRequest<B> for Claims
where
    B: Send,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| ErrorResponse {
                    http_code: StatusCode::UNAUTHORIZED.as_u16(),
                    message: "unauthorized".to_string(),
                })?;

        let secret = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| ErrorResponse {
                http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: "config not set".to_string(),
            })?
            .secret
            .to_owned();

        let claim = Claims::from_token(&secret, bearer.token()).map_err(|_| ErrorResponse {
            http_code: StatusCode::UNAUTHORIZED.as_u16(),
            message: "unauthorized".to_string(),
        })?;

        Ok(claim)
    }
}
