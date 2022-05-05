use chrono::NaiveDateTime;

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
    pub date_added: NaiveDateTime,
}

impl From<tanoshi_lib::models::MangaInfo> for Manga {
    fn from(m: tanoshi_lib::models::MangaInfo) -> Self {
        Self {
            id: 0,
            source_id: m.source_id,
            title: m.title,
            author: m.author,
            genre: m.genre,
            status: m.status,
            description: m.description,
            path: m.path,
            cover_url: m.cover_url,
            date_added: chrono::NaiveDateTime::from_timestamp(0, 0),
        }
    }
}
