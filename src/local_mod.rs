//! local_mod.rs

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use std::fs;
use unwrap::unwrap;
use uncased::UncasedStr;

use crate::clear_line;

// $ dropbox_backup_to_external_disk list_local /mnt/d/DropBoxBackup2
// $ clear; cargo run --bin dropbox_backup_to_external_disk -- list_local /mnt/d/DropBoxBackup2

pub fn list_local(base_path: &str) {
    println!("start list_local {}", base_path);
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
        // println!("{}",str_path);
        // path.is_dir() is slow. entry.file-type().is_dir() is fast
        if entry.file_type().is_dir() {
            println!(
                "{}{}Folder: {}",
                term_cursor::Goto(0,5),
                clear_line(),
                str_path.trim_start_matches(base_path)
            );
            println!("{}{}Folder_count: {}", term_cursor::Goto(0,6),clear_line(), folder_count);

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
    println!("local list sort {}", "");
    let mut sorted_local: Vec<&str> = output_string.lines().collect();
    use rayon::prelude::*;
    sorted_local.par_sort_unstable_by(|a,b|{
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
    let path_list_for_trash = "temp_data/list_for_trash.csv";
    let list_for_trash = std::fs::read_to_string(path_list_for_trash).unwrap();
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
    // empty the list
    println!("list_for_trash emptied");
    unwrap!(std::fs::write(path_list_for_trash,""));
}

// the files from list_for_correct_time
pub fn correct_time_from_list() {
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let path_list_for_correct_time = "temp_data/list_for_correct_time.csv";
    let list_for_correct_time = std::fs::read_to_string(path_list_for_correct_time).unwrap();
    for path_to_correct_time in list_for_correct_time.lines() {
        let line: Vec<&str> = path_to_correct_time.split("\t").collect();
        println!("{} {}", line[0], line[1]);
        let local_path = format!("{}{}", base_local_path, line[0]);
        let modified = filetime::FileTime::from_system_time(unwrap!(
                humantime::parse_rfc3339(line[1])
            ));
        unwrap!(filetime::set_file_mtime(local_path, modified));
    }
    // empty the list
    println!("list_for_correct_time emptied");
    unwrap!(std::fs::write(path_list_for_correct_time,""));
}

// add lines from 
pub fn add_downloaded_to_list_local() {

    println!("add_downloaded_to_list_local");
    let path_list_just_downloaded="temp_data/list_just_downloaded.csv";
    let list_just_downloaded = std::fs::read_to_string(path_list_just_downloaded).unwrap();
    // it must be sorted, because downloads are multi-thread and not in sort order
    let mut sorted_downloaded: Vec<&str> = list_just_downloaded.lines().collect();
    use rayon::prelude::*;
    sorted_downloaded.par_sort_unstable_by(|a,b|{
        let aa: &UncasedStr = (*a).into();
        let bb: &UncasedStr = (*b).into();
        aa.cmp(bb)
    } );
    let joined_downloaded = sorted_downloaded.join("\n");
    let mut list_downloaded = fs::OpenOptions::new()
        .write(true)
        .open(path_list_just_downloaded)
        .unwrap();
    use std::io::Write;
    unwrap!(list_downloaded.write (joined_downloaded.as_bytes()));

    let path_list_local_files = "temp_data/list_local_files.csv";
    let list_local_files = std::fs::read_to_string(path_list_local_files).unwrap();
    let mut sorted_local: Vec<&str> = list_local_files.lines().collect();

    // loop the 2 lists and merge sorted
    let mut cursor_downloaded = 0;
    let mut cursor_local = 0;
    let mut line_local: Vec<&str> = vec![];
    let mut line_downloaded: Vec<&str> = vec![];
    loop {
        line_local.truncate(3);
        line_downloaded.truncate(3);

        if cursor_downloaded >= sorted_downloaded.len() && cursor_local >= sorted_local.len() {
            break;
        } else if cursor_downloaded >= sorted_downloaded.len() {
            // final lines
            break;
        } else if cursor_local >= sorted_local.len() {
            // final lines
            line_downloaded = sorted_downloaded[cursor_downloaded].split("\t").collect();
            sorted_local.push(&sorted_downloaded[cursor_downloaded]);
            cursor_downloaded += 1;
        } else {
            line_downloaded = sorted_downloaded[cursor_downloaded].split("\t").collect();
            line_local = sorted_local[cursor_local].split("\t").collect();
            // UncasedStr preserves the case in the string, but comparison is done case insensitive
            let path_downloaded: &UncasedStr = line_downloaded[0].into();
            let path_local: &UncasedStr = line_local[0].into();
            if path_downloaded.lt(path_local) {
                // insert the line
                sorted_local.insert(cursor_local, sorted_downloaded[cursor_downloaded]);
                cursor_local += 1;
                cursor_downloaded += 1;
            } else if path_downloaded.gt(path_local) { 
                cursor_local += 1;
            } else {
                // equal path. replace line
                sorted_local[cursor_local] = sorted_downloaded[cursor_downloaded];
                cursor_local += 1;
                cursor_downloaded += 1;
            }
        }
    }

    let joined = sorted_local.join("\n");
    let mut list_local = fs::OpenOptions::new()
        .write(true)
        .open(path_list_local_files)
        .unwrap();
    unwrap!(list_local.write (joined.as_bytes()));
    // empty the file temp_data/list_just_downloaded.csv 
    println!("list_just_downloaded emptied");
    unwrap!(std::fs::write(path_list_just_downloaded, ""));     
}