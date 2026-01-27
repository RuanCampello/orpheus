//! Spotify authentication module for Orpheus.
//!
//! This module handles one-time authentication with the Spotify Web API.
//! On first run, it prompts the user for their Spotify app credentials,
//! then stores them locally. Tokens are cached and automatically refreshed,
//! so users don't need to re-authenticate.

#![allow(unused)]

use std::io::{self, Write};
use std::path::PathBuf;

use directories::ProjectDirs;
use rspotify::{
    AuthCodePkceSpotify, Config, Credentials, OAuth,
    prelude::{BaseClient, OAuthClient},
    scopes,
};
use thiserror::Error;

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

#[derive(Debug, Clone)]
struct StoredCredentials {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Failed to find or create config directory")]
    ConfigDir,
    #[error("Failed to read user input: {0}")]
    InputError(String),
    #[error("The OAuth flow was not properly configured: {0}")]
    OAuthConfig(String),
    #[error("Failed to authenticate with Spotify: {0}")]
    Authentication(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Creates and authenticates a Spotify client.
///
/// This function handles the complete authentication flow:
///
/// 1. **First run**: Prompts for Spotify app credentials, then opens a browser
///    for OAuth authorization. Both credentials and tokens are cached.
/// 2. **Subsequent runs**: Loads cached credentials and tokens. If the access
///    token is expired, it's automatically refreshed using the refresh token.
///
/// # Example
///
/// ```no_run
/// use orpheus::auth::authenticate;
///
/// #[tokio::main]
/// async fn main() {
///     let spotify = authenticate().await.expect("Failed to authenticate");
///     // Now you can use `spotify` to make API calls
/// }
/// ```
pub async fn authenticate() -> Result<AuthCodePkceSpotify, AuthError> {
    let stored = get_or_prompt_credentials()?;
    let cache_path = token_cache_path()?;

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| AuthError::ConfigDir)?;
    }

    let creds = Credentials::new_pkce(&stored.client_id);

    let oauth = OAuth {
        redirect_uri: stored.redirect_uri,
        scopes: scopes!(
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
            "user-read-recently-played"
        ),
        ..Default::default()
    };

    let config = Config {
        cache_path: cache_path.clone(),
        token_cached: true,     // save token to file
        token_refreshing: true, // auto-refresh expired tokens
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::with_config(creds, oauth, config);
    let cached_token = spotify.read_token_cache(true).await;

    match cached_token {
        Ok(Some(token)) => {
            *spotify.token.lock().await.unwrap() = Some(token);
        }
        _ => {
            println!("Opening browser for Spotify authorization...");

            let auth_url = spotify
                .get_authorize_url(None)
                .map_err(|e| AuthError::Authentication(e.to_string()))?;

            spotify
                .prompt_for_token(&auth_url)
                .await
                .map_err(|e| AuthError::Authentication(e.to_string()))?;

            println!("Authentication successful!");
            println!("Token cached at: {}", cache_path.display());
        }
    }

    // try to refresh the token to ensure it's valid
    // this also saves the refreshed token to the cache
    if let Err(e) = spotify.refresh_token().await {
        let err_msg = e.to_string();
        if !err_msg.contains("refresh") {
            eprintln!("Warning: Could not refresh token: {}", err_msg);
        }
    }

    Ok(spotify)
}

pub fn config_dir() -> Result<PathBuf, AuthError> {
    ProjectDirs::from("", "", "orpheus")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or(AuthError::ConfigDir)
}

fn token_cache_path() -> Result<PathBuf, AuthError> {
    let mut path = config_dir()?;
    path.push("token_cache.json");
    Ok(path)
}

fn credentials_path() -> Result<PathBuf, AuthError> {
    let mut path = config_dir()?;
    path.push("credentials.txt");
    Ok(path)
}

fn prompt(message: &str) -> Result<String, AuthError> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn prompt_secret(message: &str) -> Result<String, AuthError> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn load_credentials() -> Result<Option<StoredCredentials>, AuthError> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    let lines: Vec<&str> = content.lines().collect();

    Ok(match lines.len() >= 3 {
        true => Some(StoredCredentials {
            client_id: lines[0].to_string(),
            client_secret: lines[1].to_string(),
            redirect_uri: lines[2].to_string(),
        }),
        false => None,
    })
}

fn save_credentials(creds: &StoredCredentials) -> Result<(), AuthError> {
    let path = credentials_path()?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| AuthError::ConfigDir)?;
    }

    let content = format!(
        "{}\n{}\n{}\n",
        creds.client_id, creds.client_secret, creds.redirect_uri
    );

    std::fs::write(&path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

fn prompt_for_credentials() -> Result<StoredCredentials, AuthError> {
    println!("To use Orpheus, you need to create a Spotify Developer App:");
    println!();
    println!("  1. Go to: https://developer.spotify.com/dashboard");
    println!("  2. Log in with your Spotify account");
    println!("  3. Click 'Create App'");
    println!("  4. Fill in a name and description (anything you like)");
    println!("  5. Set the Redirect URI to: http://127.0.0.1:8888/callback");
    println!("  6. Check 'Web API' under 'Which API/SDKs are you planning to use?'");
    println!("  7. Accept the terms and click 'Save'");
    println!("  8. Click 'Settings' to find your Client ID and Client Secret");
    println!();

    let client_id = prompt("Enter your Client ID: ")?;
    if client_id.is_empty() {
        return Err(AuthError::InputError(
            "Client ID cannot be empty".to_string(),
        ));
    }

    let client_secret = prompt_secret("Enter your Client Secret: ")?;
    if client_secret.is_empty() {
        return Err(AuthError::InputError(
            "Client Secret cannot be empty".to_string(),
        ));
    }

    let redirect_uri = prompt("Enter your Redirect URI [http://127.0.0.1:8888/callback]: ")?;
    let redirect_uri = if redirect_uri.is_empty() {
        "http://127.0.0.1:8888/callback".to_string()
    } else {
        redirect_uri
    };

    Ok(StoredCredentials {
        client_id,
        client_secret,
        redirect_uri,
    })
}

fn get_or_prompt_credentials() -> Result<StoredCredentials, AuthError> {
    if let Some(creds) = load_credentials()? {
        return Ok(creds);
    }
    let creds = prompt_for_credentials()?;
    save_credentials(&creds)?;
    println!("✓ Credentials saved to: {}", credentials_path()?.display());

    Ok(creds)
}
