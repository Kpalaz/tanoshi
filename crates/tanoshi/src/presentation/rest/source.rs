use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use http::StatusCode;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    config::GLOBAL_CONFIG,
    domain::{
        entities::source::Filters,
        repositories::source::SourceRepositories,
        services::source::{SourceError, SourceService},
    },
    infrastructure::{auth::Claims, encrypt::encrypt_url},
};

use super::ErrorResponse;

impl From<SourceError> for ErrorResponse {
    fn from(e: SourceError) -> Self {
        let (status_code, message) = match e {
            SourceError::ExtensionError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("extension error: {e}"),
            ),
        };

        Self {
            http_code: status_code.as_u16(),
            message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub version: String,
    pub rustc_version: String,
    pub lib_version: String,
    pub icon: String,
    pub has_update: bool,
}

#[derive(Debug, Serialize)]
pub struct FetchInstalledSourcesResponse {
    pub sources: Vec<Source>,
}

pub async fn fetch_installed_sources<R>(
    _claim: Claims,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<Json<FetchInstalledSourcesResponse>, ErrorResponse>
where
    R: SourceRepositories,
{
    let sources = svc
        .get_installed_sources()
        .await?
        .into_par_iter()
        .map(|s| Source {
            id: s.id,
            name: s.name,
            url: s.url,
            version: s.version,
            rustc_version: s.rustc_version,
            lib_version: s.lib_version,
            icon: s.icon,
            has_update: s.has_update,
        })
        .collect();

    Ok(Json(FetchInstalledSourcesResponse { sources }))
}

#[derive(Debug, Serialize)]
pub struct FetchAvailableSourcesResponse {
    pub sources: Vec<Source>,
}

pub async fn fetch_available_sources<R>(
    _claim: Claims,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<Json<FetchAvailableSourcesResponse>, ErrorResponse>
where
    R: SourceRepositories,
{
    let repo_url = GLOBAL_CONFIG
        .get()
        .map(|cfg| cfg.extension_repository.to_owned())
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "no config set".to_string(),
        })?;

    let sources = svc
        .get_available_sources(&repo_url)
        .await?
        .into_par_iter()
        .map(|s| Source {
            id: s.id,
            name: s.name,
            url: s.url,
            version: s.version,
            rustc_version: s.rustc_version,
            lib_version: s.lib_version,
            icon: s.icon,
            has_update: s.has_update,
        })
        .collect();

    Ok(Json(FetchAvailableSourcesResponse { sources }))
}

#[derive(Debug, Serialize)]
pub struct FetchSourceResponse {
    pub source: Source,
}

pub async fn fetch_source<R>(
    _claim: Claims,
    Path(id): Path<i64>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<Json<FetchSourceResponse>, ErrorResponse>
where
    R: SourceRepositories,
{
    let s = svc.get_source_by_id(id).await?;

    Ok(Json(FetchSourceResponse {
        source: Source {
            id: s.id,
            name: s.name,
            url: s.url,
            version: s.version,
            rustc_version: s.rustc_version,
            lib_version: s.lib_version,
            icon: s.icon,
            has_update: s.has_update,
        },
    }))
}

pub async fn install_source<R>(
    _claim: Claims,
    Path(id): Path<i64>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<(), ErrorResponse>
where
    R: SourceRepositories,
{
    let repo_url = GLOBAL_CONFIG
        .get()
        .map(|cfg| cfg.extension_repository.to_owned())
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "no config set".to_string(),
        })?;

    svc.install_source(&repo_url, id).await?;

    Ok(())
}

pub async fn update_source<R>(
    _claim: Claims,
    Path(id): Path<i64>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<(), ErrorResponse>
where
    R: SourceRepositories,
{
    let repo_url = GLOBAL_CONFIG
        .get()
        .map(|cfg| cfg.extension_repository.to_owned())
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "no config set".to_string(),
        })?;

    svc.update_source(&repo_url, id).await?;

    Ok(())
}

pub async fn uninstall_source<R>(
    _claim: Claims,
    Path(id): Path<i64>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<(), ErrorResponse>
where
    R: SourceRepositories,
{
    svc.uninstall_source(id).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct FetchPopularMangaParams {
    page: i64,
}

#[derive(Debug, Serialize)]
pub struct Manga {
    pub source_id: i64,
    pub title: String,
    pub path: String,
    pub cover_url: String,
}

#[derive(Debug, Serialize)]
pub struct FetchPopularMangaResponse {
    pub manga: Vec<Manga>,
}

pub async fn fetch_popular_manga<R>(
    _claim: Claims,
    Path(source_id): Path<i64>,
    Query(params): Query<FetchPopularMangaParams>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<Json<FetchPopularMangaResponse>, ErrorResponse>
where
    R: SourceRepositories,
{
    let secret = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: format!("config not set"),
        })?
        .secret
        .to_owned();

    let manga = svc
        .get_popular_manga(source_id, params.page)
        .await?
        .into_par_iter()
        .map(|m| Manga {
            source_id: m.source_id,
            title: m.title,
            path: m.path,
            cover_url: encrypt_url(&secret, &m.cover_url).unwrap(),
        })
        .collect();

    Ok(Json(FetchPopularMangaResponse { manga }))
}

#[derive(Debug, Deserialize)]
pub struct FetchLatestMangaParams {
    page: i64,
}

#[derive(Debug, Serialize)]
pub struct FetchLatestMangaResponse {
    pub manga: Vec<Manga>,
}

pub async fn fetch_latest_manga<R>(
    _claim: Claims,
    Path(source_id): Path<i64>,
    Query(params): Query<FetchLatestMangaParams>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<Json<FetchLatestMangaResponse>, ErrorResponse>
where
    R: SourceRepositories,
{
    let secret = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: format!("config not set"),
        })?
        .secret
        .to_owned();

    let manga = svc
        .get_latest_manga(source_id, params.page)
        .await?
        .into_par_iter()
        .map(|m| Manga {
            source_id: m.source_id,
            title: m.title,
            path: m.path,
            cover_url: encrypt_url(&secret, &m.cover_url).unwrap(),
        })
        .collect();

    Ok(Json(FetchLatestMangaResponse { manga }))
}

#[derive(Debug, Deserialize)]
pub struct SearchMangaRequest {
    pub query: Option<String>,
    pub filters: Option<Filters>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMangaParams {
    pub page: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchMangaResponse {
    pub manga: Vec<Manga>,
}

pub async fn search_manga<R>(
    _claim: Claims,
    Path(source_id): Path<i64>,
    Query(params): Query<SearchMangaParams>,
    Json(req): Json<SearchMangaRequest>,
    Extension(svc): Extension<SourceService<R>>,
) -> Result<Json<FetchLatestMangaResponse>, ErrorResponse>
where
    R: SourceRepositories,
{
    let secret = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| ErrorResponse {
            http_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: format!("config not set"),
        })?
        .secret
        .to_owned();

    let manga = svc
        .search_manga(source_id, params.page, req.query, req.filters)
        .await?
        .into_par_iter()
        .map(|m| Manga {
            source_id: m.source_id,
            title: m.title,
            path: m.path,
            cover_url: encrypt_url(&secret, &m.cover_url).unwrap(),
        })
        .collect();

    Ok(Json(FetchLatestMangaResponse { manga }))
}
