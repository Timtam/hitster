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
