#[cfg(debug_assertions)]
use dotenv;
use monzo2discord::{ClientOpt, DiscordWebhook, Monzo2DiscordError};
use oauth2::{basic::BasicClient as OauthClient, CsrfToken};
use reqwest::Client as HTTPClient;
use rocket::{get, launch, response::Redirect, routes, Response, Rocket, State};
use serde_json;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;
use structopt::StructOpt;

type CsrfMap = Mutex<HashMap<String, DiscordWebhook>>; // TODO this should be a TTL cache

#[get("/login?<webhook>")]
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

    Ok(Redirect::to(url.into_string()))
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
mod test {}
