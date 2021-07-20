//! remote_mod.rs
use dropbox_sdk::default_client::UserAuthDefaultClient;
use dropbox_sdk::files;

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use unwrap::unwrap;
use uncased::UncasedStr;

use crate::*;

pub fn test_connection() {
    let token = get_short_lived_access_token();
    let client = UserAuthDefaultClient::new(token);
    match files::list_folder(&client, &files::ListFolderArg::new("".to_string())) {
        Ok(Ok(_result)) => println!(
            "{}",
            Green.paint("test connection and authorization: ok")
        ),
        Ok(Err(e)) => println!("error: {}", e),
        Err(e) => println!("error: {}", e),
    }
}

fn get_short_lived_access_token() -> String {
    // The user must prepare the short-lived access token in the environment variable
    let token = match env::var("DBX_OAUTH_TOKEN") {
        Ok(token) => {
            println!("short-lived access token read from env var DBX_OAUTH_TOKEN.");
            token
        }
        Err(_err) => {
            panic!("Error: The short-lived access token is not found in the env variable DBX_OAUTH_TOKEN.");
        }
    };
    // return
    token
}

// get remote list in parallel
// first get the first level of folders and then request in parallel sub-folders recursively
pub fn list_remote(){   
    println!("list_remote()");
    let token = get_short_lived_access_token();
    let token_clone2 = token.to_owned().clone();
    let client = UserAuthDefaultClient::new(token_clone2.to_owned());

    // walkdir non-recursive for the first level of folders
    let (folder_list, file_list) = list_remote_folder(&client,"/",0, false);
    let mut folder_list_all = folder_list;
    let mut file_list_all = file_list;

    // these folders will request walkdir recursive in parallel

    // channel for inter-thread communication.
    let (tx, rx) = std::sync::mpsc::channel();
    // threadpool with 3 threads

    let mut pool = scoped_threadpool::Pool::new(3);
    pool.scoped(|scoped| {
        let mut thread_num = 0;
        for folder_path in &folder_list_all{
            let folder_path = folder_path.clone();
            let tx_clone2 = mpsc::Sender::clone(&tx);
            let token_clone2 = token.to_owned().clone();
            // execute in a separate threads, or waits for a free thread from the pool
            scoped.execute(move || {                
                let client = UserAuthDefaultClient::new(token_clone2.to_owned());
                // recursive walkdir
                let (folder_list, file_list) = list_remote_folder(&client,&folder_path,thread_num, true);
                // folder_list is appended to folder_list_all in every thread
                unwrap!( tx_clone2.send((folder_list, file_list)));                    
            });
            thread_num = increment_and_loop(thread_num, 0, 2);
        }          
        drop(tx);
    });
    // the receiver from the threads message
    for msg in rx {
        let (folder_list, file_list) = msg;
        folder_list_all.extend_from_slice  (&folder_list);
        file_list_all.extend_from_slice(&file_list);
        println!("{}folders {}",term_cursor::Goto(0,14), folder_list_all.len());
        println!("{}files {}",term_cursor::Goto(0,15),file_list_all.len());
    }
    sort_remote_list_and_write_to_file(file_list_all);
}

pub fn list_remote_folder(client:&UserAuthDefaultClient,path:&str,thread_num:i32, recursive:bool)->(Vec<String>,Vec<String>){
    let mut folder_list:Vec<String> = vec![];
    let mut file_list:Vec<String> = vec![];
    match list_directory(&client, path, recursive) {
        Ok(Ok(iterator)) => {
            let mut folder_count = 0;
            for entry_result in iterator {
                match entry_result {
                    Ok(Ok(files::Metadata::Folder(entry))) => {
                        // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                        let folder_path = entry.path_display.unwrap_or(entry.name);                        
                        println!(
                            "{}{}Folder: {}",
                            term_cursor::Goto(0,4+thread_num*2),
                            clear_line(),
                            &folder_path
                        );
                        folder_list.push(folder_path);
                        println!("{}{}Folder_count: {}", term_cursor::Goto(0,5+thread_num*2),clear_line(), folder_count);
                        folder_count += 1;                        
                    }
                    Ok(Ok(files::Metadata::File(entry))) => {
                        // write csv tab delimited
                        file_list.push(format!(
                            "{}\t{}\t{}\n",
                            // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                            entry.path_display.unwrap_or(entry.name),
                            entry.client_modified,
                            entry.size
                        ));
                    }
                    Ok(Ok(files::Metadata::Deleted(entry))) => {
                        panic!("{}{}unexpected deleted entry: {:?}", term_cursor::Goto(0,10),clear_line(), entry);
                    }
                    Ok(Err(e)) => {
                        println!(
                            "{}{}Error from files/list_folder_continue: {}",
                            term_cursor::Goto(0,10),
                            clear_line(),
                            e
                        );
                        break;
                    }
                    Err(e) => {
                        println!("{}{}API request error: {}", term_cursor::Goto(0,10),clear_line(), e);
                        break;
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("Error from files/list_folder: {}", e);
        }
        Err(e) => {
            println!("API request error: {}", e);
        }
    }
    // return
    (folder_list, file_list)
}

pub fn sort_remote_list_and_write_to_file(mut file_list_all:Vec<String>){
    println!("remote list sort {}", "");        
    use rayon::prelude::*;
    file_list_all.par_sort_unstable_by(|a,b|{
        let aa: &UncasedStr = a.as_str().into();
        let bb: &UncasedStr = b.as_str().into();
        aa.cmp(bb)
    } );
    // join to string and write to file
    let string_file_list_all = file_list_all.join("\n");
    println!("remote list sorted local len(): {}", string_file_list_all.len());            
    unwrap!(fs::write("temp_data/list_remote_files.csv", string_file_list_all));
}

/// download one file
pub fn download(download_path: &str) {
    let token = get_short_lived_access_token();
    let client = UserAuthDefaultClient::new(token);
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    download_with_client(download_path, &client, &base_local_path,1);
}
/// download one file with client
pub fn download_with_client(download_path: &str, client: &UserAuthDefaultClient, base_local_path: &str, thread_num:i32) {
    //println!("start download: {}", download_path);
    let mut bytes_out = 0u64;
    let download_arg = files::DownloadArg::new(download_path.to_string());
    let local_path = format!("{}{}", base_local_path, download_path);
    // println!("to local path: {}", local_path);
    // create folder if it does not exist
    let path = std::path::PathBuf::from(&local_path);
    let parent = path.parent().unwrap();
    if !std::path::Path::new(&parent).exists() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let base_temp_download_path = format!("{}_temp_download", &base_local_path);
    if !std::path::Path::new(&base_temp_download_path).exists() {
        std::fs::create_dir_all(&base_temp_download_path).unwrap();
    }
    let temp_local_path = format!("{}{}", base_temp_download_path, download_path);
    // create temp folder if it does not exist
    let temp_path = std::path::PathBuf::from(&temp_local_path);
    let temp_parent = temp_path.parent().unwrap();
    if !std::path::Path::new(&temp_parent).exists() {
        std::fs::create_dir_all(temp_parent).unwrap();
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&temp_local_path)
        .unwrap();

    let mut modified: Option<filetime::FileTime> = None;
    let mut s_modified="".to_string();
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
                            //println!("\n");
                            break 'download;
                        }
                        Ok(len) => {
                            bytes_out += len as u64;
                            if let Some(total) = download_result.content_length {      
                                let (x, y) = unwrap!(term_cursor::get_pos());
                                println!("{}{}{:.01}% of {:.02} Mb downloading {}", term_cursor::Goto(0,thread_num),clear_line(), bytes_out as f64 / total as f64 * 100.,total as f64 / 1000000.,download_path);
                                unwrap!(term_cursor::set_pos(x, y));
                            } else {
                                let (x, y) = unwrap!(term_cursor::get_pos()); 
                                println!("{}{}{} Mb downloaded {}",term_cursor::Goto(0,thread_num),clear_line(), bytes_out as f64 / 1000000.,download_path);
                                unwrap!(term_cursor::set_pos(x, y));
                            }
                        }
                        Err(e) => {
                            println!("Read error: {}", e);
                            continue 'download; // do another request and resume
                        }
                    }
                }
            }
            Ok(Err(download_error)) => {
                println!("Download error: {}", download_error);
            }
            Err(request_error) => {
                println!("Error: {}", request_error);
            }
        }

        break 'download;
    }
    let atime = unwrap!(modified);
    let mtime = unwrap!(modified);
    unwrap!(filetime::set_file_times(&temp_local_path, atime, mtime));
    // move-rename the completed download file o his final folder
    unwrap!( std::fs::rename(&temp_local_path, &local_path));
    // write to file list_just_downloaded. 
    // multi-thread no problem: append is atomic on most OS <https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create>
    let list_just_downloaded = "temp_data/list_just_downloaded.csv";
    let line_to_append = format!("{}\t{}\t{}\n", download_path, s_modified, bytes_out);
    print!("{}",&line_to_append);
    let mut just_downloaded = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(list_just_downloaded)
        .unwrap();
    unwrap!(just_downloaded.write (line_to_append.as_bytes()));
}

pub fn download_from_list() {
    println!("download_from_list");
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let list_for_download = std::fs::read_to_string("temp_data/list_for_download.csv").unwrap();
    let token = get_short_lived_access_token();
    let base_local_path_ref = &base_local_path;
    let client = UserAuthDefaultClient::new(token);
    let client_ref=&client;
    // 3 threads to download in parallel
    let mut pool = scoped_threadpool::Pool::new(3);
    pool.scoped(|scoped| {
        let mut thread_num = 1;
        for path in list_for_download.lines(){    
            // execute in a separate threads, or waits for a free thread from the pool     
            scoped.execute(move || {                
                download_with_client(path, client_ref, base_local_path_ref, thread_num);
            });
            thread_num = increment_and_loop(thread_num,1,3);            
        }
    });
}

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
