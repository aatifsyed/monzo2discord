use reqwest;
use rocket::{
    async_trait,
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Monzo2DiscordError {
    #[error("Couldn't manage internal state")]
    StateError,

    #[error("Discord doesn't think this is a valid webhook")]
    InvalidWebhook(reqwest::Response),

    #[error("Couldn't make a web request for some low-level reason")]
    WebError(#[from] reqwest::Error),

    #[error("An outgoing POST wasn't accepted")]
    PostFailed(reqwest::Response),

    #[error("Must provide query parameters webhook_id and webhook_token")]
    NoWebhook,
}

pub struct DiscordWebhook {
    id: u64,
    token: String,
    address: String,
}

#[async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for DiscordWebhook {
    type Error = Monzo2DiscordError;
    async fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        if let (Some(Ok(id)), Some(Ok(token))) = (
            request.query_value::<u64>("webhook_id"),
            request.query_value::<String>("webhook_token"),
        ) {
            if let Outcome::Success(shared_client) = request.guard::<State<reqwest::Client>>().await
            {
                let client = shared_client.inner();
                match Self::new(client, id, token).await {
                    Ok(webhook) => Outcome::Success(webhook),
                    Err(err) => Outcome::Failure((Status::BadRequest, err)),
                }
            } else {
                Outcome::Failure((Status::InternalServerError, Monzo2DiscordError::StateError))
            }
        } else {
            Outcome::Failure((Status::BadRequest, Monzo2DiscordError::NoWebhook))
        }
    }
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
            reqwest::StatusCode::OK => Ok(Self { id, token, address }),
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
            reqwest::StatusCode::OK => Ok(()),
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
