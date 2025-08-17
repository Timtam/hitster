use crate::{
    games::PackPayload,
    hits::{HitPayload, HitSearchQuery},
    responses::{PacksResponse, PaginatedResponse},
    services::ServiceStore,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;

#[openapi(tag = "Hits")]
#[get("/hits/packs")]
pub fn get_all_packs(serv: &State<ServiceStore>) -> Json<PacksResponse> {
    let hs = serv.hit_service();
    let hsl = hs.lock();

    let packs = hsl
        .get_packs()
        .iter()
        .fold(vec![], |mut p: Vec<PackPayload>, pp| {
            p.push(PackPayload {
                id: pp.id,
                name: pp.name.clone(),
                hits: hsl.get_hits_for_packs(&[pp.id]).len(),
            });
            p
        });

    drop(hsl);
    drop(hs);

    Json(PacksResponse { packs })
}

#[openapi(tag = "Hits")]
#[get("/hits/search?<query..>")]
pub fn search_hits(
    query: HitSearchQuery,
    svc: &State<ServiceStore>,
) -> Json<PaginatedResponse<HitPayload>> {
    let hs = svc.hit_service();

    Json(
        Some(hs.lock().search_hits(&query))
            .map(|res| PaginatedResponse {
                results: res
                    .results
                    .into_iter()
                    .map(|h| (&h).into())
                    .collect::<Vec<_>>(),
                total: res.total,
                start: res.start,
                end: res.end,
            })
            .unwrap(),
    )
}
