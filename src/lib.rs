//! dropbox_files lib.rs

mod local_mod;
mod remote_mod;
mod utils_mod;

pub use local_mod::*;
pub use remote_mod::*;
pub use utils_mod::*;

use std::fs;
use unwrap::unwrap;

pub fn sort_lists(){

    let list_remote_files = "data/list_remote_files.csv";
    let list_local_files = "data/list_local_files.csv";
    // the files are NOT sorted
    // some folders have different case. Use case insensitive sort - lexical sort.
    eprintln!("before lexical sort{}", "");
    use lexical_sort::{lexical_cmp, StringSort};

    let content_remote = unwrap!(fs::read_to_string(list_remote_files));
    let mut sorted_remote: Vec<&str> = content_remote.lines().collect();
    eprintln!("read and collect remote{}", "");
    sorted_remote.string_sort_unstable(lexical_cmp);
    eprintln!("sorted remote len(): {}", sorted_remote.len());
    let joined = sorted_remote.join("\n");
    unwrap!(fs::write(list_remote_files, joined));

    let content_local = unwrap!(fs::read_to_string(list_local_files));
    let mut sorted_local: Vec<&str> = content_local.lines().collect();
    eprintln!("read and collect local {}", "");
    sorted_local.string_sort_unstable(lexical_cmp);
    eprintln!("sorted local len(): {}", sorted_local.len());
    let joined = sorted_local.join("\n");
    unwrap!(fs::write(list_local_files, joined));
}

// the list must be already sorted for this to work correctly
pub fn compare_sorted_lists() {
    
    let list_remote_files = "data/list_remote_files.csv";
    let list_local_files = "data/list_local_files.csv";
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
                println!("Ordering Less: {}   {}", line_web[0], line_local[0]);
                for_download.push(line_web[0].to_string());
                cursor_web += 1;
            } else if line_web[0].to_lowercase().gt(&line_local[0].to_lowercase()) {
                println!("Ordering Greater: {}   {}", line_web[0], line_local[0]);
                for_delete.push(line_local[0].to_string());
                cursor_local += 1;
            } else {
                // equal names. check date and size
                //println!("Equal names: {}   {}",line_web[0],line_local[0]);
                if line_web[1] != line_local[1] || line_web[2] != line_local[2] {
                    println!("Equal names: {}   {}", line_web[0], line_local[0]);
                    println!(
                        "Different date or size {} {} {} {}",
                        line_web[1], line_local[1], line_web[2], line_local[2]
                    );
                    for_download.push(line_web[0].to_string());
                }
                cursor_local += 1;
                cursor_web += 1;
            }
        }
    }
    let joined = for_download.join("\n");
    unwrap!(fs::write("data/list_for_download.csv", joined));
    let joined = for_delete.join("\n");
    unwrap!(fs::write("data/list_for_delete.csv", joined));
}
