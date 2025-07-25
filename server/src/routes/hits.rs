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
    let hs = serv.hit_service();
    let hsl = hs.lock();

    let hits = hsl.get_available_hits();

    let packs = hits.iter().fold(
        HashMap::<String, usize>::new(),
        |mut p: HashMap<String, usize>, h| {
            h.packs.iter().for_each(|pp| {
                let name = hsl.get_pack(*pp).unwrap().name.clone();

                p.insert(name.clone(), *p.get::<String>(&name).unwrap_or(&0) + 1);
            });
            p
        },
    );

    drop(hsl);
    drop(hs);

    Ok(Json(PacksResponse { packs }))
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
