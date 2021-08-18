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

    let cargo_pkg_name = env!("CARGO_PKG_NAME");

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
                list_and_sync(path);
                ns_print_ms("list_and_sync", ns_started);
            }
            _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
        },
        Some("sync_only") => {
            let ns_started = ns_start("sync_only");
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
            _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
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
            _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
        },
        Some("second_backup") => match env::args().nth(2).as_deref() {
            Some(path) => second_backup(path),
            _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
        },
        _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
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
            "test",
            "list_and_sync",
            "sync_only",
            "remote_list",
            "local_list",
            "compare_lists",
            "move_or_rename_local_files",
            "trash_from_list",
            "correct_time_from_list",
            "download_from_list",
            "one_file_download",
            "second_backup",
        ];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    }
    /*
    // the second level if needed
    else if last_word == "new" {
        let sub_commands = vec!["with_lib"];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    }
    */
}

fn print_help() {
    println!("");
    println!(
        "{}1. Before first use, create your private Dropbox app:{}",
        *YELLOW, *RESET
    );
    println!( "- open browser on {}<https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>{}", *GREEN, *RESET );
    println!("- click Create app, choose Scoped access, choose Full dropbox");
    println!(
        "- choose a globally unique app name like {}`backup_{}`{}",
        *GREEN,
        chrono::offset::Utc::now()
            .format("%Y%m%dT%H%M%SZ")
            .to_string(),
        *RESET
    );
    println!("- go to tab Permissions, check `files.metadata.read` and `files.content.read`, click Submit, close browser");
    println!("");

    println!(
        "{}2. Before every use, create a short-lived access token (secret):{}",
        *YELLOW, *RESET
    );
    println!( "- open browser on {}<https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>{}", *GREEN, *RESET );
    println!(
        ". choose your existing private Dropbox app like {}`backup_{}`{}",
        *GREEN,
        chrono::offset::Utc::now()
            .format("%Y%m%dT%H%M%SZ")
            .to_string(),
        *RESET
    );
    println!(
        "- click button `Generate` to generated short-lived access token and copy it, close browser"
    );
    println!(
        "- In you Linux terminal session set a short-lived private/secret environment variable:"
    );
    println!(
        "$ {}export DBX_OAUTH_TOKEN={}here paste the access token",
        *GREEN, *RESET
    );

    println!("- test if the authentication works: ");
    println!("$ {}dropbox_backup_to_external_disk test{}", *GREEN, *RESET);
    println!("");

    println!("{}Commands:{}", *YELLOW, *RESET);

    println!("Full list and sync - from dropbox to external disk
This command has 2 phases. 
1. First it lists all remote and local files. That can take a lot of time if you have lot of files.
For faster work it uses concurrent threads. 
If you interrupt the execution with ctrl+c in this phase, before the lists are completed, the lists are empty.
You will need to rerun the command and wait for the lists to be fully completed.
2. The second phase is the same as the command `sync_only`. 
It can be interrupted with crl+c. The next `sync_only` will continue where it was interrupted. ");
    println!(
        "   $ {}dropbox_backup_to_external_disk list_and_sync /mnt/d/DropBoxBackup1{}\n",
        *GREEN, *RESET
    );

    println!("Sync only - one-way sync from dropbox to external disk
It starts the sync only. Does NOT list again the remote and local files, the lists must already be completed 
from the first command `list_and_sync`.
It can be interrupted with crl+c. The next `sync_only` will continue where it was interrupted.");
    println!(
        "  $ {}dropbox_backup_to_external_disk sync_only{}\n",
        *GREEN, *RESET
    );

    println!("Second backup");
    println!("One-way sync from backup_1 external disk to backup_2 external disk.");
    println!(
        "  $ {}dropbox_backup_to_external_disk second_backup /mnt/f/DropBoxBackup2{}",
        *GREEN, *RESET
    );
    println!("");

    println!(
        "{}Just for debugging purpose, you can run every step separately.{}",
        *YELLOW, *RESET
    );
    println!("Test connection and authorization:");
    println!(
        "  $ {}dropbox_backup_to_external_disk test{}",
        *GREEN, *RESET
    );
    println!("List all files in your remote Dropbox to `list_remote_files.csv`:");
    println!(
        "  $ {}dropbox_backup_to_external_disk remote_list{}",
        *GREEN, *RESET
    );
    println!("List local files to `list_local_files.csv`:");
    println!(
        "  $ {}dropbox_backup_to_external_disk local_list /mnt/d/DropBoxBackup1{}",
        *GREEN, *RESET
    );
    println!("Compare lists and create `list_for_download.csv`, `list_for_trash.csv` and `list_for_correct_time.csv`:");
    println!(
        "  $ {}dropbox_backup_to_external_disk compare_lists{}",
        *GREEN, *RESET
    );
    println!("Move or rename local files (check with content_hash that they are equal):");
    println!(
        "  $ {}dropbox_backup_to_external_disk move_or_rename_local_files{}",
        *GREEN, *RESET
    );
    println!("Move to trash folder from `list_for_trash.csv`:");
    println!(
        "  $ {}dropbox_backup_to_external_disk trash_from_list{}",
        *GREEN, *RESET
    );
    println!("Correct time of files from `list_for_correct_time.csv`:");
    println!(
        "  $ {}dropbox_backup_to_external_disk correct_time_from_list{}",
        *GREEN, *RESET
    );
    println!("Download files from `list_for_download.csv`:");
    println!(
        "  $ {}dropbox_backup_to_external_disk download_from_list{}",
        *GREEN, *RESET
    );
    println!("One single file download:");
    println!(
        "  $ {}dropbox_backup_to_external_disk one_file_download <path>{}",
        *GREEN, *RESET
    );
    println!("");
}
