//! local_mod.rs

use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use unwrap::unwrap;

pub fn list_local(base_path: &str) {
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
        .open("data/list_local_files.csv")
        .unwrap();
    let dir = Path::new(base_path);
    unwrap!(traverse_dir(dir, &mut file, base_path));
}

pub fn traverse_dir(dir: &Path, output_file: &mut File, base_path:&str) -> io::Result<()> {
    if dir.is_dir() {
        eprintln!("Folder: {}",dir.to_str().unwrap().trim_start_matches(base_path));
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let str_path = unwrap!(path.to_str());
            if path.is_dir() {
                unwrap!(traverse_dir(&path, output_file,base_path));
            } else {
                // write csv tab delimited
                let metadata = fs::metadata(&path)?;
                use chrono::offset::Utc;
                use chrono::DateTime;
                let datetime: DateTime<Utc> = unwrap!(metadata.modified()).into();

                if let Err(e) = writeln!(
                    output_file,
                    "{}\t{}\t{}",
                    str_path.trim_start_matches(base_path),
                    datetime.format("%Y-%m-%dT%TZ"),
                    metadata.len()
                ) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
    }
    Ok(())
}
