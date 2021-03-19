use async_trait::async_trait;
use oauth2::{
    basic::BasicClient as OauthClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use reqwest;
use rocket::{self, http::Status, response::Responder, Request, Response};
use std::convert::Into;
use std::io::Cursor;
use structopt::StructOpt;
use thiserror;
use url;

/// An `oauth2::Client` can take a...
/// ```rust,ignore
/// FnOnce(HttpRequest) -> F
/// where
/// F: Future<Output = Result<HttpResponse, RE>>,
/// RE: std::error::Error + 'static
/// ```
/// And we want to use the pre-existing reqwest client, so trait it in.
#[async_trait]
pub trait OauthHttpClient {
    async fn oauth_http_client(
        &self,
        request: oauth2::HttpRequest,
    ) -> Result<oauth2::HttpResponse, reqwest::Error>;
}

#[async_trait]
impl OauthHttpClient for reqwest::Client {
    async fn oauth_http_client(
        &self,
        request: oauth2::HttpRequest,
    ) -> Result<oauth2::HttpResponse, reqwest::Error> {
        let request = self
            .request(request.method, request.url)
            .body(request.body)
            .headers(request.headers)
            .build()?;

        let response = self.execute(request).await?;

        Ok(oauth2::HttpResponse {
            status_code: response.status(),
            headers: response.headers().to_owned(),
            body: response.bytes().await?.to_vec(),
        })
    }
}

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

/// Allow different errors to become HTTP responses
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

#[derive(Debug, StructOpt)]
#[structopt(name = "foo")]
pub struct ClientOpt {
    #[structopt(short = "i", long, env, parse(from_str=parsers::client_id))]
    client_id: ClientId,
    #[structopt(short = "s", long, env, parse(from_str=parsers::client_secret), hide_env_values = true)]
    client_secret: ClientSecret,
    #[structopt(short, long, env, parse(try_from_str=parsers::auth_url), default_value = "https://auth.monzo.com")]
    auth_url: AuthUrl,
    #[structopt(short, long, env, parse(try_from_str=parsers::redirect_url))]
    redirect_url: RedirectUrl,
    #[structopt(short, long, env, parse(try_from_str=parsers::token_url), default_value = "https://api.monzo.com/oauth2/token")]
    token_url: TokenUrl,
}

impl ClientOpt {
    pub fn to_oauth_client(self) -> OauthClient {
        OauthClient::new(
            self.client_id,
            Some(self.client_secret),
            self.auth_url,
            Some(self.token_url),
        )
        .set_redirect_uri(self.redirect_url)
    }
}

/// For structopt parsing
mod parsers {
    use super::*;
    pub fn client_id(s: &str) -> ClientId {
        ClientId::new(s.to_owned())
    }
    pub fn client_secret(s: &str) -> ClientSecret {
        ClientSecret::new(s.to_owned())
    }
    pub fn auth_url(s: &str) -> Result<AuthUrl, url::ParseError> {
        AuthUrl::new(s.to_owned())
    }
    pub fn redirect_url(s: &str) -> Result<RedirectUrl, url::ParseError> {
        RedirectUrl::new(s.to_owned())
    }
    pub fn token_url(s: &str) -> Result<TokenUrl, url::ParseError> {
        TokenUrl::new(s.to_owned())
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
