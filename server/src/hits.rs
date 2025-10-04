use crate::{GlobalEvent, HitsterConfig, services::ServiceStore};
use async_process::Command;
use hitster_core::{Hit, HitId, HitsterData, Pack};
use rocket::{
    Orbit, Rocket,
    fairing::{Fairing, Info, Kind},
    tokio::{
        select,
        sync::broadcast::{Sender, channel, error::RecvError},
    },
};
use rocket_db_pools::Database;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{
    collections::{HashMap, HashSet},
    convert::From,
    env,
    fs::{create_dir_all, read_dir, remove_file},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(FromRow)]
struct PackRow {
    name: String,
    id: Uuid,
    last_modified: OffsetDateTime,
    custom: bool,
}

#[derive(FromRow)]
struct HitRow {
    id: Uuid,
    title: String,
    artist: String,
    yt_id: String,
    belongs_to: String,
    year: u32,
    playback_offset: u16,
    last_modified: OffsetDateTime,
    downloaded: bool,
    custom: bool,
}

#[derive(FromRow)]
struct HitPackRow {
    hit_id: Uuid,
    pack_id: Uuid,
    custom: bool,
}

/// a hit metadata relevant in a game

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct HitPayload {
    /// artist of the song
    pub artist: String,
    /// title of the song
    pub title: String,
    /// any movie, musical or whatever the song is known for
    pub belongs_to: String,
    /// the year the song was released in
    pub year: u32,
    /// the packs the song lives in
    pub packs: Vec<Uuid>,
    /// the unique hit id
    pub id: Uuid,
}

impl From<&Hit> for HitPayload {
    fn from(hit: &Hit) -> Self {
        Self {
            artist: hit.artist.clone(),
            title: hit.title.clone(),
            belongs_to: hit.belongs_to.clone(),
            year: hit.year,
            packs: hit.packs.clone(),
            id: hit.id,
        }
    }
}

/// the full hit dataset as it is used within the Hits-specific endpoints

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema, Deserialize)]
pub struct FullHitPayload {
    /// artist of the song
    pub artist: String,
    /// title of the song
    pub title: String,
    /// any movie, musical or whatever the song is known for
    pub belongs_to: String,
    /// the year the song was released in
    pub year: u32,
    /// the packs the song lives in
    pub packs: Vec<Uuid>,
    /// the YouTube video time offset at which the song starts playing
    pub playback_offset: u16,
    /// the unique hit id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    /// the YouTube video ID
    pub yt_id: String,
}

impl From<&Hit> for FullHitPayload {
    fn from(hit: &Hit) -> Self {
        Self {
            artist: hit.artist.clone(),
            title: hit.title.clone(),
            belongs_to: hit.belongs_to.clone(),
            year: hit.year,
            packs: hit.packs.clone(),
            id: Some(hit.id),
            yt_id: hit.yt_id.clone(),
            playback_offset: hit.playback_offset,
        }
    }
}

pub fn get_hitster_data() -> &'static HitsterData {
    static DATA: OnceLock<HitsterData> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_yml::from_str::<HitsterData>(include_str!("../../etc/hits.yml")).unwrap()
    })
}

pub async fn get_hitster_data_from_db<T>(e: &mut T, custom: bool) -> HitsterData
where
    for<'e> &'e mut T: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let mut data = HitsterData::new(vec![], vec![]);
    let packs = sqlx::query_as!(
        PackRow,
        r#"
SELECT
    id AS "id: Uuid",
    name,
    last_modified AS "last_modified: OffsetDateTime",
    custom
FROM packs WHERE marked_for_deletion = ?"#,
        false,
    )
    .fetch_all(&mut *e)
    .await
    .unwrap();

    for pack in packs.iter() {
        if custom || !pack.custom {
            data.insert_pack(Pack {
                id: pack.id,
                name: pack.name.clone(),
                last_modified: pack.last_modified,
            });
        }
    }

    let hits = sqlx::query_as!(
        HitRow,
        r#"
SELECT
    id AS "id: Uuid",
    title,
    artist,
    yt_id,
    belongs_to,
    year AS "year: u32",
    playback_offset AS "playback_offset: u16",
    last_modified AS "last_modified: OffsetDateTime",
    downloaded,
    custom
FROM hits WHERE marked_for_deletion = ?"#,
        false,
    )
    .fetch_all(&mut *e)
    .await
    .unwrap()
    .into_iter()
    .map(|h| (h.id, h))
    .collect::<HashMap<Uuid, HitRow>>();

    let hits_packs = sqlx::query_as!(
        HitPackRow,
        r#"
SELECT
    hit_id AS "hit_id: Uuid",
    pack_id AS "pack_id: Uuid",
    custom
FROM hits_packs WHERE marked_for_deletion = ?"#,
        false,
    )
    .fetch_all(&mut *e)
    .await
    .unwrap()
    .into_iter()
    .fold(HashMap::<Uuid, Vec<Uuid>>::new(), |mut m, h| {
        if custom || !h.custom {
            m.entry(h.hit_id).or_default().push(h.pack_id);
        }
        m
    });

    for hit in hits.values().filter(|h| custom || !h.custom).map(|h| Hit {
        title: h.title.clone(),
        artist: h.artist.clone(),
        id: h.id,
        yt_id: h.yt_id.clone(),
        year: h.year,
        playback_offset: h.playback_offset,
        last_modified: h.last_modified,
        belongs_to: h.belongs_to.clone(),
        packs: hits_packs.get(&h.id).cloned().unwrap_or_default(),
        downloaded: h.downloaded,
    }) {
        data.insert_hit(hit);
    }

    data
}

#[derive(Copy, Clone, Deserialize, Eq, JsonSchema, PartialEq, FromFormField)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[field(value = "ascending")]
    Ascending,
    #[field(value = "descending")]
    Descending,
}

/// sorting criteria for searching hits

#[derive(Clone, Deserialize, JsonSchema, FromFormField, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[field(value = "title")]
    Title,
    #[field(value = "artist")]
    Artist,
    #[field(value = "year")]
    Year,
    #[field(value = "belongs_to")]
    BelongsTo,
}

/// a search query for searching hits

#[derive(Deserialize, JsonSchema, FromForm)]
pub struct HitSearchQuery {
    /// The sorting criteria that determine the order of returned hits. Earlier criteria have priority.
    pub sort_by: Option<Vec<SortBy>>,
    /// wether to sort in ascending or descending order
    pub sort_direction: Option<SortDirection>,
    /// a text to search for in title, artist and belongs to fields. fuzzy search is applied so the result might not be 100% accurate on purpose.
    pub query: Option<String>,
    /// the packs that you want to limit the search to
    pub packs: Option<Vec<Uuid>>,
    /// the start of the pagination (default 1)
    pub start: Option<usize>,
    /// amount of search results you want to get
    pub amount: Option<usize>,
}

impl Default for HitSearchQuery {
    fn default() -> Self {
        Self {
            sort_by: Some(vec![SortBy::Title]),
            sort_direction: Some(SortDirection::Ascending),
            query: Some(String::from("")),
            packs: Some(vec![]),
            start: Some(1),
            amount: Some(50),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DownloadHitData {
    in_file: PathBuf,
    hit: Hit,
}

#[derive(Default)]
pub struct HitDownloadService {}

#[rocket::async_trait]
impl Fairing for HitDownloadService {
    fn info(&self) -> Info {
        Info {
            name: "Download hits in background",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let db = HitsterConfig::fetch(rocket).unwrap().0.clone();
        let hit_service = Arc::new(rocket.state::<ServiceStore>().unwrap().hit_service());
        let dl_sender = channel::<Hit>(100000).0;
        let process_sender = channel::<DownloadHitData>(100000).0;
        let event_sender = Arc::new(rocket.state::<Sender<GlobalEvent>>().unwrap().clone());
        let _ = create_dir_all(Hit::download_dir().as_str());

        hit_service
            .lock()
            .set_download_info(dl_sender.clone(), process_sender.clone());

        rocket::tokio::spawn({
            let db = db.clone();
            let dl_sender = dl_sender.clone();
            let event_sender = Arc::clone(&event_sender);
            let hit_service = Arc::clone(&hit_service);
            async move {
                let paths = read_dir(Hit::download_dir()).unwrap();
                let mut files: HashSet<String> = HashSet::new();

                for p in paths.flatten() {
                    files.insert(p.file_name().into_string().unwrap());
                }

                let packs = sqlx::query_as!(
                    PackRow,
                    r#"
SELECT
    id AS "id: Uuid",
    name,
    last_modified AS "last_modified: OffsetDateTime",
    custom
FROM packs WHERE marked_for_deletion = ?"#,
                    false
                )
                .fetch_all(&db)
                .await
                .unwrap();

                for pack in packs.iter() {
                    hit_service.lock().insert_pack(Pack {
                        id: pack.id,
                        name: pack.name.clone(),
                        last_modified: pack.last_modified,
                    });
                }

                let hits = sqlx::query_as!(
                    HitRow,
                    r#"
SELECT
    id AS "id: Uuid",
    title,
    artist,
    yt_id,
    belongs_to,
    year AS "year: u32",
    playback_offset AS "playback_offset: u16",
    last_modified AS "last_modified: OffsetDateTime",
    downloaded,
    custom
FROM hits WHERE marked_for_deletion = ?"#,
                    false
                )
                .fetch_all(&db)
                .await
                .unwrap()
                .into_iter()
                .map(|h| (h.id, h))
                .collect::<HashMap<Uuid, HitRow>>();

                let hits_packs = sqlx::query_as!(
                    HitPackRow,
                    r#"
SELECT
    hit_id AS "hit_id: Uuid",
    pack_id AS "pack_id: Uuid",
    custom
FROM hits_packs WHERE marked_for_deletion = ?"#,
                    false
                )
                .fetch_all(&db)
                .await
                .unwrap()
                .into_iter()
                .fold(HashMap::<Uuid, Vec<Uuid>>::new(), |mut m, h| {
                    m.entry(h.hit_id).or_default().push(h.pack_id);
                    m
                });

                for mut hit in hits.values().map(|h| Hit {
                    title: h.title.clone(),
                    artist: h.artist.clone(),
                    id: h.id,
                    yt_id: h.yt_id.clone(),
                    year: h.year,
                    playback_offset: h.playback_offset,
                    last_modified: h.last_modified,
                    belongs_to: h.belongs_to.clone(),
                    packs: hits_packs.get(&h.id).cloned().unwrap_or_default(),
                    downloaded: h.downloaded,
                }) {
                    if hit.exists() {
                        files.remove(&format!("{}_{}.mp3", hit.yt_id, hit.playback_offset));
                        if !hit.downloaded {
                            let _ = sqlx::query!(
                                "UPDATE hits SET downloaded = ? WHERE id = ?",
                                true,
                                hit.id
                            )
                            .execute(&db)
                            .await;
                            hit.downloaded = true;
                        }
                    } else {
                        if hit.downloaded {
                            let _ = sqlx::query!(
                                "UPDATE hits SET downloaded = ? WHERE id = ?",
                                false,
                                hit.id
                            )
                            .execute(&db)
                            .await;
                            hit.downloaded = false;
                        }
                        let _ = dl_sender.send(hit.clone());
                        let available = hit_service
                            .lock()
                            .get_hits()
                            .iter()
                            .filter(|h| h.downloaded)
                            .count();
                        let downloading = hit_service.lock().downloading();
                        let processing = hit_service.lock().processing();
                        let _ = event_sender.send(GlobalEvent::ProcessHits {
                            available,
                            downloading,
                            processing,
                        });
                    }
                    hit_service.lock().insert_hit(hit);
                }

                for file in files.into_iter() {
                    let _ = remove_file(format!(
                        "{}/{}",
                        Hit::download_dir().as_str(),
                        file.as_str()
                    ));
                }
            }
        });

        rocket::tokio::spawn({
            let dl_sender = dl_sender.clone();
            let event_sender = Arc::clone(&event_sender);
            let hit_service = Arc::clone(&hit_service);
            let process_sender = process_sender.clone();
            async move {
                rocket::info!("Starting background download of hits");
                let mut rx = dl_sender.subscribe();

                loop {
                    let hit = select! {
                        hit = rx.recv() => match hit {
                            Ok(hit) => hit,
                            Err(RecvError::Closed) => break,
                            Err(RecvError::Lagged(_)) => continue,
                        },
                    };

                    hit_service.lock().set_downloading(true);
                    let available = hit_service
                        .lock()
                        .get_hits()
                        .iter()
                        .filter(|h| h.downloaded)
                        .count();
                    let downloading = hit_service.lock().downloading();
                    let processing = hit_service.lock().processing();
                    let _ = event_sender.send(GlobalEvent::ProcessHits {
                        available,
                        downloading,
                        processing,
                    });
                    #[cfg(feature = "native_dl")]
                    {
                        use filesize::PathExt;
                        use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};

                        let in_file =
                            Path::new(&Hit::download_dir()).join(format!("{}.opus", hit.yt_id));

                        let options = VideoOptions {
                            quality: VideoQuality::HighestAudio,
                            filter: VideoSearchOptions::Audio,
                            ..Default::default()
                        };
                        if let Ok(video) = Video::new_with_options(hit.yt_id.as_str(), options) {
                            let in_dl = video
                                .download(format!(
                                    "{}/{}.opus",
                                    Hit::download_dir().as_str(),
                                    hit.yt_id
                                ))
                                .await;

                            if in_dl.is_err()
                                || !in_file.is_file()
                                || in_file.size_on_disk().unwrap_or(0) == 0
                            {
                                if in_dl.is_err() {
                                    rocket::warn!(
                                        "Error downloading hit with rusty_ytdl: {artist}: {title}, error: {error}",
                                        artist = hit.artist,
                                        title = hit.title,
                                        error = in_dl.unwrap_err()
                                    );
                                }
                                if in_file.is_file() {
                                    remove_file(&in_file).unwrap();
                                }
                            } else {
                                let _ = process_sender.send(DownloadHitData { in_file, hit });
                                hit_service.lock().set_downloading(!dl_sender.is_empty());
                                let available = hit_service
                                    .lock()
                                    .get_hits()
                                    .iter()
                                    .filter(|h| h.downloaded)
                                    .count();
                                let downloading = hit_service.lock().downloading();
                                let processing = hit_service.lock().processing();
                                let _ = event_sender.send(GlobalEvent::ProcessHits {
                                    available,
                                    downloading,
                                    processing,
                                });
                                continue;
                            }
                        }
                    }

                    #[cfg(feature = "yt_dl")]
                    {
                        let in_file =
                            Path::new(&Hit::download_dir()).join(format!("{}.m4a", hit.yt_id));

                        let mut command = Command::new("yt-dlp");
                        command
                            .current_dir(env::current_dir().unwrap())
                            .args(["-f", "bestaudio[ext=m4a]"])
                            .args(["-o", in_file.to_str().unwrap()])
                            .args(["--extractor-args", "youtube:player-client=default,mweb"])
                            .arg(format!("https://www.youtube.com/watch?v={}", hit.yt_id));

                        let output = command.output().await;

                        if let Ok(ref output_res) = output
                            && !output_res.status.success()
                        {
                            rocket::warn!(
                                "Error downloading hit with yt-dlp: {artist}: {title}, error: {error}",
                                artist = hit.artist,
                                title = hit.title,
                                error = String::from_utf8_lossy(&output_res.stderr)
                            );
                        } else if output.is_err() {
                            rocket::warn!(
                                "error when trying to run yt-dlp. Maybe it isn't installed?"
                            );
                        } else {
                            process_sender
                                .send(DownloadHitData { in_file, hit })
                                .unwrap();
                        }
                    }
                    hit_service.lock().set_downloading(!dl_sender.is_empty());
                    let available = hit_service
                        .lock()
                        .get_hits()
                        .iter()
                        .filter(|h| h.downloaded)
                        .count();
                    let downloading = hit_service.lock().downloading();
                    let processing = hit_service.lock().processing();
                    let _ = event_sender.send(GlobalEvent::ProcessHits {
                        available,
                        downloading,
                        processing,
                    });
                }
            }
        });

        rocket::tokio::spawn({
            let db = db.clone();
            let event_sender = Arc::clone(&event_sender);
            let hit_service = Arc::clone(&hit_service);
            let process_sender = process_sender.clone();
            async move {
                let mut rx = process_sender.subscribe();

                loop {
                    let hit_data = select! {
                        hit_data = rx.recv() => match hit_data {
                            Ok(hit_data) => hit_data,
                            Err(RecvError::Closed) => break,
                            Err(RecvError::Lagged(_)) => continue,
                        },
                    };

                    hit_service.lock().set_processing(true);
                    let available = hit_service
                        .lock()
                        .get_hits()
                        .iter()
                        .filter(|h| h.downloaded)
                        .count();
                    let downloading = hit_service.lock().downloading();
                    let processing = hit_service.lock().processing();
                    let _ = event_sender.send(GlobalEvent::ProcessHits {
                        available,
                        downloading,
                        processing,
                    });
                    if !hit_data.hit.exists() {
                        let out_file = Path::new(&Hit::download_dir()).join(format!(
                            "{}_{}.mp3",
                            hit_data.hit.yt_id, hit_data.hit.playback_offset
                        ));

                        let mut command = Command::new("ffmpeg-normalize");
                        command
                            .current_dir(env::current_dir().unwrap())
                            .arg(&hit_data.in_file)
                            .args(["-ar", "44100"])
                            .args(["-b:a", "128k"])
                            .args(["-c:a", "libmp3lame"])
                            .args(["-e", &format!("-ss {}", hit_data.hit.playback_offset)])
                            .args(["--extension", "mp3"])
                            .args(["-o", out_file.to_str().unwrap()])
                            .arg("-sn")
                            .args(["-t", "-18.0"])
                            .arg("-vn");

                        let _ = command
                            .output()
                            .await
                            .expect("Failed to execute ffmpeg process!");

                        remove_file(hit_data.in_file).unwrap();
                    }
                    let _ = sqlx::query!(
                        "UPDATE hits SET downloaded = ? WHERE id = ?",
                        true,
                        hit_data.hit.id,
                    )
                    .execute(&db)
                    .await;
                    let mut hs = hit_service.lock();
                    if hs.remove_hit(&HitId::Id(hit_data.hit.id)) {
                        // only insert the hit if it could be removed
                        // this makes sure that we only insert the hit if it is still available
                        // e.g. when the hit got deleted while the hit is still downloading, it will not be within the hit service anymore
                        // and we thus won't insert it here anymore either
                        hs.insert_hit(hit_data.hit);
                    }
                    hs.set_processing(!process_sender.is_empty());
                    let available = hs.get_hits().iter().filter(|h| h.downloaded).count();
                    let downloading = hs.downloading();
                    let processing = hs.processing();

                    drop(hs);

                    let _ = event_sender.send(GlobalEvent::ProcessHits {
                        available,
                        downloading,
                        processing,
                    });
                }
            }
        });
    }
}

/// information necessary for creating a new pack

#[derive(Deserialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct CreatePackPayload {
    /// the name of the new pack
    pub name: String,
}
