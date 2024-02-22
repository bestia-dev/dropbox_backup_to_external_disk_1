//! dropbox_backup_to_external_disk lib.rs

#![doc=include_str!("../README.md")]

mod local_mod;
mod remote_mod;
mod utils_mod;

use std::fs;

pub use local_mod::*;
pub use remote_mod::*;
pub use utils_mod::*;

#[allow(unused_imports)]
use uncased::UncasedStr;
use unwrap::unwrap;

pub struct AppConfig {
    pub path_list_base_local_path: &'static str,
    pub path_list_source_files: &'static str,
    pub path_list_destination_files: &'static str,
    pub path_list_source_folders: &'static str,
    pub path_list_destination_folders: &'static str,
    pub path_list_destination_readonly_files: &'static str,
    pub path_list_for_download: &'static str,
    pub path_list_for_trash: &'static str,
    pub path_list_for_correct_time: &'static str,
    pub path_list_just_downloaded_or_moved: &'static str,
    pub path_list_for_trash_folders: &'static str,
    pub path_list_for_create_folders: &'static str,
}

/// list and sync is the complete process for backup in one command
pub fn list_and_sync(base_path: &str, app_config: &'static AppConfig) {
    all_list_remote_and_local(base_path, app_config);
    press_enter_to_continue_timeout_5_sec();
    sync_only(app_config);
}

/// all list remote and local
pub fn all_list_remote_and_local(base_path: &str, app_config: &'static AppConfig) {
    let _hide_cursor_terminal = crate::start_hide_cursor_terminal();
    println!("{}{}dropbox_backup_to_external_disk list_and_sync{}", at_line(1), *YELLOW, *RESET);
    ns_start("");
    // start 2 threads, first for remote list and second for local list
    use std::thread;
    let base_path = base_path.to_string();
    let handle_2 = thread::spawn(move || {
        println!("{}{}3 threads for source (remote files):{}", at_line(3), *GREEN, *RESET);
        // prints at rows 4,5,6 and 7,8,9
        list_remote(app_config);
    });
    let handle_1 = thread::spawn(move || {
        println!("{}{}1 thread for destination (local files):{}", at_line(12), *GREEN, *RESET);
        // prints at rows 13,14,15,16
        list_local(&base_path, app_config);
    });
    // wait for both threads to finish
    handle_1.join().unwrap();
    handle_2.join().unwrap();

    println!("{}{}", at_line(20), *CLEAR_LINE);
}

/// sync_only can be stopped with ctrl+c and then restarted if downloading takes lots of time.  
/// No need to repeat the "list" that takes lots of times.  
pub fn sync_only(app_config: &'static AppConfig) {
    println!("{}compare remote and local lists{}", *YELLOW, *RESET);
    compare_files(app_config);
    println!("{}rename or move equal files{}", *YELLOW, *RESET);
    move_or_rename_local_files(app_config);
    println!("{}move to trash from list{}", *YELLOW, *RESET);
    trash_from_list(app_config);
    println!("{}correct time from list{}", *YELLOW, *RESET);
    correct_time_from_list(app_config);
    press_enter_to_continue_timeout_5_sec();
    download_from_list(app_config);
}

/// compare list: the lists and produce list_for_download, list_for_trash, list_for_correct_time
pub fn compare_files(app_config: &'static AppConfig) {
    add_just_downloaded_to_list_local(app_config);
    compare_lists_internal(
        app_config.path_list_source_files,
        app_config.path_list_destination_files,
        app_config.path_list_for_download,
        app_config.path_list_for_trash,
        app_config.path_list_for_correct_time,
    );
}

/// compare list: the lists must be already sorted for this to work correctly
fn compare_lists_internal(path_list_source_files: &str, path_list_destination_files: &str, path_list_for_download: &str, path_list_for_trash: &str, path_list_for_correct_time: &str) {
    let string_list_source_files = unwrap!(fs::read_to_string(path_list_source_files));
    let vec_list_source_files: Vec<&str> = string_list_source_files.lines().collect();
    println!("{}: {}", path_list_source_files.split("/").collect::<Vec<&str>>()[1], vec_list_source_files.len());
    let string_list_destination_files = unwrap!(fs::read_to_string(path_list_destination_files));
    let vec_list_destination_files: Vec<&str> = string_list_destination_files.lines().collect();
    println!("{}: {}", path_list_destination_files.split("/").collect::<Vec<&str>>()[1], vec_list_destination_files.len());

    let mut vec_for_download: Vec<String> = vec![];
    let mut vec_for_trash: Vec<String> = vec![];
    let mut vec_for_correct_time: Vec<String> = vec![];
    let mut cursor_source = 0;
    let mut cursor_destination = 0;
    //avoid making new allocations or shadowing inside a loop
    let mut vec_line_destination: Vec<&str> = vec![];
    let mut vec_line_source: Vec<&str> = vec![];
    //let mut i = 0;
    loop {
        vec_line_destination.truncate(3);
        vec_line_source.truncate(3);

        if cursor_source >= vec_list_source_files.len() && cursor_destination >= vec_list_destination_files.len() {
            break;
        } else if cursor_source >= vec_list_source_files.len() {
            // final lines
            vec_for_trash.push(vec_list_destination_files[cursor_destination].to_string());
            cursor_destination += 1;
        } else if cursor_destination >= vec_list_destination_files.len() {
            // final lines
            vec_for_download.push(vec_list_source_files[cursor_source].to_string());
            cursor_source += 1;
        } else {
            //compare the 2 lines
            vec_line_source = vec_list_source_files[cursor_source].split("\t").collect();
            vec_line_destination = vec_list_destination_files[cursor_destination].split("\t").collect();
            // UncasedStr preserves the case in the string, but comparison is done case insensitive
            let path_source: &UncasedStr = vec_line_source[0].into();
            let path_destination: &UncasedStr = vec_line_destination[0].into();

            //println!("{}",path_source);
            //println!("{}",path_destination);
            if path_source.lt(path_destination) {
                //println!("lt");
                vec_for_download.push(vec_list_source_files[cursor_source].to_string());
                cursor_source += 1;
            } else if path_source.gt(path_destination) {
                //println!("gt" );
                vec_for_trash.push(vec_list_destination_files[cursor_destination].to_string());
                cursor_destination += 1;
            } else {
                //println!("eq");
                // equal names. check date and size
                // println!("Equal names: {}   {}",path_remote,path_destination);
                // if equal size and time difference only in seconds, then correct destination time
                if vec_line_source[2] == vec_line_destination[2] && vec_line_source[1] != vec_line_destination[1] && vec_line_source[1][0..17] == vec_line_destination[1][0..17] {
                    vec_for_correct_time.push(format!("{}\t{}", path_destination, vec_line_source[1]));
                } else if vec_line_source[1] != vec_line_destination[1] || vec_line_source[2] != vec_line_destination[2] {
                    //println!("Equal names: {}   {}", path_remote, path_destination);
                    //println!(
                    //"Different date or size {} {} {} {}",
                    //line_remote[1], line_destination[1], line_remote[2], line_local[2]
                    //);
                    vec_for_download.push(vec_list_source_files[cursor_source].to_string());
                }
                // else the metadata is the same, no action
                cursor_destination += 1;
                cursor_source += 1;
            }
        }
    }
    println!("{}: {}", path_list_for_download.split("/").collect::<Vec<&str>>()[1], vec_for_download.len());
    let string_for_download = vec_for_download.join("\n");
    unwrap!(fs::write(path_list_for_download, string_for_download));

    println!("{}: {}", path_list_for_trash.split("/").collect::<Vec<&str>>()[1], vec_for_trash.len());
    let string_for_trash = vec_for_trash.join("\n");
    unwrap!(fs::write(path_list_for_trash, string_for_trash));

    println!("{}: {}", path_list_for_correct_time.split("/").collect::<Vec<&str>>()[1], vec_for_correct_time.len());
    let string_for_correct_time = vec_for_correct_time.join("\n");
    unwrap!(fs::write(path_list_for_correct_time, string_for_correct_time));
}

/// compare folders and write folders to trash into path_list_for_trash_folders
/// the list is already sorted
pub fn compare_folders(string_list_source_folders: &str, string_list_destination_folders: &str, file_list_for_trash_folders: &mut FileTxt, file_list_for_create_folders: &mut FileTxt) {
    let vec_list_source_folders: Vec<&str> = string_list_source_folders.lines().collect();
    let vec_list_destination_folders: Vec<&str> = string_list_destination_folders.lines().collect();

    let mut vec_for_trash: Vec<String> = vec![];
    file_list_for_trash_folders.empty().unwrap();
    let mut vec_for_create: Vec<String> = vec![];
    file_list_for_create_folders.empty().unwrap();
    let mut cursor_source = 0;
    let mut cursor_destination = 0;

    loop {
        if cursor_source >= vec_list_source_folders.len() && cursor_destination >= vec_list_destination_folders.len() {
            // all lines are processed
            //dbg!("break");
            break;
        } else if cursor_destination >= vec_list_destination_folders.len() {
            // final lines
            //dbg!("final create_empty_folders ", vec_list_source_folders[cursor_source]);
            vec_for_create.push(vec_list_source_folders[cursor_source].to_string());
            cursor_source += 1;
        } else if cursor_source >= vec_list_source_folders.len() {
            // final lines
            //dbg!("final trash ", vec_list_destination_folders[cursor_destination]);
            vec_for_trash.push(vec_list_destination_folders[cursor_destination].to_string());
            cursor_destination += 1;
        } else {
            // compare the 2 lines
            // UncasedStr preserves the case in the string, but comparison is done case insensitive
            let path_source: &UncasedStr = vec_list_source_folders[cursor_source].into();
            let path_destination: &UncasedStr = vec_list_destination_folders[cursor_destination].into();
            if path_source.lt(path_destination) {
                //dbg!("create_empty_folders {}", vec_list_source_folders[cursor_source]);
                vec_for_create.push(vec_list_source_folders[cursor_source].to_string());
                cursor_source += 1;
            } else if path_source.gt(path_destination) {
                //dbg!("trash {}", vec_list_destination_folders[cursor_destination]);
                vec_for_trash.push(vec_list_destination_folders[cursor_destination].to_string());
                cursor_destination += 1;
            } else {
                // else no action, just increment cursors
                // dbg!("same lines, just increment");
                cursor_destination += 1;
                cursor_source += 1;
            }
        }
    }
    let string_for_trash = vec_for_trash.join("\n");
    file_list_for_trash_folders.write_str(&string_for_trash).unwrap();
    let string_for_create = vec_for_create.join("\n");
    file_list_for_create_folders.write_str(&string_for_create).unwrap();
}
