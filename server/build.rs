use std::{
    collections::HashMap,
    env, fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};
use uuid::Uuid;

fn main() {
    let mut hasher: DefaultHasher;
    let mut ids: HashMap<u64, Uuid> = HashMap::new();
    let mut file_content: String = String::from("");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hits.rs");
    let csv_path = Path::new("./etc/hits.csv");
    let csv_data = fs::read_to_string(csv_path).expect("Unable to read hits");
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(csv_data.as_bytes());

    for result in csv_reader.records() {
        let record = result.unwrap();
        hasher = DefaultHasher::new();
        record.get(0).unwrap().hash(&mut hasher);
        record.get(2).unwrap().hash(&mut hasher);
        record.get(1).unwrap().hash(&mut hasher);
        let id = *ids.entry(hasher.finish()).or_insert_with(|| Uuid::new_v4());

        if record.get(5).unwrap() != "" {
            file_content += format!(
                "Hit {{
            artist: \"{}\",
            title: \"{}\",
            year: {},
            yt_url: \"{}\",
            playback_offset: {},
            pack: \"{}\",
            belongs_to: \"{}\",
            id: Uuid::parse_str(\"{}\").unwrap(),
        }},",
                record.get(0).unwrap(),
                record.get(2).unwrap(),
                record.get(1).unwrap(),
                record.get(5).unwrap(),
                record.get(6).unwrap_or("0"),
                record.get(3).unwrap(),
                record.get(4).unwrap(),
                id.to_string(),
            )
            .as_str();
        }
    }

    fs::write(
        dest_path,
        format!(
            "pub fn get_all() -> &'static Vec<Hit> {{
            static HITS: OnceLock<Vec<Hit>> = OnceLock::new();
            HITS.get_or_init(|| {{
            vec![{}]
         }})}}
        ",
            file_content
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./etc/hits.csv");
}
