use super::catalogue::{chapter::ReadProgress, Manga};
use crate::domain::repositories::{
    history::HistoryRepository, library::LibraryRepository, manga::MangaRepository,
    tracker::TrackerRepository,
};
use async_graphql::{dataloader::Loader, Result};
use chrono::NaiveDateTime;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    history_repo: H,
    library_repo: L,
    manga_repo: M,
    tracker_repo: T,
}

impl<H, L, M, T> DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    pub fn new(history_repo: H, library_repo: L, manga_repo: M, tracker_repo: T) -> Self {
        Self {
            history_repo,
            library_repo,
            manga_repo,
            tracker_repo,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserFavoriteId(pub i64, pub i64);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserFavoriteId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = bool;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserFavoriteId],
    ) -> Result<HashMap<UserFavoriteId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let manga_id_set: HashSet<i64> = keys.iter().map(|key| key.1).collect();

        let res = self
            .library_repo
            .get_manga_from_library(user_id)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|manga| {
                (
                    UserFavoriteId(user_id, manga.id),
                    manga_id_set.get(&manga.id).is_some(),
                )
            })
            .collect();

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserFavoritePath(pub i64, pub String);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserFavoritePath> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = bool;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserFavoritePath],
    ) -> Result<HashMap<UserFavoritePath, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let manga_path_set: HashSet<String> = keys.iter().map(|key| key.1.clone()).collect();

        let res = self
            .library_repo
            .get_manga_from_library(user_id)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|manga| {
                let is_library = manga_path_set.get(&manga.path).is_some();
                (UserFavoritePath(user_id, manga.path), is_library)
            })
            .collect();

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLastReadId(pub i64, pub i64);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserLastReadId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = NaiveDateTime;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserLastReadId],
    ) -> Result<HashMap<UserLastReadId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();

        let res = self
            .history_repo
            .get_history_chapters_by_manga_ids(user_id, &manga_ids)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|chapter| (UserLastReadId(user_id, chapter.manga_id), chapter.read_at))
            .collect();

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserUnreadChaptersId(pub i64, pub i64);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserUnreadChaptersId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = i64;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserUnreadChaptersId],
    ) -> Result<HashMap<UserUnreadChaptersId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();

        let res = self
            .history_repo
            .get_unread_chapters_by_manga_ids(user_id, &manga_ids)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|(manga_id, count)| (UserUnreadChaptersId(user_id, manga_id), count))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserHistoryId(pub i64, pub i64);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserHistoryId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = ReadProgress;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserHistoryId],
    ) -> Result<HashMap<UserHistoryId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let chapter_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();

        let res = self
            .history_repo
            .get_history_chapters_by_chapter_ids(user_id, &chapter_ids)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|chapter| {
                (
                    UserHistoryId(user_id, chapter.chapter_id),
                    ReadProgress {
                        at: chapter.read_at,
                        last_page: chapter.last_page_read,
                        is_complete: chapter.is_complete,
                    },
                )
            })
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MangaId(pub i64);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<MangaId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = Manga;

    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[MangaId]) -> Result<HashMap<MangaId, Self::Value>, Self::Error> {
        let keys: Vec<i64> = keys.iter().map(|key| key.0).collect();
        let res = self
            .manga_repo
            .get_manga_by_ids(&keys)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|m| (MangaId(m.id), m.into()))
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserTrackerMangaId(pub i64, pub i64);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserTrackerMangaId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = Vec<(String, Option<String>)>;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserTrackerMangaId],
    ) -> Result<HashMap<UserTrackerMangaId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let manga_ids: Vec<i64> = keys.iter().map(|key| key.1).collect();

        let res = self
            .tracker_repo
            .get_tracked_manga_id_by_manga_ids(user_id, &manga_ids)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .iter()
            .group_by(|m| UserTrackerMangaId(user_id, m.manga_id))
            .into_iter()
            .map(|(key, group)| {
                (
                    key,
                    (group
                        .map(|v| (v.tracker.clone(), v.tracker_manga_id.clone()))
                        .collect()),
                )
            })
            .collect();
        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserCategoryId(pub i64, pub Option<i64>);

#[async_trait::async_trait]
impl<H, L, M, T> Loader<UserCategoryId> for DatabaseLoader<H, L, M, T>
where
    H: HistoryRepository + 'static,
    L: LibraryRepository + 'static,
    M: MangaRepository + 'static,
    T: TrackerRepository + 'static,
{
    type Value = i64;

    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[UserCategoryId],
    ) -> Result<HashMap<UserCategoryId, Self::Value>, Self::Error> {
        let user_id = keys
            .iter()
            .next()
            .map(|key| key.0)
            .ok_or_else(|| anyhow::anyhow!("no user id"))?;

        let res = self
            .library_repo
            .get_category_count(user_id)
            .await
            .map_err(|e| Arc::new(anyhow::anyhow!("{e}")))?
            .into_par_iter()
            .map(|(category_id, count)| (UserCategoryId(user_id, category_id), count))
            .collect();
        Ok(res)
    }
}