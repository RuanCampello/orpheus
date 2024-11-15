use crate::internal::config::Config;
use crate::internal::{get_spotify, get_token, Client};
use rspotify::oauth2::SpotifyOAuth;
use std::time::Duration;

mod internal;
mod terminal;
mod tui;

const SCOPES: [&str; 14] = [
    "playlist-read-collaborative",
    "playlist-read-private",
    "playlist-modify-private",
    "playlist-modify-public",
    "user-follow-read",
    "user-follow-modify",
    "user-library-modify",
    "user-library-read",
    "user-modify-playback-state",
    "user-read-currently-playing",
    "user-read-playback-state",
    "user-read-playback-position",
    "user-read-private",
    "user-read-recently-played",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_config = Config::new();

    let mut oauth = SpotifyOAuth::default()
        .client_id(&client_config.client_id)
        .client_secret(&client_config.client_secret)
        .redirect_uri(&client_config.get_redirect_uri())
        .scope(&SCOPES.join(" "))
        .build();

    let tick_rate = Duration::from_millis(500);

    match get_token(&mut oauth, client_config.get_port()).await {
        Some(token_info) => {
            println!("TOKEN {token_info:?}");

            let (spotify, exp) = get_spotify(token_info);
            let client = Client::new(spotify, oauth);

            terminal::run(tick_rate, client).await?;
        }
        None => println!("Failed to authorize Spotify"),
    };

    Ok(())
}
