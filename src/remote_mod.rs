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
use std::thread;
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

fn get_short_lived_access_token() -> dropbox_sdk::oauth2::Authorization {
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
    dropbox_sdk::oauth2::Authorization::from_access_token(token)
}

// get remote list in parallel
// first get the first level of folders and then request in parallel sub-folders recursively
pub fn list_remote(){   
    //println!("list_remote()");
    let token = get_short_lived_access_token();
    let token_clone2 = token.to_owned().clone();
    let client = UserAuthDefaultClient::new(token_clone2.to_owned());
    print!("{}",term_cursor::Goto(0,10));
    println!("num folders: {}",0);
    println!("num files: {}",0);

    // walkdir non-recursive for the first level of folders
    let (folder_list, file_list) = list_remote_folder(&client,"/",0, false);
    let folder_list_root = folder_list.clone();
    let mut folder_list_all = folder_list;
    let mut file_list_all = file_list;
    print!("{}",term_cursor::Goto(0,10));
    println!("num folders: {}", folder_list_all.len());
    println!("num files: {}",file_list_all.len());

    // these folders will request walkdir recursive in parallel

    // channel for inter-thread communication.
    let (tx, rx) = std::sync::mpsc::channel();
    
    // loop in a new thread, so the send msg will come immediately
    let _sender_thread = thread::spawn(move || {
        // threadpool with 3 threads
        let pool = rayon::ThreadPoolBuilder::new().num_threads(3).build().unwrap();
        pool.scope(|scoped|{
            for folder_path in &folder_list_root{
                let folder_path = folder_path.clone();
                let tx_clone2 = mpsc::Sender::clone(&tx);
                let token_clone2 = token.to_owned().clone();
                // execute in a separate threads, or waits for a free thread from the pool
                scoped.spawn(move |_s| {                
                    let client = UserAuthDefaultClient::new(token_clone2.to_owned());
                    // recursive walkdir
                    let thread_num =unwrap!(rayon::current_thread_index()) as i32;
                    let (folder_list, file_list) = list_remote_folder(&client,&folder_path,thread_num, true);
                    // folder_list is appended to folder_list_all in every thread
                    unwrap!( tx_clone2.send((folder_list, file_list)));                    
                });
            
            }          
            drop(tx);            
        });
    });
    
    // the receiver reads all msgs from the queue, until senders exist - drop(tx)
    for msg in &rx {
        let (folder_list, file_list) = msg;
        folder_list_all.extend_from_slice  (&folder_list);
        file_list_all.extend_from_slice(&file_list);
        // why this does not print on the terminal in real time? But only at the end of all threads?
        print!("{}",term_cursor::Goto(0,10));
        println!("num folders: {}", folder_list_all.len());
        println!("num files: {}", file_list_all.len());
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
                        // for 3 threads this is lines: 4,5, 6,7, 8,9, so summary can be on 10,11 and list_local on 16,17
                        print!("{}",term_cursor::Goto(0,4+thread_num*2));
                        println!(
                            "{}Folder: {}",                            
                            clear_line(),
                            &folder_path
                        );                        
                        println!("{}Folder_count: {}", clear_line(), folder_count);
                        folder_list.push(folder_path);
                        folder_count += 1;                        
                    }
                    Ok(Ok(files::Metadata::File(entry))) => {
                        // write csv tab delimited
                        file_list.push(format!(
                            "{}\t{}\t{}",
                            // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                            entry.path_display.unwrap_or(entry.name),
                            entry.client_modified,
                            entry.size
                        ));
                    }
                    Ok(Ok(files::Metadata::Deleted(entry))) => {
                        print!("{}",term_cursor::Goto(0,10));
                        panic!("{}unexpected deleted entry: {:?}",clear_line(), entry);
                    }
                    Ok(Err(e)) => {
                        print!("{}",term_cursor::Goto(0,13));
                        println!(
                            "{}Error from files/list_folder_continue: {}",                            
                            clear_line(),
                            e
                        );
                        break;
                    }
                    Err(e) => {
                        print!("{}",term_cursor::Goto(0,10));
                        println!("{}API request error: {}",clear_line(), e);
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
    print!("{}",term_cursor::Goto(0,13));
    println!("remote list sort");
    
    use rayon::prelude::*;
    file_list_all.par_sort_unstable_by(|a,b|{
        let aa: &UncasedStr = a.as_str().into();
        let bb: &UncasedStr = b.as_str().into();
        aa.cmp(bb)
    } );
    // join to string and write to file
    let string_file_list_all = file_list_all.join("\n");
    print!("{}",term_cursor::Goto(0,13));
    println!("remote list sorted local len(): {}",  string_file_list_all.len());            
    unwrap!(fs::write("temp_data/list_remote_files.csv", string_file_list_all));
}

/// download one file
pub fn download(path_to_download: &str) {
    let token = get_short_lived_access_token();
    let client = UserAuthDefaultClient::new(token);
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
     // channel for inter-thread communication.
     let (tx, rx) = std::sync::mpsc::channel();
     let path_to_download = path_to_download.to_string();
     let _sender_thread = thread::spawn(move || {           
        let base_local_path_ref = &base_local_path;
        let client_ref=&client; 
        let thread_num = 0;
        let tx_clone2 = mpsc::Sender::clone(&tx);                     
        download_with_client(&path_to_download, client_ref, base_local_path_ref, thread_num,tx_clone2); 
        drop(tx);
    });  
    // the receiver reads all msgs from the queue, until all senders exist - drop(tx)
    // only this thread writes to the terminal, to avoid race in cursor position
    for msg in &rx {
        let (string_to_print, thread_num) = msg; 
        if thread_num != -1{
            println!("\r{}{}","\x1b[1F", &string_to_print);  
        } else{
            println!("{}", &string_to_print);
        }
    }
}

/// download one file with client
pub fn download_with_client(download_path: &str, client: &UserAuthDefaultClient, base_local_path: &str, thread_num:i32,tx_clone:mpsc::Sender<(String, i32)>) {
    //log::trace!("download_with_client: {}",download_path);
    let mut bytes_out = 0u64;
    //let download_arg = files::DownloadArg::new(crate::escape_non_ascii(download_path));
    let download_arg = files::DownloadArg::new(download_path.to_string());
    log::trace!("download_arg: {}", &download_arg.path);    
    let local_path = format!("{}{}", base_local_path, download_path);
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
                            break 'download;
                        }
                        Ok(len) => {
                            bytes_out += len as u64;
                            if let Some(total) = download_result.content_length {    
                                let string_to_print = format!("{}{:.01}% of {:.02} Mb downloading {}", clear_line(), bytes_out as f64 / total as f64 * 100.,total as f64 / 1000000.,download_path);
                                unwrap!( tx_clone.send((string_to_print, thread_num)));                                                                
                            } else {                                
                                let string_to_print = format!("{}{} Mb downloaded {}",clear_line(), bytes_out as f64 / 1000000.,download_path);
                                unwrap!( tx_clone.send((string_to_print, thread_num)));    
                            }
                        }
                        Err(e) => {
                            let string_to_print = format!("Read error: {}", e);
                            unwrap!( tx_clone.send((string_to_print, -1)));  
                            continue 'download; // do another request and resume
                        }
                    }
                }
            }
            Ok(Err(download_error)) => {
                let string_to_print = format!("Download error: {}", download_error);
                unwrap!( tx_clone.send((string_to_print, -1)));  
            }
            Err(request_error) => {
                let string_to_print = format!("Error: {}", request_error);
                unwrap!( tx_clone.send((string_to_print, -1)));  
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
    let line_to_append = format!("{}\t{}\t{}", download_path, s_modified, bytes_out);
    let string_to_print = format!("{}",&line_to_append);
    unwrap!( tx_clone.send((string_to_print, -1)));      
    let mut just_downloaded = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(list_just_downloaded)
        .unwrap();
    unwrap!( writeln!(just_downloaded,"{}",line_to_append));
}

pub fn download_from_list() {
    term_cursor::clear().unwrap();
    println!("download_from_list {}",hide_cursor() );
    print!("{}",term_cursor::Goto(0,7));
    println!("{}",clear_line());
    
    let base_local_path = std::fs::read_to_string("temp_data/base_local_path.csv").unwrap();
    let list_for_download = std::fs::read_to_string("temp_data/list_for_download.csv").unwrap();
    let token = get_short_lived_access_token();    
    let client = UserAuthDefaultClient::new(token);    
    // channel for inter-thread communication.
    let (tx, rx) = std::sync::mpsc::channel();

    // loop in a new thread, so the send msg will come immediately
    let _sender_thread = thread::spawn(move || {
        let base_local_path_ref = &base_local_path;
        let client_ref=&client;       
        // 3 threads to download in parallel
        let pool = rayon::ThreadPoolBuilder::new().num_threads(3).build().unwrap();
        pool.scope(|scoped|{
            for line_path_to_download in list_for_download.lines(){   
                let line: Vec<&str> = line_path_to_download.split("\t").collect();
                let path_to_download = line[0]; 
                let tx_clone2 = mpsc::Sender::clone(&tx); 
                // execute in a separate threads, or waits for a free thread from the pool                
                scoped.spawn(move |_s| {  
                    let thread_num =unwrap!(rayon::current_thread_index()) as i32;                    
                    download_with_client(path_to_download, client_ref, base_local_path_ref, thread_num,tx_clone2); 
                });         
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
        if thread_num != -1{
            let (x,y) = unwrap!(term_cursor::get_pos());
            print!("{}",term_cursor::Goto(0,3+thread_num));    
            println!("{}", &string_to_print);  
            unwrap!( term_cursor::set_pos(x, y));
            if thread_num == 0 {
                string_to_print_1 = string_to_print;
            } else if thread_num == 1 {
                string_to_print_2 = string_to_print;
            } else if thread_num == 2 {
                string_to_print_3 = string_to_print;
            }            
        }else{    
            let (x,y) = unwrap!(term_cursor::get_pos());
            // there is annoying jumping because of scrolling
            // let clear first and write second
            println!("{}{}", term_cursor::Goto(0,1),clear_line());
            println!("{}", clear_line());
            println!("{}", clear_line());  
            println!("{}", clear_line());  
            println!("{}", clear_line());  
            println!("{}", clear_line());
            println!("{}", clear_line());
            unwrap!( term_cursor::set_pos(x, y));

            println!("{}", &string_to_print);
            // print the first 6 lines, because of scrolling            
            let (x,y) = unwrap!(term_cursor::get_pos());
            println!("{}{}download_from_list", term_cursor::Goto(0,1),clear_line());
            println!("{}", clear_line());
            println!("{}", &string_to_print_1);  
            println!("{}", &string_to_print_2);  
            println!("{}", &string_to_print_3);  
            println!("{}", clear_line());
            unwrap!( term_cursor::set_pos(x, y));
        }              
    }
    print!("{}",unhide_cursor());
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
        &files::ListFolderArg::new(crate::escape_non_ascii(&requested_path)).with_recursive(recursive),
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
                &files::ListFolderContinueArg::new(crate::escape_non_ascii(&cursor)),
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

pub fn remote_content_hash(remote_path: &str)->Option<String> {
    let token = get_short_lived_access_token();
    let client = UserAuthDefaultClient::new(token);
    let arg = files::GetMetadataArg::new(crate::escape_non_ascii(remote_path));
    let res_res_metadata = dropbox_sdk::files::get_metadata(&client, &arg);
    
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
            println!("Error get metadata: {}",e);  
            return None;          
        }
        Err(e) => {
            println!("API request error: {}", e);     
            return None;       
        }
    }
}