#[cfg(feature = "js")]
use rquickjs::{FromJs, IntoJs};
use serde::{Deserialize, Serialize};

/// A type represent manga details, normalized across source
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "js", derive(FromJs, IntoJs))]
#[cfg_attr(feature = "js", quickjs(rename_all = "camelCase"))]
pub struct MangaInfo {
    pub source_id: i64,
    pub title: String,
    pub author: Vec<String>,
    pub genre: Vec<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub cover_url: String,
}