use crate::{
    HitsterConfig,
    games::PackPayload,
    hits::{FullHitPayload, HitPayload, HitSearchQuery},
    responses::{
        DeleteHitError, GetHitError, MessageResponse, PacksResponse, PaginatedResponse,
        UpdateHitError,
    },
    services::ServiceStore,
    users::UserAuthenticator,
};
use hitster_core::{Hit, HitId, Permissions};
use rocket::{State, serde::json::Json};
use rocket_db_pools::{
    Connection,
    sqlx::{self, FromRow},
};
use rocket_okapi::openapi;
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(FromRow)]
struct HitRow {
    custom: bool,
}

#[derive(FromRow)]
struct HitPackRow {
    hit_id: Uuid,
    pack_id: Uuid,
    custom: bool,
    marked_for_deletion: bool,
}

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

#[openapi(tag = "Hits")]
#[get("/hits/<hit_id>")]
pub fn get_hit(
    hit_id: &str,
    svc: &State<ServiceStore>,
) -> Result<Json<FullHitPayload>, GetHitError> {
    let hs = svc.hit_service();
    let hsl = hs.lock();

    Uuid::parse_str(hit_id)
        .ok()
        .and_then(|hit_id| hsl.get_hit(&HitId::Id(hit_id)))
        .map(|h| Json(h.into()))
        .ok_or(GetHitError {
            message: "hit id not found".into(),
            http_status_code: 404,
        })
}

#[openapi(tag = "Hits")]
#[patch("/hits", format = "json", data = "<hit>")]
pub async fn update_hit(
    hit: Json<FullHitPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, UpdateHitError> {
    if !user.0.permissions.contains(Permissions::CAN_WRITE_HITS) {
        return Err(UpdateHitError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    let hs = serv.hit_service();

    if hs.lock().get_hit(&HitId::Id(hit.id)).is_none() {
        return Err(UpdateHitError {
            message: "hit not found".into(),
            http_status_code: 404,
        });
    }

    let new_hit = Hit {
        id: hit.id,
        title: hit.title.clone(),
        artist: hit.artist.clone(),
        packs: hit.packs.clone(),
        belongs_to: hit.belongs_to.clone(),
        yt_id: hit.yt_id.clone(),
        playback_offset: hit.playback_offset,
        last_modified: OffsetDateTime::now_utc(),
        year: hit.year,
    };
    let exists = new_hit.exists();

    hs.lock().remove_hit(&HitId::Id(hit.id));

    let _ = sqlx::query!(
        "
UPDATE hits SET
    title = $1,
    artist = $2,
    yt_id = $3,
    year = $4,
    playback_offset = $5,
    belongs_to = $6,
    last_modified = $7,
    downloaded = $8
    WHERE id = $9",
        new_hit.title,
        new_hit.artist,
        new_hit.yt_id,
        new_hit.year,
        new_hit.playback_offset,
        new_hit.belongs_to,
        new_hit.last_modified,
        exists,
        new_hit.id,
    )
    .execute(&mut **db)
    .await;

    let mut hits_packs = sqlx::query_as!(
        HitPackRow,
        r#"
SELECT
    hit_id AS "hit_id: Uuid",
    pack_id AS "pack_id: Uuid",
    custom,
    marked_for_deletion
FROM hits_packs WHERE hit_id = ?"#,
        hit.id
    )
    .fetch_all(&mut **db)
    .await
    .unwrap()
    .into_iter()
    .map(|row| (row.pack_id, row))
    .collect::<HashMap<Uuid, HitPackRow>>();

    for pack in new_hit.packs.iter() {
        if let Some(row) = hits_packs.get(pack) {
            if row.marked_for_deletion {
                let _ = sqlx::query!(
                    r#"
UPDATE
    hits_packs
SET 
    marked_for_deletion = ?
WHERE hit_id = ? AND pack_id = ?"#,
                    false,
                    row.hit_id,
                    row.pack_id
                )
                .execute(&mut **db)
                .await;
            }
            let id = row.pack_id;
            hits_packs.remove(&id);
        } else {
            let _ = sqlx::query!(
                r#"
INSERT INTO
    hits_packs (
    hit_id, 
    pack_id, 
    custom, 
    marked_for_deletion
) VALUES (
    ?, ?, ?, ?)"#,
                new_hit.id,
                pack,
                true,
                false
            )
            .execute(&mut **db)
            .await;
        }
    }

    for row in hits_packs.values() {
        if row.custom {
            let _ = sqlx::query!(
                r#"
DELETE FROM hits_packs
WHERE hit_id = ? AND pack_id = ?"#,
                row.hit_id,
                row.pack_id
            )
            .execute(&mut **db)
            .await;
        } else if !row.marked_for_deletion {
            let _ = sqlx::query!(
                r#"
UPDATE
    hits_packs
SET 
    marked_for_deletion = ?
WHERE hit_id = ? AND pack_id = ?"#,
                true,
                row.hit_id,
                row.pack_id
            )
            .execute(&mut **db)
            .await;
        }
    }

    if exists {
        hs.lock().insert_hit(new_hit);
    } else {
        hs.lock().download_hit(new_hit);
    }

    Ok(Json(MessageResponse {
        message: "hit updated successfully".into(),
        r#type: "success".into(),
    }))
}

#[openapi(tag = "Hits")]
#[delete("/hits/<hit_id>")]
pub async fn delete_hit(
    hit_id: Uuid,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, DeleteHitError> {
    if !user.0.permissions.contains(Permissions::CAN_WRITE_HITS) {
        return Err(DeleteHitError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    let hs = serv.hit_service();

    if hs.lock().get_hit(&HitId::Id(hit_id)).is_none() {
        return Err(DeleteHitError {
            message: "hit not found".into(),
            http_status_code: 404,
        });
    }

    hs.lock().remove_hit(&HitId::Id(hit_id));

    let hit = sqlx::query_as!(HitRow, "SELECT custom FROM hits WHERE id = ?", hit_id)
        .fetch_one(&mut **db)
        .await
        .unwrap();

    if hit.custom {
        let _ = sqlx::query!("DELETE FROM hits WHERE id = ?", hit_id)
            .execute(&mut **db)
            .await;
        let _ = sqlx::query!("DELETE FROM hits_packs WHERE hit_id = ?", hit_id)
            .execute(&mut **db)
            .await;
    } else {
        let _ = sqlx::query!(
            "UPDATE hits SET marked_for_deletion = ? WHERE id = ?",
            true,
            hit_id
        )
        .execute(&mut **db)
        .await;
    }

    Ok(Json(MessageResponse {
        message: "hit deleted successfully".into(),
        r#type: "success".into(),
    }))
}
