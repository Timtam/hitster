use crate::{GlobalEvent, HitsterConfig, services::ServiceStore};
use async_process::Command;
use hitster_core::{Hit, HitId, HitIssue, HitIssueType, HitsterData, Pack};
use rocket::{
    Orbit, Rocket,
    fairing::{Fairing, Info, Kind},
    tokio::{
        select,
        sync::{
            Mutex, Semaphore,
            broadcast::{Sender, channel, error::RecvError},
        },
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
#[cfg(feature = "yt_dl")]
use time::Duration;
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
    /// whether the hit has been downloaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded: Option<bool>,
    /// any issues reported for the hit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<HitIssue>>,
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
            downloaded: None,
            issues: None,
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
    /// whether the hit has been downloaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded: Option<bool>,
    /// any issues reported for the hit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<HitIssue>>,
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
            downloaded: None,
            issues: None,
        }
    }
}

#[derive(Copy, Clone, Deserialize, Eq, JsonSchema, PartialEq, FromFormField, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HitQueryPart {
    #[field(value = "issues")]
    Issues,
    #[field(value = "downloaded")]
    Downloaded,
}

pub fn get_hitster_data() -> &'static HitsterData {
    static DATA: OnceLock<HitsterData> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_yml::from_str::<HitsterData>(include_str!("../../etc/hits.yml")).unwrap()
    })
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

/// optional filters for searching hits

#[derive(Copy, Clone, Deserialize, Eq, JsonSchema, PartialEq, FromFormField, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HitSearchFilter {
    #[field(value = "has_issues")]
    HasIssues,
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
    /// optional parts to include in the response
    pub parts: Option<Vec<HitQueryPart>>,
    /// optional filters to apply before pagination
    pub filters: Option<Vec<HitSearchFilter>>,
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
            parts: None,
            filters: None,
        }
    }
}

#[derive(Deserialize, JsonSchema, FromForm)]
pub struct HitPartsQuery {
    /// optional parts to include in the response
    pub parts: Option<Vec<HitQueryPart>>,
}

#[derive(Clone, Debug)]
pub struct DownloadHitData {
    in_file: PathBuf,
    hit: Hit,
}

const UNAVAILABLE_ISSUE_MESSAGE: &str = "youtube video is unavailable";
const DOWNLOAD_FAILED_ISSUE_MESSAGE: &str = "hit failed to download";

#[derive(Default)]
pub struct HitDownloadService {}

#[cfg(feature = "yt_dl")]
async fn check_hit_availability(hit: &Hit) -> Result<bool, String> {
    let mut command = Command::new("yt-dlp");
    command
        .current_dir(env::current_dir().unwrap())
        .args(["--skip-download", "--no-warnings", "--no-progress"])
        .args(["--extractor-args", "youtube:player-client=default,mweb"])
        .arg(format!("https://www.youtube.com/watch?v={}", hit.yt_id));

    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let combined = format!("{stderr}\n{stdout}");
                let unavailable_markers = [
                    "Video unavailable",
                    "This video is unavailable",
                    "Private video",
                    "This video is private",
                ];
                if unavailable_markers.iter().any(|m| combined.contains(m)) {
                    Ok(false)
                } else {
                    Err(combined)
                }
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(feature = "yt_dl")]
async fn ensure_yt_dlp_is_updated(update_time: &Arc<Mutex<OffsetDateTime>>) {
    let mut last_update_time = update_time.lock().await;
    let now = OffsetDateTime::now_utc();

    if *last_update_time + Duration::hours(12) >= now {
        return;
    }

    let mut command = Command::new("yt-dlp");
    command.current_dir(env::current_dir().unwrap()).arg("-U");

    let output = command.output().await;

    if let Ok(ref output_res) = output
        && !output_res.status.success()
    {
        rocket::warn!(
            "Error updating yt-dlp. error: {error}",
            error = String::from_utf8_lossy(&output_res.stderr)
        );
    } else if output.is_err() {
        rocket::warn!("error when trying to run yt-dlp. Maybe it isn't installed?");
    }

    *last_update_time = OffsetDateTime::now_utc();
}

#[cfg(all(not(feature = "yt_dl"), feature = "native_dl"))]
async fn check_hit_availability(hit: &Hit) -> Result<bool, String> {
    use rusty_ytdl::{Video, VideoError};

    let video = Video::new(hit.yt_id.as_str()).map_err(|e| e.to_string())?;

    match video.get_basic_info().await {
        Ok(_) => Ok(true),
        Err(VideoError::VideoNotFound)
        | Err(VideoError::VideoSourceNotFound)
        | Err(VideoError::VideoIsPrivate) => Ok(false),
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(all(not(feature = "yt_dl"), not(feature = "native_dl")))]
async fn check_hit_availability(_hit: &Hit) -> Result<bool, String> {
    Err("no youtube availability checker configured".to_string())
}

async fn upsert_auto_issue(
    db: &sqlx::SqlitePool,
    event_sender: &Sender<GlobalEvent>,
    hit_id: Uuid,
    message: &str,
) {
    let now = OffsetDateTime::now_utc();
    let existing = sqlx::query_scalar::<_, String>(
        "SELECT id FROM hit_issues WHERE hit_id = ? AND type = 'auto' AND message = ?",
    )
    .bind(hit_id)
    .bind(message)
    .fetch_optional(db)
    .await;

    match existing {
        Ok(Some(issue_id)) => {
            if let Err(err) = sqlx::query("UPDATE hit_issues SET last_modified = ? WHERE id = ?")
                .bind(now)
                .bind(issue_id)
                .execute(db)
                .await
            {
                rocket::warn!(
                    "Failed to update auto hit issue for {hit_id}: {err}",
                    hit_id = hit_id,
                    err = err
                );
            }
        }
        Ok(None) => {
            let new_issue = HitIssue {
                id: Uuid::new_v4(),
                hit_id,
                r#type: HitIssueType::Auto,
                message: message.to_string(),
                created_at: now,
                last_modified: now,
            };
            if let Err(err) = sqlx::query(
                "INSERT INTO hit_issues (id, hit_id, type, message, created_at, last_modified) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(new_issue.id)
            .bind(hit_id)
            .bind("auto")
            .bind(message)
            .bind(now)
            .bind(now)
            .execute(db)
            .await
            {
                rocket::warn!(
                    "Failed to insert auto hit issue for {hit_id}: {err}",
                    hit_id = hit_id,
                    err = err
                );
            } else {
                let _ = event_sender.send(GlobalEvent::CreateHitIssue(new_issue));
            }
        }
        Err(err) => {
            rocket::warn!(
                "Failed to query hit issues for {hit_id}: {err}",
                hit_id = hit_id,
                err = err
            );
        }
    }
}

async fn clear_auto_issue(
    db: &sqlx::SqlitePool,
    event_sender: &Sender<GlobalEvent>,
    hit_id: Uuid,
    message: &str,
) {
    match sqlx::query_scalar::<_, String>(
        "SELECT id FROM hit_issues WHERE hit_id = ? AND type = 'auto' AND message = ?",
    )
    .bind(hit_id)
    .bind(message)
    .fetch_all(db)
    .await
    {
        Ok(issue_ids) => {
            for issue_id in issue_ids {
                match sqlx::query("DELETE FROM hit_issues WHERE hit_id = ? AND id = ?")
                    .bind(hit_id)
                    .bind(&issue_id)
                    .execute(db)
                    .await
                {
                    Ok(result) if result.rows_affected() > 0 => match Uuid::parse_str(&issue_id) {
                        Ok(issue_uuid) => {
                            let _ = event_sender.send(GlobalEvent::DeleteHitIssue {
                                hit_id,
                                issue_id: issue_uuid,
                            });
                        }
                        Err(err) => {
                            rocket::warn!(
                                "Failed to parse auto hit issue id {issue_id} for {hit_id}: {err}",
                                issue_id = issue_id,
                                hit_id = hit_id,
                                err = err
                            );
                        }
                    },
                    Ok(_) => {}
                    Err(err) => {
                        rocket::warn!(
                            "Failed to clear auto hit issue {issue_id} for {hit_id}: {err}",
                            issue_id = issue_id,
                            hit_id = hit_id,
                            err = err
                        );
                    }
                }
            }
        }
        Err(err) => {
            rocket::warn!(
                "Failed to query auto hit issues for {hit_id}: {err}",
                hit_id = hit_id,
                err = err
            );
        }
    }
}

async fn upsert_unavailable_issue(
    db: &sqlx::SqlitePool,
    event_sender: &Sender<GlobalEvent>,
    hit_id: Uuid,
) {
    upsert_auto_issue(db, event_sender, hit_id, UNAVAILABLE_ISSUE_MESSAGE).await;
}

async fn clear_unavailable_issue(
    db: &sqlx::SqlitePool,
    event_sender: &Sender<GlobalEvent>,
    hit_id: Uuid,
) {
    clear_auto_issue(db, event_sender, hit_id, UNAVAILABLE_ISSUE_MESSAGE).await;
}

async fn upsert_download_failed_issue(
    db: &sqlx::SqlitePool,
    event_sender: &Sender<GlobalEvent>,
    hit_id: Uuid,
) {
    upsert_auto_issue(db, event_sender, hit_id, DOWNLOAD_FAILED_ISSUE_MESSAGE).await;
}

async fn clear_download_failed_issue(
    db: &sqlx::SqlitePool,
    event_sender: &Sender<GlobalEvent>,
    hit_id: Uuid,
) {
    clear_auto_issue(db, event_sender, hit_id, DOWNLOAD_FAILED_ISSUE_MESSAGE).await;
}

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
        let availability_sender = channel::<Hit>(100000).0;
        let event_sender = Arc::new(rocket.state::<Sender<GlobalEvent>>().unwrap().clone());
        #[cfg(feature = "yt_dl")]
        let yt_dlp_update_time = Arc::new(Mutex::new(OffsetDateTime::UNIX_EPOCH));
        let _ = create_dir_all(Hit::download_dir().as_str());

        hit_service
            .lock()
            .set_download_info(dl_sender.clone(), process_sender.clone());
        hit_service
            .lock()
            .set_availability_sender(availability_sender.clone());

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
            let db = db.clone();
            let mut rx = availability_sender.subscribe();
            let in_flight: Arc<Mutex<HashSet<Uuid>>> = Arc::new(Mutex::new(HashSet::new()));
            let max_checks = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            let semaphore = Arc::new(Semaphore::new(max_checks));
            let event_sender = Arc::clone(&event_sender);
            #[cfg(feature = "yt_dl")]
            let yt_dlp_update_time = Arc::clone(&yt_dlp_update_time);

            async move {
                loop {
                    let hit = select! {
                        hit = rx.recv() => match hit {
                            Ok(hit) => hit,
                            Err(RecvError::Closed) => break,
                            Err(RecvError::Lagged(_)) => continue,
                        },
                    };

                    {
                        let mut guard = in_flight.lock().await;
                        if !guard.insert(hit.id) {
                            continue;
                        }
                    }

                    let db = db.clone();
                    let in_flight = Arc::clone(&in_flight);
                    let event_sender = Arc::clone(&event_sender);
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    #[cfg(feature = "yt_dl")]
                    let yt_dlp_update_time = Arc::clone(&yt_dlp_update_time);

                    rocket::tokio::spawn(async move {
                        let _permit = permit;
                        let hit_id = hit.id;
                        #[cfg(feature = "yt_dl")]
                        ensure_yt_dlp_is_updated(&yt_dlp_update_time).await;
                        let result = check_hit_availability(&hit).await;

                        match result {
                            Ok(true) => {
                                clear_unavailable_issue(&db, event_sender.as_ref(), hit_id).await;
                            }
                            Ok(false) => {
                                upsert_unavailable_issue(&db, event_sender.as_ref(), hit_id).await;
                            }
                            Err(err) => {
                                rocket::warn!(
                                    "Availability check failed for {hit_id}: {err}",
                                    hit_id = hit_id,
                                    err = err
                                );
                            }
                        }

                        let mut guard = in_flight.lock().await;
                        guard.remove(&hit_id);
                    });
                }
            }
        });

        rocket::tokio::spawn({
            let db = db.clone();
            let dl_sender = dl_sender.clone();
            let event_sender = Arc::clone(&event_sender);
            let hit_service = Arc::clone(&hit_service);
            let process_sender = process_sender.clone();
            #[cfg(feature = "yt_dl")]
            let yt_dlp_update_time = Arc::clone(&yt_dlp_update_time);
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
                                if let Err(in_dl) = in_dl {
                                    rocket::warn!(
                                        "Error downloading hit with rusty_ytdl: {artist}: {title}, error: {error}",
                                        artist = &hit.artist,
                                        title = &hit.title,
                                        error = in_dl
                                    );
                                }
                                upsert_download_failed_issue(&db, event_sender.as_ref(), hit.id)
                                    .await;
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
                        } else {
                            rocket::warn!(
                                "Error initializing rusty_ytdl for {artist}: {title}",
                                artist = &hit.artist,
                                title = &hit.title
                            );
                            upsert_download_failed_issue(&db, event_sender.as_ref(), hit.id).await;
                        }
                    }

                    #[cfg(feature = "yt_dl")]
                    {
                        ensure_yt_dlp_is_updated(&yt_dlp_update_time).await;
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

                        if let Ok(ref output_res) = output {
                            if !output_res.status.success() {
                                rocket::warn!(
                                    "Error downloading hit with yt-dlp: {artist}: {title}, error: {error}",
                                    artist = &hit.artist,
                                    title = &hit.title,
                                    error = String::from_utf8_lossy(&output_res.stderr)
                                );
                                upsert_download_failed_issue(&db, event_sender.as_ref(), hit.id)
                                    .await;
                            } else {
                                process_sender
                                    .send(DownloadHitData { in_file, hit })
                                    .unwrap();
                            }
                        } else {
                            rocket::warn!(
                                "error when trying to run yt-dlp. Maybe it isn't installed?"
                            );
                            upsert_download_failed_issue(&db, event_sender.as_ref(), hit.id).await;
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
                    let mut hit_data = select! {
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
                            .args(["--preset", "streaming-video"])
                            .args(["-ar", "44100"])
                            .args(["-b:a", "128k"])
                            .args(["-c:a", "libmp3lame"])
                            .args(["-e", &format!("-ss {}", hit_data.hit.playback_offset)])
                            .args(["--extension", "mp3"])
                            .args(["-o", out_file.to_str().unwrap()])
                            .arg("-sn")
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
                    clear_unavailable_issue(&db, event_sender.as_ref(), hit_data.hit.id).await;
                    clear_download_failed_issue(&db, event_sender.as_ref(), hit_data.hit.id).await;
                    hit_data.hit.downloaded = true;
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
