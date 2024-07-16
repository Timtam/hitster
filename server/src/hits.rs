use filesize::PathExt;
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
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
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock,
    },
    thread,
    time::Duration,
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

// copied from ffmpeg-loudness-norm

fn progress_thread() -> (Arc<AtomicBool>, thread::JoinHandle<()>) {
    const PROGRESS_CHARS: [&str; 12] = ["⠂", "⠃", "⠁", "⠉", "⠈", "⠘", "⠐", "⠰", "⠠", "⠤", "⠄", "⠆"];
    let finished = Arc::new(AtomicBool::new(false));
    let stop_signal = Arc::clone(&finished);
    let handle = thread::spawn(move || {
        for pc in PROGRESS_CHARS.iter().cycle() {
            if stop_signal.load(Ordering::Relaxed) {
                break;
            };
            eprint!("Processing {}\r", pc);
            thread::sleep(Duration::from_millis(250));
        }
    });
    (finished, handle)
}

#[derive(Default)]
pub struct HitsterDownloader {}

#[rocket::async_trait]
impl Fairing for HitsterDownloader {
    fn info(&self) -> Info {
        Info {
            kind: Kind::Ignite,
            name: "Download hits",
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let download_dir = Hit::download_dir();

        let _ = create_dir_all(download_dir.as_str());

        println!("Starting download of missing hits. This may take a while...");

        for hit in get_all().iter() {
            if !hit.exists() {
                let mut in_file =
                    Path::new(&Hit::download_dir()).join(format!("{}.opus", hit.yt_id));
                let out_file = Path::new(&Hit::download_dir())
                    .join(format!("{}_{}.mp3", hit.yt_id, hit.playback_offset));
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
                                .current_dir(&env::current_dir().unwrap())
                                .args(["-f", "bestaudio[ext=m4a]"])
                                .args(["-o", in_file.to_str().unwrap()])
                                .arg(&format!("https://www.youtube.com/watch?v={}", hit.yt_id));

                            let output = {
                                let (finished, _) = progress_thread();
                                let output_res = command.output();
                                finished.store(true, Ordering::SeqCst);
                                output_res.expect("Failed to execute ffmpeg process!")
                            };

                            if !output.status.success() {
                                println!("{}", String::from_utf8_lossy(&output.stderr));
                                println!("Download failed with yt-dlp, skipping...");
                                continue;
                            }
                        } else {
                            println!("Download failed, skipping...");
                            continue;
                        }
                    }

                    println!(
                        "Processing {} to mp3...",
                        in_file.extension().unwrap().to_str().unwrap()
                    );

                    let mut command = Command::new("ffmpeg-normalize");
                    command
                        .current_dir(&env::current_dir().unwrap())
                        .arg(&in_file)
                        .args(["-ar", "44100"])
                        .args(["-b:a", "128k"])
                        .args(["-c:a", "libmp3lame"])
                        .args(["-e", &format!("-ss {}", hit.playback_offset)])
                        .args(["--extension", "mp3"])
                        .args(["-o", out_file.to_str().unwrap()])
                        .arg("-sn")
                        .args(["-t", "-18.0"])
                        .arg("-vn");

                    {
                        let (finished, _) = progress_thread();
                        let output_res = command.output();
                        finished.store(true, Ordering::SeqCst);
                        output_res.expect("Failed to execute ffmpeg process!");
                    }

                    remove_file(in_file).unwrap();
                }
            }
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

        Ok(rocket)
    }
}
