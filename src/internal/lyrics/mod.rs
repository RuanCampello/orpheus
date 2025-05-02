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
    #[error("Lyrics for this song was not found")]
    NotFound,
}

const BASE_URL: &str = "https://lrclib.net";

pub struct Lyra {
    client: Client,
}

type Lyrics = (String, bool);

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
            .await?
            .json::<SearchResponse>()
            .await?;

        Ok(res)
    }

    pub async fn get_song_lyrics(&self, artist: &str, name: &str) -> Result<Lyrics, Error> {
        let search = self
            .search(artist, name.split("-").next().unwrap_or(name))
            .await?;

        let has_lyrics = search
            .synced_lyrics
            .as_ref()
            .map_or(false, |s| !s.is_empty())
            || search.plain_lyrics.is_some();

        if !has_lyrics {
            return Err(Error::NotFound);
        }

        let is_synced = search.synced_lyrics.is_some();

        Ok(match is_synced {
            true => (search.synced_lyrics.unwrap(), is_synced),
            false => (search.plain_lyrics.unwrap(), is_synced),
        })
    }
}
