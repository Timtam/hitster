use crate::{responses::PacksResponse, services::ServiceStore};
use rocket::{serde::json::Json, State};
use rocket_okapi::openapi;
use std::collections::HashMap;

#[openapi(tag = "Hits")]
#[get("/hits/packs")]
pub fn get_all_packs(serv: &State<ServiceStore>) -> Json<PacksResponse> {
    let hits = serv.hit_service().lock().get_all();

    Json(PacksResponse {
        packs: hits.iter().fold(HashMap::new(), |mut p, h| {
            p.insert(h.pack.clone(), *p.get(&h.pack).unwrap_or(&0) + 1);
            p
        }),
    })
}
