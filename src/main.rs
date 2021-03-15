#![feature(proc_macro_hygiene, decl_macro)]
use monzo2discord::DiscordWebhook;
use rocket::{get, routes};

#[get("/login")]
fn login(discord: DiscordWebhook) -> &'static str {
    "Hello, world!"
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![login])
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::local;
}
