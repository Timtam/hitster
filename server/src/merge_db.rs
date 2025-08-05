use crate::{HitsterConfig, hits::get_hitster_data};
use hitster_core::{HitId, Pack};
use multi_key_map::MultiKeyMap;
use rocket::{
    Build, Rocket,
    fairing::{self, Fairing, Info, Kind},
};
use rocket_db_pools::{
    Database,
    sqlx::{self, FromRow},
};
use std::collections::{HashMap, HashSet};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(FromRow)]
struct PackRow {
    id: Uuid,
    name: String,
    last_modified: OffsetDateTime,
    custom: bool,
}

#[derive(FromRow)]
struct HitRow {
    id: Uuid,
    yt_id: String,
    last_modified: OffsetDateTime,
    custom: bool,
    marked_for_deletion: bool,
}

#[derive(FromRow)]
struct HitPackRow {
    hit_id: Uuid,
    pack_id: Uuid,
    custom: bool,
    marked_for_deletion: bool,
}

#[derive(Default)]
pub struct MergeDbService {}

#[rocket::async_trait]
impl Fairing for MergeDbService {
    fn info(&self) -> Info {
        Info {
            name: "Merge hits from codebase into database",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let db = HitsterConfig::fetch(&rocket).unwrap();
        let static_hits = get_hitster_data();

        rocket::debug!("Starting merging database...");

        rocket::debug!("Loaded {} static packs", static_hits.get_packs().len());
        rocket::debug!("Loaded {} static hits", static_hits.get_hits().len());

        let packs = sqlx::query_as!(
            PackRow,
            r#"
SELECT
    id AS "id: Uuid",
    name,
    last_modified AS "last_modified: OffsetDateTime",
    custom
FROM packs"#
        )
        .fetch_all(&db.0)
        .await
        .unwrap()
        .into_iter()
        .fold(HashMap::<Uuid, PackRow>::new(), |mut m, row| {
            m.insert(row.id, row);
            m
        });

        rocket::debug!("Loaded {} packs from db", packs.len());

        let hits = sqlx::query_as!(
            HitRow,
            r#"
SELECT
    id AS "id: Uuid",
    yt_id,
    last_modified AS "last_modified: OffsetDateTime",
    custom,
    marked_for_deletion
FROM hits"#
        )
        .fetch_all(&db.0)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (vec![HitId::Id(row.id), HitId::YtId(row.yt_id.clone())], row))
        .collect::<MultiKeyMap<HitId, HitRow>>();

        rocket::debug!("Loaded {} hits from db", hits.values().count());

        let static_packs = static_hits
            .get_packs()
            .into_iter()
            .map(|p| (p.id, p))
            .collect::<HashMap<Uuid, &Pack>>();

        for static_pack in static_packs.values() {
            if !packs.contains_key(&static_pack.id) {
                rocket::debug!(
                    "Inserting new pack {} ({})",
                    static_pack.name,
                    static_pack.id
                );
                let _ = sqlx::query!(
                    "
INSERT INTO packs (
    id,
    name,
    last_modified,
    custom,
    marked_for_deletion) VALUES (
    $1,
    $2,
    $3,
    $4,
    $5)",
                    static_pack.id,
                    static_pack.name,
                    static_pack.last_modified,
                    false,
                    false
                )
                .execute(&db.0)
                .await;
            } else if packs.get(&static_pack.id).unwrap().last_modified < static_pack.last_modified
            {
                rocket::debug!(
                    "Updating pack {} ({}), (old {}, new {})",
                    static_pack.name,
                    static_pack.id,
                    packs.get(&static_pack.id).unwrap().last_modified,
                    static_pack.last_modified
                );
                let _ = sqlx::query!(
                    "
UPDATE packs
SET
    name = $1,
    last_modified = $2
WHERE id = $3",
                    static_pack.name,
                    static_pack.last_modified,
                    static_pack.id
                )
                .execute(&db.0)
                .await;
            }
        }

        for pack in packs.values() {
            if !static_packs.contains_key(&pack.id) && !pack.custom {
                rocket::debug!("Deleting old pack {} ({})", pack.name, pack.id);
                let _ = sqlx::query!("DELETE FROM packs WHERE id = $1", pack.id)
                    .execute(&db.0)
                    .await;
            }
        }

        let hits_packs = sqlx::query_as!(
            HitPackRow,
            r#"
SELECT
    hit_id AS "hit_id: Uuid",
    pack_id AS "pack_id: Uuid",
    custom,
    marked_for_deletion
FROM hits_packs"#
        )
        .fetch_all(&db.0)
        .await
        .unwrap()
        .into_iter()
        .fold(HashMap::<HitId, Vec<HitPackRow>>::new(), |mut m, row| {
            if m.contains_key(&HitId::Id(row.hit_id)) {
                m.get_mut(&HitId::Id(row.hit_id)).unwrap().push(row);
            } else {
                m.insert(HitId::Id(row.hit_id), vec![row]);
            }

            m
        });

        rocket::debug!(
            "Loaded {} hits to packs assocations from db",
            hits_packs.len()
        );

        for static_hit in static_hits.get_hits().into_iter() {
            if !hits.contains_key(&HitId::Id(static_hit.id)) {
                let hit = hits.get(&HitId::YtId(static_hit.yt_id.clone()));
                if hit.is_some() {
                    // the link is already in use, but not under this id
                    // delete the entry in the db
                    rocket::debug!(
                        "Delete accidental duplicate {} (same yt id as {}: {} ({}))",
                        hit.unwrap().id,
                        static_hit.artist,
                        static_hit.title,
                        static_hit.id
                    );
                    let _ = sqlx::query!("DELETE FROM hits WHERE yt_id = $1", static_hit.yt_id)
                        .execute(&db.0)
                        .await;
                }
                // the hit is entirely new and needs to be created
                let exists = static_hit.exists();
                let marked_for_deletion = hit.map(|h| h.marked_for_deletion).unwrap_or(false);
                rocket::debug!(
                    "Insert new hit {}: {} ({})",
                    static_hit.artist,
                    static_hit.title,
                    static_hit.id
                );
                let _ = sqlx::query!(
                    "
INSERT INTO hits (
    id,
    title,
    artist,
    yt_id,
    year,
    playback_offset,
    belongs_to,
    last_modified,
    downloaded,
    custom,
    marked_for_deletion) VALUES (
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?,
    ?)",
                    static_hit.id,
                    static_hit.title,
                    static_hit.artist,
                    static_hit.yt_id,
                    static_hit.year,
                    static_hit.playback_offset,
                    static_hit.belongs_to,
                    static_hit.last_modified,
                    exists,
                    false,
                    marked_for_deletion
                )
                .execute(&db.0)
                .await;
                // and insert the packs associated
                for pack in hit
                    .and_then(|hit| hits_packs.get(&HitId::Id(hit.id)))
                    .map(|p| {
                        p.iter()
                            .map(|row| {
                                let pack = packs.get(&row.pack_id).unwrap();
                                Pack {
                                    id: row.pack_id,
                                    name: pack.name.clone(),
                                    last_modified: pack.last_modified,
                                }
                            })
                            .collect::<HashSet<_>>()
                    })
                    .unwrap_or(HashSet::new())
                    .union(
                        &static_hit
                            .packs
                            .iter()
                            .map(|p| (*static_packs.get(p).unwrap()).clone())
                            .collect::<HashSet<_>>(),
                    )
                {
                    let custom = hits_packs
                        .get(&HitId::Id(static_hit.id))
                        .and_then(|p| p.iter().find(|p| p.pack_id == pack.id).map(|p| p.custom))
                        .unwrap_or(false);
                    let marked_for_deletion = hits_packs
                        .get(&HitId::Id(static_hit.id))
                        .and_then(|p| {
                            p.iter()
                                .find(|p| p.pack_id == pack.id)
                                .map(|p| p.marked_for_deletion)
                        })
                        .unwrap_or(false);
                    rocket::debug!(
                        "Insert association of hit {} ({}: {}) with pack {} ({}) (custom: {})",
                        static_hit.artist,
                        static_hit.title,
                        static_hit.id,
                        pack.name,
                        pack.id,
                        custom
                    );
                    let _ = sqlx::query!(
                        "
INSERT INTO hits_packs (
    hit_id, 
    pack_id, 
    custom, 
    marked_for_deletion) VALUES (
    $1,
    $2,
    $3,
    $4)",
                        static_hit.id,
                        pack.id,
                        custom,
                        marked_for_deletion
                    )
                    .execute(&db.0)
                    .await;
                }
                continue; // no need to remove unnecessary associations
            } else if let Some(hit) = hits.get(&HitId::Id(static_hit.id))
                && hit.last_modified < static_hit.last_modified
            {
                // the hit exists and got updated in the meantime
                rocket::debug!(
                    "Updating hit {}: {} ({}) (old {}, new {})",
                    static_hit.artist,
                    static_hit.title,
                    static_hit.id,
                    hit.last_modified,
                    static_hit.last_modified
                );
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
    downloaded = $8,
    custom = $9
    WHERE id = $10",
                    static_hit.title,
                    static_hit.artist,
                    static_hit.yt_id,
                    static_hit.year,
                    static_hit.playback_offset,
                    static_hit.belongs_to,
                    static_hit.last_modified,
                    false,
                    false,
                    static_hit.id,
                )
                .execute(&db.0)
                .await;
            }
            // we will check for new and old associations either if the hit was updated or inserted anew
            // check for new associations
            for pack in static_hit.packs.iter() {
                if !hits_packs
                    .get(&HitId::Id(static_hit.id))
                    .map(|packs| packs.iter().find(|p| p.pack_id == *pack).is_some())
                    .unwrap_or(false)
                {
                    rocket::debug!(
                        "Insert new association of hit {}: {} ({}) with pack {} ({})",
                        static_hit.artist,
                        static_hit.title,
                        static_hit.id,
                        static_hits.get_pack(*pack).unwrap().name,
                        pack
                    );
                    let _ = sqlx::query!(
                        "
INSERT INTO hits_packs (
    hit_id, 
    pack_id, 
    custom, 
    marked_for_deletion) VALUES (
    ?, 
    ?, 
    ?, 
    ?)",
                        static_hit.id,
                        pack,
                        false,
                        false
                    )
                    .execute(&db.0)
                    .await;
                }
            }
            // check for deleted associations
            for pack in hits_packs.get(&HitId::Id(static_hit.id)).unwrap().iter() {
                if !pack.custom && !static_hit.packs.contains(&pack.pack_id) {
                    // this association has been removed in the static dataset
                    rocket::debug!(
                        "Delete dangling association from hit {}: {} ({}) to pack {} ({})",
                        static_hit.artist,
                        static_hit.title,
                        static_hit.id,
                        packs.get(&pack.pack_id).unwrap().name,
                        pack.pack_id
                    );
                    let _ = sqlx::query!(
                        "DELETE FROM hits_packs WHERE hit_id = ? AND pack_id = ?",
                        pack.hit_id,
                        pack.pack_id
                    )
                    .execute(&db.0)
                    .await;
                }
            }
        }

        for hit in hits.values() {
            if static_hits.get_hit(&HitId::Id(hit.id)).is_none() && !hit.custom {
                rocket::debug!("Deleting old hit {}", hit.id);
                let _ = sqlx::query!("DELETE FROM hits WHERE id = $1", hit.id)
                    .execute(&db.0)
                    .await;
            }
        }

        rocket::debug!("Finished merging database.");

        Ok(rocket)
    }
}
