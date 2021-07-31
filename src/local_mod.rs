//! local_mod.rs
//! Module contains all functions for local external disk.

use crate::clear_line;
#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use dropbox_content_hasher::DropboxContentHasher;
use log::error;
use std::fs;
use std::io::Write;
use std::path;
use uncased::UncasedStr;
use unwrap::unwrap;

/// list all local files and folders. It can take some time.
pub fn list_local(base_path: &str) {
    println!("start list_local {}", base_path);
    // empty the file. I want all or nothing result here if the process is terminated prematurely.
    unwrap!(fs::write("temp_data/list_local_files.csv", ""));
    // just_loaded is obsolete once I got the fresh local list
    unwrap!(fs::write("temp_data/list_just_downloaded_or_moved.csv", ""));
    save_base_path(base_path);
    // write data to a big string in memory
    let mut output_string = String::with_capacity(1024 * 1024);

    use walkdir::WalkDir;

    let mut folder_count = 0;
    let mut file_count = 0;
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
                term_cursor::Goto(0, 16),
                clear_line(),
                str_path.trim_start_matches(base_path)
            );
            println!(
                "{}{}local_folder_count: {}",
                term_cursor::Goto(0, 17),
                clear_line(),
                folder_count
            );

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
                println!(
                    "{}{}local_file_count: {}",
                    term_cursor::Goto(0, 18),
                    clear_line(),
                    file_count
                );

                file_count += 1;
            }
        }
        //ns_print("WalkDir entry end", ns_started);
    }
    // region: sort
    println!("{}local list sort...", term_cursor::Goto(0, 19));
    let sorted_string = crate::sort_string_lines(&output_string);
    println!(
        "{}list_local_files lines: {}",
        term_cursor::Goto(0, 19),
        sorted_string.lines().count()
    );
    // end region: sort
    unwrap!(fs::write("temp_data/list_local_files.csv", sorted_string));
}

/// saves the base local path for later commands like "/mnt/f/DropBoxBackup1"
pub fn save_base_path(base_path: &str) {
    if !path::Path::new(base_path).exists() {
        println!("error: base_path not exists {}", base_path);
        std::process::exit(1);
    }
    fs::write("temp_data/base_local_path.csv", base_path).unwrap();
}

/// Files are often moved or renamed
/// After compare, the same file (with different path or name) will be in the list_for_trash and in the list_for_download.
/// First for every trash line, we search list_for_download for same size and modified.
/// If found, get the remote_metadata with content_hash and calculate local_content_hash.
/// If they are equal move or rename, else nothing: it will be trashed and downloaded eventually.
/// Remove also the lines in files list_for_trash and list_for_download.
pub fn move_or_rename_local_files() {
    let base_local_path = fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let path_list_for_trash = "temp_data/list_for_trash.csv";
    let list_for_trash = fs::read_to_string(path_list_for_trash).unwrap();

    let path_list_for_download = "temp_data/list_for_download.csv";
    let list_for_download = fs::read_to_string(path_list_for_download).unwrap();

    let token = crate::remote_mod::get_short_lived_access_token();
    let client = dropbox_sdk::default_client::UserAuthDefaultClient::new(token);
    // write the renamed files to list_just_downloaded_or_moved, later they will be added to list_local_files.csv
    let list_just_downloaded_or_moved = "temp_data/list_just_downloaded_or_moved.csv";
    let mut just_downloaded = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(list_just_downloaded_or_moved)
        .unwrap();
    let mut count_moved = 0;
    for line_for_trash in list_for_trash.lines() {
        let vec_line_for_trash: Vec<&str> = line_for_trash.split("\t").collect();
        let string_path_for_trash = vec_line_for_trash[0];
        let global_path_to_trash = format!("{}{}", &base_local_path, string_path_for_trash);
        let path_to_trash = path::Path::new(&global_path_to_trash);
        // if path does not exist ignore, probably it eas moved or trashed earlier
        if path_to_trash.exists() {
            let modified_for_trash = vec_line_for_trash[1];
            let size_for_trash = vec_line_for_trash[2];
            // search in list_for_download for possible candidates
            for line_for_download in list_for_download.lines() {
                let vec_line_for_download: Vec<&str> = line_for_download.split("\t").collect();
                let path_for_download = vec_line_for_download[0];
                let modified_for_download = vec_line_for_download[1];
                let size_for_download = vec_line_for_download[2];
                if modified_for_trash == modified_for_download
                    && size_for_trash == size_for_download
                {
                    // same size and date. Let's check the content_hash to be sure.
                    let local_content_hash = format!(
                        "{:x}",
                        unwrap!(DropboxContentHasher::hash_file(path_to_trash))
                    );
                    let remote_content_hash = unwrap!(crate::remote_mod::remote_content_hash(
                        path_for_download,
                        &client
                    ));

                    if local_content_hash == remote_content_hash {
                        let move_from = path_to_trash;
                        let move_to = format!("{}{}", base_local_path, path_for_download);
                        println!("move {}  ->  {}", &move_from.to_string_lossy(), move_to);
                        let parent = unwrap!(path::Path::parent(path::Path::new(&move_to)));
                        if !parent.exists() {
                            fs::create_dir_all(&parent).unwrap();
                        }
                        unwrap!(fs::rename(&move_from, &move_to));
                        // append to just_downloaded
                        unwrap!(writeln!(just_downloaded, "{}", line_for_download));
                        count_moved += 1;
                        break;
                    }
                }
            }
        }
    }
    println!("moved or renamed: {}", count_moved);
}

/// move to trash folder the files from list_for_trash
/// ignore if the file does not exist anymore
pub fn trash_from_list() {
    let base_local_path = fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let now_string = chrono::Local::now()
        .format("trash_%Y-%m-%d_%H-%M-%S")
        .to_string();
    let base_trash_path = format!("{}_{}", &base_local_path, &now_string);
    if !path::Path::new(&base_trash_path).exists() {
        fs::create_dir_all(&base_trash_path).unwrap();
    }
    //move the files in the same directory structure
    let path_list_for_trash = "temp_data/list_for_trash.csv";
    let list_for_trash = fs::read_to_string(path_list_for_trash).unwrap();
    for line_path_for_trash in list_for_trash.lines() {
        let line: Vec<&str> = line_path_for_trash.split("\t").collect();
        let string_path_for_trash = line[0];
        let path_for_trash = path::Path::new(string_path_for_trash);
        // move to trash if file exists. Nothing if it does not exist, maybe is deleted when moved or in a move_to_trash before.
        if path_for_trash.exists() {
            let move_from = format!("{}{}", base_local_path, string_path_for_trash);
            let move_to = format!("{}{}", base_trash_path, string_path_for_trash);
            println!("{}  ->  {}", move_from, move_to);
            let parent = unwrap!(path::Path::parent(path::Path::new(&move_to)));
            if !parent.exists() {
                fs::create_dir_all(&parent).unwrap();
            }
            unwrap!(fs::rename(&move_from, &move_to));
        }
    }
    // empty the list if all is successful
    unwrap!(fs::write(path_list_for_trash, ""));
}

/// modify the files from list_for_correct_time
pub fn correct_time_from_list() {
    let base_local_path = fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let path_list_for_correct_time = "temp_data/list_for_correct_time.csv";
    let list_for_correct_time = fs::read_to_string(path_list_for_correct_time).unwrap();
    let token = crate::remote_mod::get_short_lived_access_token();
    let client = dropbox_sdk::default_client::UserAuthDefaultClient::new(token);
    for path_to_correct_time in list_for_correct_time.lines() {
        let line: Vec<&str> = path_to_correct_time.split("\t").collect();
        let remote_path = line[0];
        println!("{}", remote_path);
        let remote_content_hash = unwrap!(crate::remote_mod::remote_content_hash(
            &remote_path,
            &client
        ));
        let local_path = format!("{}{}", base_local_path, remote_path);
        let local_content_hash = format!(
            "{:x}",
            unwrap!(DropboxContentHasher::hash_file(&local_path))
        );
        if local_content_hash == remote_content_hash {
            let modified =
                filetime::FileTime::from_system_time(unwrap!(humantime::parse_rfc3339(line[1])));
            unwrap!(filetime::set_file_mtime(local_path, modified));
        } else {
            error!("correct_time content_hash different: {}", remote_path);
        }
    }
    // empty the list
    unwrap!(fs::write(path_list_for_correct_time, ""));
}

/// add lines from just_downloaded to list_local. Only before compare.
pub fn add_just_downloaded_to_list_local() {
    let path_list_just_downloaded = "temp_data/list_just_downloaded_or_moved.csv";
    let string_just_downloaded = fs::read_to_string(path_list_just_downloaded).unwrap();
    if !string_just_downloaded.is_empty() {
        // it must be sorted, because downloads are multi-thread and not in sort order
        let string_sorted_just_downloaded = crate::sort_string_lines(&string_just_downloaded);
        let mut vec_sorted_downloaded: Vec<&str> = string_sorted_just_downloaded.lines().collect();
        // It is forbidden to have duplicate lines
        vec_sorted_downloaded.dedup();
        println!(
            "add_just_downloaded_to_list_local: {}",
            vec_sorted_downloaded.len()
        );
        unwrap!(fs::write(
            path_list_just_downloaded,
            &string_sorted_just_downloaded
        ));

        let path_list_local_files = "temp_data/list_local_files.csv";
        let string_local_files = fs::read_to_string(path_list_local_files).unwrap();
        let mut vec_sorted_local: Vec<&str> = string_local_files.lines().collect();

        // loop the 2 lists and merge sorted
        let mut cursor_downloaded = 0;
        let mut cursor_local = 0;
        let mut vec_line_local: Vec<&str> = vec![];
        let mut vec_line_downloaded: Vec<&str> = vec![];
        loop {
            vec_line_local.truncate(3);
            vec_line_downloaded.truncate(3);

            if cursor_downloaded >= vec_sorted_downloaded.len()
                && cursor_local >= vec_sorted_local.len()
            {
                break;
            } else if cursor_downloaded >= vec_sorted_downloaded.len() {
                // final lines
                break;
            } else if cursor_local >= vec_sorted_local.len() {
                // final lines
                vec_line_downloaded = vec_sorted_downloaded[cursor_downloaded]
                    .split("\t")
                    .collect();
                vec_sorted_local.push(&vec_sorted_downloaded[cursor_downloaded]);
                cursor_downloaded += 1;
            } else {
                vec_line_downloaded = vec_sorted_downloaded[cursor_downloaded]
                    .split("\t")
                    .collect();
                vec_line_local = vec_sorted_local[cursor_local].split("\t").collect();
                // UncasedStr preserves the case in the string, but comparison is done case insensitive
                let path_downloaded: &UncasedStr = vec_line_downloaded[0].into();
                let path_local: &UncasedStr = vec_line_local[0].into();
                if path_downloaded.lt(path_local) {
                    // insert the line
                    vec_sorted_local.insert(cursor_local, vec_sorted_downloaded[cursor_downloaded]);
                    cursor_local += 1;
                    cursor_downloaded += 1;
                } else if path_downloaded.gt(path_local) {
                    cursor_local += 1;
                } else {
                    // equal path. replace line
                    vec_sorted_local[cursor_local] = vec_sorted_downloaded[cursor_downloaded];
                    cursor_local += 1;
                    cursor_downloaded += 1;
                }
            }
        }

        let new_local_files = vec_sorted_local.join("\n");
        unwrap!(fs::write(path_list_local_files, &new_local_files));

        // empty the file temp_data/list_just_downloaded_or_moved.csv
        // println!("list_just_downloaded_or_moved emptied");
        unwrap!(fs::write(path_list_just_downloaded, ""));
    }
}
