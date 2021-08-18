//! dropbox_backup_to_external_disk lib.rs

// region: auto_md_to_doc_comments include README.md A //!
//! # dropbox_backup_to_external_disk
//!
//! **One way sync from dropbox to external disc**  
//! ***[repository](https://github.com/lucianobestia/dropbox_backup_to_external_disk/); version: 2021.818.1453  date: 2021-08-18 authors: Luciano Bestia***  
//!
//! [![Lines in Rust code](https://img.shields.io/badge/Lines_in_Rust-1730-green.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in Doc comments](https://img.shields.io/badge/Lines_in_Doc_comments-180-blue.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in Comments](https://img.shields.io/badge/Lines_in_comments-136-purple.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in examples](https://img.shields.io/badge/Lines_in_examples-0-yellow.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//! [![Lines in tests](https://img.shields.io/badge/Lines_in_tests-0-orange.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//!
//! [![Licence](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/blob/main/LICENSE) [![Rust](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/workflows/RustAction/badge.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
//!
//! ## Motivation
//!
//! On my Dropbox "remote drive" I have more than 1 Terabyte of data in 200 000 files.  
//! I own now 4 notebooks and 2 android phones and 1 tablet and not a single one has an internal drive with more than 1 Terabyte. I use dropbox `Selective Sync` to sync only the bare minimum I temporarily need on the local device. But I want to have a backup of all of my data. I must have a backup. Better, I want to have 2 backups of all the data on 2 external hard disks in different locations. So if Dropbox go bankrupt, I still have all my data.  
//! The original Dropbox Sync app works great for the internal HD, but is "not recommended" for external drives. I also need only one way sync: from remote to local. There exist apps for that:
//!
//! - rclone
//! - dropbox_uploader
//!
//! But I wanted to write something mine for fun, learning Rust and using my own apps.
//! I have a lot of files, so I wanted to list them first - remote and local. Then compare the lists and finally download the files.  
//! Obsolete files will "move to trash folder", so I can inspect what and how to delete manually.  
//! The dropbox remote storage will always be read_only, nothing will be modified there, never, no permission for that.  
//!
//! ## Try it
//!
//! There are a few manual steps for the security of you files on Dropbox. Authentication on internet is a complex topic.  
//! You should be logged in Linux terminal (also in WSL2) with your account. So things you do, are not visible to others. You will set some local environment variables that are private/secret to your linux Session.  After you logout from you Linux session these local environment variables will be deleted.  
//! The executable will create a sub-directory `temp_data` in the current directory. Maybe it is best if you create a dedicated directory `~/dropbox_backup_to_external_disk/` just for this executable.
//! Download the latest release from [Github](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/releases) and make the file executable and enable auto-completion:
//!
//! ```bash
//! cd ~
//! mkdir dropbox_backup_to_external_disk
//! cd dropbox_backup_to_external_disk
//! chmod +x dropbox_backup_to_external_disk
//! complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk
//! ```
//!
//! Run the executable with --help and follow carefully the instructions 1. and 2.  
//!
//! ```bash
//! dropbox_backup_to_external_disk --help
//! ```
//!
//! ![screenshot_1](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/screenshot_1.png "screenshot_1") ![screenshot_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/screenshot_2.png "screenshot_2") ![list_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/list_2.png "list_2") ![list_3](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/list_3.png "list_3") ![list_4](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/list_4.png "list_4")
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
//! I use [cargo-auto](https://crates.io/crates/cargo-auto) for automation tasks in rust language. Install it:
//!
//! ```bash
//! cargo install cargo-auto
//! ```
//!
//! List user-defined automation tasks in `automation_tasks_rs`:
//!
//! ```bash
//! cargo auto
//! ```
//!
//! I use WSL2 on Win10 to develope and execute this CLI in Debian Linux.  
//! The external disk path from WSL2 looks like this: `/mnt/d/DropBoxBackup1`. CLI saves the list of the local files metadata in `temp_data/list_local_files.csv`.  
//! Save the list all the files metadata from the remote Dropbox to the file `temp_data/list_remote_files.csv`.
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
//! Dropbox has made a `Stone` thingy that contains all the API definition. From there is possible to generate code boilerplate for different languages for the api-client.  
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
//! ![dropbox_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/dropbox_2.png "dropbox_2") ![dropbox_1](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/main/images/dropbox_1.png "dropbox_1")
//!
//! ## rename or move
//!
//! Often a file is renamed or moved to another folder. I can try to recognize if there is the same file in list_for_trash and list_for download, but I cannot use the file path or name. Instead I use the metadata: size, date modified and content_hash.  
//!
//! ## bash auto-completion
//!
//! This executable is prepared for auto-completion in bash.  
//! Run this command to define auto-completion in bash for the current session:  
//!
//! ```bash
//! complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk
//! ```
//!
//! To make it permanent add this command to the file `~/.bashrc` or some other file that runs commands on bash initialization.  
//!
//! ## Learn something new every day
//!
//! ### REGEX adventure with non-breaking space and CRLF
//!
//! We all know space. But there are other non-visible characters that are very similar and sometimes impossible to distinguish. Tab is one of them, but it is not so difficult to spot with a quick try.  
//! But nbsp non-breaking space, often used in HTML is a catastrophe. There is no way to tell it apart from the normal space. I used a regex to find a match with some spaces. It worked right for a years. Yesterday it didn't work. If I changed space to `\s` in the regex expression, it worked, but not with space. I tried everything and didn't find the cause. Finally I deleted and inserted the space. It works. But how? After a detailed analysis I discovered it was a non-breakable space. This is unicode 160 or \xa0, instead of normal space unicode 32 \x20. Now I will try to find them all and replace with normal space. What a crazy world.  
//! And another REGEX surprise. I try to have all text files delimited with the unix standard LF. But somehow the windows standard got mixed and I didn't recognize it. The regex for `end of line` $ didn't work for CRLF. When I changed it to LF, the good life is back and all works.
//!
//! ### Text files
//!
//! Simple text files are a terrible way to store data that needs to be changed. It is ok for write once and then read. But there is not a good way to modify only one line inside a big text file. The recommended approach is read all, modify, save all. If the memory is not big enough then use a buffer to read a segment, modify, save a segment, repeat to end.  
//! There is another approach called memory map to file, but everybody is trying to avoid it because some other process could modify the file when in use and make it garbage.  
//! Sounds like a database is always a better choice for more agile development.  
//! In this project I will create additional files that only append lines. Some kind of journal. And later use this to modify the big text files in one go. For example: list_just_downloaded_or_moved.csv is added to list_local_files.csv.  
//!
//! ### termion
//!
//! After using some small crates to help me with Linux tty terminal ansi codes, I am happy to finally use only the `termion` crate.  
//! It has all I need.  
//!
//! ## cargo crev reviews and advisory
//!
//! We leave in times of danger with `supply chain attacks`.  
//! It is recommended to always use [cargo-crev](https://github.com/crev-dev/cargo-crev)  
//! to verify the trustworthiness of each of your dependencies.  
//! Please, spread this info.  
//! You can also read reviews quickly on the web. Example for the crate `num-traits`:  
//! <https://web.crev.dev/rust-reviews/crate/num-traits/>  
//!
//! ## open-source free and free as a beer
//!
//! My open-source projects are free and free as a beer (MIT license).  
//! I just love programming.  
//! But I need also to drink. If you find my projects and tutorials helpful, please buy me a beer or two donating on my [paypal](https://www.paypal.com/paypalme/LucianoBestia). You know the price of a beer in your local bar ;-)  
//! So I can drink a free beer for your health :-)  
//! [Na zdravje](https://translate.google.com/?hl=en&sl=sl&tl=en&text=Na%20zdravje&op=translate) !
//!
// endregion: auto_md_to_doc_comments include README.md A //!

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

/// list and sync is the complete process for backup in one command
pub fn list_and_sync(base_path: &str) {
    let _hide_cursor_terminal = crate::start_hide_cursor_terminal();
    print!("{}", *CLEAR_ALL);
    println!(
        "{}{}{}dropbox_backup_to_external_disk list_and_sync{}",
        at_line(1),
        *CLEAR_LINE,
        *YELLOW,
        *RESET
    );
    ns_start("");
    // start 2 threads, first for remote list and second for local list
    use std::thread;
    let base_path = base_path.to_string();
    let handle_2 = thread::spawn(move || {
        println!(
            "{}{}{}Threads for remote:{}",
            at_line(3),
            *CLEAR_LINE,
            *GREEN,
            *RESET
        );
        // prints at rows 4,5,6 and 7,8,9
        list_remote();
    });
    let handle_1 = thread::spawn(move || {
        println!(
            "{}{}{}Thread for local:{}",
            at_line(12),
            *CLEAR_LINE,
            *GREEN,
            *RESET
        );
        // prints at rows 13,14,15,16
        list_local(&base_path);
    });
    // wait for both threads to finish
    handle_1.join().unwrap();
    handle_2.join().unwrap();
    println!("{}{}", at_line(20), *CLEAR_LINE);
    press_enter_to_continue_timeout_5_sec();
    sync_only();
}

/// sync_only can be stopped and then restarted if downloading takes a lot of time.
/// no need to repeat the "list" that takes a lot of timeS
pub fn sync_only() {
    println!("{}compare remote and local lists{}", *YELLOW, *RESET);
    compare_lists();
    println!("{}rename or move equal files{}", *YELLOW, *RESET);
    move_or_rename_local_files();
    println!("{}move to trash from list{}", *YELLOW, *RESET);
    trash_from_list();
    println!("{}correct time from list{}", *YELLOW, *RESET);
    correct_time_from_list();
    press_enter_to_continue_timeout_5_sec();
    download_from_list();
}

pub fn compare_lists() {
    add_just_downloaded_to_list_local();
    let path_list_source_files = "temp_data/list_remote_files.csv";
    let path_list_destination_files = "temp_data/list_local_files.csv";
    let path_list_for_download = "temp_data/list_for_download.csv";
    let path_list_for_trash = "temp_data/list_for_trash.csv";
    let path_list_for_correct_time = "temp_data/list_for_correct_time.csv";
    compare_lists_internal(
        path_list_source_files,
        path_list_destination_files,
        path_list_for_download,
        path_list_for_trash,
        path_list_for_correct_time,
    );
}

pub fn compare2_lists() {
    add2_just_downloaded_to_list_local();
    let path_list_source_files = "temp_data/list_local_files.csv";
    let path_list_destination_files = "temp_data/list2_local_files.csv";
    let path_list_for_download = "temp_data/list2_for_download.csv";
    let path_list_for_trash = "temp_data/list2_for_trash.csv";
    let path_list_for_correct_time = "temp_data/list2_for_correct_time.csv";
    compare_lists_internal(
        path_list_source_files,
        path_list_destination_files,
        path_list_for_download,
        path_list_for_trash,
        path_list_for_correct_time,
    );
}

/// compare list: the lists must be already sorted for this to work correctly
fn compare_lists_internal(
    path_list_source_files: &str,
    path_list_destination_files: &str,
    path_list_for_download: &str,
    path_list_for_trash: &str,
    path_list_for_correct_time: &str,
) {
    let string_remote = unwrap!(fs::read_to_string(path_list_source_files));
    let vec_remote_lines: Vec<&str> = string_remote.lines().collect();
    println!(
        "{}: {}",
        path_list_source_files.split("/").collect::<Vec<&str>>()[1],
        vec_remote_lines.len()
    );
    let string_destination = unwrap!(fs::read_to_string(path_list_destination_files));
    let vec_destination_lines: Vec<&str> = string_destination.lines().collect();
    println!(
        "{}: {}",
        path_list_destination_files
            .split("/")
            .collect::<Vec<&str>>()[1],
        vec_destination_lines.len()
    );

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

        if cursor_source >= vec_remote_lines.len()
            && cursor_destination >= vec_destination_lines.len()
        {
            break;
        } else if cursor_source >= vec_remote_lines.len() {
            // final lines
            vec_for_trash.push(vec_destination_lines[cursor_destination].to_string());
            cursor_destination += 1;
        } else if cursor_destination >= vec_destination_lines.len() {
            // final lines
            vec_for_download.push(vec_remote_lines[cursor_source].to_string());
            cursor_source += 1;
        } else {
            vec_line_source = vec_remote_lines[cursor_source].split("\t").collect();
            vec_line_destination = vec_destination_lines[cursor_destination]
                .split("\t")
                .collect();
            // UncasedStr preserves the case in the string, but comparison is done case insensitive
            let path_source: &UncasedStr = vec_line_source[0].into();
            let path_destination: &UncasedStr = vec_line_destination[0].into();

            //println!("{}",path_source);
            //println!("{}",path_destination);
            if path_source.lt(path_destination) {
                //println!("lt");
                vec_for_download.push(vec_remote_lines[cursor_source].to_string());
                cursor_source += 1;
            } else if path_source.gt(path_destination) {
                //println!("gt" );
                vec_for_trash.push(vec_destination_lines[cursor_destination].to_string());
                cursor_destination += 1;
            } else {
                //println!("eq");
                // equal names. check date and size
                // println!("Equal names: {}   {}",path_remote,path_destination);
                // if equal size and time difference only in seconds, then correct destination time
                if vec_line_source[2] == vec_line_destination[2]
                    && vec_line_source[1] != vec_line_destination[1]
                    && vec_line_source[1][0..17] == vec_line_destination[1][0..17]
                {
                    vec_for_correct_time
                        .push(format!("{}\t{}", path_destination, vec_line_source[1]));
                } else if vec_line_source[1] != vec_line_destination[1]
                    || vec_line_source[2] != vec_line_destination[2]
                {
                    //println!("Equal names: {}   {}", path_remote, path_destination);
                    //println!(
                    //"Different date or size {} {} {} {}",
                    //line_remote[1], line_destination[1], line_remote[2], line_local[2]
                    //);
                    vec_for_download.push(vec_remote_lines[cursor_source].to_string());
                }
                // else the metadata is the same, no action
                cursor_destination += 1;
                cursor_source += 1;
            }
        }
    }
    println!(
        "{}: {}",
        path_list_for_download.split("/").collect::<Vec<&str>>()[1],
        vec_for_download.len()
    );
    let string_for_download = vec_for_download.join("\n");
    unwrap!(fs::write(path_list_for_download, string_for_download));

    println!(
        "{}: {}",
        path_list_for_trash.split("/").collect::<Vec<&str>>()[1],
        vec_for_trash.len()
    );
    let string_for_trash = vec_for_trash.join("\n");
    unwrap!(fs::write(path_list_for_trash, string_for_trash));

    println!(
        "{}: {}",
        path_list_for_correct_time.split("/").collect::<Vec<&str>>()[1],
        vec_for_correct_time.len()
    );
    let string_for_correct_time = vec_for_correct_time.join("\n");
    unwrap!(fs::write(
        path_list_for_correct_time,
        string_for_correct_time
    ));
}

/// after the first backup from dropbox, we want to make a second backup from the first backup
/// ideally, we put it somewhere safe in a distant location
/// having 2 external disks on the same computer, is faster to just copy files then to question for calculating hash
/// no need to move files or correct time. Just copy it. It is faster.
pub fn second_backup(base_path: &str) {
    list2_local(base_path);
    // compare list_local_files and list2_local_files
    compare2_lists();
    trash2_from_list();
    // copy instead of download, no multi-thread
    copy_from_list2_for_download("temp_data/list2_for_download.csv");
    // just copy also the files for correct time. It is faster then hash.
    copy_from_list2_for_download("temp_data/list2_for_correct_time.csv");
    println!("{}compare local and local2 lists{}", *YELLOW, *RESET);
    compare2_lists();
}
