//! dropbox_backup_to_external_disk lib.rs

// region: lmake_md_to_doc_comments include README.md A //!
//! # dropbox_backup_to_external_disk
//!
//! **one way sync from dropbox to an external disc**  
//! ***[repo](https://github.com/lucianobestia/dropbox_backup_to_external_disk/); version: 0.1.298  date: 2021-07-31 authors: Luciano Bestia***  
//!
//! [![Lines in Rust code](https://img.shields.io/badge/Lines_in_Rust-1242-green.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in Doc comments](https://img.shields.io/badge/Lines_in_Doc_comments-140-blue.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in Comments](https://img.shields.io/badge/Lines_in_comments-102-purple.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in examples](https://img.shields.io/badge/Lines_in_examples-0-yellow.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in tests](https://img.shields.io/badge/Lines_in_tests-0-orange.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//!
//! [![Licence](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/blob/master/LICENSE) [![Rust](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/workflows/RustAction/badge.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//!
//! On my Dropbox "remote drive" I have more than 1 Terabyte of data in 200 000 files.  
//! I own now 4 notebooks and 2 android phones and 1 tablet and not a single one has an internal drive with more than 1 Terabyte. I use dropbox `Selective Sync` to sync only the bare minimum I temporarily need on the local device. But I want to have a backup of all of my data. I must have a backup. Better, I want to have 2 backups of all the data on 2 external hard disks in different locations. So if Dropbox go bankrupt, I still have all my data.  
//! The original Dropbox Sync app works great for the internal HD, but is "not recommended" for external drives. I also need only one way sync: from remote to local. There exist apps for that:
//!
//! - rclone
//! - dropbox_uploader
//!
//! But I wanted to write something mine for fun, learning Rust and using my own apps.
//! I have a lot of files, so I wanted to list them first, then compare with the local files and finally download them.  
//! Obsolete files will "move to trash folder", so I can inspect what and how to remove manually.  
//! The dropbox remote storage will always be read_only, nothing will be modified there, never, no permission for that.  
//!
//! ## Try it
//!
//! You should be logged in Linux terminal with your account. So things you do, are not visible to others.  
//! You will set some local environment variables that are private/secret to your linux Session.  
//! After you logout from you Linux session the local environment variables will be deleted.  
//! You have to be in the project folder where cargo.toml is.  
//! Build the CLI:  
//! `$ cargo make debug`  
//! Follow carefully the instructions.  
//! Before the first use, create your Dropbox app.  
//! Before every use generate your "short-lived access token" and in Linux bash write the "token" into the environment variable like this:  
//! `$ export DBX_OAUTH_TOKEN=here paste the token`  
//! Make a temporary alias for easy of use (it lasts only for this session lifetime) :  
//! `$ alias dropbox_backup_to_external_disk=target/debug/dropbox_backup_to_external_disk`  
//! Test the connection and permission:  
//! `$ dropbox_backup_to_external_disk test`  
//!   
//! Later, use `$ dropbox_backup_to_external_disk --help` to get all the instructions and commands.  
//!
//! ![screenshot_1](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/screenshot_1.png "screenshot_1") ![screenshot_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/screenshot_2.png "screenshot_2") ![list_](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/list_2.png "list_")  
//!
//! ## Warning
//!
//! I don't know why, but WSL2 sometimes does not see all the folders of the external disk.  
//! Instead of 12.000 folders it sees only 28 ???  
//! Be careful !  
//! I then restart my Win10 and the problem magically disappears.
//!
//! ## Development
//!
//! I use WSL2 on Win10 to develope and execute this CLI in Debian Linux.  
//! The external disk path from WSL2 looks like this: `/mnt/d/DropBoxBackup1`. CLI lists the local files metadata in `temp_data/list_local_files.csv`.  
//! List all the files metadata from the remote Dropbox to the file `temp_data/list_remote_files.csv`.
//! Tab delimited with metadata: path (with name), datetime modified, size.
//! The remote path is not really case-sensitive. They try to make it case-preserve, but this apply only to the last part of the path. Before that it is random-case.
//! For big dropbox remotes it can take a while to complete. After the first level folders are listed, I use 3 threads in a ThreadPool to get sub-folders recursively in parallel. It makes it much faster. Also the download of files is in parallel on multiple threads.  
//! The sorting of lists is also done in parallel with the crate Rayon.  
//! Once the lists are complete the CLI will compare them and create files:  
//! `list_for_correct_time.csv`  
//! `list_for_download.csv`  
//! `list_for_trash.csv`  
//! With this files the CLI will:  
//! `move_or_rename_local_files` using the content_hash to be sure they are equal  
//! `trash_from_list` will move the obsolete files into a trash folder  
//! `correct_time_from_list` sometimes it is needed  
//! `download_from_list` - this can take a lot of time and it can be stopped with ctrl+c
//!
//! ## DropBox api2 - Stone sdk
//!
//! Dropbox has made a `Stone` thing that contains all the API definition. From there is possible to generate code boilerplate for different languages for the api-client.  
//! For Rust there is this quasi official project:  
//! <https://crates.io/crates/dropbox-sdk>  
//!
//! ## Authorization OAuth2
//!
//! Authorization on the internet is a mess. Dropbox api uses OAuth2.
//! Every app must be authorized on Dropbox and have its own `app key` and `app secret`.  
//! For commercial programs they probably embed them into the binary code somehow. But for OpenSource projects it is not possible to keep a secret. So the workaround is: every user must create a new `dropbox app` exclusive only to him. Creating a new app is simple. This app will stay forever in `development status` in dropbox, to be more private and secure. The  
//! `$ dropbox_backup_to_external_disk --help`  
//! has the detailed instructions.  
//! Then every time before use we need generate the "short-lived access token" for security reasons.  
//! ![dropbox_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/dropbox_2.png "dropbox_2") ![dropbox_1](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/dropbox_1.png "dropbox_1")
//!
//! ## rename or move
//!
//! Often a file is renamed or moved to another folder. I can try to recognize if there is the same file in list_for_trash and list_for download, but I cannot use the file path or name. Instead I use the metadata: size, date modified and content_hash.  
//!
//! ## REGEX adventure with non-breaking space and CRLF
//!
//! We all know space. But there are other non-visible characters that are very similar and sometimes impossible to distinguish. Tab is one of them, but it is not so difficult to spot with a quick try.  
//! But nbsp non-breaking space, often used in HTML is a catastrophe. There is no way to tell it apart from the normal space. I used a regex to find a match with some spaces. It worked right for a years. Yesterday it didn't work. If I changed space to `\s` in the regex expression, it worked, but not with space. I tried everything and didn't find the cause. Finally I deleted and inserted the space. It works. But how? After a detailed analysis I discovered it was a non-breakable space. This is unicode 160 or \xa0, instead of normal space unicode 32 \x20. Now I will try to find them all and replace with normal space. What a crazy world.  
//! And another REGEX surprise. I try to have all text files delimited with the unix standard LF. But somehow the windows standard got mixed and I didn't recognize it. The regex for `end of line` $ didn't work for CRLF. When I changed it to LF, the good life is back and all works.
//!
//! ## Text files
//!
//! Simple text files are a terrible way to store data that needs to be changed. It is ok for write once and then read. But there is not a good way to modify only one line inside a big text file. The recommended approach is read all, modify, save all. If the memory is not big enough then use a buffer to read a segment, modify, save a segment, repeat to end.  
//! There is another approach called memory map to file, but everybody is trying to avoid it because some other process could modify the file when in use and make it garbage.  
//! Sounds like a database is always a better choice for more agile development.  
//! In this project I will create additional files that only append lines. Some kind of journal. And later use this to modify the big text files in one go. For example: list_just_downloaded_or_moved.csv is added to list_local_files.csv.  
//!
// endregion: lmake_md_to_doc_comments include README.md A //!

mod local_mod;
mod remote_mod;
mod utils_mod;

use std::{fs, thread::sleep};

pub use local_mod::*;
pub use remote_mod::*;
pub use utils_mod::*;

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use uncased::UncasedStr;
use unwrap::unwrap;

/// list and sync is the complete process for backup in one command
pub fn list_and_sync(base_path: &str) {
    print!("{}", term_cursor::Clear);
    println!(
        "{}{}{}{}",
        term_cursor::Goto(0, 1),
        clear_line(),
        "dropbox_backup_to_external_disk list_and_sync",
        hide_cursor()
    );
    ns_start("");
    // start 2 threads, first for remote list and second for local list
    use std::thread;
    let base_path = base_path.to_string();
    let handle_2 = thread::spawn(move || {
        println!(
            "{}{}{}",
            term_cursor::Goto(0, 3),
            clear_line(),
            Green.paint("Threads for remote:")
        );
        // prints at rows 10,11,12
        list_remote();
    });
    let handle_1 = thread::spawn(move || {
        println!(
            "{}{}{}",
            term_cursor::Goto(0, 15),
            clear_line(),
            Green.paint("Thread for local:")
        );
        // prints at rows 5, 6, 7
        list_local(&base_path);
    });
    // wait for both threads to finish
    handle_1.join().unwrap();
    handle_2.join().unwrap();
    println!(
        "{}{}{}{}",
        term_cursor::Goto(0, 20),
        clear_line(),
        Green.paint(""),
        unhide_cursor()
    );

    sync_only();
}

/// sync_only can be stopped and then restarted if downloading takes a lot of time.
/// no need to repeat the "list" that takes a lot of timeS
pub fn sync_only() {
    println!("{}", Yellow.paint("compare remote and local lists"));
    compare_lists();
    println!("{}", Yellow.paint("rename or move equal files"));
    move_or_rename_local_files();
    println!("{}", Yellow.paint("move to trash from list"));
    trash_from_list();
    println!("{}", Yellow.paint("correct time from list"));
    correct_time_from_list();
    println!("{}", Yellow.paint("download from list"));
    // wait 2 second, just to see the result on the screen
    sleep(std::time::Duration::new(2, 0));
    download_from_list();
}

/// compare list: the lists must be already sorted for this to work correctly
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
                if vec_line_remote[2] == vec_line_local[2]
                    && vec_line_remote[1] != vec_line_local[1]
                    && vec_line_remote[1][0..17] == vec_line_local[1][0..17]
                {
                    vec_for_correct_time.push(format!("{}\t{}", path_local, vec_line_remote[1]));
                } else if vec_line_remote[1] != vec_line_local[1]
                    || vec_line_remote[2] != vec_line_local[2]
                {
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
    unwrap!(fs::write(
        "temp_data/list_for_download.csv",
        string_for_download
    ));

    println!("list_for_trash: {}", vec_for_trash.len());
    let string_for_trash = vec_for_trash.join("\n");
    unwrap!(fs::write("temp_data/list_for_trash.csv", string_for_trash));

    println!("list_for_correct_time: {}", vec_for_correct_time.len());
    let string_for_correct_time = vec_for_correct_time.join("\n");
    unwrap!(fs::write(
        "temp_data/list_for_correct_time.csv",
        string_for_correct_time
    ));
}
