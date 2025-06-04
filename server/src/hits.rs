use crossbeam_channel::unbounded;
use filesize::PathExt;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};
use serde::Serialize;
use std::{
    cmp::PartialEq,
    collections::HashSet,
    env,
    fs::{create_dir_all, read_dir, remove_file},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};
use uuid::Uuid;

include!(concat!(env!("OUT_DIR"), "/hits.rs"));

#[derive(Clone, Eq, Debug, Serialize, JsonSchema)]
pub struct Hit {
    pub artist: &'static str,
    pub title: &'static str,
    pub belongs_to: &'static str,
    pub year: u32,
    pub pack: &'static str,
    #[serde(skip)]
    pub playback_offset: u16,
    pub id: Uuid,
    #[serde(skip)]
    pub yt_id: &'static str,
}

impl Hit {
    pub fn download_dir() -> String {
        env::var("DOWNLOAD_DIRECTORY").unwrap_or("./hits".to_string())
    }

    pub fn file(&self) -> PathBuf {
        Path::new(&Hit::download_dir()).join(format!("{}_{}.mp3", self.yt_id, self.playback_offset))
    }

    pub fn exists(&self) -> bool {
        self.file().is_file()
    }
}

impl PartialEq for Hit {
    fn eq(&self, h: &Self) -> bool {
        self.yt_id == h.yt_id
    }
}

impl Hash for Hit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.yt_id.hash(state);
    }
}

struct DownloadHitData {
    in_file: PathBuf,
    hit: &'static Hit,
}

pub fn download_hits() {
    let (s, r) = unbounded::<DownloadHitData>();
    let download_dir = Hit::download_dir();

    let _ = create_dir_all(download_dir.as_str());

    rocket::tokio::spawn(async move {
        for hit in get_all().iter() {
            if !hit.exists() {
                let mut in_file =
                    Path::new(&Hit::download_dir()).join(format!("{}.opus", hit.yt_id));
                let options = VideoOptions {
                    quality: VideoQuality::HighestAudio,
                    filter: VideoSearchOptions::Audio,
                    ..Default::default()
                };
                if let Ok(video) = Video::new_with_options(hit.yt_id, options) {
                    println!(
                        "Download {}: {} to {}.opus",
                        hit.artist, hit.title, hit.yt_id
                    );

                    let in_dl = video
                        .download(format!("{}/{}.opus", download_dir.as_str(), hit.yt_id))
                        .await;

                    if in_dl.is_err()
                        || !in_file.is_file()
                        || in_file.size_on_disk().unwrap_or(0) == 0
                    {
                        if in_dl.is_err() {
                            println!("{}", in_dl.unwrap_err());
                        }
                        if env::var("USE_YT_DLP").is_ok() {
                            println!("Using yt-dlp...");
                            if in_file.is_file() {
                                remove_file(&in_file).unwrap();
                            }
                            in_file.set_extension("m4a");
                            let mut command = Command::new("yt-dlp");
                            command
                                .current_dir(env::current_dir().unwrap())
                                .args(["-f", "bestaudio[ext=m4a]"])
                                .args(["-o", in_file.to_str().unwrap()])
                                .arg(format!("https://www.youtube.com/watch?v={}", hit.yt_id));

                            let output_res =
                                command.output().expect("Failed to execute ffmpeg process!");

                            if !output_res.status.success() {
                                println!("{}", String::from_utf8_lossy(&output_res.stderr));
                                println!("Download failed with yt-dlp, skipping...");
                                continue;
                            }
                        } else {
                            println!("Download failed, skipping...");
                            continue;
                        }
                    }

                    s.send(DownloadHitData { in_file, hit }).unwrap();
                }
            }
        }
    });

    rocket::tokio::spawn(async move {
        while let Ok(hit_data) = r.recv() {
            println!(
                "Processing {} to mp3...",
                hit_data.in_file.extension().unwrap().to_str().unwrap()
            );

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

        println!("Download finished.");

        println!("Cleaning up unused hits...");

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

        println!("Finished cleanup.");
    });
}
