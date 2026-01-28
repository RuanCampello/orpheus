mod auth;
mod config;

use rspotify::prelude::OAuthClient;

#[tokio::main]
async fn main() {
    let spotify = match auth::authenticate().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    };

    match spotify.current_user().await {
        Ok(user) => println!(
            "{}",
            user.display_name.unwrap_or_else(|| user.id.to_string())
        ),
        Err(e) => {
            eprintln!("Failed to fetch user profile: {}", e);
            std::process::exit(1);
        }
    }
}
