use crate::domain::entities::{chapter::Chapter, manga::Manga};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MangaRepositoryError {
    #[error("database return error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait MangaRepositories {
    async fn get_manga_by_id(&self, id: i64) -> Result<Manga, MangaRepositoryError>;
    async fn get_manga_by_ids(&self, ids: &[i64]) -> Result<Vec<Manga>, MangaRepositoryError>;
    async fn get_manga_by_source_path(
        &self,
        source_id: i64,
        path: &str,
    ) -> Result<Manga, MangaRepositoryError>;
    async fn insert_manga(&self, manga: &Manga) -> Result<i64, MangaRepositoryError>;
    async fn get_chapters_by_manga_id(
        &self,
        manga_id: i64,
    ) -> Result<Vec<Chapter>, MangaRepositoryError>;
}
