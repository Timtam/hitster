use crate::{responses::PacksResponse, services::ServiceStore};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use std::collections::HashMap;

#[openapi(tag = "Hits")]
#[get("/hits/packs")]
pub fn get_all_packs(serv: &State<ServiceStore>) -> Json<PacksResponse> {
    let hs = serv.hit_service();
    let hsl = hs.lock();

    let packs = hsl.get_packs().iter().fold(
        HashMap::<String, usize>::new(),
        |mut p: HashMap<String, usize>, pp| {
            p.insert(
                pp.name.clone(),
                *p.get::<String>(&pp.name).unwrap_or(&0) + 1,
            );
            p
        },
    );

    drop(hsl);
    drop(hs);

    Json(PacksResponse { packs })
}
