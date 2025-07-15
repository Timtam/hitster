use crate::{
    hits::DownloadingGuard,
    responses::{HitsStatusResponse, PacksResponse, ServerBusyError},
    services::ServiceStore,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use std::collections::HashMap;

#[openapi(tag = "Hits")]
#[get("/hits/packs")]
pub fn get_all_packs(
    serv: &State<ServiceStore>,
    _g: DownloadingGuard,
) -> Result<Json<PacksResponse>, ServerBusyError> {
    let hits = serv.hit_service().lock().get_all();

    Ok(Json(PacksResponse {
        packs: hits.iter().fold(
            HashMap::<String, usize>::new(),
            |mut p: HashMap<String, usize>, h| {
                p.insert(h.pack.clone(), *p.get::<String>(&h.pack).unwrap_or(&0) + 1);
                p
            },
        ),
    }))
}

#[openapi(tag = "Hits")]
#[get("/hits/status")]
pub fn get_status(serv: &State<ServiceStore>) -> Json<HitsStatusResponse> {
    let hits_status = serv.hit_service().lock().get_progress();
    Json(HitsStatusResponse {
        downloaded: hits_status.0,
        all: hits_status.1,
        finished: hits_status.2,
    })
}
