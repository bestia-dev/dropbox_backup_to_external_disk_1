//! local_mod.rs

use crate::terminal_ansi_mod::*;

use lexical_sort::{lexical_cmp, StringSort};
use std::fs;
use unwrap::unwrap;
#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};

// $ dbx_download list_local /mnt/d/DropBoxBackup2
// $ clear; cargo run --bin dbx_download -- list_local /mnt/d/DropBoxBackup2

pub fn list_local(base_path: &str) {
    eprintln!("start list_local");
    save_base_path(base_path);
    // write data to a big string in memory
    let mut output_string = String::with_capacity(1024 * 1024);

    use walkdir::WalkDir;

    let mut folder_count = 0;
    for entry in WalkDir::new(base_path) {
        //let mut ns_started = ns_start("WalkDir entry start");
        let entry: walkdir::DirEntry = entry.unwrap();
        let path = entry.path();
        let str_path = unwrap!(path.to_str());
        // path.is_dir() is slow. entry.file-type().is_dir() is fast
        if entry.file_type().is_dir() {
            
            println!("{}Folder: {}",ansi_set_row(5), str_path.trim_start_matches(base_path));
            
            println!("{}Folder_count: {}",ansi_set_row(6), folder_count);

            folder_count += 1;
        } else {
            // write csv tab delimited
            // metadata() in wsl/Linux is slow. Nothing to do here.
            //ns_started = ns_print("metadata start", ns_started);
            if let Ok(metadata) = entry.metadata() {
                //ns_started = ns_print("metadata end", ns_started);
                use chrono::offset::Utc;
                use chrono::DateTime;
                let datetime: DateTime<Utc> = unwrap!(metadata.modified()).into();

                output_string.push_str(&format!(
                    "{}\t{}\t{}\n",
                    str_path.trim_start_matches(base_path),
                    datetime.format("%Y-%m-%dT%TZ"),
                    metadata.len()
                ));
            }
        }
        //ns_print("WalkDir entry end", ns_started);
    }
    //#region: sort
    eprintln!("local list lexical sort{}", "");
    let mut sorted_local: Vec<&str> = output_string.lines().collect();
    sorted_local.string_sort_unstable(lexical_cmp);
    let joined = sorted_local.join("\n");
    eprintln!("local list sorted local len(): {}", sorted_local.len());
    //#end region: sort

    // join to string and write to file
    unwrap!(fs::write("temp_data/list_local_files.csv", joined));
}

/// remember the base local path for later commands
pub fn save_base_path(base_path: &str) {
    if !std::path::Path::new(base_path).exists() {
        eprintln!("error: base_path not exists {}", base_path);
        std::process::exit(1);
    }
    std::fs::write("temp_data/base_local_path.csv", base_path).unwrap();
}
