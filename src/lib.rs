use reqwest::{self, StatusCode};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Monzo2DiscordError {
    #[error("Discord doesn't think this is a valid webhook")]
    InvalidWebhook(reqwest::Response),

    #[error("Couldn't make a web request for some low-level reason")]
    WebError(#[from] reqwest::Error),

    #[error("A POST wasn't accepted")]
    PostFailed(reqwest::Response),
}

pub struct DiscordWebhook {
    #[allow(dead_code)]
    id: u64,
    #[allow(dead_code)]
    token: String,
    address: String,
}

impl DiscordWebhook {
    pub async fn new(
        client: &reqwest::Client,
        id: u64,
        token: String,
    ) -> Result<Self, Monzo2DiscordError> {
        let address = format!("https://discord.com/api/webhooks/{}/{}", id, token);
        let response = client.get(&address).send().await?;
        match response.status() {
            StatusCode::OK => Ok(Self { id, token, address }),
            _ => Err(Monzo2DiscordError::InvalidWebhook(response)),
        }
    }
    pub async fn post(
        &self,
        client: &reqwest::Client,
        message: String,
    ) -> Result<(), Monzo2DiscordError> {
        let response = client.post(&self.address).body(message).send().await?;
        match response.status() {
            StatusCode::OK => Ok(()),
            _ => Err(Monzo2DiscordError::PostFailed(response)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        unimplemented!();
    }
}
