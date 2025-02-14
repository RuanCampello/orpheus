use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub response: SearchResponseBody,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponseBody {
    pub hits: Vec<Hit>,
}

#[derive(Debug, Deserialize)]
pub struct Hit {
    #[serde(rename = "type")]
    pub _type: String,
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub url: String,
}
