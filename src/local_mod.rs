// local_mod.rs
//! Module contains all functions for local external disk.

#[allow(unused_imports)]
use dropbox_content_hasher::DropboxContentHasher;
use log::error;
use std::fs;
use std::io::Write;
use std::path;
use uncased::UncasedStr;
use unwrap::unwrap;

use crate::*;

/// list all local files and folders. It can take some time.
pub fn list_local(base_path: &str, app_config: &'static AppConfig) {
    // empty the file. I want all or nothing result here if the process is terminated prematurely.
    let path_list = app_config.path_list_destination_files;
    // just_loaded is obsolete once I got the fresh local list
    save_base_path(base_path, app_config);
    list_local_internal(
        base_path,
        path_list,
        app_config.path_list_just_downloaded_or_moved,
    );
}
/// for second backup: list all local files and folders. It can take some time.
pub fn list2_local(base2_path: &str, app_config: &'static AppConfig) {
    // empty the file. I want all or nothing result here if the process is terminated prematurely.
    // just_loaded is obsolete once I got the fresh local list
    save2_base_path(base2_path, app_config);
    list_local_internal(
        base2_path,
        app_config.path_list2_local_files,
        app_config.path_list2_just_downloaded_or_moved,
    );
}

/// list all local files and folders. It can take some time.
fn list_local_internal(base_path: &str, path_list: &str, path_just_downloaded: &str) {
    // empty the file. I want all or nothing result here if the process is terminated prematurely.
    unwrap!(fs::write(path_list, ""));
    // just_loaded is obsolete once I got the fresh local list
    unwrap!(fs::write(path_just_downloaded, ""));
    // write data to a big string in memory
    let mut output_string = String::with_capacity(1024 * 1024);
    let (x_screen_len, _y_screen_len) = unwrap!(termion::terminal_size());
    use walkdir::WalkDir;

    let mut folder_count = 0;
    let mut file_count = 0;
    for entry in WalkDir::new(base_path) {
        //let mut ns_started = ns_start("WalkDir entry start");
        let entry: walkdir::DirEntry = entry.unwrap();
        let path = entry.path();
        let str_path = unwrap!(path.to_str());
        // path.is_dir() is slow. entry.file-type().is_dir() is fast
        if entry.file_type().is_dir() {
            println!(
                "{}{}Folder: {}",
                at_line(13),
                *CLEAR_LINE,
                shorten_string(str_path.trim_start_matches(base_path), x_screen_len - 9),
            );
            println!(
                "{}{}local_folder_count: {}",
                at_line(14),
                *CLEAR_LINE,
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
                    at_line(15),
                    *CLEAR_LINE,
                    file_count
                );

                file_count += 1;
            }
        }
        //ns_print("WalkDir entry end", ns_started);
    }
    // region: sort
    println!("{}local list sort...", at_line(16));
    let sorted_string = crate::sort_string_lines(&output_string);
    println!(
        "{}list_local_files sorted lines: {}",
        at_line(16),
        sorted_string.lines().count()
    );
    // end region: sort
    unwrap!(fs::write(path_list, sorted_string));
}

/// saves the base local path for later use like "/mnt/d/DropBoxBackup1"
pub fn save_base_path(base_path: &str, app_config: &'static AppConfig) {
    if !path::Path::new(base_path).exists() {
        println!("error: base_path not exists {}", base_path);
        std::process::exit(1);
    }
    fs::write(app_config.path_list_base_local_path, base_path).unwrap();
}

/// saves the base local path for later use like "/mnt/f/DropBoxBackup2"
pub fn save2_base_path(base2_path: &str, app_config: &'static AppConfig) {
    if !path::Path::new(base2_path).exists() {
        println!("error: base2_path not exists {}", base2_path);
        std::process::exit(1);
    }
    fs::write(app_config.path_list2_base2_local_path, base2_path).unwrap();
}

/// The source file can be on dropbox or on external disk Backup_1
pub enum RemoteKind {
    Client {
        client: dropbox_sdk::default_client::UserAuthDefaultClient,
    },
    RemoteBasePath {
        remote_base_local_path: String,
    },
}

impl RemoteKind {
    /// 2 different ways of getting the content_hash
    /// it depends if the file is on the remote dropbox or on the local disk
    fn get_content_hash(&self, path_for_download: &str) -> String {
        match self {
            RemoteKind::Client { client } => {
                unwrap!(crate::remote_mod::remote_content_hash(
                    path_for_download,
                    &client
                ))
            }
            RemoteKind::RemoteBasePath {
                remote_base_local_path,
            } => {
                let global_path_to_download =
                    format!("{}{}", &remote_base_local_path, path_for_download);
                let path_global_path_to_download = path::Path::new(&global_path_to_download);
                if path_global_path_to_download.exists() {
                    format!(
                        "{:x}",
                        unwrap!(DropboxContentHasher::hash_file(
                            path_global_path_to_download
                        ))
                    )
                } else {
                    "".to_string()
                }
            }
        }
    }
}

/// Files are often moved or renamed  
/// After compare, the same file (with different path or name) will be in the list_for_trash and in the list_for_download.  
/// First for every trash line, we search list_for_download for same size and modified.  
/// If found, get the remote_metadata with content_hash and calculate local_content_hash.  
/// If they are equal move or rename, else nothing: it will be trashed and downloaded eventually.  
/// Remove also the lines in files list_for_trash and list_for_download.  
pub fn move_or_rename_local_files(app_config: &'static AppConfig) {
    let to_base_local_path = fs::read_to_string(app_config.path_list_base_local_path).unwrap();
    let token = crate::remote_mod::get_short_lived_access_token();
    let client = dropbox_sdk::default_client::UserAuthDefaultClient::new(token);
    move_or_rename_local_files_internal(
        RemoteKind::Client { client },
        &to_base_local_path,
        app_config.path_list_for_trash,
        app_config.path_list_for_download,
        app_config.path_list_just_downloaded_or_moved,
    );
}

/// internal function
fn move_or_rename_local_files_internal(
    client_or_base_path: RemoteKind,
    to_base_local_path: &str,
    path_list_for_trash: &str,
    path_list_for_download: &str,
    list_just_downloaded_or_moved: &str,
) {
    let list_for_trash = fs::read_to_string(path_list_for_trash).unwrap();
    let list_for_download = fs::read_to_string(path_list_for_download).unwrap();

    // write the renamed files to list_just_downloaded_or_moved, later they will be added to list_local_files.csv
    let mut just_downloaded = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(list_just_downloaded_or_moved)
        .unwrap();
    let mut count_moved = 0;
    for line_for_trash in list_for_trash.lines() {
        let vec_line_for_trash: Vec<&str> = line_for_trash.split("\t").collect();
        let string_path_for_trash = vec_line_for_trash[0];
        let global_path_to_trash = format!("{}{}", &to_base_local_path, string_path_for_trash);
        let path_global_path_to_trash = path::Path::new(&global_path_to_trash);
        // if path does not exist ignore, probably it eas moved or trashed earlier
        if path_global_path_to_trash.exists() {
            let modified_for_trash = vec_line_for_trash[1];
            let size_for_trash = vec_line_for_trash[2];
            let file_name_for_trash: Vec<&str> = string_path_for_trash.split("/").collect();
            let file_name_for_trash = unwrap!(file_name_for_trash.last());

            // search in list_for_download for possible candidates
            // first try exact match with name, date, size because it is fast
            let mut is_moved = false;
            for line_for_download in list_for_download.lines() {
                let vec_line_for_download: Vec<&str> = line_for_download.split("\t").collect();
                let path_for_download = vec_line_for_download[0];
                let modified_for_download = vec_line_for_download[1];
                let size_for_download = vec_line_for_download[2];
                let file_name_for_download: Vec<&str> = path_for_download.split("/").collect();
                let file_name_for_download = unwrap!(file_name_for_download.last());

                if modified_for_trash == modified_for_download
                    && size_for_trash == size_for_download
                    && file_name_for_trash == file_name_for_download
                {
                    move_internal(
                        path_global_path_to_trash,
                        &to_base_local_path,
                        path_for_download,
                    );
                    unwrap!(writeln!(just_downloaded, "{}", line_for_download));
                    count_moved += 1;
                    is_moved = true;
                    break;
                }
            }
            // if the exact match didn't move the file, then check the content_hash (slow)
            if is_moved == false {
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
                            unwrap!(DropboxContentHasher::hash_file(path_global_path_to_trash))
                        );
                        let remote_content_hash =
                            client_or_base_path.get_content_hash(path_for_download);

                        if local_content_hash == remote_content_hash {
                            move_internal(
                                path_global_path_to_trash,
                                &to_base_local_path,
                                path_for_download,
                            );
                            unwrap!(writeln!(just_downloaded, "{}", line_for_download));
                            count_moved += 1;
                            break;
                        }
                    }
                }
            }
        }
    }
    println!("moved or renamed: {}", count_moved);
}

/// internal code to move file
fn move_internal(
    path_global_path_to_trash: &path::Path,
    to_base_local_path: &str,
    path_for_download: &str,
) {
    let move_from = path_global_path_to_trash;
    let move_to = format!("{}{}", to_base_local_path, path_for_download);
    println!("move {}  ->  {}", &move_from.to_string_lossy(), move_to);
    let parent = unwrap!(path::Path::parent(path::Path::new(&move_to)));
    if !parent.exists() {
        fs::create_dir_all(&parent).unwrap();
    }
    if path::Path::new(&move_to).exists() {
        let mut perms = unwrap!(fs::metadata(&move_to)).permissions();
        if perms.readonly() == true {
            perms.set_readonly(false);
            unwrap!(fs::set_permissions(&move_to, perms));
        }
    }
    if path::Path::new(&move_from).exists() {
        let mut perms = unwrap!(fs::metadata(&move_from)).permissions();
        if perms.readonly() == true {
            perms.set_readonly(false);
            unwrap!(fs::set_permissions(&move_from, perms));
        }
    }
    unwrap!(fs::rename(&move_from, &move_to));
}

/// Move to trash folder the files from list_for_trash.  
/// Ignore if the file does not exist anymore.  
pub fn trash_from_list(app_config: &'static AppConfig) {
    let base_local_path = fs::read_to_string(app_config.path_list_base_local_path).unwrap();
    let path_list_local_files = app_config.path_list_destination_files;
    trash_from_list_internal(
        &base_local_path,
        app_config.path_list_for_trash,
        path_list_local_files,
    );
}

/// Move to trash folder the files from list_for_trash.  
/// Ignore if the file does not exist anymore.  
pub fn trash2_from_list(app_config: &'static AppConfig) {
    let base2_local_path = fs::read_to_string(app_config.path_list2_base2_local_path).unwrap();
    trash_from_list_internal(
        &base2_local_path,
        app_config.path_list2_for_trash,
        app_config.path_list2_local_files,
    );
}

/// internal
pub fn trash_from_list_internal(
    base_local_path: &str,
    path_list_for_trash: &str,
    path_list_local_files: &str,
) {
    let list_for_trash = fs::read_to_string(path_list_for_trash).unwrap();
    if list_for_trash.is_empty() {
        println!("{}: 0", path_list_for_trash);
    } else {
        let now_string = chrono::Local::now()
            .format("trash_%Y-%m-%d_%H-%M-%S")
            .to_string();
        let base_trash_path = format!("{}_{}", base_local_path, &now_string);
        if !path::Path::new(&base_trash_path).exists() {
            fs::create_dir_all(&base_trash_path).unwrap();
        }
        //move the files in the same directory structure
        for line_path_for_trash in list_for_trash.lines() {
            let line: Vec<&str> = line_path_for_trash.split("\t").collect();
            let string_path_for_trash = line[0];
            let move_from = format!("{}{}", base_local_path, string_path_for_trash);
            let path_move_from = path::Path::new(&move_from);
            // move to trash if file exists. Nothing if it does not exist, maybe is deleted when moved or in a move_to_trash before.
            if path_move_from.exists() {
                let move_to = format!("{}{}", base_trash_path, string_path_for_trash);
                println!("{}  ->  {}", move_from, move_to);
                let parent = unwrap!(path::Path::parent(path::Path::new(&move_to)));
                if !parent.exists() {
                    fs::create_dir_all(&parent).unwrap();
                }
                unwrap!(fs::rename(&move_from, &move_to));
            }
        }

        // remove lines from list_local_files.csv
        let string_local_files = fs::read_to_string(path_list_local_files).unwrap();
        let vec_sorted_local: Vec<&str> = string_local_files.lines().collect();
        // I must create a new vector.
        let mut string_new_local = String::with_capacity(string_local_files.len());
        println!("sorting local list... It will take a minute or two.");
        for line in vec_sorted_local {
            if !list_for_trash.contains(line) {
                string_new_local.push_str(line);
                string_new_local.push_str("\n");
            }
        }
        // save the new local
        unwrap!(fs::write(path_list_local_files, &string_new_local));

        // empty the list if all is successful
        // println!("empty the list if all is successful");
        unwrap!(fs::write(path_list_for_trash, ""));
    }
}

/// modify the date od files from list_for_correct_time
pub fn correct_time_from_list(app_config: &'static AppConfig) {
    let token = crate::remote_mod::get_short_lived_access_token();
    let client = dropbox_sdk::default_client::UserAuthDefaultClient::new(token);
    let base_local_path = fs::read_to_string(app_config.path_list_base_local_path).unwrap();
    correct_time_from_list_internal(
        RemoteKind::Client { client },
        &base_local_path,
        app_config.path_list_for_correct_time,
    );
}

/// modify the date od files from list_for_correct_time
fn correct_time_from_list_internal(
    client_or_base_path: RemoteKind,
    base_local_path: &str,
    path_list_for_correct_time: &str,
) {
    let list_for_correct_time = fs::read_to_string(path_list_for_correct_time).unwrap();
    for path_to_correct_time in list_for_correct_time.lines() {
        let line: Vec<&str> = path_to_correct_time.split("\t").collect();
        let remote_path = line[0];
        let local_path = format!("{}{}", base_local_path, remote_path);
        if path::Path::new(&local_path).exists() {
            let remote_content_hash = client_or_base_path.get_content_hash(remote_path);
            let local_content_hash = format!(
                "{:x}",
                unwrap!(DropboxContentHasher::hash_file(&local_path))
            );
            if local_content_hash == remote_content_hash {
                let modified = filetime::FileTime::from_system_time(unwrap!(
                    humantime::parse_rfc3339(line[1])
                ));
                unwrap!(filetime::set_file_mtime(local_path, modified));
            } else {
                error!("correct_time content_hash different: {}", remote_path);
            }
        }
    }
    // empty the list
    unwrap!(fs::write(path_list_for_correct_time, ""));
}

/// add just downloaded files to list_local (from dropbox remote)
pub fn add_just_downloaded_to_list_local(app_config: &'static AppConfig) {
    let path_list_local_files = app_config.path_list_destination_files;
    add_just_downloaded_to_list_local_internal(
        app_config.path_list_just_downloaded_or_moved,
        path_list_local_files,
    );
}

/// add just downloaded files to list_local (from external disk)
pub fn add2_just_downloaded_to_list_local(app_config: &'static AppConfig) {
    add_just_downloaded_to_list_local_internal(
        app_config.path_list2_just_downloaded_or_moved,
        app_config.path_list2_local_files,
    );
}

/// add lines from just_downloaded to list_local. Only before compare.
fn add_just_downloaded_to_list_local_internal(
    path_list_just_downloaded: &str,
    path_list_local_files: &str,
) {
    let string_just_downloaded = fs::read_to_string(path_list_just_downloaded).unwrap();
    if !string_just_downloaded.is_empty() {
        // it must be sorted, because downloads are multi-thread and not in sort order
        let string_sorted_just_downloaded = crate::sort_string_lines(&string_just_downloaded);
        let mut vec_sorted_downloaded: Vec<&str> = string_sorted_just_downloaded.lines().collect();
        // It is forbidden to have duplicate lines
        vec_sorted_downloaded.dedup();
        println!(
            "{}: {}",
            path_list_just_downloaded.split("/").collect::<Vec<&str>>()[1],
            vec_sorted_downloaded.len()
        );
        unwrap!(fs::write(
            path_list_just_downloaded,
            &string_sorted_just_downloaded
        ));

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

/// copies files from external disk backup_1 to backup_2
pub fn copy_from_list2_for_download(path_list2_for_download: &str, app_config: &'static AppConfig) {
    let list2_for_download = fs::read_to_string(path_list2_for_download).unwrap();
    let base_source_path = fs::read_to_string(app_config.path_list_base_local_path).unwrap();
    let base_local_path = fs::read_to_string(app_config.path_list2_base2_local_path).unwrap();

    if !list2_for_download.is_empty() {
        println!(
            "{}{}copy_from_list2_for_download{}",
            at_line(1),
            *YELLOW,
            *RESET
        );

        let mut just_downloaded = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(app_config.path_list2_just_downloaded_or_moved)
            .unwrap();
        for line_path_to_download in list2_for_download.lines() {
            let line: Vec<&str> = line_path_to_download.split("\t").collect();
            let path_to_download = line[0];
            let path_from = format!("{}{}", base_source_path, path_to_download);
            let path_to = format!("{}{}", base_local_path, path_to_download);

            let parent = unwrap!(path::Path::parent(path::Path::new(&path_to)));
            if !parent.exists() {
                fs::create_dir_all(&parent).unwrap();
            }
            println!("{} -> {}", &path_from, &path_to);
            // remove readonly attribute
            if path::Path::new(&path_to).exists() {
                let mut perms = unwrap!(fs::metadata(&path_to)).permissions();
                if perms.readonly() == true {
                    perms.set_readonly(false);
                    unwrap!(fs::set_permissions(&path_to, perms));
                }
            }
            unwrap!(fs::copy(&path_from, &path_to));
            // copy also the modified file date time
            use chrono::offset::Utc;
            use chrono::DateTime;
            let modified_system_time = unwrap!(unwrap!(fs::metadata(&path_from)).modified());
            let modified_date_time_utc: DateTime<Utc> = modified_system_time.into();
            let modified_file_time = filetime::FileTime::from_system_time(modified_system_time);
            unwrap!(filetime::set_file_times(
                &path_to,
                modified_file_time,
                modified_file_time
            ));

            // write to file list2_just_downloaded_or_moved.
            let line_to_append = format!(
                "{}\t{}\t{}",
                path_to_download,
                modified_date_time_utc.format("%Y-%m-%dT%TZ"),
                unwrap!(fs::metadata(&path_to)).len()
            );
            println!("{}", &line_to_append);
            unwrap!(writeln!(just_downloaded, "{}", line_to_append));
        }
    } else {
        println!("list2_for_download: 0");
    }
}
