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
        None | Some("--help") | Some("-h") => print_help(&cargo_pkg_name),
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
        Some("list_remote") => {
            print!("{}", *CLEAR_ALL);
            println!(
                "{}{}{}list_remote into temp_data/list_remote_files.csv{}",
                at_line(1),
                *CLEAR_LINE,
                *YELLOW,
                *RESET
            );
            let ns_started = ns_start("");
            list_remote();
            ns_print_ms("list_remote", ns_started);
        }
        Some("list_local") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", *CLEAR_ALL);
                println!(
                    "{}{}{}list_local into temp_data/list_local_files.csv{}",
                    at_line(1),
                    *CLEAR_LINE,
                    *YELLOW,
                    *RESET
                );
                let ns_started = ns_start("");
                list_local(path);
                ns_print_ms("list_local", ns_started);
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
        Some("download") => match env::args().nth(2).as_deref() {
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
            "list_remote",
            "list_local",
            "compare_lists",
            "move_or_rename_local_files",
            "trash_from_list",
            "correct_time_from_list",
            "download_from_list",
            "download",
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

fn print_help(cargo_pkg_name: &str) {
    println!("");
    println!(
        "{}1. Before first use, create your private Dropbox app:{}",
        *YELLOW, *RESET
    );
    println!( "- open browser on {}<https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>{}", *GREEN, *RESET );
    println!("- click Create app, choose Scoped access, choose Full dropbox");
    println!(
        "- choose a unique app name like {}`backup_{}`{}",
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
        "- click button Generate to generated short-lived access token and copy it, close browser"
    );
    println!(
        "- In you Linux terminal session set a short-lived private/secret environment variable:"
    );
    println!(
        "$ {}export DBX_OAUTH_TOKEN={}here paste the access token",
        *GREEN, *RESET
    );
    println!("- create an alias for easy of use:");
    println!(
        "$ {}alias dropbox_backup_to_external_disk=target/debug/dropbox_backup_to_external_disk{}",
        *GREEN, *RESET
    );
    println!("- test if the authentication works: ");
    println!("$ {}dropbox_backup_to_external_disk test{}", *GREEN, *RESET);
    println!("");

    println!("{}Commands:{}", *YELLOW, *RESET);
    println!("Full list and sync - from dropbox to external disk.");
    println!(
        "It lists remote and local files (that takes a lot of time) and then starts the sync."
    );
    println!(
        "$ {}{} list_and_sync /mnt/d/DropBoxBackup2{}",
        *GREEN, cargo_pkg_name, *RESET
    );
    println!("Sync only - from dropbox to external disk.");
    println!("It starts the sync only. Does NOT list again the remote and local files, because it takes a lot of time.");
    println!("It continues where the previous sync has finished.");
    println!("$ {}{} sync_only{}", *GREEN, cargo_pkg_name, *RESET);
    println!("");

    println!(
        "{}For debugging purpose, you can run every step separately.{}",
        *YELLOW, *RESET
    );
    println!("Test connection and authorization:");
    println!("$ {}{} test{}", *GREEN, cargo_pkg_name, *RESET);
    println!("List all files in your remote Dropbox to list_remote_files.csv:");
    println!("$ {}{} list_remote{}", *GREEN, cargo_pkg_name, *RESET);
    println!("List local files to list_local_files.csv:");
    println!(
        "$ {}{} list_local /mnt/d/DropBoxBackup2{}",
        *GREEN, cargo_pkg_name, *RESET
    );
    println!("Compare lists and create list_for_download.csv, list_for_trash.csv and list_for_correct_time.csv:");
    println!("$ {}{} compare_lists{}", *GREEN, cargo_pkg_name, *RESET);
    println!("move_or_rename_local_files:");
    println!(
        "$ {}{} move_or_rename_local_files{}",
        *GREEN, cargo_pkg_name, *RESET
    );
    println!("Move to trash folder from list_for_trash.csv:");
    println!("$ {}{} trash_from_list{}", *GREEN, cargo_pkg_name, *RESET);
    println!("Correct time of files from list_for_correct_time.csv:");
    println!(
        "$ {}{} correct_time_from_list{}",
        *GREEN, cargo_pkg_name, *RESET
    );
    println!("Download files from list_for_download.csv:");
    println!(
        "$ {}{} download_from_list{}",
        *GREEN, cargo_pkg_name, *RESET
    );
    println!("Download one single file:");
    println!("$ {}{} download <path>{}", *GREEN, cargo_pkg_name, *RESET);

    println!("Second backup:");
    println!(
        "$ {}{} second_backup <path>{}",
        *GREEN, cargo_pkg_name, *RESET
    );
}
