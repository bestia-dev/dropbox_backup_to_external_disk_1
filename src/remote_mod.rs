// remote_mod.rs
//! Module contains all the communication with the remote dropbox storage.

use dropbox_sdk::default_client::UserAuthDefaultClient;
use dropbox_sdk::files;

#[allow(unused_imports)]
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path;
use std::sync::mpsc;
use std::thread;
use uncased::UncasedStr;
use unwrap::unwrap;

use crate::*;

/// test authentication with dropbox.com
pub fn test_connection() {
    let token = get_short_lived_access_token();
    let client = UserAuthDefaultClient::new(token);
    match files::list_folder(&client, &files::ListFolderArg::new("".to_string())) {
        Ok(Ok(_result)) => println!("{}test connection and authorization: ok{}", *GREEN, *RESET),
        Ok(Err(e)) => panic!("error: {}", e),
        Err(e) => panic!("error: {}", e),
    }
}

/// get token from env variable
pub fn get_short_lived_access_token() -> dropbox_sdk::oauth2::Authorization {
    // The user must prepare the short-lived access token in the environment variable
    let token = match env::var("DBX_OAUTH_TOKEN") {
        Ok(token) => {
            //println!("short-lived access token read from env var DBX_OAUTH_TOKEN.");
            token
        }
        Err(_err) => {
            panic!("Error: The short-lived access token is not found in the env variable DBX_OAUTH_TOKEN.");
        }
    };
    // return
    dropbox_sdk::oauth2::Authorization::from_access_token(token)
}

/// get remote list in parallel
/// first get the first level of folders and then request in parallel sub-folders recursively
pub fn list_remote() {
    // empty the file. I want all or nothing result here if the process is terminated prematurely.
    unwrap!(fs::write("temp_data/list_remote_files.csv", ""));

    let token = get_short_lived_access_token();
    let token_clone2 = token.to_owned().clone();
    let client = UserAuthDefaultClient::new(token_clone2.to_owned());

    // channel for inter-thread communication.
    let (tx, rx) = mpsc::channel();
    let tx_clone3 = mpsc::Sender::clone(&tx);

    let (x_screen_len, _y_screen_len) = unwrap!(termion::terminal_size());

    // walkdir non-recursive for the first level of folders
    let (folder_list, file_list) =
        list_remote_folder(&client, "/", 0, false, tx_clone3, x_screen_len);
    let folder_list_root = folder_list.clone();
    let mut folder_list_all = folder_list;
    let mut file_list_all = file_list;

    // these folders will request walkdir recursive in parallel
    // loop in a new thread, so the send msg will come immediately
    let _sender_thread = thread::spawn(move || {
        // threadpool with 3 threads
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(3)
            .build()
            .unwrap();
        pool.scope(|scoped| {
            for folder_path in &folder_list_root {
                let folder_path = folder_path.clone();
                let tx_clone2 = mpsc::Sender::clone(&tx);
                let tx_clone4 = mpsc::Sender::clone(&tx);
                let token_clone2 = token.to_owned().clone();
                // execute in a separate threads, or waits for a free thread from the pool
                scoped.spawn(move |_s| {
                    let client = UserAuthDefaultClient::new(token_clone2.to_owned());
                    // recursive walkdir
                    let thread_num = unwrap!(rayon::current_thread_index()) as i32;
                    let (folder_list, file_list) = list_remote_folder(
                        &client,
                        &folder_path,
                        thread_num,
                        true,
                        tx_clone2,
                        x_screen_len,
                    );
                    unwrap!(tx_clone4.send((Some(folder_list), Some(file_list), 0, 0)));
                });
            }
            drop(tx);
        });
    });

    // the receiver reads all msgs from the queue, until senders exist - drop(tx)
    let mut all_folder_count = 0;
    let mut all_file_count = 0;
    for msg in &rx {
        let (folder_list, file_list, folder_count, file_count) = msg;
        if let Some(folder_list) = folder_list {
            folder_list_all.extend_from_slice(&folder_list);
        }
        if let Some(file_list) = file_list {
            file_list_all.extend_from_slice(&file_list);
        }
        all_folder_count += folder_count;
        all_file_count += file_count;
        println!(
            "{}{}remote_folder_count: {}",
            at_line(7),
            *CLEAR_LINE,
            all_folder_count
        );
        println!(
            "{}{}remote_file_count: {}",
            at_line(8),
            *CLEAR_LINE,
            all_file_count
        );
    }

    sort_remote_list_and_write_to_file(file_list_all);
}

/// list remote folder
pub fn list_remote_folder(
    client: &UserAuthDefaultClient,
    path: &str,
    thread_num: i32,
    recursive: bool,
    tx_clone: mpsc::Sender<(Option<Vec<String>>, Option<Vec<String>>, i32, i32)>,
    x_screen_len: u16,
) -> (Vec<String>, Vec<String>) {
    let mut folder_list: Vec<String> = vec![];
    let mut file_list: Vec<String> = vec![];
    let screen_line = 4 + thread_num as u16;
    match list_directory(&client, path, recursive) {
        Ok(Ok(iterator)) => {
            for entry_result in iterator {
                match entry_result {
                    Ok(Ok(files::Metadata::Folder(entry))) => {
                        // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                        let folder_path = entry.path_display.unwrap_or(entry.name);
                        // for 3 threads this is lines: 4,5, 6,7, 8,9, so summary can be on 10,11 and list_local on 16,17
                        println!(
                            "{}{}{}. Folder: {}",
                            at_line(screen_line),
                            *CLEAR_LINE,
                            thread_num,
                            shorten_string(&folder_path, x_screen_len - 11)
                        );
                        folder_list.push(folder_path);
                        unwrap!(tx_clone.send((None, None, 1, 0)));
                    }
                    Ok(Ok(files::Metadata::File(entry))) => {
                        // write csv tab delimited
                        // avoid strange files *com.dropbox.attrs
                        // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                        let file_path = entry.path_display.unwrap_or(entry.name);
                        if !file_path.ends_with("com.dropbox.attrs") {
                            file_list.push(format!(
                                "{}\t{}\t{}",
                                file_path, entry.client_modified, entry.size
                            ));
                            unwrap!(tx_clone.send((None, None, 0, 1)));
                        }
                    }
                    Ok(Ok(files::Metadata::Deleted(entry))) => {
                        panic!(
                            "{}{}{}unexpected deleted entry: {:?}{}",
                            at_line(screen_line),
                            *CLEAR_LINE,
                            *RED,
                            entry,
                            *RESET
                        );
                    }
                    Ok(Err(e)) => {
                        println!(
                            "{}{}{}Error from files/list_folder_continue: {}{}",
                            at_line(screen_line),
                            *CLEAR_LINE,
                            *RED,
                            e,
                            *RESET
                        );
                        break;
                    }
                    Err(e) => {
                        println!(
                            "{}{}{}API request error: {}{}",
                            at_line(screen_line),
                            *CLEAR_LINE,
                            *RED,
                            e,
                            *RESET
                        );
                        break;
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!(
                "{}{}{}Error from files/list_folder: {}{}",
                at_line(screen_line),
                *CLEAR_LINE,
                *RED,
                e,
                *RESET
            );
        }
        Err(e) => {
            println!(
                "{}{}{}API request error: {}{}",
                at_line(screen_line),
                *CLEAR_LINE,
                *RED,
                e,
                *RESET
            );
        }
    }
    // return
    (folder_list, file_list)
}

/// sort and write to file
pub fn sort_remote_list_and_write_to_file(mut file_list_all: Vec<String>) {
    print!("{}remote list sort", at_line(9));

    use rayon::prelude::*;
    file_list_all.par_sort_unstable_by(|a, b| {
        let aa: &UncasedStr = a.as_str().into();
        let bb: &UncasedStr = b.as_str().into();
        aa.cmp(bb)
    });
    // join to string and write to file
    println!(
        "{}list_remote_files sorted lines: {}",
        at_line(9),
        file_list_all.len()
    );
    let string_file_list_all = file_list_all.join("\n");
    unwrap!(fs::write(
        "temp_data/list_remote_files.csv",
        string_file_list_all
    ));
}

/// download one file
pub fn download_one_file(path_to_download: &str) {
    let token = get_short_lived_access_token();
    let client = UserAuthDefaultClient::new(token);
    let base_local_path = fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let (x_screen_len, _y_screen_len) = unwrap!(termion::terminal_size());
    // channel for inter-thread communication.
    let (tx, rx) = mpsc::channel();
    let path_to_download = path_to_download.to_string();
    let _sender_thread = thread::spawn(move || {
        let base_local_path_ref = &base_local_path;
        let client_ref = &client;
        let thread_num = 0;
        let tx_clone2 = mpsc::Sender::clone(&tx);
        download_internal(
            &path_to_download,
            client_ref,
            base_local_path_ref,
            thread_num,
            tx_clone2,
            x_screen_len,
        );
        drop(tx);
    });
    // the receiver reads all msgs from the queue, until all senders exist - drop(tx)
    // only this thread writes to the terminal, to avoid race in cursor position
    for msg in &rx {
        let (string_to_print, thread_num) = msg;
        if thread_num != -1 {
            println!("\r{}{}", "\x1b[1F", &string_to_print);
        } else {
            println!("{}", &string_to_print);
        }
    }
}

/// download one file with client object UserAuthDefaultClient
fn download_internal(
    download_path: &str,
    client: &UserAuthDefaultClient,
    base_local_path: &str,
    thread_num: i32,
    tx_clone: mpsc::Sender<(String, i32)>,
    x_screen_len: u16,
) {
    //log::trace!("download_with_client: {}",download_path);
    let mut bytes_out = 0u64;
    let download_arg = files::DownloadArg::new(download_path.to_string());
    log::trace!("download_arg: {}", &download_arg.path);
    let local_path = format!("{}{}", base_local_path, download_path);
    // create folder if it does not exist
    let path_of_local_path = path::PathBuf::from(&local_path);
    let parent = path_of_local_path.parent().unwrap();
    if !path::Path::new(&parent).exists() {
        fs::create_dir_all(parent).unwrap();
    }
    let base_temp_download_path = format!("{}_temp_download", &base_local_path);
    if !path::Path::new(&base_temp_download_path).exists() {
        fs::create_dir_all(&base_temp_download_path).unwrap();
    }
    let temp_local_path = format!("{}{}", base_temp_download_path, download_path);
    // create temp folder if it does not exist
    let temp_path = path::PathBuf::from(&temp_local_path);
    let temp_parent = temp_path.parent().unwrap();
    if !path::Path::new(&temp_parent).exists() {
        fs::create_dir_all(temp_parent).unwrap();
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&temp_local_path)
        .unwrap();

    let mut modified: Option<filetime::FileTime> = None;
    let mut s_modified = "".to_string();
    // I will download to a temp folder and then move the file to the right folder only when the download is complete.
    'download: loop {
        let result = files::download(client, &download_arg, Some(bytes_out), None);
        match result {
            Ok(Ok(download_result)) => {
                let mut body = download_result.body.expect("no body received!");
                if modified.is_none() {
                    s_modified = download_result.result.client_modified.clone();
                    modified = Some(filetime::FileTime::from_system_time(unwrap!(
                        humantime::parse_rfc3339(&s_modified)
                    )));
                };
                loop {
                    // limit read to 1 MiB per loop iteration so we can output progress
                    let mut input_chunk = (&mut body).take(1024 * 1024);
                    match io::copy(&mut input_chunk, &mut file) {
                        Ok(0) => {
                            break 'download;
                        }
                        Ok(len) => {
                            bytes_out += len as u64;
                            if let Some(total) = download_result.content_length {
                                let string_to_print = format!(
                                    "{}{:.01}% of {:.02} Mb downloading {}",
                                    *CLEAR_LINE,
                                    bytes_out as f64 / total as f64 * 100.,
                                    total as f64 / 1000000.,
                                    shorten_string(download_path, x_screen_len - 31)
                                );
                                unwrap!(tx_clone.send((string_to_print, thread_num)));
                            } else {
                                let string_to_print = format!(
                                    "{}{} Mb downloaded {}",
                                    *CLEAR_LINE,
                                    bytes_out as f64 / 1000000.,
                                    shorten_string(download_path, x_screen_len - 31)
                                );
                                unwrap!(tx_clone.send((string_to_print, thread_num)));
                            }
                        }
                        Err(e) => {
                            let string_to_print = format!("{}Read error: {}{}", *RED, e, *RESET);
                            unwrap!(tx_clone.send((string_to_print, -1)));
                            continue 'download; // do another request and resume
                        }
                    }
                }
            }
            Ok(Err(download_error)) => {
                let string_to_print =
                    format!("{}Download error: {}{}", *RED, download_error, *RESET);
                unwrap!(tx_clone.send((string_to_print, -1)));
            }
            Err(request_error) => {
                let string_to_print = format!("{}Error: {}{}", *RED, request_error, *RESET);
                unwrap!(tx_clone.send((string_to_print, -1)));
            }
        }
        break 'download;
    }
    let atime = unwrap!(modified);
    let mtime = unwrap!(modified);
    unwrap!(filetime::set_file_times(&temp_local_path, atime, mtime));

    //Some files are read-only. For example .git files.
    //Check the attribute, remember it and remove the read-only.
    //let mut is_read_only = false;
    if path_of_local_path.exists() {
        let mut perms = unwrap!(fs::metadata(&path_of_local_path)).permissions();
        if perms.readonly() == true {
            //is_read_only = true;
            perms.set_readonly(false);
            unwrap!(fs::set_permissions(&path_of_local_path, perms));
        }
    }

    // move-rename the completed download file to his final folder
    unwrap!(fs::rename(&temp_local_path, &local_path));

    // write to file list_just_downloaded_or_moved.
    // multi-thread no problem: append is atomic on most OS <https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create>
    let list_just_downloaded_or_moved = "temp_data/list_just_downloaded_or_moved.csv";
    let line_to_append = format!("{}\t{}\t{}", download_path, s_modified, bytes_out);
    let string_to_print = format!("{}", &line_to_append);
    unwrap!(tx_clone.send((string_to_print, -1)));
    let mut just_downloaded = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(list_just_downloaded_or_moved)
        .unwrap();
    unwrap!(writeln!(just_downloaded, "{}", line_to_append));
}

/// download files from list
pub fn download_from_list() {
    let base_local_path = fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let list_for_download = fs::read_to_string("temp_data/list_for_download.csv").unwrap();

    if !list_for_download.is_empty() {
        let mut hide_cursor_terminal = crate::start_hide_cursor_terminal();
        let (x_screen_len, _y_screen_len) = unwrap!(termion::terminal_size());
        let (_x, y) = get_pos(&mut hide_cursor_terminal);

        println!("{}{}download_from_list{}", at_line(1), *YELLOW, *RESET);
        print!("{}", at_line(y));

        let token = get_short_lived_access_token();
        let client = UserAuthDefaultClient::new(token);
        // channel for inter-thread communication.
        let (tx, rx) = mpsc::channel();

        // loop in a new thread, so the send msg will come immediately
        let _sender_thread = thread::spawn(move || {
            let base_local_path_ref = &base_local_path;
            let client_ref = &client;
            // 3 threads to download in parallel
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(3)
                .build()
                .unwrap();
            pool.scope(|scoped| {
                for line_path_to_download in list_for_download.lines() {
                    let line: Vec<&str> = line_path_to_download.split("\t").collect();
                    let path_to_download = line[0];
                    let modified_for_download = line[1];
                    let file_size: i32 = line[2].parse().unwrap();
                    if file_size == 0 {
                        // create an empty file, because download empty file causes error 416
                        let local_path = format!("{}{}", base_local_path, path_to_download);
                        let path = path::PathBuf::from(&local_path);
                        let parent = path.parent().unwrap();
                        if !path::Path::new(&parent).exists() {
                            fs::create_dir_all(parent).unwrap();
                        }
                        let mut file = fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(&local_path)
                            .unwrap();
                        unwrap!(writeln!(file, "{}", ""));
                        // change the file date
                        let system_time = unwrap!(humantime::parse_rfc3339(modified_for_download));
                        let modified = filetime::FileTime::from_system_time(system_time);
                        let atime = modified;
                        let mtime = modified;
                        unwrap!(filetime::set_file_times(&local_path, atime, mtime));

                        unwrap!(tx.send((format!("{}", &local_path), -1)));

                        // append to list_just_downloaded
                        let list_just_downloaded_or_moved =
                            "temp_data/list_just_downloaded_or_moved.csv";
                        let mut just_downloaded = fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(list_just_downloaded_or_moved)
                            .unwrap();
                        unwrap!(writeln!(just_downloaded, "{}", line_path_to_download));
                    } else {
                        let tx_clone2 = mpsc::Sender::clone(&tx);
                        // execute in a separate threads, or waits for a free thread from the pool
                        scoped.spawn(move |_s| {
                            let thread_num = unwrap!(rayon::current_thread_index()) as i32;
                            download_internal(
                                path_to_download,
                                client_ref,
                                base_local_path_ref,
                                thread_num,
                                tx_clone2,
                                x_screen_len,
                            );
                        });
                    }
                }
                drop(tx);
            });
        });
        // the receiver reads all msgs from the queue, until senders exist - drop(tx)
        // only this thread writes to the terminal, to avoid race in cursor position

        let mut string_to_print_1 = "".to_string();
        let mut string_to_print_2 = "".to_string();
        let mut string_to_print_3 = "".to_string();
        for msg in &rx {
            let (string_to_print, thread_num) = msg;
            if thread_num != -1 {
                let (_x, y) = get_pos(&mut hide_cursor_terminal);
                println!("{}{}", at_line(3 + thread_num as u16), &string_to_print);
                print!("{}", at_line(y),);
                if thread_num == 0 {
                    string_to_print_1 = string_to_print;
                } else if thread_num == 1 {
                    string_to_print_2 = string_to_print;
                } else if thread_num == 2 {
                    string_to_print_3 = string_to_print;
                }
            } else {
                let (_x, y) = get_pos(&mut hide_cursor_terminal);
                // there is annoying jumping because of scrolling
                // let clear first and write second
                println!("{}{}", at_line(7), termion::clear::BeforeCursor);
                print!("{}", at_line(y));

                println!("{}", &string_to_print);

                let (_x, y) = get_pos(&mut hide_cursor_terminal);
                // print the first 6 lines, because of scrolling
                println!("{}{}download_from_list{}", at_line(1), *YELLOW, *RESET);
                println!("{}", *CLEAR_LINE);
                println!("{}", &string_to_print_1);
                println!("{}", &string_to_print_2);
                println!("{}", &string_to_print_3);
                println!("{}", *CLEAR_LINE);
                print!("{}", at_line(y));
            }
        }
        // delete the temp folder
        let base_local_path = fs::read_to_string("temp_data/base_local_path.csv").unwrap();
        let base_temp_download_path = format!("{}_temp_download", &base_local_path);
        fs::remove_dir_all(base_temp_download_path).unwrap_or(());
    } else {
        println!("list_for_download: 0");
    }
    println!("{}compare remote and local lists{}", *YELLOW, *RESET);
    compare_lists();
}

/// list directory
fn list_directory<'a>(
    client: &'a UserAuthDefaultClient,
    path: &str,
    recursive: bool,
) -> dropbox_sdk::Result<Result<DirectoryIterator<'a>, files::ListFolderError>> {
    assert!(
        path.starts_with('/'),
        "path needs to be absolute (start with a '/')"
    );
    let requested_path = if path == "/" {
        // Root folder should be requested as empty string
        String::new()
    } else {
        path.to_owned()
    };
    match files::list_folder(
        client,
        &files::ListFolderArg::new(requested_path).with_recursive(recursive),
    ) {
        Ok(Ok(result)) => {
            let cursor = if result.has_more {
                Some(result.cursor)
            } else {
                None
            };

            Ok(Ok(DirectoryIterator {
                client,
                cursor,
                buffer: result.entries.into(),
            }))
        }
        Ok(Err(e)) => Ok(Err(e)),
        Err(e) => Err(e),
    }
}

/// iterator for Directory on remote Dropbox storage
struct DirectoryIterator<'a> {
    client: &'a UserAuthDefaultClient,
    buffer: VecDeque<files::Metadata>,
    cursor: Option<String>,
}

impl<'a> Iterator for DirectoryIterator<'a> {
    type Item = dropbox_sdk::Result<Result<files::Metadata, files::ListFolderContinueError>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.buffer.pop_front() {
            Some(Ok(Ok(entry)))
        } else if let Some(cursor) = self.cursor.take() {
            match files::list_folder_continue(
                self.client,
                &files::ListFolderContinueArg::new(cursor),
            ) {
                Ok(Ok(result)) => {
                    self.buffer.extend(result.entries.into_iter());
                    if result.has_more {
                        self.cursor = Some(result.cursor);
                    }
                    self.buffer.pop_front().map(|entry| Ok(Ok(entry)))
                }
                Ok(Err(e)) => Some(Ok(Err(e))),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}

/// get content_hash from remote
pub fn remote_content_hash(remote_path: &str, client: &UserAuthDefaultClient) -> Option<String> {
    let arg = files::GetMetadataArg::new(remote_path.to_string());
    let res_res_metadata = dropbox_sdk::files::get_metadata(client, &arg);

    match res_res_metadata {
        Ok(Ok(files::Metadata::Folder(_entry))) => {
            return None;
        }
        Ok(Ok(files::Metadata::File(entry))) => {
            return Some(unwrap!(entry.content_hash));
        }
        Ok(Ok(files::Metadata::Deleted(_entry))) => {
            return None;
        }
        Ok(Err(e)) => {
            println!("Error get metadata: {}", e);
            return None;
        }
        Err(e) => {
            println!("API request error: {}", e);
            return None;
        }
    }
}
