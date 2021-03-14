#![feature(proc_macro_hygiene, decl_macro)]
use reqwest;
use rocket::{get, http::RawStr, routes};

#[get("/login?<webhook_id>&<webhook_token>")]
fn login(webhook_id: u64, webhook_token: &RawStr) -> &'static str {
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
