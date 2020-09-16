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

}

pub fn compare_for_dn() {
    // if the files are already sorted, then re-sorting is much faster.
    let file_web = "my-file3.csv";
    let file_local = "my-local3.csv";
    // the files are NOT sorted
    // some folders have different case. Use case insensitive sort - lexical sort.
    eprintln!("before lexical sort{}", "");
    //use lexical_sort::{lexical_cmp, StringSort};
    let content_web = unwrap!(fs::read_to_string(file_web));
    let sorted_web: Vec<&str> = content_web.lines().collect();
    //eprintln!("read and collect{}", "");
    //sorted_web.string_sort_unstable(lexical_cmp);
    //eprintln!("sorted web len(): {}", sorted_web.len());
    //let joined = sorted_web.join("\n");
    //unwrap!(fs::write("my-file3.csv", joined));

    let content_local = unwrap!(fs::read_to_string(file_local));
    let sorted_local: Vec<&str> = content_local.lines().collect();
    //eprintln!("read and collect{}", "");
    //sorted_local.string_sort_unstable(lexical_cmp);
    //eprintln!("sorted local len(): {}", sorted_local.len());
    //let joined = sorted_local.join("\n");
    //unwrap!(fs::write("my-local3.csv", joined));

    let mut for_dn: Vec<String> = vec![];
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
        if cursor_web >= sorted_web.len() && cursor_local >= sorted_local.len() {
            break;
        } else if cursor_web >= sorted_web.len() {
            line_local = sorted_local[cursor_local].split("\t").collect();
            for_delete.push(line_local[0].to_string());
            cursor_local += 1;
        } else if cursor_local >= sorted_local.len() {
            line_web = sorted_web[cursor_web].split("\t").collect();
            for_dn.push(line_web[0].to_string());
            cursor_web += 1;
        } else {
            line_web = sorted_web[cursor_web].split("\t").collect();
            line_local = sorted_local[cursor_local].split("\t").collect();
            if line_web[0].to_lowercase().lt(&line_local[0].to_lowercase()) {
                println!("Ordering Less: {}   {}", line_web[0], line_local[0]);
                for_dn.push(line_web[0].to_string());
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
                    for_dn.push(line_web[0].to_string());
                }
                cursor_local += 1;
                cursor_web += 1;
            }
        }
    }
    let joined = for_dn.join("\n");
    unwrap!(fs::write("compare_for_dn.csv", joined));
    let joined = for_delete.join("\n");
    unwrap!(fs::write("compare_for_delete.csv", joined));
}
