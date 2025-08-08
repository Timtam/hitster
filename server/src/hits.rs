use crate::{HitsterConfig, responses::MessageResponse, services::ServiceStore};
use crossbeam_channel::unbounded;
use hitster_core::{Hit, HitsterData, Pack};
use rocket::{
    Orbit, Rocket, State,
    fairing::{Fairing, Info, Kind},
    http::Status,
    request::{FromRequest, Outcome, Request},
    serde::json::Json,
};
use rocket_db_pools::Database;
use rocket_okapi::{
    r#gen::OpenApiGenerator,
    okapi::{schemars, schemars::JsonSchema},
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use serde::Serialize;
use sqlx::FromRow;
use std::{
    collections::{HashMap, HashSet},
    convert::From,
    env,
    fs::{create_dir_all, read_dir, remove_file},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, OnceLock},
};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(FromRow)]
struct PackRow {
    name: String,
    id: Uuid,
    last_modified: OffsetDateTime,
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
}

#[derive(FromRow)]
struct HitPackRow {
    hit_id: Uuid,
    pack_id: Uuid,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct HitPayload {
    pub artist: String,
    pub title: String,
    pub belongs_to: String,
    pub year: u32,
    pub packs: Vec<Uuid>,
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

pub fn get_hitster_data() -> &'static HitsterData {
    static DATA: OnceLock<HitsterData> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_yml::from_str::<HitsterData>(include_str!("../../etc/hits.yml")).unwrap()
    })
}

struct DownloadHitData {
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
        let (s, r) = unbounded::<DownloadHitData>();
        let download_dir = Hit::download_dir();

        let _ = create_dir_all(download_dir.as_str());

        rocket::tokio::spawn({
            let db = db.clone();
            let hit_service = Arc::clone(&hit_service);
            async move {
                rocket::info!("Starting background download of hits");

                let packs = sqlx::query_as!(
                    PackRow,
                    r#"
SELECT
    id AS "id: Uuid",
    name,
    last_modified AS "last_modified: OffsetDateTime"
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
    downloaded
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
    pack_id AS "pack_id: Uuid"
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

                let hits = {
                    let mut hh = vec![];
                    for h in hits.values().map(|h| Hit {
                        title: h.title.clone(),
                        artist: h.artist.clone(),
                        id: h.id,
                        yt_id: h.yt_id.clone(),
                        year: h.year,
                        playback_offset: h.playback_offset,
                        last_modified: h.last_modified,
                        belongs_to: h.belongs_to.clone(),
                        packs: hits_packs.get(&h.id).cloned().unwrap_or_default(),
                    }) {
                        if h.exists() {
                            if !hits.get(&h.id).unwrap().downloaded {
                                let _ = sqlx::query!(
                                    "UPDATE hits SET downloaded = ? WHERE id = ?",
                                    true,
                                    h.id
                                )
                                .execute(&db)
                                .await;
                            }
                            hit_service.lock().insert_hit(h);
                        } else {
                            if hits.get(&h.id).unwrap().downloaded {
                                let _ = sqlx::query!(
                                    "UPDATE hits SET downloaded = ? WHERE id = ?",
                                    false,
                                    h.id
                                )
                                .execute(&db)
                                .await;
                            }
                            hh.push(h);
                        }
                    }
                    hh
                };

                for hit in hits.into_iter() {
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
                                .download(format!("{}/{}.opus", download_dir.as_str(), hit.yt_id))
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
                                let _ = s.send(DownloadHitData { in_file, hit });
                                return;
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

                        let output_res =
                            command.output().expect("Failed to execute ffmpeg process!");

                        if !output_res.status.success() {
                            rocket::warn!(
                                "Error downloading hit with yt-dlp: {artist}: {title}, error: {error}",
                                artist = hit.artist,
                                title = hit.title,
                                error = String::from_utf8_lossy(&output_res.stderr)
                            );
                            continue;
                        }

                        s.send(DownloadHitData { in_file, hit }).unwrap();
                    }
                }
            }
        });

        rocket::tokio::spawn({
            let db = db.clone();
            let hit_service = Arc::clone(&hit_service);
            async move {
                while let Ok(hit_data) = r.recv() {
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

                        let _ = command.output().expect("Failed to execute ffmpeg process!");

                        remove_file(hit_data.in_file).unwrap();
                    }
                    let _ = sqlx::query!(
                        "UPDATE hits SET downloaded = ? WHERE id = ?",
                        true,
                        hit_data.hit.id,
                    )
                    .execute(&db)
                    .await;
                    hit_service.lock().insert_hit(hit_data.hit);
                }

                rocket::info!("Download finished.");

                rocket::info!("Cleaning up unused hits...");

                let paths = read_dir(Hit::download_dir()).unwrap();
                let mut files: HashSet<String> = HashSet::new();

                for p in paths.flatten() {
                    files.insert(p.file_name().into_string().unwrap());
                }

                for hit in get_hitster_data().get_hits().iter() {
                    files.remove(&format!("{}_{}.mp3", hit.yt_id, hit.playback_offset));
                }

                for file in files.into_iter() {
                    let _ = remove_file(format!(
                        "{}/{}",
                        Hit::download_dir().as_str(),
                        file.as_str()
                    ));
                }

                rocket::info!("Finished cleanup.");
                hit_service.lock().set_finished_downloading();
            }
        });
    }
}

pub struct DownloadingGuard {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DownloadingGuard {
    type Error = Json<MessageResponse>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let serv = req.guard::<&State<ServiceStore>>().await.unwrap();
        let hit_service = serv.hit_service();

        if hit_service.lock().get_progress().2 {
            Outcome::Success(Self {})
        } else {
            Outcome::Error((
                Status::ServiceUnavailable,
                Json(MessageResponse {
                    message: "server is still downloading hits".into(),
                    r#type: "error".into(),
                }),
            ))
        }
    }
}

impl OpenApiFromRequest<'_> for DownloadingGuard {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}
