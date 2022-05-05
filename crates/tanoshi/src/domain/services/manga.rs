use crate::domain::{
    entities::manga::Manga,
    repositories::{
        manga::{MangaRepositories, MangaRepositoryError},
        source::{SourceRepositories, SourceRepositoryError},
    },
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MangaError {
    #[error("repository return error: {0}")]
    RepositoryError(#[from] MangaRepositoryError),
    #[error("source repository return error: {0}")]
    SourceRepositoryError(#[from] SourceRepositoryError),
}

#[derive(Clone)]
pub struct MangaService<R, S>
where
    R: MangaRepositories,
    S: SourceRepositories,
{
    repo: R,
    source_repo: S,
}

impl<R, S> MangaService<R, S>
where
    R: MangaRepositories,
    S: SourceRepositories,
{
    pub fn new(repo: R, source_repo: S) -> Self {
        Self { repo, source_repo }
    }

    pub async fn get_manga_by_id(&self, id: i64, refresh: bool) -> Result<Manga, MangaError> {
        let mut manga = self.repo.get_manga_by_id(id).await?;
        if refresh {
            let mut m = self
                .source_repo
                .get_manga_by_source_path(manga.source_id, &manga.path)
                .await?;

            m.id = manga.id;

            self.repo.insert_manga(&m).await?;

            manga = self.repo.get_manga_by_id(id).await?;
        }

        Ok(manga)
    }

    pub async fn get_manga_by_source_path(
        &self,
        source_id: i64,
        path: &str,
        refresh: bool,
    ) -> Result<Manga, MangaError> {
        let mut res = self.repo.get_manga_by_source_path(source_id, path).await;
        if res.is_err() || refresh {
            let m = self
                .source_repo
                .get_manga_by_source_path(source_id, path)
                .await?;

            self.repo.insert_manga(&m).await?;

            res = self.repo.get_manga_by_source_path(source_id, path).await;
        }

        Ok(res?)
    }
}
