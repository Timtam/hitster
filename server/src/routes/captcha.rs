use altcha_lib_rs::{Challenge, ChallengeOptions};
use chrono::Utc;
use rocket::{http::Status, serde::json::Json};
use std::env;

#[get("/altcha")]
pub async fn get_altcha_challenge() -> Result<Json<Challenge>, (Status, String)> {
    let hmac = env::var("ALTCHA_KEY").unwrap_or("".to_string());

    if hmac.is_empty() {
        return Err((Status::NoContent, "altcha not enabled".to_string()));
    }

    let res = altcha_lib_rs::create_challenge(ChallengeOptions {
        hmac_key: &hmac,
        expires: Some(Utc::now() + chrono::TimeDelta::minutes(5)),
        ..Default::default()
    });
    res.map(Json)
        .map_err(|e| (Status::Conflict, format!("{:?}", e)))
}
