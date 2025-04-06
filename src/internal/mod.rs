pub mod config;
pub(super) mod image;
mod lyrics;
pub(super) mod text;

use crate::internal::lyrics::Lyra;
use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth, TokenInfo};
use rspotify::util::{process_token, request_token};
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, SystemTime};

pub(super) struct Client {
    pub spotify: Spotify,
    pub lyra: Lyra,
    oauth: SpotifyOAuth,
}

impl Client {
    pub fn new(spotify: Spotify, oauth: SpotifyOAuth) -> Self {
        Self {
            spotify,
            oauth,
            lyra: Lyra::new(),
        }
    }
}

pub fn debug<I: Debug>(filename: &str, items: &[I]) {
    let mut file = File::create(filename).unwrap();
    for item in items {
        file.write_all(format!("{item:#?}").as_bytes()).unwrap();
    }
}

pub async fn get_token(spotify_oauth: &mut SpotifyOAuth, port: u16) -> Option<TokenInfo> {
    match spotify_oauth.get_cached_token().await {
        Some(token) => Some(token),
        None => match redirect_to_authorize(spotify_oauth, port) {
            Ok(mut url) => process_token(spotify_oauth, &mut url).await,
            Err(()) => {
                println!("Starting webserver failed. Continuing with manual authentication");
                request_token(spotify_oauth);
                println!("Enter the URL you were redirected to: ");
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => process_token(spotify_oauth, &mut input).await,
                    Err(_) => None,
                }
            }
        },
    }
}

pub fn get_spotify(token_info: TokenInfo) -> (Spotify, SystemTime) {
    let token_expiry = {
        if let Some(expires_at) = token_info.expires_at {
            SystemTime::UNIX_EPOCH + Duration::from_secs(expires_at as u64)
                - Duration::from_secs(10)
        } else {
            SystemTime::now()
        }
    };

    let client_credential = SpotifyClientCredentials::default()
        .token_info(token_info)
        .build();

    let spotify = Spotify::default()
        .client_credentials_manager(client_credential)
        .build();

    (spotify, token_expiry)
}

fn redirect_to_authorize(spotify_oauth: &mut SpotifyOAuth, port: u16) -> Result<String, ()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port));

    match listener {
        Ok(listener) => {
            request_token(spotify_oauth);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Some(url) = handle_connection(stream) {
                            return Ok(url);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                };
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    Err(())
}

fn handle_connection(mut stream: TcpStream) -> Option<String> {
    let mut buff = [0; 1000];
    let _ = stream.read(&mut buff);

    match String::from_utf8(buff.to_vec()) {
        Ok(req) => {
            let split: Vec<&str> = req.split_whitespace().collect();

            if split.len() > 1 {
                return Some(split[1].to_string());
            }
            None
        }
        Err(e) => {
            eprintln!("Malformed request: {e}");
            None
        }
    }
}
