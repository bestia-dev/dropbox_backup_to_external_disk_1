//! local_mod.rs

use crate::terminal_ansi_mod::*;

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use std::fs;
use unwrap::unwrap;
use uncased::UncasedStr;

// $ dropbox_backup_to_external_disk list_local /mnt/d/DropBoxBackup2
// $ clear; cargo run --bin dropbox_backup_to_external_disk -- list_local /mnt/d/DropBoxBackup2

pub fn list_local(base_path: &str) {
    println!("start list_local");
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
            println!(
                "{}Folder: {}",
                ansi_set_row(5),
                str_path.trim_start_matches(base_path)
            );

            println!("{}Folder_count: {}", ansi_set_row(6), folder_count);

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
    println!("local list sort{}", "");
    let mut sorted_local: Vec<&str> = output_string.lines().collect();

    sorted_local.sort_by(|a,b|{
        let aa: &UncasedStr = (*a).into();
        let bb: &UncasedStr = (*b).into();
        aa.cmp(bb)
    } );

    let joined = sorted_local.join("\n");
    println!("local list sorted local len(): {}", sorted_local.len());
    //#end region: sort

    // join to string and write to file
    unwrap!(fs::write("temp_data/list_local_files.csv", joined));
}

/// remember the base local path for later commands
pub fn save_base_path(base_path: &str) {
    if !std::path::Path::new(base_path).exists() {
        println!("error: base_path not exists {}", base_path);
        std::process::exit(1);
    }
    std::fs::write("temp_data/base_local_path.csv", base_path).unwrap();
}

// the files from list_for_trash move to trash folder
pub fn trash_from_list() {
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let now_string = chrono::Local::now()
        .format("trash_%Y-%m-%d_%H-%M-%S")
        .to_string();
    let base_trash_path = format!("{}_{}", &base_local_path, &now_string);
    if !std::path::Path::new(&base_trash_path).exists() {
        std::fs::create_dir_all(&base_trash_path).unwrap();
    }
    //move the files in the same directory structure
    let list_for_trash = std::fs::read_to_string("temp_data/list_for_trash.csv").unwrap();
    for path_to_trash in list_for_trash.lines() {
        let move_from = format!("{}{}", base_local_path, path_to_trash);
        let move_to = format!("{}{}", base_trash_path, path_to_trash);
        println!("{}  ->  {}", move_from, move_to);
        let parent = unwrap!(std::path::Path::parent(std::path::Path::new(&move_to)));
        if !parent.exists() {
            std::fs::create_dir_all(&parent).unwrap();
        }
        unwrap!(std::fs::rename(&move_from, &move_to));
    }
}

// the files from list_for_correct_time
pub fn correct_time_from_list() {
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let list_for_correct_time = std::fs::read_to_string("temp_data/list_for_correct_time.csv").unwrap();
    for path_to_correct_time in list_for_correct_time.lines() {
        let line: Vec<&str> = path_to_correct_time.split("\t").collect();
        println!("{} {}", line[0], line[1]);
        let local_path = format!("{}{}", base_local_path, line[0]);
        let modified = filetime::FileTime::from_system_time(unwrap!(
                humantime::parse_rfc3339(line[1])
            ));
        unwrap!(filetime::set_file_mtime(local_path, modified));
    }
}

// add lines from 
pub fn list_local_add_downloaded() {
    println!("list_local_add_downloaded");
    let list_just_downloaded = std::fs::read_to_string("temp_data/list_just_downloaded.csv").unwrap();

    let path_list_local_files = "temp_data/list_local_files.csv";
    let mut list_local_files = std::fs::read_to_string(path_list_local_files).unwrap();
    list_local_files.push_str(&list_just_downloaded);
    let mut sorted_local: Vec<&str> = list_local_files.lines().collect();
    sorted_local.sort_by(|a,b|{
        let aa: &UncasedStr = (*a).into();
        let bb: &UncasedStr = (*b).into();
        aa.cmp(bb)
    } );
    let joined = sorted_local.join("\n");
    let mut list_local = fs::OpenOptions::new()
        .write(true)
        .open(path_list_local_files)
        .unwrap();
    use std::io::Write;
    unwrap!(list_local.write (joined.as_bytes()));
    println!("end");
}