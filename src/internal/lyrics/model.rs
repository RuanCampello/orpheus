use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    // pub id: u32,
    // pub track_name: String,
    // pub artist_name: String,
    // pub duration: u32,
    pub plain_lyrics: String,
    pub synced_lyrics: Option<String>,
}
