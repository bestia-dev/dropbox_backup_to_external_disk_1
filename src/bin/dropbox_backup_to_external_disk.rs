//! dropbox_backup_to_external_disk.rs

use dropbox_backup_to_external_disk::*;
use std::env;

fn main() {
    pretty_env_logger::init();
    ctrlc::set_handler(move || {
        println!("terminated with ctrl+c. {}", *UNHIDE_CURSOR);
        std::process::exit(exitcode::OK);
    })
    .expect("Error setting Ctrl-C handler");

    //create the directory temp_data/
    std::fs::create_dir_all("temp_data").unwrap();

    match env::args().nth(1).as_deref() {
        None | Some("--help") | Some("-h") => print_help(),
        Some("completion") => completion(),
        Some("test") => {
            let ns_started = ns_start("test");
            test_connection();
            ns_print_ms("test", ns_started);
        }
        Some("list_and_sync") => match env::args().nth(2).as_deref() {
            Some(path) => {
                let ns_started = ns_start("list_and_sync");
                print!("{}", *CLEAR_ALL);
                list_and_sync(path);
                ns_print_ms("list_and_sync", ns_started);
            }
            _ => println!("Unrecognized arguments. Try dropbox_backup_to_external_disk --help"),
        },
        Some("sync_only") => {
            let ns_started = ns_start("sync_only");
            print!("{}", *CLEAR_ALL);
            sync_only();
            ns_print_ms("sync_only", ns_started);
        }
        Some("remote_list") => {
            print!("{}", *CLEAR_ALL);
            println!(
                "{}{}{}remote_list into temp_data/list_remote_files.csv{}",
                at_line(1),
                *CLEAR_LINE,
                *YELLOW,
                *RESET
            );
            let ns_started = ns_start("");
            test_connection();
            list_remote();
            ns_print_ms("remote_list", ns_started);
        }
        Some("local_list") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", *CLEAR_ALL);
                println!(
                    "{}{}{}local_list into temp_data/list_local_files.csv{}",
                    at_line(1),
                    *CLEAR_LINE,
                    *YELLOW,
                    *RESET
                );
                let ns_started = ns_start("");
                list_local(path);
                ns_print_ms("local_list", ns_started);
            }
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },
        Some("all_list") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", *CLEAR_ALL);
                println!(
                    "{}{}{}remote and local lists into temp_data{}",
                    at_line(1),
                    *CLEAR_LINE,
                    *YELLOW,
                    *RESET
                );
                let ns_started = ns_start("");
                test_connection();
                all_list_remote_and_local(path);
                ns_print_ms("all_list", ns_started);
            }
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },

        Some("compare_lists") => {
            let ns_started = ns_start("compare lists");
            println!("{}compare remote and local lists{}", *YELLOW, *RESET);
            compare_lists();
            ns_print_ms("compare_lists", ns_started);
        }
        Some("move_or_rename_local_files") => {
            let ns_started = ns_start("move_or_rename_local_files");
            move_or_rename_local_files();
            ns_print_ms("move_or_rename_local_files", ns_started);
        }
        Some("trash_from_list") => {
            let ns_started = ns_start("trash from temp_data/list_for_trash.csv");
            trash_from_list();
            ns_print_ms("trash_from_list", ns_started);
        }
        Some("correct_time_from_list") => {
            let ns_started =
                ns_start("correct time of files from temp_data/list_for_correct_time.csv");
            correct_time_from_list();
            ns_print_ms("correct_time_from_list", ns_started);
        }
        Some("download_from_list") => {
            let ns_started = ns_start("download from temp_data/list_for_download.csv");
            download_from_list();
            ns_print_ms("download_from_list", ns_started);
        }
        Some("one_file_download") => match env::args().nth(2).as_deref() {
            Some(path) => download_one_file(path),
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },
        Some("second_backup") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", *CLEAR_ALL);
                second_backup(path)
            }
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },
        _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
    }
}

/// sub-command for bash auto-completion of `cargo auto` using the crate `dev_bestia_cargo_completion`
/// `complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk`
fn completion() {
    /// println one, more or all sub_commands
    fn completion_return_one_or_more_sub_commands(
        sub_commands: Vec<&str>,
        word_being_completed: &str,
    ) {
        let mut sub_found = false;
        for sub_command in sub_commands.iter() {
            if sub_command.starts_with(word_being_completed) {
                println!("{}", sub_command);
                sub_found = true;
            }
        }
        if sub_found == false {
            // print all sub-commands
            for sub_command in sub_commands.iter() {
                println!("{}", sub_command);
            }
        }
    }

    let args: Vec<String> = std::env::args().collect();
    // `complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk`
    // this completion always sends this arguments:
    // 0. executable path
    // 1. word completion
    // 2. executable file name
    // 3. word_being_completed (even if it is empty)
    // 4. last_word
    let word_being_completed = args[3].as_str();
    let last_word = args[4].as_str();

    if last_word == "dropbox_backup_to_external_disk" {
        let sub_commands = vec![
            "--help",
            "-h",
            "all_list",
            "compare_lists",
            "correct_time_from_list",
            "download_from_list",
            "list_and_sync",
            "local_list",
            "move_or_rename_local_files",
            "one_file_download",
            "remote_list",
            "second_backup",
            "sync_only",
            "test",
            "trash_from_list",
        ];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    }
    // the second level if needed
    else if last_word == "list_and_sync" || last_word == "local_list" || last_word == "all_list" {
        let sub_commands = vec!["/mnt/d/DropboxBackup1"];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    } else if last_word == "second_backup" {
        let sub_commands = vec!["/mnt/f/DropboxBackup2"];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    }
}

/// print help
fn print_help() {
    println!(
        r#"
{y}1. Before first use, create your private Dropbox app:{rs}
- open browser on {g}<https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>{rs}
- click Create app, choose Scoped access, choose Full dropbox
- choose a globally unique app name like {g}`backup_{date}`{rs}
- go to tab Permissions, check `files.metadata.read` and `files.content.read`, click Submit, close browser

{y}2. Before every use, create a short-lived access token (secret):{rs}
- open browser on {g}<https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>{rs}
- choose your existing private Dropbox app like {g}`backup_{date}`{rs}
- click button `Generate` to generated short-lived access token and copy it, close browser
- In you Linux terminal session set a short-lived private/secret environment variable:
  $ {g}export DBX_OAUTH_TOKEN={rs}here paste the access token
- test if the authentication works:
  $ {g}dropbox_backup_to_external_disk test{rs}

{y}Commands:{rs}
Full list and sync - from dropbox to external disk
This command has 2 phases. 
1. First it lists all remote and local files. That can take a lot of time if you have lot of files.
For faster work it uses concurrent threads. 
If you interrupt the execution with ctrl+c in this phase, before the lists are completed, the lists are empty.
You will need to rerun the command and wait for the lists to be fully completed.
2. The second phase is the same as the command `sync_only`. 
It can be interrupted with crl+c. The next `sync_only` will continue where it was interrupted.
   $ {g}dropbox_backup_to_external_disk list_and_sync /mnt/d/DropBoxBackup1{rs}

Sync only - one-way sync from dropbox to external disk
It starts the sync only. Does NOT list again the remote and local files, the lists must already be completed 
from the first command `list_and_sync`.
It can be interrupted with crl+c. The next `sync_only` will continue where it was interrupted
  $ {g}dropbox_backup_to_external_disk sync_only{rs}

Second backup
One-way sync from backup_1 external disk to backup_2 external disk.
  $ {g}dropbox_backup_to_external_disk second_backup /mnt/f/DropBoxBackup2{rs}

{y}Just for debugging purpose, you can run every step separately.{rs}
Test connection and authorization:
  $ {g}dropbox_backup_to_external_disk test{rs}
List remote files from Dropbox to `list_remote_files.csv`:
  $ {g}dropbox_backup_to_external_disk remote_list{rs}
List local files to `list_local_files.csv`:
  $ {g}dropbox_backup_to_external_disk local_list /mnt/d/DropBoxBackup1{rs}
List all - both remote and local files to `temp_date/`:
  $ {g}dropbox_backup_to_external_disk all_list /mnt/d/DropBoxBackup1{rs}  
Compare lists and generate `list_for_download.csv`, `list_for_trash.csv` and `list_for_correct_time.csv`:
  $ {g}dropbox_backup_to_external_disk compare_lists{rs}
Move or rename local files if they are equal in trash_from_list and download_from_list:
  $ {g}dropbox_backup_to_external_disk move_or_rename_local_files{rs}
Move to trash folder from `list_for_trash.csv`:
  $ {g}dropbox_backup_to_external_disk trash_from_list{rs}
Correct time of files from `list_for_correct_time.csv`:
  $ {g}dropbox_backup_to_external_disk correct_time_from_list{rs}
Download files from `list_for_download.csv`:
  $ {g}dropbox_backup_to_external_disk download_from_list{rs}
One single file download:
  $ {g}dropbox_backup_to_external_disk one_file_download <path>{rs}

Run in `~/dropbox_backup_to_external_disk` directory to gain auto-completion:
  $ alias dropbox_backup_to_external_disk=./dropbox_backup_to_external_disk
  $ complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk
open-source: https://github.com/bestia-dev/dropbox_backup_to_external_disk
    "#,
        g = *GREEN,
        y = *YELLOW,
        rs = *RESET,
        date = chrono::offset::Utc::now().format("%Y%m%dT%H%M%SZ"),
    );
}
