mod model;

use crate::internal::lyrics::model::{SearchResponse, SearchResponseBody};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use scraper::{Html, Selector};
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

const BASE_URL: &str = "https://api.genius.com";

pub struct Lyra {
    client: Client,
}

impl Lyra {
    pub fn new() -> Self {
        let token = dotenv::var("GENIUS_TOKEN").expect("API was not found");
        let token = HeaderValue::from_str(&format!("Bearer {token}"))
            .expect("Failed to create auth header");

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, token);

        let client = reqwest::ClientBuilder::default()
            .default_headers(headers)
            .build()
            .expect("Failed to create client");

        Self { client }
    }

    async fn search(&self, name: &str) -> Result<SearchResponseBody, Error> {
        let res = self
            .client
            .get(&format!("{BASE_URL}/search?q={name}"))
            .send()
            .await?
            .json::<SearchResponse>()
            .await?;

        Ok(res.response)
    }

    pub async fn get_song_lyrics(&self, name: &str) -> Result<String, Error> {
        let search = self.search(name).await?;
        let Some(track) = search.hits.first() else {
            return Err(Error::NotFound(name.to_string()));
        };

        let html = self
            .client
            .get(&track.result.url)
            .send()
            .await?
            .text()
            .await?;

        let html = Html::parse_document(&html);
        let selector = Selector::parse("div[data-lyrics-container='true']").unwrap();

        let lyrics = html
            .select(&selector)
            .flat_map(|element| element.text())
            .filter(|line| !line.trim_start().starts_with('['))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(lyrics)
    }
}
