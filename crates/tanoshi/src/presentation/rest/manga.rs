use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::{
    repositories::{manga::MangaRepositories, source::SourceRepositories},
    services::manga::MangaService,
};

use super::ErrorResponse;

#[derive(Debug, Serialize)]
pub struct Manga {
    pub id: i64,
    pub source_id: i64,
    pub title: String,
    pub author: Vec<String>,
    pub genre: Vec<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub cover_url: String,
}

#[derive(Debug, Deserialize)]
pub struct FetchMangaByIdParams {
    pub refresh: bool,
}

#[derive(Debug, Serialize)]
pub struct FetchMangaByIdResponse {
    pub manga: Manga,
}

pub async fn fetch_manga_by_id<R, S>(
    Path(id): Path<i64>,
    Query(params): Query<FetchMangaByIdParams>,
    Extension(svc): Extension<MangaService<R, S>>,
) -> Result<Json<FetchMangaByIdResponse>, ErrorResponse>
where
    R: MangaRepositories,
    S: SourceRepositories,
{
    todo!()
}
