use reqwest;
use rocket::{self, http::Status, response::Responder, Request, Response};
use std::convert::Into;
use std::io::Cursor;
use thiserror;
use url;

#[derive(thiserror::Error, Debug)]
pub enum Monzo2DiscordError {
    #[error("The user-provided webhook isn't valid")]
    InvalidWebhook(#[from] InvalidWebhookError),

    #[error("Couldn't make a web request: {:?}", .0)]
    WebError(#[from] reqwest::Error),

    #[error("An outgoing POST wasn't accepted: {}", .0.status())]
    PostFailed(reqwest::Response),
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidWebhookError {
    #[error("Discord wouldn't confirm that this is a webhook: {}", .0.status())]
    DiscordError(reqwest::Response),

    #[error("Host of URL must be `discord.com`, and path must be `/api/webhooks/...`, not {}", .0)]
    DisallowedUrl(String),

    #[error("Couldn't parse URL: {:?}", .0)]
    ParseError(#[from] url::ParseError),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Monzo2DiscordError {
    fn respond_to(self, _request: &'r Request<'_>) -> Result<Response<'o>, Status> {
        let status = match self {
            Monzo2DiscordError::InvalidWebhook(_) => Status::BadRequest,
            Monzo2DiscordError::WebError(_) => Status::InternalServerError,
            Monzo2DiscordError::PostFailed(_) => Status::FailedDependency,
        };
        let body = format!("{}:\n{:#?}", self, self);
        let response = Response::build()
            .status(status)
            .sized_body(body.len(), Cursor::new(body))
            .finalize();
        Ok(response)
    }
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
            Some("discord.com") if parsed.path().starts_with("/api/webhooks") => {
                // URL looks OK. Check with discord.
                let response = client.get(&address).send().await?;
                match response.status() {
                    reqwest::StatusCode::OK => Ok(Self { address }),
                    _ => Err(InvalidWebhookError::DiscordError(response).into()),
                }
            }
            _ => Err(InvalidWebhookError::DisallowedUrl(address).into()),
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
                InvalidWebhookError::DisallowedUrl(_)
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
