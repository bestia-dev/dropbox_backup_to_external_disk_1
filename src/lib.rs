//! dropbox_backup_to_external_disk lib.rs

mod local_mod;
mod remote_mod;
mod utils_mod;

use std::fs;

pub use local_mod::*;
pub use remote_mod::*;
pub use utils_mod::*;

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use unwrap::unwrap;

pub fn list_and_sync(base_path: &str) {
    print!("{}", term_cursor::Clear);
    println!("{}{}{}", term_cursor::Goto(0,1),clear_line(), "dropbox_backup_to_external_disk list_and_sync");
    ns_start("");
    // start 2 threads, first for remote list and second for local list
    use std::thread;
    let base_path = base_path.to_string();
    let handle_1 = thread::spawn(move || {
        println!("{}{}{}", term_cursor::Goto(0,4),clear_line(), Green.paint("first thread:"));
        // prints at rows 5, 6, 7
        list_local(&base_path);
    });
    let handle_2 = thread::spawn(move || {
        println!("{}{}{}", term_cursor::Goto(0,9),clear_line(), Green.paint("second thread:"));
        // prints at rows 10,11,12
        list_remote();
    });
    // wait for both threads to finish
    handle_1.join().unwrap();
    handle_2.join().unwrap();
    sync_only();
  
}

pub fn sync_only(){
    println!("{}", Yellow.paint("add downloaded files to list_local"));
    add_downloaded_to_list_local();
    println!("{}", Yellow.paint("compare remote and local lists"));
    compare_sorted_lists();
    println!("{}", Yellow.paint("move to trash from list"));
    trash_from_list();
    println!("{}", Yellow.paint("correct time from list"));
    correct_time_from_list();
    println!("{}", Yellow.paint("download from list"));
    download_from_list();  
}

// the list must be already sorted for this to work correctly
pub fn compare_sorted_lists() {
    use uncased::UncasedStr;
    let list_remote_files = "temp_data/list_remote_files.csv";
    let list_local_files = "temp_data/list_local_files.csv";
    let content_remote = unwrap!(fs::read_to_string(list_remote_files));
    let sorted_remote: Vec<&str> = content_remote.lines().collect();
    let content_local = unwrap!(fs::read_to_string(list_local_files));
    let sorted_local: Vec<&str> = content_local.lines().collect();

    let mut for_download: Vec<String> = vec![];
    let mut for_trash: Vec<String> = vec![];
    let mut for_correct_time: Vec<String> = vec![];
    let mut cursor_remote = 0;
    let mut cursor_local = 0;
    //avoid making new allocations or shadowing inside a loop
    let mut line_local: Vec<&str> = vec![];
    let mut line_remote: Vec<&str> = vec![];
    //let mut i = 0;
    loop {
        line_local.truncate(3);
        line_remote.truncate(3);

        if cursor_remote >= sorted_remote.len() && cursor_local >= sorted_local.len() {
            break;
        } else if cursor_remote >= sorted_remote.len() {
            // final lines
            line_local = sorted_local[cursor_local].split("\t").collect();
            for_trash.push(line_local[0].to_string());
            cursor_local += 1;
        } else if cursor_local >= sorted_local.len() {
            // final lines
            line_remote = sorted_remote[cursor_remote].split("\t").collect();
            for_download.push(line_remote[0].to_string());
            cursor_remote += 1;
        } else {
            line_remote = sorted_remote[cursor_remote].split("\t").collect();
            line_local = sorted_local[cursor_local].split("\t").collect();
            // UncasedStr preserves the case in the string, but comparison is done case insensitive
            let path_remote: &UncasedStr = line_remote[0].into();
            let path_local: &UncasedStr = line_local[0].into();

            //println!("{}",path_remote);
            //println!("{}",path_local);
            if path_remote.lt(path_local) {
                //println!("lt");
                for_download.push(path_remote.to_string());
                cursor_remote += 1;
            } else if path_remote.gt(path_local) { 
                //println!("gt" );
                for_trash.push(path_local.to_string());
                cursor_local += 1;
            } else {
                //println!("eq");
                // equal names. check date and size
                // println!("Equal names: {}   {}",path_remote,path_local);
                // if equal size and time difference only in seconds, then correct local time
                if line_remote[2] == line_local[2] && line_remote[1] != line_local[1] && line_remote[1][0..17] == line_local[1][0..17]{
                    for_correct_time.push(format!("{}\t{}",path_local,line_remote[1] ));
                } else if line_remote[1] != line_local[1] || line_remote[2] != line_local[2] {
                    //println!("Equal names: {}   {}", path_remote, path_local);
                    //println!(
                    //"Different date or size {} {} {} {}",
                    //line_remote[1], line_local[1], line_remote[2], line_local[2]
                    //);
                    for_download.push(path_remote.to_string());
                }
                // else the metadata is the same, no action
                cursor_local += 1;
                cursor_remote += 1;
            }
        }
    }
    let joined = for_download.join("\n");
    unwrap!(fs::write("temp_data/list_for_download.csv", joined));
    let joined = for_trash.join("\n");
    unwrap!(fs::write("temp_data/list_for_trash.csv", joined));
    let correct_time = for_correct_time.join("\n");
    unwrap!(fs::write("temp_data/list_for_correct_time.csv", correct_time));
}

// clears the line on the terminal \x1b[0K  clears from cursor to end of line
pub fn clear_line()->&'static str{
    "\x1b[0K"
}