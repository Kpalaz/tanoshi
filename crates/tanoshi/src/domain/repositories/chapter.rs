use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::chapter::Chapter;

#[derive(Debug, Error)]
pub enum ChapterRepositoryError {}

#[async_trait]
pub trait ChapterRepositories {
    async fn get_chapter_by_id(&self, id: i64) -> Result<Chapter, ChapterRepositoryError>;
    async fn get_downloaded_pages(
        &self,
        chapter_id: i64,
    ) -> Result<Vec<String>, ChapterRepositoryError>;
}
