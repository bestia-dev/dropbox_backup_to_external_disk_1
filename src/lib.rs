//! dbx_download lib.rs

mod local_mod;
mod remote_mod;
mod terminal_ansi_mod;
mod utils_mod;

pub use local_mod::*;
pub use remote_mod::*;
pub use terminal_ansi_mod::*;
pub use utils_mod::*;

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use std::fs;
use unwrap::unwrap;

pub fn one_way_sync(base_path: &str) {
    ansi_clear_screen();
    println!("{}{}", ansi_set_row(1), "dbx_download one_way_sync");
    ns_start("");
    // start 2 threads, first for remote list and second for local list
    use std::thread;
    let base_path = base_path.to_string();
    let handle_1 = thread::spawn(move || {
        print!("{}{}", ansi_set_row(4), Green.paint("first thread:"));
        // prints at rows 5, 6, 7
        list_local(&base_path);
    });
    let handle_2 = thread::spawn(move || {
        print!("{}{}", ansi_set_row(9), Green.paint("second thread:"));
        // prints at rows 10,11,12
        list_remote();
    });
    // wait for both threads to finish
    handle_1.join().unwrap();
    handle_2.join().unwrap();
    println!("{}{}", ansi_set_row(13), Yellow.paint("compare lists"));
    compare_sorted_lists();
    println!("{}", Yellow.paint("download from list"));
    download_from_list();
}
// the list must be already sorted for this to work correctly
pub fn compare_sorted_lists() {
    let list_remote_files = "temp_data/list_remote_files.csv";
    let list_local_files = "temp_data/list_local_files.csv";
    let content_remote = unwrap!(fs::read_to_string(list_remote_files));
    let sorted_remote: Vec<&str> = content_remote.lines().collect();
    let content_local = unwrap!(fs::read_to_string(list_local_files));
    let sorted_local: Vec<&str> = content_local.lines().collect();

    let mut for_download: Vec<String> = vec![];
    let mut for_delete: Vec<String> = vec![];
    let mut cursor_web = 0;
    let mut cursor_local = 0;
    //avoid making new allocations or shadowing inside a loop
    let mut line_local: Vec<&str> = vec![];
    let mut line_web: Vec<&str> = vec![];
    //let mut i = 0;
    loop {
        line_local.truncate(3);
        line_web.truncate(3);
        //if i > 3 {break;}
        //i += 1;
        if cursor_web >= sorted_remote.len() && cursor_local >= sorted_local.len() {
            break;
        } else if cursor_web >= sorted_remote.len() {
            line_local = sorted_local[cursor_local].split("\t").collect();
            for_delete.push(line_local[0].to_string());
            cursor_local += 1;
        } else if cursor_local >= sorted_local.len() {
            line_web = sorted_remote[cursor_web].split("\t").collect();
            for_download.push(line_web[0].to_string());
            cursor_web += 1;
        } else {
            line_web = sorted_remote[cursor_web].split("\t").collect();
            line_local = sorted_local[cursor_local].split("\t").collect();
            if line_web[0].to_lowercase().lt(&line_local[0].to_lowercase()) {
                //println!("Ordering Less: {}   {}", line_web[0], line_local[0]);
                for_download.push(line_web[0].to_string());
                cursor_web += 1;
            } else if line_web[0].to_lowercase().gt(&line_local[0].to_lowercase()) {
                //println!("Ordering Greater: {}   {}", line_web[0], line_local[0]);
                for_delete.push(line_local[0].to_string());
                cursor_local += 1;
            } else {
                // equal names. check date and size
                //println!("Equal names: {}   {}",line_web[0],line_local[0]);
                if line_web[1] != line_local[1] || line_web[2] != line_local[2] {
                    //println!("Equal names: {}   {}", line_web[0], line_local[0]);
                    //println!(
                    //"Different date or size {} {} {} {}",
                    //line_web[1], line_local[1], line_web[2], line_local[2]
                    //);
                    for_download.push(line_web[0].to_string());
                }
                cursor_local += 1;
                cursor_web += 1;
            }
        }
    }
    let joined = for_download.join("\n");
    unwrap!(fs::write("temp_data/list_for_download.csv", joined));
    let joined = for_delete.join("\n");
    unwrap!(fs::write("temp_data/list_for_delete.csv", joined));
}
