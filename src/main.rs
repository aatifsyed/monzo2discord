#[cfg(debug_assertions)]
use dotenv;
use monzo2discord::{ClientOpt, DiscordWebhook, Monzo2DiscordError, CsrfToken};
use oauth2::{basic::BasicClient as OauthClient};
use reqwest::Client as HTTPClient;
use rocket::{get, launch, response::Redirect, routes, Response, Rocket, State};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

type SecretMap = Arc<Mutex<HashMap<CsrfToken, DiscordWebhook>>>; // TODO this should be a TTL cache

#[get("/login?<webhook>")]
async fn login(
    http_client: State<'_, HTTPClient>,
    oauth_client: State<'_, OauthClient>,
    secret_map: State<'_, SecretMap>,
    webhook: &str,
) -> Result<Redirect, Monzo2DiscordError> {
    let webhook = DiscordWebhook::new(&http_client, webhook.into()).await?;
    let (url, state) = oauth_client.authorize_url(CsrfToken::new_random).url();
    let foo = secret_map.lock().unwrap();
    *foo.insert(webhook, webhook);
    Ok(Redirect::to(url.into_string()))
}

#[launch]
fn rocket() -> Rocket {
    #[cfg(debug_assertions)] // dev
    dotenv::dotenv().ok();

    let client_opt = ClientOpt::from_args();
    let secret_map = SecretMap::new(Mutex::new(HashMap::new()));

    rocket::ignite()
        .mount("/", routes![login])
        .manage(HTTPClient::new())
        .manage(client_opt.to_oauth_client())
        .manage(secret_map)
}

#[cfg(test)]
mod test {}
