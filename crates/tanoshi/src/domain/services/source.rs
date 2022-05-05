use crate::domain::{
    entities::{
        manga::Manga,
        source::{Filters, Source},
    },
    repositories::source::{SourceRepositories, SourceRepositoryError},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("extension return error: {0}")]
    ExtensionError(#[from] SourceRepositoryError),
}

#[derive(Clone)]
pub struct SourceService<R>
where
    R: SourceRepositories,
{
    repo: R,
}

impl<R> SourceService<R>
where
    R: SourceRepositories,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_installed_sources(&self) -> Result<Vec<Source>, SourceError> {
        let sources = self.repo.installed_sources().await?;

        Ok(sources)
    }

    pub async fn get_available_sources(&self, repo_url: &str) -> Result<Vec<Source>, SourceError> {
        let sources = self.repo.available_sources(repo_url).await?;

        Ok(sources)
    }

    pub async fn get_source_by_id(&self, id: i64) -> Result<Source, SourceError> {
        let source = self.repo.get_source_by_id(id).await?;

        Ok(source)
    }

    pub async fn install_source(&self, repo_url: &str, id: i64) -> Result<(), SourceError> {
        self.repo.install_source(repo_url, id).await?;

        Ok(())
    }

    pub async fn update_source(&self, repo_url: &str, id: i64) -> Result<(), SourceError> {
        self.repo.update_source(repo_url, id).await?;

        Ok(())
    }

    pub async fn uninstall_source(&self, id: i64) -> Result<(), SourceError> {
        self.repo.uninstall_source(id).await?;

        Ok(())
    }

    pub async fn get_popular_manga(
        &self,
        source_id: i64,
        page: i64,
    ) -> Result<Vec<Manga>, SourceError> {
        let fetched_manga = self.repo.get_popular_manga(source_id, page).await?;

        Ok(fetched_manga)
    }

    pub async fn get_latest_manga(
        &self,
        source_id: i64,
        page: i64,
    ) -> Result<Vec<Manga>, SourceError> {
        let fetched_manga = self.repo.get_latest_manga(source_id, page).await?;

        Ok(fetched_manga)
    }

    pub async fn search_manga(
        &self,
        source_id: i64,
        page: i64,
        query: Option<String>,
        filters: Option<Filters>,
    ) -> Result<Vec<Manga>, SourceError> {
        let fetched_manga = self
            .repo
            .search_manga(source_id, page, query, filters)
            .await?;

        Ok(fetched_manga)
    }
}
