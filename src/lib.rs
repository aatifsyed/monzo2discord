use async_trait::async_trait;
use chrono::{DateTime, Utc};
use oauth2::{
    basic::BasicClient as OauthClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use reqwest;
use rocket::{self, http::Status, response::Responder, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json;
use serenity;
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

    #[error("An outgoing POST wasn't accepted")]
    WebhookNotExecuted(reqwest::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidWebhookError {
    #[error("Discord wouldn't confirm that this is a webhook")]
    DiscordError,

    #[error("Host of URL inconsistent. Got {:?}, wanted {:?}", .given, .configured)]
    DisallowedHost {
        given: url::Url,
        configured: url::Url,
    },

    #[error("Couldn't parse URL: {:?}", .0)]
    UrlParseError(#[from] url::ParseError),
}

/// Allow different errors to become HTTP responses
impl<'r, 'o: 'r> Responder<'r, 'o> for Monzo2DiscordError {
    fn respond_to(self, _request: &'r Request<'_>) -> Result<Response<'o>, Status> {
        let status = match self {
            Monzo2DiscordError::InvalidWebhook(_) => Status::BadRequest,
            Monzo2DiscordError::WebError(_) => Status::InternalServerError,
            Monzo2DiscordError::WebhookNotExecuted(_) => Status::FailedDependency,
        };
        let body = format!("{}:\n{:#?}", self, self);
        let response = Response::build()
            .status(status)
            .sized_body(body.len(), Cursor::new(body))
            .finalize();
        Ok(response)
    }
}

/// Represents a webhook
#[derive(Debug)]
pub struct Webhook {
    url: url::Url,
}

impl Webhook {
    pub async fn execute<B: std::fmt::Display>(
        &self,
        client: &reqwest::Client,
        body: B,
    ) -> Result<(), Monzo2DiscordError> {
        let body = Message {
            content: format!("{}", body),
        };
        let body = serde_json::to_string(&body).unwrap();
        client
            .post(self.url.clone())
            .body(body)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .error_for_status()
            .map(|_| ())
            .map_err(|e| Monzo2DiscordError::WebhookNotExecuted(e))
    }
}
#[derive(Serialize)]
struct Message {
    content: String,
}

/// Represents communication with discord
pub struct Discord {
    /// Where is discord?
    pub url: url::Url,
}

impl Default for Discord {
    fn default() -> Self {
        Self {
            url: url::Url::parse("https://discord.com").unwrap(),
        }
    }
}

impl Discord {
    pub async fn create_webhook(
        &self,
        client: &reqwest::Client,
        webhook: &str,
    ) -> Result<Webhook, Monzo2DiscordError> {
        // Should be a valid URL
        let mut webhook = match url::Url::parse(webhook) {
            Ok(p) => p,
            Err(e) => return Err(Monzo2DiscordError::InvalidWebhook(e.into())),
        };

        if webhook.host() != self.url.host() {
            return Err(InvalidWebhookError::DisallowedHost {
                given: webhook,
                configured: self.url.clone(),
            }
            .into());
        }

        webhook.set_fragment(None);
        webhook.set_query(None);

        client
            .get(webhook.clone())
            .send()
            .await?
            // Lean on serenity to validate for us
            .json::<serenity::model::webhook::Webhook>()
            .await
            .map_err::<Monzo2DiscordError, _>(|_| InvalidWebhookError::DiscordError.into())?;

        Ok(Webhook { url: webhook })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[non_exhaustive]
enum MonzoObject {
    #[serde(rename = "transaction.created")]
    TransactionCreated {
        account_id: String,
        amount: isize,
        created: DateTime<Utc>,
        currency: String,
        description: String,
        id: String,
        category: String,
        is_load: bool,
        settled: DateTime<Utc>,
        /// We only want this field for the name really
        #[serde(alias = "merchant")]
        counterparty: Named,
    },
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct Named {
    name: String,
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
    use httpmock::{Method, MockServer};
    /// We rely on `serenity`'s webhook object parsing to validate our webhooks.
    /// This is the smallest valid webhook
    fn valid_webhook_object(id: &str, token: &str) -> String {
        format!(
            r#"{{
            "application_id": null,
            "avatar": null,
            "channel_id": "100",
            "guild_id": null,
            "id": "{}",
            "name": null,
            "token": "{}",
            "type": 1
            }}"#,
            id, token
        )
    }
    fn transaction_created_object() -> &'static str {
        r#"{
            "type": "transaction.created",
            "data": {
                "account_id": "acc_00008gju41AHyfLUzBUk8A",
                "amount": -350,
                "created": "2015-09-04T14:28:40Z",
                "currency": "GBP",
                "description": "Ozone Coffee Roasters",
                "id": "tx_00008zjky19HyFLAzlUk7t",
                "category": "eating_out",
                "is_load": false,
                "settled": "2015-09-05T14:28:40Z",
                "merchant": {
                    "address": {
                        "address": "98 Southgate Road",
                        "city": "London",
                        "country": "GB",
                        "latitude": 51.54151,
                        "longitude": -0.08482400000002599,
                        "postcode": "N1 3JD",
                        "region": "Greater London"
                    },
                    "created": "2015-08-22T12:20:18Z",
                    "group_id": "grp_00008zIcpbBOaAr7TTP3sv",
                    "id": "merch_00008zIcpbAKe8shBxXUtl",
                    "logo": "https://pbs.twimg.com/profile_images/527043602623389696/68_SgUWJ.jpeg",
                    "emoji": "🍞",
                    "name": "The De Beauvoir Deli Co.",
                    "category": "eating_out"
                }
            }
        }"#
    }
    #[tokio::test]
    async fn valid_webhooks() {
        let discord_server = MockServer::start();
        let http_client = reqwest::Client::new();

        let (id, token) = ("123", "456");
        let path = format!("/api/webhooks/{}/{}", id, token);

        let webhook_endpoint = discord_server.mock(|when, then| {
            when.method(Method::GET).path(&path);
            then.status(200)
                .header("Content-Type", "application/json")
                .body(valid_webhook_object(id, token));
        });

        let discord_client = Discord {
            url: url::Url::parse(&discord_server.url("")).unwrap(),
        };

        discord_client
            .create_webhook(&http_client, &discord_server.url(&path))
            .await
            .unwrap();
        webhook_endpoint.assert();

        let bad_webhook = discord_server.mock(|when, then| {
            when.method(Method::GET).path("/api/webhooks/abc/def");
            then.status(400);
        });

        discord_client
            .create_webhook(&http_client, &discord_server.url("/api/webhooks/abc/def"))
            .await
            .unwrap_err();
        bad_webhook.assert();
    }

    #[tokio::test]
    async fn post_webhook() {
        let discord_server = MockServer::start();
        let http_client = reqwest::Client::new();
        let path = "/api/webhooks/123/456";

        let message = r#"Hello from "Aatif""#;
        let message_json = Message {
            content: message.to_string(),
        };
        let message_json = serde_json::to_string(&message_json).unwrap();

        let webhook_endpoint = discord_server.mock(|when, then| {
            when.method(Method::POST).path(path).body(message_json);
            then.status(200);
        });

        let webhook = Webhook {
            url: url::Url::parse(&discord_server.url(path)).unwrap(),
        };

        webhook.execute(&http_client, message).await.unwrap();

        webhook_endpoint.assert();
    }

    #[test]
    fn serde_transaction_created() {
        let incoming = transaction_created_object();

        let parsed: MonzoObject = serde_json::from_str(incoming).unwrap();
        match parsed {
            MonzoObject::TransactionCreated { counterparty, .. }
                if counterparty
                    == Named {
                        name: "The De Beauvoir Deli Co.".to_owned(),
                    } =>
            {
                ()
            }
            _ => panic!(),
        }
    }
}
