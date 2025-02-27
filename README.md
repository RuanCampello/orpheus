<p align="center"><h1 align="center">Orpheus</h1></p>
<p align="center">
	<em><code>❯ Spotify TUI Client</code></em>
</p>
<p align="center">Control your Spotify within the terminal.</p>
<br>

## Installation

1. Clone this repository:

```sh
git clone https://github.com/RuanCampello/orpheus.git
cd orpheus
```

2. Environment Variables:

To run this project locally, set up a `.env` file in the root directory with the following variables:

[Get Spotify API Credentials](https://developer.spotify.com/documentation/web-api/)

```env
# Spotify API credentials
CLIENT_ID=
CLIENT_SECRET=

RSPOTIFY_REDIRECT_URI=http://localhost:8888/callback
```

3. Run the Application:

```sh
cargo run
```

> [!NOTE]
> Make sure to have a Spotify client open when launching the TUI. It's a limitation of Spotify's web API. It can be a
> desktop app, the web client or the Spotify Daemon


Checkout the Spotify Daemon [here](https://github.com/Spotifyd/spotifyd).