//! local_mod.rs

use lexical_sort::{lexical_cmp, StringSort};
use std::fs;
use unwrap::unwrap;
 
// $ dbx_download list_local /mnt/d/DropBoxBackup2
// $ clear; cargo run --bin dbx_download -- list_local /mnt/d/DropBoxBackup2

pub fn list_local(base_path: &str) {
    // remember the base local path for later commands
    if !std::path::Path::new(base_path).exists() {
        eprintln!("error: base_path not exists {}", base_path);
        std::process::exit(1);
    }
    std::fs::write("data/base_local_path.csv", base_path).unwrap();

    // write data to a big string in memory
    let mut output_string = String::with_capacity(1024 * 1024);

    use walkdir::WalkDir;
    
    //make space for fixed ansi terminal lines
    // clear screen
    print!("\x1B[2J");
    //print!("dbx_download list_local\n\n");
    let mut folder_count=0;
    for entry in WalkDir::new(base_path) {
        //let mut ns_started = ns_start("WalkDir entry start");
        let entry:walkdir::DirEntry = entry.unwrap();
        let path = entry.path();
        let str_path = unwrap!(path.to_str());
        // path.is_dir() is slow. entry.file-type().is_dir() is fast
        if entry.file_type().is_dir() {
            // How to print on the same Line? Use the \r, but it does not work every time.
            // https://www.lihaoyi.com/post/BuildyourownCommandLinewithANSIescapecodes.html
            /*
            Move up/down is not working because some lines are too long and they change the cursor position
            It is better to use fixed row position numbers.
            Up: \x1b[{n}A moves cursor up by n
Down: \x1b[{n}B moves cursor down by n
Right: \x1b[{n}C moves cursor right by n
Left: \x1b[{n}D moves cursor left by n
Next Line: \x1b[{n}E moves cursor to beginning of line n lines down
Prev Line: \x1b[{n}F moves cursor to beginning of line n lines down
Set Column: \x1b[{n}G moves cursor to column n
Set Position: \x1b[{n};{m}H moves cursor to row n column m
Clear Screen: \x1b[{n}J clears the screen
n=0 clears from cursor until end of screen,
n=1 clears from cursor to beginning of screen
n=2 clears entire screen
Clear Line: \x1b[{n}K clears the current line
n=0 clears from cursor to end of line
n=1 clears from cursor to start of line
n=2 clears entire line

            */

            // Set Position 1,1
            print!("\x1b[1;1H");
            // clear entire line
            print!("\x1b[2K");
            println!("Folder: {}", str_path.trim_start_matches(base_path));
            
            
            // Set Position 1,2
            print!("\x1b[2;1H");
            // clear entire line
            print!("\x1b[2K");
            println!("Folder_count: {}", folder_count);
            
            
            folder_count += 1;
        } else {
            // write csv tab delimited
            // metadata() in wsl/Linux is slow. Nothing to do here.
            //ns_started = ns_print("metadata start", ns_started);
            if let Ok(metadata) = entry.metadata() {
                //ns_started = ns_print("metadata end", ns_started);
                use chrono::offset::Utc;
                use chrono::DateTime;
                let datetime: DateTime<Utc> = unwrap!(metadata.modified()).into();
                
                output_string.push_str(&format!(
                    "{}\t{}\t{}\n",
                    str_path.trim_start_matches(base_path),
                    datetime.format("%Y-%m-%dT%TZ"),
                    metadata.len()
                ));
                
            }
        }
        //ns_print("WalkDir entry end", ns_started);
    }

    // sort
    eprintln!("\nstart sorting {}", "");
    let mut sorted_local: Vec<&str> = output_string.lines().collect();
    sorted_local.string_sort_unstable(lexical_cmp);
    let joined = sorted_local.join("\n");
    eprintln!("sorted local len(): {}", sorted_local.len());
    // join to string and write to file
    unwrap!(fs::write("data/list_local_files.csv", joined));
}
