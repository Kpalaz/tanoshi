use chrono::NaiveDateTime;

pub struct Chapter {
    pub id: i64,
    pub source_id: i64,
    pub manga_id: i64,
    pub title: String,
    pub path: String,
    pub number: f64,
    pub scanlator: String,
    pub uploaded: NaiveDateTime,
}

impl From<tanoshi_lib::models::ChapterInfo> for Chapter {
    fn from(ch: tanoshi_lib::models::ChapterInfo) -> Self {
        Self {
            id: 0,
            source_id: ch.source_id,
            manga_id: 0,
            title: ch.title,
            path: ch.path,
            number: ch.number,
            scanlator: ch.scanlator.unwrap_or_default(),
            uploaded: chrono::NaiveDateTime::from_timestamp(ch.uploaded, 0),
        }
    }
}
