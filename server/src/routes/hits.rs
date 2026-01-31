use crate::{
    HitsterConfig,
    games::PackPayload,
    hits::{
        CreatePackPayload, FullHitPayload, HitPartsQuery, HitPayload, HitQueryPart, HitSearchQuery,
    },
    responses::{
        CreateHitError, CreateHitIssueError, CreatePackError, DeleteHitError, DeletePackError,
        ExportHitsError, GetHitError, MessageResponse, PacksResponse, PaginatedResponse,
        UpdateHitError, UpdatePackError, Yaml,
    },
    routes::captcha::verify_captcha,
    services::ServiceStore,
    users::UserAuthenticator,
};
use hitster_core::{Hit, HitId, HitIssue, HitIssueType, HitsterData, Pack, Permissions};
use rocket::{State, serde::json::Json};
use rocket_db_pools::{
    Connection,
    sqlx::{self, FromRow},
};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Deserialize;
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

fn includes_part(parts: &Option<Vec<HitQueryPart>>, part: HitQueryPart) -> bool {
    parts.as_ref().map(|p| p.contains(&part)).unwrap_or(false)
}

#[derive(FromRow)]
struct HitRow {
    custom: bool,
}

#[derive(FromRow)]
struct PackRow {
    custom: bool,
}

#[derive(FromRow)]
struct HitPackRow {
    hit_id: Uuid,
    pack_id: Uuid,
    custom: bool,
    marked_for_deletion: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateHitIssuePayload {
    pub message: String,
    pub altcha_token: String,
}

/// # Get all packs
///
/// This endpoint returns all packs currently available on this server.

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

    Json(PacksResponse { packs })
}

/// # Search for hits
///
/// Search for hits in the database. The search will be executed using fuzzy search, so approximated results will be returned as well.
/// The results will be paginated, use the parameters to specify the page size.
/// Use the optional `parts` parameter to include additional hit data in the response.
/// Supported values are `issues` and `downloaded`.
/// Issue data is only returned if the caller has the `READ_ISSUES` permission.

#[openapi(tag = "Hits")]
#[get("/hits/search?<query..>")]
pub async fn search_hits(
    query: HitSearchQuery,
    svc: &State<ServiceStore>,
    user: Option<UserAuthenticator>,
    mut db: Connection<HitsterConfig>,
) -> Json<PaginatedResponse<HitPayload>> {
    let hs = svc.hit_service();
    let res = hs.lock().search_hits(&query);
    let include_download_status = includes_part(&query.parts, HitQueryPart::Downloaded);
    let include_issues = includes_part(&query.parts, HitQueryPart::Issues);
    let can_read_issues = user
        .as_ref()
        .map(|u| u.0.permissions.contains(Permissions::READ_ISSUES))
        .unwrap_or(false);

    let issues_by_hit = if include_issues && can_read_issues {
        let mut qb = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
            "SELECT id, hit_id, type, message, created_at, last_modified FROM hit_issues WHERE hit_id IN (",
        );
        let mut separated = qb.separated(", ");
        res.results.iter().for_each(|hit| {
            separated.push_bind(hit.id);
        });
        separated.push_unseparated(") ORDER BY created_at ASC");

        qb.build_query_as::<HitIssue>()
            .fetch_all(&mut **db)
            .await
            .unwrap_or_default()
            .into_iter()
            .fold(HashMap::<Uuid, Vec<HitIssue>>::new(), |mut map, issue| {
                map.entry(issue.hit_id).or_default().push(issue);
                map
            })
    } else {
        HashMap::new()
    };

    Json(PaginatedResponse {
        results: res
            .results
            .into_iter()
            .map(|h| {
                let mut payload: HitPayload = (&h).into();
                if include_download_status {
                    payload.downloaded = Some(h.downloaded);
                }
                if include_issues && can_read_issues {
                    payload.issues = Some(issues_by_hit.get(&h.id).cloned().unwrap_or_default());
                }
                payload
            })
            .collect::<Vec<_>>(),
        total: res.total,
        start: res.start,
        end: res.end,
    })
}

/// # Get detailed hit information
///
/// Retrieve all information about a hit independent from a game.
/// Use the optional `parts` parameter to include additional hit data in the response.
/// Supported values are `issues` and `downloaded`.
/// Issue data is only returned if the caller has the `READ_ISSUES` permission.

#[openapi(tag = "Hits")]
#[get("/hits/<hit_id>?<query..>")]
pub async fn get_hit(
    hit_id: &str,
    query: HitPartsQuery,
    svc: &State<ServiceStore>,
    user: Option<UserAuthenticator>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<FullHitPayload>, GetHitError> {
    let hs = svc.hit_service();
    let hit = {
        let hsl = hs.lock();
        Uuid::parse_str(hit_id)
            .ok()
            .and_then(|hit_id| hsl.get_hit(&HitId::Id(hit_id)).cloned())
    }
    .ok_or(GetHitError {
        message: "hit id not found".into(),
        http_status_code: 404,
    })?;

    let include_download_status = includes_part(&query.parts, HitQueryPart::Downloaded);
    let include_issues = includes_part(&query.parts, HitQueryPart::Issues);
    let can_read_issues = user
        .as_ref()
        .map(|u| u.0.permissions.contains(Permissions::READ_ISSUES))
        .unwrap_or(false);

    let mut payload: FullHitPayload = (&hit).into();

    if include_download_status {
        payload.downloaded = Some(hit.downloaded);
    }

    if include_issues && can_read_issues {
        let issues = sqlx::query_as!(
            HitIssue,
            r#"
SELECT 
    id AS "id: Uuid", 
    hit_id AS "hit_id: Uuid", 
    type, 
    message, 
    created_at AS "created_at: OffsetDateTime", 
    last_modified AS "last_modified: OffsetDateTime" 
FROM hit_issues WHERE hit_id = ? ORDER BY created_at ASC"#,
            hit.id
        )
        .fetch_all(&mut **db)
        .await
        .unwrap_or_default();
        payload.issues = Some(issues);
    }

    Ok(Json(payload))
}

/// # Create a new hit issue
///
/// Report an issue for a hit. The authenticated user needs to have issue write permissions.

#[openapi(tag = "Hits")]
#[post("/hits/<hit_id>/issues", format = "json", data = "<issue>")]
pub async fn create_hit_issue(
    hit_id: Uuid,
    issue: Json<CreateHitIssuePayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<HitIssue>, CreateHitIssueError> {
    if !user.0.permissions.contains(Permissions::WRITE_ISSUES) {
        return Err(CreateHitIssueError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    if !verify_captcha(&issue.altcha_token) {
        return Err(CreateHitIssueError {
            message: "altcha solution incorrect".into(),
            http_status_code: 403,
        });
    }

    let message = issue.message.trim();
    if message.is_empty() {
        return Err(CreateHitIssueError {
            message: "issue message is required".into(),
            http_status_code: 400,
        });
    }

    let hs = serv.hit_service();

    if hs.lock().get_hit(&HitId::Id(hit_id)).is_none() {
        return Err(CreateHitIssueError {
            message: "hit not found".into(),
            http_status_code: 404,
        });
    }

    let now = OffsetDateTime::now_utc();
    let new_issue = HitIssue {
        id: Uuid::new_v4(),
        hit_id,
        r#type: HitIssueType::Custom,
        message: message.to_string(),
        created_at: now,
        last_modified: now,
    };

    if sqlx::query(
        "INSERT INTO hit_issues (id, hit_id, type, message, created_at, last_modified) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new_issue.id)
    .bind(new_issue.hit_id)
    .bind("custom")
    .bind(&new_issue.message)
    .bind(new_issue.created_at)
    .bind(new_issue.last_modified)
    .execute(&mut **db)
    .await
    .is_err()
    {
        return Err(CreateHitIssueError {
            message: "failed to create issue".into(),
            http_status_code: 500,
        });
    }

    Ok(Json(new_issue))
}

/// # Update a hit
///
/// Update a hit's info. This endpoint is only usable if the authenticated user has the permission to write hits.
/// If the YouTube ID or playback offset changed, the hit will be added to the download queue.

#[openapi(tag = "Hits")]
#[patch("/hits/<hit_id>", format = "json", data = "<hit>")]
pub async fn update_hit(
    hit_id: Uuid,
    hit: Json<FullHitPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, UpdateHitError> {
    if !user.0.permissions.contains(Permissions::WRITE_HITS) {
        return Err(UpdateHitError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    let hs = serv.hit_service();

    if hs.lock().get_hit(&HitId::Id(hit_id)).is_none() {
        return Err(UpdateHitError {
            message: "hit not found".into(),
            http_status_code: 404,
        });
    }

    let mut new_hit = Hit {
        id: hit_id,
        title: hit.title.clone(),
        artist: hit.artist.clone(),
        packs: hit.packs.clone(),
        belongs_to: hit.belongs_to.clone(),
        yt_id: hit.yt_id.clone(),
        playback_offset: hit.playback_offset,
        last_modified: OffsetDateTime::now_utc(),
        year: hit.year,
        downloaded: false,
    };
    new_hit.downloaded = new_hit.exists();

    hs.lock().remove_hit(&HitId::Id(hit_id));

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
        new_hit.downloaded,
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
        hit_id
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

    if !new_hit.downloaded {
        hs.lock().download_hit(new_hit.clone());
    }

    hs.lock().insert_hit(new_hit);

    Ok(Json(MessageResponse {
        message: "hit updated successfully".into(),
        r#type: "success".into(),
    }))
}

/// # Delete a hit
///
/// Delete a hit. The authenticated user needs to have write permissions for hits.

#[openapi(tag = "Hits")]
#[delete("/hits/<hit_id>")]
pub async fn delete_hit(
    hit_id: Uuid,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, DeleteHitError> {
    if !user.0.permissions.contains(Permissions::WRITE_HITS) {
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

/// # Delete a pack
///
/// Delete a pack from the server. The authenticated user needs to have pack write permissions.

#[openapi(tag = "Hits")]
#[delete("/hits/packs/<pack_id>")]
pub async fn delete_pack(
    pack_id: Uuid,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, DeletePackError> {
    if !user.0.permissions.contains(Permissions::WRITE_PACKS) {
        return Err(DeletePackError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    let hs = serv.hit_service();

    if hs.lock().get_pack(pack_id).is_none() {
        return Err(DeletePackError {
            message: "pack not found".into(),
            http_status_code: 404,
        });
    }

    hs.lock().remove_pack(pack_id);

    let pack = sqlx::query_as!(PackRow, "SELECT custom FROM packs WHERE id = ?", pack_id)
        .fetch_one(&mut **db)
        .await
        .unwrap();

    if pack.custom {
        let _ = sqlx::query!("DELETE FROM packs WHERE id = ?", pack_id)
            .execute(&mut **db)
            .await;
        let _ = sqlx::query!("DELETE FROM hits_packs WHERE pack_id = ?", pack_id)
            .execute(&mut **db)
            .await;
    } else {
        let _ = sqlx::query!(
            "UPDATE packs SET marked_for_deletion = ? WHERE id = ?",
            true,
            pack_id
        )
        .execute(&mut **db)
        .await;
    }

    Ok(Json(MessageResponse {
        message: "pack deleted successfully".into(),
        r#type: "success".into(),
    }))
}

/// # Create a new pack
///
/// Create a new pack. The authenticated user needs to have pack write permissions.

#[openapi(tag = "Hits")]
#[post("/hits/packs", format = "json", data = "<pack>")]
pub async fn create_pack(
    pack: Json<CreatePackPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<PackPayload>, CreatePackError> {
    if !user.0.permissions.contains(Permissions::WRITE_PACKS) {
        return Err(CreatePackError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    if sqlx::query!("SELECT * FROM packs WHERE name = ?", pack.name)
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .is_some()
    {
        return Err(CreatePackError {
            message: "a pack with that name already exists".into(),
            http_status_code: 409,
        });
    }

    let pack = Pack {
        name: pack.name.clone(),
        id: Uuid::new_v4(),
        last_modified: OffsetDateTime::now_utc(),
    };

    let _ = sqlx::query!(
        r#"
INSERT INTO packs (
    id, name, last_modified, custom, marked_for_deletion) VALUES (
    ?, ?, ?, ?, ?)"#,
        pack.id,
        pack.name,
        pack.last_modified,
        true,
        false
    )
    .execute(&mut **db)
    .await;

    let hs = serv.hit_service();

    hs.lock().insert_pack(pack.clone());

    Ok(Json(PackPayload {
        name: pack.name.clone(),
        id: pack.id,
        hits: 0,
    }))
}

/// # Update a pack
///
/// Update a pack. The authenticated user needs to have pack write permissions.

#[openapi(tag = "Hits")]
#[patch("/hits/packs/<pack_id>", format = "json", data = "<pack>")]
pub async fn update_pack(
    pack_id: Uuid,
    pack: Json<CreatePackPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, UpdatePackError> {
    if !user.0.permissions.contains(Permissions::WRITE_PACKS) {
        return Err(UpdatePackError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    let hs = serv.hit_service();

    if hs.lock().get_pack(pack_id).is_some() {
        let pack = Pack {
            name: pack.name.clone(),
            id: pack_id,
            last_modified: OffsetDateTime::now_utc(),
        };

        let _ = sqlx::query!(
            r#"
UPDATE packs SET
    name = $1,
    last_modified = $2
WHERE id = $3"#,
            pack.name,
            pack.last_modified,
            pack.id,
        )
        .execute(&mut **db)
        .await;

        hs.lock().insert_pack(pack.clone());

        Ok(Json(MessageResponse {
            message: "pack updated successfully".into(),
            r#type: "success".into(),
        }))
    } else {
        Err(UpdatePackError {
            message: "a pack with that id doesn't exist".into(),
            http_status_code: 404,
        })
    }
}

/// # Create a new hit
///
/// Create a new hit. The hit will be added to the download queue and will be available in all new games once the download finishes.
/// The authenticated user needs to have hit write permissions.

#[openapi(tag = "Hits")]
#[post("/hits", format = "json", data = "<hit>")]
pub async fn create_hit(
    hit: Json<FullHitPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<FullHitPayload>, CreateHitError> {
    if !user.0.permissions.contains(Permissions::WRITE_HITS) {
        return Err(CreateHitError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    if sqlx::query("SELECT 1 FROM hits WHERE yt_id = ?")
        .bind(&hit.yt_id)
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .is_some()
    {
        return Err(CreateHitError {
            message: "a hit for that YouTube ID already exists".into(),
            http_status_code: 409,
        });
    }

    let mut hit = Hit {
        title: hit.title.clone(),
        artist: hit.artist.clone(),
        last_modified: OffsetDateTime::now_utc(),
        id: Uuid::new_v4(),
        yt_id: hit.yt_id.clone(),
        belongs_to: hit.belongs_to.clone(),
        playback_offset: hit.playback_offset,
        year: hit.year,
        packs: hit.packs.clone(),
        downloaded: false,
    };

    hit.downloaded = hit.exists();

    let _ = sqlx::query!(
        r#"
INSERT INTO hits (
    id,
    artist,
    title,
    year,
    belongs_to,
    yt_id,
    playback_offset,
    last_modified,
    downloaded,
    custom,
    marked_for_deletion
) VALUES (
    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        hit.id,
        hit.artist,
        hit.title,
        hit.year,
        hit.belongs_to,
        hit.yt_id,
        hit.playback_offset,
        hit.last_modified,
        hit.downloaded,
        true,
        false
    )
    .execute(&mut **db)
    .await;

    for pack in hit.packs.iter() {
        let _ = sqlx::query!(
            r#"
INSERT INTO hits_packs (
    hit_id,
    pack_id,
    custom,
    marked_for_deletion) VALUES (
    ?, ?, ?, ?)"#,
            hit.id,
            pack,
            true,
            false
        )
        .execute(&mut **db)
        .await;
    }

    let hs = serv.hit_service();

    if !hit.downloaded {
        hs.lock().download_hit(hit.clone());
    }

    hs.lock().insert_hit(hit.clone());

    Ok(Json((&hit).into()))
}

/// # Export hits database
///
/// This endpoint allows authenticated users with hits write permissions to
/// export the hits database of this server in the YAML format used when
/// deploying hits within the codebase. Use this to transfer hits between server instances.
///
/// The query and pack parameters behave similarly to those within the /hits/search endpoint.

#[openapi(tag = "Hits")]
#[get("/hits/export?<query>&<pack>")]
pub async fn export_hits(
    query: Option<&str>,
    pack: Option<Vec<Uuid>>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
) -> Result<Yaml, ExportHitsError> {
    if !user.0.permissions.contains(Permissions::WRITE_HITS) {
        return Err(ExportHitsError {
            message: "permission denied".into(),
            http_status_code: 401,
        });
    }

    let hs = serv.hit_service();
    let hsl = hs.lock();

    let hsq = HitSearchQuery {
        query: query.map(|q| q.to_string()),
        packs: pack,
        start: Some(1),
        amount: Some(hsl.get_hits().len()),
        ..Default::default()
    };

    let results = hsl.search_hits(&hsq);

    let mut data = HitsterData::new(vec![], vec![]);

    results.results.iter().for_each(|h| {
        h.packs.iter().for_each(|p| {
            if data.get_pack(*p).is_none() {
                data.insert_pack(hsl.get_pack(*p).cloned().unwrap());
            }
        });
        data.insert_hit(h.clone());
    });

    Ok(Yaml(serde_yml::to_string(&data).unwrap()))
}
