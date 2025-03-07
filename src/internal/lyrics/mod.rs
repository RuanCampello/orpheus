mod model;

use crate::internal::lyrics::model::SearchResponse;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request error: {0}")]
    Client(#[from] reqwest::Error),
    #[error("Parsing error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Lyrics for song {0} was not found")]
    NotFound(String),
}

const BASE_URL: &str = "https://lrclib.net";

pub struct Lyra {
    client: Client,
}

impl Lyra {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str("orpheus").unwrap());

        let client = reqwest::ClientBuilder::default()
            .default_headers(headers)
            .build()
            .expect("Failed to create client");

        Self { client }
    }

    async fn search(&self, artist: &str, name: &str) -> Result<SearchResponse, Error> {
        let res = self
            .client
            .get(&format!(
                "{BASE_URL}/api/get?artist_name={artist}&track_name={name}"
            ))
            .send()
            .await?.json::<SearchResponse>().await?;

        Ok(res)
    }

    pub async fn get_song_lyrics(&self, artist: &str, name: &str) -> Result<String, Error> {
        let search = self
            .search(artist, name.split("-").next().unwrap_or(name))
            .await?;

        Ok(search.synced_lyrics.unwrap_or(search.plain_lyrics))
    }
}
