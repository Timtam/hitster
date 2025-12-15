use altcha_lib_rs::{Challenge, ChallengeOptions};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::Utc;
use rocket::{http::Status, serde::json::Json};
use std::{
    env,
    sync::{OnceLock, RwLock},
    time::Duration,
};
use timed_set::TimedSet;

pub fn verify_captcha(payload: &str) -> bool {
    static TOKENS: OnceLock<RwLock<TimedSet<String>>> = OnceLock::new();

    let tokens = TOKENS.get_or_init(|| RwLock::new(TimedSet::new(Duration::from_mins(15))));

    if !tokens.read().unwrap().contains(&payload.to_string()) {
        let decoded_payload = BASE64_STANDARD.decode(payload);
        if let Ok(decoded_payload) = decoded_payload {
            let string_payload = std::str::from_utf8(decoded_payload.as_slice());
            if let Ok(string_payload) = string_payload {
                let hmac = env::var("ALTCHA_KEY").unwrap_or("".to_string());
                if hmac.is_empty() {
                    false
                } else {
                    if altcha_lib_rs::verify_json_solution(string_payload, &hmac, true).is_ok() {
                        tokens.write().unwrap().add(payload.to_string());
                        true
                    } else {
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

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
