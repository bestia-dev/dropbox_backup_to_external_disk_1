//! dropbox_backup_to_external_disk.rs

use dropbox_backup_to_external_disk::*;
use ansi_term::Colour::{Yellow, Green};
use std::env;

fn main() {
    pretty_env_logger::init();
    ctrlc::set_handler(move || {
        println!("terminated with ctrl+c. {}",unhide_cursor());
        std::process::exit(exitcode::OK);
    }).expect("Error setting Ctrl-C handler");

    let cargo_pkg_name = env!("CARGO_PKG_NAME");

    match env::args().nth(1).as_deref() {
        None | Some("--help") | Some("-h") => print_help(&cargo_pkg_name),
        Some("test") =>  {
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
            print!("{}", term_cursor::Clear);
            println!(
                "{}{}{}{}",
                term_cursor::Goto(0,1),
                clear_line(),
                "list_remote into temp_data/list_remote_files.csv",
                hide_cursor()
            );
            let ns_started = ns_start("");
            list_remote();
            println!("{}", unhide_cursor());
            ns_print_ms("list_remote", ns_started);
        }
        Some("list_local") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", term_cursor::Clear);
                println!(
                    "{}{}{}",
                    term_cursor::Goto(0,1),
                    clear_line(),
                    "list_local into temp_data/list_local_files.csv"
                );
                let ns_started = ns_start("");
                list_local(path);
                ns_print_ms("list_local", ns_started);
            }
            _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
        },
        Some("compare_lists") => {
            let ns_started = ns_start("compare lists");
            compare_lists();
            ns_print_ms("compare_lists", ns_started);
        }
        Some("move_or_rename_local_files") => {
            let ns_started = ns_start("move_or_rename_local_files");
            move_or_rename_local_files();
            ns_print_ms("move_or_rename_local_files", ns_started);
        },  
        Some("trash_from_list") => {
            let ns_started = ns_start("trash from temp_data/list_for_trash.csv");
            trash_from_list();
            ns_print_ms("trash_from_list", ns_started);
        }
        Some("correct_time_from_list") => {
            let ns_started = ns_start("correct time of files from temp_data/list_for_correct_time.csv");
            correct_time_from_list();
            ns_print_ms("correct_time_from_list", ns_started);
        }
        Some("download_from_list") => {
            let ns_started = ns_start("download from temp_data/list_for_download.csv");
            download_from_list();
            ns_print_ms("download_from_list", ns_started);
        }
        Some("download") => match env::args().nth(2).as_deref() {
            Some(path) => download(path),
            _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
        },
            
        _ => println!("Unrecognized arguments. Try {} --help", &cargo_pkg_name),
    }
}

fn print_help(cargo_pkg_name: &str) {
    println!("{}", Yellow.paint("1. Before first use, create your private Dropbox app:"));
    println!("- open browser on <{}>",Green.paint("https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps"));
    println!("- click Create app, choose Scoped access, choose Full dropbox");
    println!("- choose a unique app name like `{}{}`",Green.paint("backup_"),Green.paint(chrono::offset::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()),);
    println!("- go to tab Permissions, check `files.metadata.read` and `files.content.read`, click Submit, close browser");
    println!("");

    println!("{}", Yellow.paint("2. Before every use, create a short-lived access token (secret):"));
    println!("- open browser on <{}>",Green.paint("https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps"));
    println!(". choose your existing private Dropbox app like `{}{}`",Green.paint("backup_"),Green.paint(chrono::offset::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()),);
    println!("- click button Generate to generated short-lived access token and copy it, close browser");
    println!("- In you Linux terminal session set a short-lived private/secret environment variable:");
    println!("$ {}{}",Green.paint("export DBX_OAUTH_TOKEN="),"here paste the access token");
    println!("- create an alias for easy of use:"); 
    println!("$ {}",Green.paint("alias dropbox_backup_to_external_disk=target/debug/dropbox_backup_to_external_disk"));
    println!("- test if the authentication works: ");
    println!("$ {}",Green.paint("dropbox_backup_to_external_disk test"));
    println!("");

    println!("{}", Yellow.paint("Commands:"));
    println!("Full list and sync - from dropbox to external disk.");
    println!("It lists remote and local files (that takes a lot of time) and then starts the sync.");
    println!("$ {} {}", Green.paint(cargo_pkg_name), Green.paint("list_and_sync /mnt/d/DropBoxBackup2"));
    println!("Sync only - from dropbox to external disk.");
    println!("It starts the sync only. Does NOT list again the remote and local files, because it takes a lot of time.");
    println!("It continues where the previous sync has finished.");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("sync_only"));
    println!("");

    println!("{}", Yellow.paint("For debugging purpose, you can run every step separately."));
    println!("Test connection and authorization:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("test"));
    println!("List all files in your remote Dropbox to list_remote_files.csv:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("list_remote"));
    println!("List local files to list_local_files.csv:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("list_local /mnt/d/DropBoxBackup2"));
    println!("Compare lists and create list_for_download.csv, list_for_trash.csv and list_for_correct_time.csv:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("compare_lists"));
    println!("move_or_rename_local_files:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("move_or_rename_local_files"));
    println!("Move to trash folder from list_for_trash.csv:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("trash_from_list"));
    println!("Correct time of files from list_for_correct_time.csv:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("correct_time_from_list"));
    println!("Download files from list_for_download.csv:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("download_from_list"));
    println!("Download one single file:");
    println!("$ {} {}", Green.paint(cargo_pkg_name),Green.paint("download <path>"));

}
