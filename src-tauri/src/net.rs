use reqwest::blocking::Client;
use std::time::Duration;

pub fn client() -> Client {
    Client::builder()
        .user_agent(concat!("190x4-discord-kit/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(90))
        .build()
        .expect("http client")
}
