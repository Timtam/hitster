use regex::Regex;
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{create_dir_all, remove_file},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

include!(concat!(env!("OUT_DIR"), "/hits.rs"));

#[derive(Clone, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Hit {
    pub artist: String,
    pub title: String,
    pub belongs_to: String,
    pub year: u32,
    pub pack: String,
    #[serde(skip)]
    pub yt_url: String,
    #[serde(skip)]
    pub playback_offset: u16,
}

impl Hit {
    pub fn yt_id(&self) -> Option<String> {
        let yt_id: Regex = Regex::new(
            r"^.*((youtu.be\/)|(v\/)|(\/u\/\w\/)|(embed\/)|(watch\?))\??v?=?([^#&?]*).*",
        )
        .unwrap();

        yt_id
            .captures(self.yt_url.as_str())
            .map(|caps| caps[7].to_string())
    }

    pub fn download_dir() -> String {
        env::var("DOWNLOAD_DIRECTORY").unwrap_or("./hits".to_string())
    }

    pub fn file(&self) -> Option<PathBuf> {
        self.yt_id()
            .map(|id| Path::new(&Hit::download_dir()).join(format!("{}.mp3", id)))
    }

    pub fn exists(&self) -> bool {
        self.file().map(|p| p.is_file()).unwrap_or(false)
    }
}

impl PartialEq for Hit {
    fn eq(&self, other: &Self) -> bool {
        self.artist == other.artist && self.title == other.title && self.year == other.year
    }
}

impl Hash for Hit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.artist.hash(state);
        self.title.hash(state);
        self.year.hash(state);
    }
}

// alot of stuff copied from ffmpeg-loudness-norm

#[derive(Serialize, Deserialize, Debug)]
struct Loudness {
    input_i: String,
    input_tp: String,
    input_lra: String,
    input_thresh: String,
    target_offset: String,
}

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

        let _ = create_dir_all(download_dir.clone());

        println!("Starting download of missing hits. This may take a while...");

        for hit in get_all().iter() {
            if let Some(id) = hit.yt_id() {
                if !hit.exists() {
                    let options = VideoOptions {
                        quality: VideoQuality::HighestAudio,
                        filter: VideoSearchOptions::Audio,
                        ..Default::default()
                    };
                    if let Ok(video) = Video::new_with_options(id.as_str(), options) {
                        println!(
                            "Download {}: {} to {}.opus",
                            hit.artist.as_str(),
                            hit.title.as_str(),
                            id.as_str()
                        );

                        if video
                            .download(format!("{}/{}.opus", download_dir.as_str(), id))
                            .await
                            .is_ok()
                        {
                            let in_file = format!("{}/{}.opus", download_dir.as_str(), id);
                            let out_file = format!("{}/{}.mp3", download_dir.as_str(), id);
                            let offset = format!("{}", hit.playback_offset);

                            println!("Measure loudness of song...");

                            let mut command = Command::new("ffmpeg");
                            command
                                .current_dir(&env::current_dir().unwrap())
                                .arg("-i")
                                .arg(&in_file)
                                .arg("-hide_banner")
                                .args(&["-vn", "-af"])
                                .arg(format!(
                                    "loudnorm=I={}:LRA={}:tp={}:print_format=json",
                                    -18.0, 12.0, -1.0
                                ))
                                .args(&["-f", "null", "-"]);

                            let output = {
                                let (finished, _) = progress_thread();
                                let output_res = command.output();
                                finished.store(true, Ordering::SeqCst);
                                output_res.expect("Failed to execute ffmpeg process!")
                            };

                            let output_s = String::from_utf8_lossy(&output.stderr);
                            if output.status.success() {
                                let loudness: Loudness = {
                                    let json: String = {
                                        let lines: Vec<&str> = output_s.lines().collect();
                                        let (_, lines) = lines.split_at(lines.len() - 14);
                                        lines
                                            .into_iter()
                                            .take(12)
                                            .map(|s| *s)
                                            .collect::<Vec<_>>()
                                            .join("\n")
                                    };
                                    serde_json::from_str(&json).unwrap()
                                };

                                let af = format!("loudnorm=linear=true:I={}:LRA={}:TP={}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:print_format=summary",
                                    -18.0, 12.0, -1.0,
                                    loudness.input_i,
                                    loudness.input_tp,
                                    loudness.input_lra,
                                    loudness.input_thresh,
                                    loudness.target_offset,
                                );

                                println!("Processing opus to mp3...");

                                let mut command = Command::new("ffmpeg");
                                command
                                    .current_dir(&env::current_dir().unwrap())
                                    .arg("-nostdin")
                                    .arg("-y")
                                    .arg("-i")
                                    .arg(&in_file)
                                    .args(["-ss", offset.as_str()])
                                    .arg("-vn")
                                    .arg("-sn")
                                    .arg("-dn")
                                    .args(["-af", af.as_str()])
                                    .arg(&out_file);

                                {
                                    let (finished, _) = progress_thread();
                                    let output_res = command.output();
                                    finished.store(true, Ordering::SeqCst);
                                    output_res.expect("Failed to execute ffmpeg process!");
                                }

                                remove_file(in_file.as_str()).unwrap();
                            }
                        } else {
                            println!("Unable to download video.");
                        }
                    }
                }
            }
        }

        println!("Download finished.");

        Ok(rocket)
    }
}
