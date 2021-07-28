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
use uncased::UncasedStr;

pub fn list_and_sync(base_path: &str) {
    print!("{}", term_cursor::Clear);
    println!("{}{}{}", term_cursor::Goto(0,1),clear_line(), "dropbox_backup_to_external_disk list_and_sync");
    ns_start("");
    // start 2 threads, first for remote list and second for local list
    use std::thread;
    let base_path = base_path.to_string();
    let handle_2 = thread::spawn(move || {
        println!("{}{}{}", term_cursor::Goto(0,3),clear_line(), Green.paint("Threads for remote:"));
        // prints at rows 10,11,12
        list_remote();
    });
    let handle_1 = thread::spawn(move || {
        println!("{}{}{}", term_cursor::Goto(0,15),clear_line(), Green.paint("Thread for local:"));
        // prints at rows 5, 6, 7
        list_local(&base_path);
    });
    // wait for both threads to finish
    handle_1.join().unwrap();
    handle_2.join().unwrap();
    println!("{}{}{}", term_cursor::Goto(0,20),clear_line(), Green.paint(""));
    sync_only();
  
}

pub fn sync_only(){
    println!("{}", Yellow.paint("compare remote and local lists"));
    compare_lists();
    println!("{}", Yellow.paint("rename or move equal files"));
    move_or_rename_local_files();
    println!("{}", Yellow.paint("move to trash from list"));
    trash_from_list();
    println!("{}", Yellow.paint("correct time from list"));
    correct_time_from_list();
    println!("{}", Yellow.paint("download from list"));
    download_from_list();  
}

// the list must be already sorted for this to work correctly
pub fn compare_lists() {
    add_just_downloaded_to_list_local();
    let path_list_remote_files = "temp_data/list_remote_files.csv";
    let path_list_local_files = "temp_data/list_local_files.csv";
    let string_remote = unwrap!(fs::read_to_string(path_list_remote_files));
    let vec_remote_lines: Vec<&str> = string_remote.lines().collect();
    let string_local = unwrap!(fs::read_to_string(path_list_local_files));
    let vec_local_lines: Vec<&str> = string_local.lines().collect();

    let mut vec_for_download: Vec<String> = vec![];
    let mut vec_for_trash: Vec<String> = vec![];
    let mut vec_for_correct_time: Vec<String> = vec![];
    let mut cursor_remote = 0;
    let mut cursor_local = 0;
    //avoid making new allocations or shadowing inside a loop
    let mut vec_line_local: Vec<&str> = vec![];
    let mut vec_line_remote: Vec<&str> = vec![];
    //let mut i = 0;
    loop {
        vec_line_local.truncate(3);
        vec_line_remote.truncate(3);

        if cursor_remote >= vec_remote_lines.len() && cursor_local >= vec_local_lines.len() {
            break;
        } else if cursor_remote >= vec_remote_lines.len() {
            // final lines
            vec_for_trash.push(vec_local_lines[cursor_local].to_string());
            cursor_local += 1;
        } else if cursor_local >= vec_local_lines.len() {
            // final lines
            vec_for_download.push(vec_remote_lines[cursor_remote].to_string());
            cursor_remote += 1;
        } else {
            vec_line_remote = vec_remote_lines[cursor_remote].split("\t").collect();
            vec_line_local = vec_local_lines[cursor_local].split("\t").collect();
            // UncasedStr preserves the case in the string, but comparison is done case insensitive
            let path_remote: &UncasedStr = vec_line_remote[0].into();
            let path_local: &UncasedStr = vec_line_local[0].into();

            //println!("{}",path_remote);
            //println!("{}",path_local);
            if path_remote.lt(path_local) {
                //println!("lt");
                vec_for_download.push(vec_remote_lines[cursor_remote].to_string());
                cursor_remote += 1;
            } else if path_remote.gt(path_local) { 
                //println!("gt" );
                vec_for_trash.push(vec_local_lines[cursor_local].to_string());
                cursor_local += 1;
            } else {
                //println!("eq");
                // equal names. check date and size
                // println!("Equal names: {}   {}",path_remote,path_local);
                // if equal size and time difference only in seconds, then correct local time
                if vec_line_remote[2] == vec_line_local[2] && vec_line_remote[1] != vec_line_local[1] && vec_line_remote[1][0..17] == vec_line_local[1][0..17]{
                    vec_for_correct_time.push(format!("{}\t{}",path_local,vec_line_remote[1] ));
                } else if vec_line_remote[1] != vec_line_local[1] || vec_line_remote[2] != vec_line_local[2] {
                    //println!("Equal names: {}   {}", path_remote, path_local);
                    //println!(
                    //"Different date or size {} {} {} {}",
                    //line_remote[1], line_local[1], line_remote[2], line_local[2]
                    //);
                    vec_for_download.push(vec_remote_lines[cursor_remote].to_string());
                }
                // else the metadata is the same, no action
                cursor_local += 1;
                cursor_remote += 1;
            }
        }
    }
    println!("list_for_download: {}", vec_for_download.len());
    let string_for_download = vec_for_download.join("\n");    
    unwrap!(fs::write("temp_data/list_for_download.csv", string_for_download));

    println!("list_for_trash: {}", vec_for_trash.len());
    let string_for_trash = vec_for_trash.join("\n");
    unwrap!(fs::write("temp_data/list_for_trash.csv", string_for_trash));

    println!("list_for_correct_time: {}", vec_for_correct_time.len());
    let string_for_correct_time = vec_for_correct_time.join("\n");
    unwrap!(fs::write("temp_data/list_for_correct_time.csv", string_for_correct_time));
}

// clears the line on the terminal \x1b[0K  clears from cursor to end of line
pub fn clear_line()->&'static str{
    "\x1b[0K"
}
pub fn hide_cursor()->&'static str{
    "\x1b[?25l"
}
pub fn unhide_cursor()->&'static str{
    "\x1b[?25h"
}
//
pub fn increment_and_loop(thread_num: i32,start_num:i32,max_num:i32)->i32{
    if thread_num < max_num{
        thread_num + 1
    }else{
        start_num
    }
}

pub fn sort_string_lines(output_string:&str)->String{
    let mut sorted_local: Vec<&str> = output_string.lines().collect();
    use rayon::prelude::*;
    sorted_local.par_sort_unstable_by(|a,b|{
        let aa: &UncasedStr = (*a).into();
        let bb: &UncasedStr = (*b).into();
        aa.cmp(bb)
    } );

    let joined = sorted_local.join("\n");  
    // return
    joined  
}

pub fn escape_non_ascii(input:&str)->String{
    let mut new_string = String::new();
    for ch in input.chars(){
        if ch.is_ascii(){
            new_string.push(ch);
        }else{
            // for dropbox <https://www.dropbox.com/developers/reference/json-encoding>
            // it must look like this: \\u010d            
            new_string.push_str( &format!("\\u{:04x}", ch as u32)	);
        }
    }
    // return
    new_string
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escape_non_ascii_01() {
        assert_eq!(escape_non_ascii("123 č 456"), "123 \\u010d 456");
    }
}