use crate::{hits::Pack, responses::PacksResponse, services::ServiceStore};
use rocket::{serde::json::Json, State};
use rocket_okapi::openapi;
use strum::VariantArray;

#[openapi(tag = "Hits")]
#[get("/hits/packs")]
pub fn get_all_packs(serv: &State<ServiceStore>) -> Json<PacksResponse> {
    let hits = serv.hit_service().lock().get_all();

    Json(PacksResponse {
        packs: Pack::VARIANTS
            .iter()
            .map(|p| (*p, hits.iter().filter(|h| h.pack == *p).count()))
            .collect::<_>(),
    })
}
