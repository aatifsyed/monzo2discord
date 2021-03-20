#[cfg(debug_assertions)]
use dotenv;
use monzo2discord::{ClientOpt, DiscordWebhook, Monzo2DiscordError, OauthHttpClient};
use oauth2::{basic::BasicClient as OauthClient, AuthorizationCode, CsrfToken};
use reqwest::Client as HTTPClient;
use rocket::{get, http::Status, launch, response::Redirect, routes, Response, Rocket, State};
use serde_json;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;
use structopt::StructOpt;

type CsrfMap = Mutex<HashMap<String, DiscordWebhook>>; // TODO this should be a TTL cache

#[get("/oauth/login?<webhook>")]
async fn login(
    http_client: State<'_, HTTPClient>,
    oauth_client: State<'_, OauthClient>,
    secret_map: State<'_, CsrfMap>,
    webhook: &str,
) -> Result<Redirect, Monzo2DiscordError> {
    let discord_webhook = DiscordWebhook::new(&http_client, webhook.into()).await?;
    let (url, csrf_token) = oauth_client.authorize_url(CsrfToken::new_random).url();

    let csrf_token = serde_json::to_string(&csrf_token).unwrap();
    let secret_map = &*secret_map; // Get a reference to the Mutex
    let mut secret_map = secret_map.lock().unwrap(); // Turn that reference into an owned value
    secret_map.insert(csrf_token, discord_webhook);

    let url = url.into_string();
    println!("Redirecting to {:?}", url);

    Ok(Redirect::to(url))
}

#[get("/oauth/callback?<code>&<state>")]
async fn oauth_callback(
    http_client: State<'_, HTTPClient>,
    oauth_client: State<'_, OauthClient>,
    secret_map: State<'_, CsrfMap>,
    code: &str,
    state: &str,
) -> Response<'static> {
    let state = state.to_owned();
    let mut value = None;

    // Drop the lock before we await
    {
        let mut secret_map = (&*secret_map).lock().unwrap();
        value = secret_map.remove(&state);
    }

    match value {
        None => Response::build().status(Status::Gone).finalize(),
        Some(discord_webhoook) => {
            match oauth_client
                .exchange_code(AuthorizationCode::new(code.to_string()))
                .request_async(|request| http_client.oauth_http_client(request))
                .await
            {
                Err(token_error) => Response::build()
                    .status(Status::ExpectationFailed)
                    .finalize(),
                Ok(token_response) => Response::new(),
            }
        }
    }
}

#[launch]
fn rocket() -> Rocket {
    #[cfg(debug_assertions)] // dev
    dotenv::dotenv().ok();

    let client_opt = ClientOpt::from_args();
    let secret_map = CsrfMap::new(HashMap::new());

    rocket::ignite()
        .mount("/", routes![login])
        .manage(HTTPClient::new())
        .manage(client_opt.to_oauth_client())
        .manage(secret_map)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn state_code_survives_journey() {
        let token = CsrfToken::new_random();
    }
}
