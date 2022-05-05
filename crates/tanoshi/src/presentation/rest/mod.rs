pub mod auth;
pub mod health;
pub mod image;
pub mod manga;
pub mod source;
pub mod user;

use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router, Server,
};
use http::StatusCode;
use serde::Serialize;
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use tanoshi_vm::prelude::SourceBus;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    domain::services::{image::ImageService, source::SourceService, user::UserService},
    infrastructure::{
        database::Pool,
        repositories::{image::ImageRepository, source::SourceRepository, user::UserRepository},
    },
};

#[derive(Serialize)]
pub struct ErrorResponse {
    pub http_code: u16,
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::from_u16(self.http_code).unwrap(), Json(self)).into_response()
    }
}

pub fn build_router(db: Pool, ext: SourceBus) -> Router<axum::body::Body> {
    let user_repository = UserRepository::new(db.clone());
    let user_service = UserService::new(user_repository);

    let image_repository = ImageRepository::new();
    let image_service = ImageService::new(image_repository);

    let source_repository = SourceRepository::new(ext);
    let source_service = SourceService::new(source_repository);

    let user_router = Router::new()
        .route(
            "/",
            get(user::fetch_user::<UserRepository>).post(user::create_user::<UserRepository>),
        )
        .route("/password", put(user::change_password::<UserRepository>))
        .route(
            "/telegram",
            put(user::update_telegram_chat_id::<UserRepository>),
        )
        .route(
            "/pushover",
            put(user::update_pushover_user_key::<UserRepository>),
        )
        .route("/all", get(user::fetch_users::<UserRepository>))
        .route("/register", post(user::register::<UserRepository>))
        .route("/login", post(user::login::<UserRepository>));

    let image_router = Router::new().route("/:url", get(image::fetch_image::<ImageRepository>));

    let source_router = Router::new()
        .route(
            "/installed",
            get(source::fetch_installed_sources::<SourceRepository>),
        )
        .route(
            "/available",
            get(source::fetch_available_sources::<SourceRepository>),
        )
        .route("/:source_id", get(source::fetch_source::<SourceRepository>))
        .route(
            "/:source_id/install",
            post(source::install_source::<SourceRepository>),
        )
        .route(
            "/:source_id/update",
            put(source::update_source::<SourceRepository>),
        )
        .route(
            "/:source_id/uninstall",
            delete(source::uninstall_source::<SourceRepository>),
        )
        .route(
            "/:source_id/popular",
            get(source::fetch_popular_manga::<SourceRepository>),
        )
        .route(
            "/:source_id/latest",
            get(source::fetch_latest_manga::<SourceRepository>),
        )
        .route(
            "/:source_id/search",
            post(source::search_manga::<SourceRepository>),
        );

    let mut app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/user", user_router)
                .nest("/source", source_router),
        )
        .nest("/image", image_router)
        .route("/health", get(health::health_check))
        .layer(Extension(user_service))
        .layer(Extension(image_service))
        .layer(Extension(source_service))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    #[cfg(feature = "embed")]
    {
        app = app.fallback(get(crate::assets::static_handler));
    }

    app
}

pub async fn serve(
    addr: &str,
    port: u16,
    router: Router<axum::body::Body>,
) -> Result<(), anyhow::Error> {
    let addr = SocketAddr::from((IpAddr::from_str(addr)?, port));
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
