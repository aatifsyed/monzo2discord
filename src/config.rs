use serde::{Deserialize};

#[derive(Deserialize)]
struct AuthConf {
    client_id: str
    client_secret: str,
    auth_url: str,
    token_url: str,
}