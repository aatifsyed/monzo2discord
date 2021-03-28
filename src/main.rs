#[cfg(debug_assertions)]
use dotenv;
use monzo2discord::{ClientOpt, Discord, Monzo2DiscordError, OauthHttpClient, Webhook};
use oauth2::{basic::BasicClient as OauthClient, AuthorizationCode, CsrfToken};
use reqwest::Client as HTTPClient;
use rocket::{
    get,
    http::{self, Status},
    launch,
    response::Redirect,
    routes, Response, Rocket, State,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Mutex;
use structopt::StructOpt;

type CsrfMap = Mutex<HashMap<String, Webhook>>; // TODO this should be a TTL cache

#[get("/oauth/login?<webhook>")]
async fn login(
    discord: State<'_, Discord>,
    http_client: State<'_, HTTPClient>,
    oauther: State<'_, OauthClient>,
    secret_map: State<'_, CsrfMap>,
    webhook: &str,
) -> Result<Redirect, Monzo2DiscordError> {
    let webhook = discord.create_webhook(&http_client, webhook).await?;
    let (url, csrf_token) = oauther.authorize_url(CsrfToken::new_random).url();

    // We want to use the token as a key. Transform to hashable
    let csrf_token = serde_json::to_string(&csrf_token).unwrap();

    let secret_map = &*secret_map; // Get a reference to the Mutex
    let mut secret_map = secret_map.lock().unwrap(); // Turn that reference into an owned value

    // Store the Cross Site Request Forgery token so that we can validate the callback
    secret_map.insert(csrf_token, webhook);

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
    let value = {
        let mut secret_map = (&*secret_map).lock().unwrap();
        secret_map.remove(&state)
    }; // Drop the lock before we await, as the borrow-checker is extending the lifetime unnecessarily

    match value {
        None => {
            // I don't recognise this CSRF. How did you get here??
            Response::build().status(Status::Gone).finalize()
        }
        Some(discord_webhook) => {
            // A real CSRF. Configure this webhook.
            match oauth_client
                .exchange_code(AuthorizationCode::new(code.to_string()))
                .request_async(|request| http_client.oauth_http_client(request))
                .await
            {
                // Monzo doesn't like us.
                Err(_token_error) => Response::build()
                    .status(Status::ExpectationFailed)
                    .finalize(),
                Ok(_token_response) => {
                    let _ = ();
                    Response::new()
                }
            }
        }
    }
}

#[launch]
fn rocket() -> Rocket {
    #[cfg(debug_assertions)] // dev
    dotenv::dotenv().ok();

    let client_opt = ClientOpt::from_args();
    let oauther = client_opt.to_oauth_client();

    let secret_map = CsrfMap::new(HashMap::new());
    let http_client = HTTPClient::new();
    let discord = Discord::default();

    rocket::ignite()
        .mount("/", routes![login])
        .manage(http_client)
        .manage(oauther)
        .manage(secret_map)
        .manage(discord)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn state_code_survives_journey() {
        let token = CsrfToken::new_random();
    }
}
