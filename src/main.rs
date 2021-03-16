#[cfg(debug_assertions)]
use dotenv;
use monzo2discord::{DiscordWebhook, Monzo2DiscordError};
use oauth2::Client as OauthClient;
use reqwest::Client as HTTPClient;
use rocket::{get, launch, response::Redirect, routes, Response, Rocket, State};
use std::env;
use std::io::Cursor;

#[get("/login?<webhook>")]
async fn login(
    client: State<'_, HTTPClient>,
    webhook: &str,
) -> Result<Redirect, Monzo2DiscordError> {
    let webhook = DiscordWebhook::new(&client, webhook.into()).await?;
    let body = "hello";
    Ok(Redirect::to("http://example.com"))
}

#[launch]
fn rocket() -> Rocket {
    #[cfg(debug_assertions)] // dev
    dotenv::dotenv().ok();

    println!("{:#?}", env::var("M2D_CLIENT_SECRET"));

    rocket::ignite()
        .mount("/", routes![login])
        .manage(HTTPClient::new())
    // .manage(OauthClient::new())
}

#[cfg(test)]
mod test {}
