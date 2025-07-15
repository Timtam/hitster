use crate::{
    responses::MessageResponse,
    services::{HitService, ServiceHandle, ServiceStore},
};
use crossbeam_channel::unbounded;
use hitster_core::Hit;
use rocket::{
    State,
    http::Status,
    request::{FromRequest, Outcome, Request},
    serde::json::Json,
};
use rocket_okapi::{
    r#gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use std::{
    collections::HashSet,
    env,
    fs::{create_dir_all, read_dir, remove_file},
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};
use uuid::Uuid;

include!(concat!(env!("OUT_DIR"), "/hits.rs"));

struct DownloadHitData {
    in_file: PathBuf,
    hit: &'static Hit,
}

pub fn download_hits(hit_service: ServiceHandle<HitService>) {
    let (s, r) = unbounded::<DownloadHitData>();
    let dl_hit_service = hit_service.clone();
    let download_dir = Hit::download_dir();

    let _ = create_dir_all(download_dir.as_str());

    rocket::tokio::spawn(async move {
        rocket::info!("Starting background download of hits");

        let hits = get_all()
            .iter()
            .filter(|h| {
                if h.exists() {
                    hit_service.lock().add(h);
                }
                !h.exists()
            })
            .collect::<Vec<_>>();

        for hit in hits.iter() {
            #[cfg(feature = "native_dl")]
            {
                use filesize::PathExt;
                use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};

                let in_file = Path::new(&Hit::download_dir()).join(format!("{}.opus", hit.yt_id));

                let options = VideoOptions {
                    quality: VideoQuality::HighestAudio,
                    filter: VideoSearchOptions::Audio,
                    ..Default::default()
                };
                if let Ok(video) = Video::new_with_options(hit.yt_id, options) {
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
                        continue;
                    }
                }
            }

            #[cfg(feature = "yt_dl")]
            {
                let in_file = Path::new(&Hit::download_dir()).join(format!("{}.m4a", hit.yt_id));

                let mut command = Command::new("yt-dlp");
                command
                    .current_dir(env::current_dir().unwrap())
                    .args(["-f", "bestaudio[ext=m4a]"])
                    .args(["-o", in_file.to_str().unwrap()])
                    .args(["--extractor-args", "youtube:player-client=default,mweb"])
                    .arg(format!("https://www.youtube.com/watch?v={}", hit.yt_id));

                let output_res = command.output().expect("Failed to execute ffmpeg process!");

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
    });

    rocket::tokio::spawn(async move {
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
            dl_hit_service.lock().add(hit_data.hit);
        }

        rocket::info!("Download finished.");

        rocket::info!("Cleaning up unused hits...");

        let paths = read_dir(Hit::download_dir()).unwrap();
        let mut files: HashSet<String> = HashSet::new();

        for p in paths.flatten() {
            files.insert(p.file_name().into_string().unwrap());
        }

        for hit in get_all().iter() {
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
        dl_hit_service.lock().set_finished_downloading();
    });
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
