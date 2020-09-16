//! remote_mod.rs

use dropbox_sdk::client_trait::HttpClient;
use dropbox_sdk::{files, HyperClient, Oauth2AuthorizeUrlBuilder, Oauth2Type};

use std::collections::VecDeque;
use std::env;
use std::io::{self, Read, Write};
// use unwrap::unwrap;

fn prompt(msg: &str) -> String {
    eprint!("{}: ", msg);
    io::stderr().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_owned()
}

pub fn test_connection() {
    let token = get_token();
    let client = HyperClient::new(token);
    match files::list_folder(&client, &files::ListFolderArg::new("".to_string())) {
        Ok(Ok(_result)) => eprintln!("test connection and authorization: ok"),
        Ok(Err(e)) => eprintln!("error: {}", e),
        Err(e) => eprintln!("error: {}", e),
    }
}

fn get_token() -> String {
    // Let the user pass the token in an environment variable, or prompt them if that's not found.
    let token = match env::var("DBX_OAUTH_TOKEN") {
        Ok(token) => {
            eprintln!("Token read from env var.");
            token
        }
        Err(_err) => {
            let client_id = prompt("Give me a Dropbox API app key");
            let client_secret = prompt("Give me a Dropbox API app secret");

            let url =
                Oauth2AuthorizeUrlBuilder::new(&client_id, Oauth2Type::AuthorizationCode).build();
            eprintln!("Open this URL in your browser:");
            eprintln!("{}", url);
            eprintln!();
            let auth_code = prompt("Then paste the code here");

            eprintln!("requesting OAuth2 token");
            match HyperClient::oauth2_token_from_authorization_code(
                &client_id,
                &client_secret,
                auth_code.trim(),
                None,
            ) {
                Ok(token) => {
                    eprintln!("got token: {}", token);
                    eprintln!("You can store this token into a env variable for temporary use.");
                    eprintln!("So you don't need to do this dance again.");
                    eprintln!(
                        "You are logged into Linux and this is mostly not shared with others."
                    );
                    eprintln!("$ export DBX_OAUTH_TOKEN={}", token);

                    // This is where you'd save the token somewhere so you don't need to do this dance
                    // again.

                    token
                }
                Err(e) => {
                    eprintln!("Error getting OAuth2 token: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };
    // return
    token
}

pub fn list_remote() {
    let token = get_token();
    let client = HyperClient::new(token);

    use std::fs::OpenOptions;
    use std::io::prelude::*;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("data/list_remote_files.csv")
        .unwrap();
    match list_directory(&client, "/", true) {
        Ok(Ok(iterator)) => {
            for entry_result in iterator {
                match entry_result {
                    Ok(Ok(files::Metadata::Folder(entry))) => {
                        // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                        println!("Folder: {}", entry.path_display.unwrap_or(entry.name));
                    }
                    Ok(Ok(files::Metadata::File(entry))) => {
                        // write csv tab delimited
                        if let Err(e) = writeln!(
                            file,
                            "{}\t{}\t{}",
                            // path_display is not 100% case accurate. Dropbox is case-insensitive and preserves the casing only for the metadata_name, not path.
                            entry.path_display.unwrap_or(entry.name),
                            entry.client_modified,
                            entry.size
                        ) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                        //println!("{}\t{}\t{}", entry.path_display.unwrap_or(entry.name), entry.client_modified, entry.size);
                    }
                    Ok(Ok(files::Metadata::Deleted(entry))) => {
                        panic!("unexpected deleted entry: {:?}", entry);
                    }
                    Ok(Err(e)) => {
                        eprintln!("Error from files/list_folder_continue: {}", e);
                        break;
                    }
                    Err(e) => {
                        eprintln!("API request error: {}", e);
                        break;
                    }
                }
            }
        }
        Ok(Err(e)) => {
            eprintln!("Error from files/list_folder: {}", e);
        }
        Err(e) => {
            eprintln!("API request error: {}", e);
        }
    }
}

pub fn download(download_path: &str) {
    let token = get_token();
    let client = HyperClient::new(token);
    eprintln!("downloading file {}", download_path);
    let mut bytes_out = 0u64;
    let download_arg = files::DownloadArg::new(download_path.to_string());
    use std::fs::OpenOptions;
    let base_local_path=std::fs::read_to_string("data/base_local_path.csv").unwrap();
    let local_path = format!("{}{}",base_local_path,download_path);
    eprintln!("to local path: {}", local_path);
    // create folder if it does not exist
    use std::path::PathBuf;
    let path = PathBuf::from(&local_path);
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(local_path)
        .unwrap();

    'download: loop {
        let result = files::download(&client, &download_arg, Some(bytes_out), None);
        match result {
            Ok(Ok(download_result)) => {
                let mut body = download_result.body.expect("no body received!");
                loop {
                    // limit read to 1 MiB per loop iteration so we can output progress
                    let mut input_chunk = (&mut body).take(1024 * 1024);
                    match io::copy(&mut input_chunk, &mut file) {
                        Ok(0) => {
                            eprint!("\r");
                            break 'download;
                        }
                        Ok(len) => {
                            bytes_out += len as u64;
                            if let Some(total) = download_result.content_length {
                                eprint!("\r{:.01}%", bytes_out as f64 / total as f64 * 100.);
                            } else {
                                eprint!("\r{} bytes", bytes_out);
                            }
                        }
                        Err(e) => {
                            eprintln!("Read error: {}", e);
                            continue 'download; // do another request and resume
                        }
                    }
                }
            }
            Ok(Err(download_error)) => {
                eprintln!("Download error: {}", download_error);
            }
            Err(request_error) => {
                eprintln!("Error: {}", request_error);
            }
        }
        break 'download;
    }
}

pub fn download_from_list(){
    // TODO: open the authorization once
    // and then download multiple files
    let base_local_path=std::fs::read_to_string("data/base_local_path.csv").unwrap();
    let list_for_download = std::fs::read_to_string("data/list_for_download.csv").unwrap();
    for download_path in list_for_download.lines(){
        // TODO: add datetime and size in list
        let local_path = format!("{}{}",base_local_path,download_path);
        //TODO: if datetime and size is not the same then overwrite
        if !std::path::Path::new(&local_path).exists(){
            download(download_path);
        }
    }
}

fn list_directory<'a>(
    client: &'a dyn HttpClient,
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
    client: &'a dyn HttpClient,
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
