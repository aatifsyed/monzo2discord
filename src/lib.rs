use reqwest;
use serde::Serialize;
use std::convert::Into;
use thiserror;
use url;

#[derive(thiserror::Error, Debug)]
pub enum Monzo2DiscordError {
    #[error("Couldn't manage internal state")]
    StateError,

    #[error("The user-provided webhook isn't valid")]
    InvalidWebhook(#[from] InvalidWebhookError),

    #[error("Couldn't make a web request for some low-level reason")]
    WebError(#[from] reqwest::Error),

    #[error("An outgoing POST wasn't accepted")]
    PostFailed(reqwest::Response),
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidWebhookError {
    #[error("Discord wouldn't confirm that this is a webhook")]
    DiscordError(reqwest::Response),

    #[error("Host of URL must be `discord.com`")]
    DisallowedHost(String),

    #[error("Couldn't parse URL")]
    ParseError(#[from] url::ParseError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct DiscordWebhook {
    address: String,
}

impl DiscordWebhook {
    /// Validates a webhook, and returns one if validation passed.
    pub async fn new(
        client: &reqwest::Client,
        address: String,
    ) -> Result<Self, Monzo2DiscordError> {
        let parsed = match url::Url::parse(&address) {
            Ok(p) => p,
            Err(e) => return Err(Monzo2DiscordError::InvalidWebhook(e.into())),
        };
        match parsed.host_str() {
            Some("discord.com") => {
                let response = client.get(&address).send().await?;
                match response.status() {
                    reqwest::StatusCode::OK => Ok(Self { address }),
                    _ => Err(InvalidWebhookError::DiscordError(response).into()),
                }
            }
            _ => Err(InvalidWebhookError::DisallowedHost(address).into()),
        }
    }
    pub async fn post(
        &self,
        client: &reqwest::Client,
        message: String,
    ) -> Result<(), Monzo2DiscordError> {
        let response = client.post(&self.address).body(message).send().await?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            _ => Err(Monzo2DiscordError::PostFailed(response)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn valid_webhooks() {
        let client = reqwest::Client::new();
        assert!(matches!(
            DiscordWebhook::new(&client, "hello".into()).await,
            Err(Monzo2DiscordError::InvalidWebhook(
                InvalidWebhookError::ParseError(_)
            ),),
        ));
        assert!(matches!(
            DiscordWebhook::new(&client, "https://google.com".into()).await,
            Err(Monzo2DiscordError::InvalidWebhook(
                InvalidWebhookError::DisallowedHost(_)
            ),),
        ));
        assert!(matches!(
            DiscordWebhook::new(
                &client,
                "https://discord.com/api/webhooks/12345/abcde".into()
            )
            .await,
            Err(Monzo2DiscordError::InvalidWebhook(
                InvalidWebhookError::DiscordError(_)
            ),),
        ));
    }
}
