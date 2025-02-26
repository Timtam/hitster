use crate::{responses::PacksResponse, services::ServiceStore};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use std::collections::HashMap;

#[openapi(tag = "Hits")]
#[get("/hits/packs")]
pub fn get_all_packs(serv: &State<ServiceStore>) -> Json<PacksResponse> {
    let hits = serv.hit_service().lock().get_all();

    Json(PacksResponse {
        packs: hits.iter().fold(
            HashMap::<&'static str, usize>::new(),
            |mut p: HashMap<&'static str, usize>, h| {
                p.insert(h.pack, *p.get::<&'static str>(&h.pack).unwrap_or(&0) + 1);
                p
            },
        ),
    })
}
