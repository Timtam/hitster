use csv;
use std::{env, fs, path::Path};

fn main() {
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
        file_content += format!(
            "Hit {{
            interpret: \"{}\".into(),
            title: \"{}\".into(),
            year: {},
        }},",
            record.get(0).unwrap(),
            record.get(2).unwrap(),
            record.get(1).unwrap()
        )
        .as_str();
    }

    fs::write(
        &dest_path,
        format!(
            "pub fn get_all() -> Vec<Hit> {{
            vec![{}]
         }}
        ",
            file_content
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./etc/hits.csv");
}
