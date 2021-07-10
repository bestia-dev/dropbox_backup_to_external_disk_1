//! dbx_download.rs

use dbx_download::*;

fn main() {
    env_logger::init();

    let bin_name = "dbx_download";

    match std::env::args().nth(1).as_deref() {
        None | Some("--help") | Some("-h") => print_help(&bin_name),
        Some("test") => test_connection(),
        Some("one_way_sync") => match std::env::args().nth(2).as_deref() {
            Some(path) => {
                let ns_started = ns_start("one_way_syncv");
                one_way_sync(path);
                ns_print("one_way_sync", ns_started);
            }
            _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
        },
        Some("list_remote") => {
            ansi_clear_screen();
            println!(
                "{}{}",
                ansi_set_row(1),
                "list_remote into temp_data/list_remote_files.csv"
            );
            let ns_started = ns_start("");
            list_remote();
            ns_print("list_remote", ns_started);
        }
        Some("list_local") => match std::env::args().nth(2).as_deref() {
            Some(path) => {
                ansi_clear_screen();
                println!(
                    "{}{}",
                    ansi_set_row(1),
                    "list_local into temp_data/list_local_files.csv"
                );
                let ns_started = ns_start("");
                ansi_clear_screen();
                list_local(path);
                ns_print("list_local", ns_started);
            }
            _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
        },
        Some("compare_sorted_lists") => {
            let ns_started = ns_start("compare sorted lists");
            compare_sorted_lists();
            ns_print("compare_sorted_lists", ns_started);
        }
        Some("download") => match std::env::args().nth(2).as_deref() {
            Some(path) => download(path),
            _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
        },
        Some("download_from_list") => {
            let ns_started = ns_start("download from temp_data/list_for_download.csv");
            download_from_list();
            ns_print("download_from_list", ns_started);
        }

        _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
    }
}

fn print_help(bin_name: &str) {
    eprintln!("usage: $ {} <command> [options] [<args>]", bin_name);
    eprintln!("  ");

    eprintln!("View this Help, usage:");
    eprintln!("  $ {}", bin_name);
    eprintln!("  $ {} --help", bin_name);
    eprintln!("  $ {} -h", bin_name);
    eprintln!("  ");

    eprintln!("Before first use, you will need to create your private Dropbox app and access token.");
    eprintln!("Follow the instruction below.");
    eprintln!("  ");

    eprintln!("One-way sync download:");
    eprintln!("  $ {} one_way_sync /mnt/d/DropBoxBackup2", bin_name);
    eprintln!("  ");

    eprintln!("For debugging purpose, you can run every step separately.");

    eprintln!("Test connection and authorization:");
    eprintln!("  $ {} test", bin_name);
    eprintln!("  ");

    eprintln!("List all files in your remote Dropbox to temp_data/list_remote_files.csv:");
    eprintln!("  $ {} list_remote", bin_name);
    eprintln!("  ");

    eprintln!("List local files to temp_data/list_local_files.csv:");
    eprintln!("  $ {} list_local /mnt/d/DropBoxBackup2", bin_name);
    eprintln!("  ");

    eprintln!("Compare lists and create temp_data/list_for_download.csv and temp_data/list_for_delete.csv:");
    eprintln!("  $ {} compare_sorted_lists", bin_name);
    eprintln!("  ");

    eprintln!("Download one file:");
    eprintln!("  $ {} download <path>", bin_name);
    eprintln!();

    eprintln!("Download files from temp_data/list_for_download.csv:");
    eprintln!("  $ {} download_from_list", bin_name);
    eprintln!();

    eprintln!("Before first use, you will need to create your private Dropbox app:");
    eprintln!("- open your browser");
    eprintln!("- login to dropbox.com");
    eprintln!("- open App console <https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>");
    eprintln!("- click Create app");
    eprintln!("- choose Scoped access");
    eprintln!("- choose Full dropbox");
    eprintln!("- choose a unique app name (it does not matter) ex. `dbx_files_20200916_181100`, write it somewhere safe");
    eprintln!("- Permission type, click Scoped App");
    eprintln!("- check `files.content.read` and `files.metadata.read`, Submit");
    eprintln!("- on the top return to the Settings tab");
    eprintln!("- App secret click Show, copy App key and App secret somewhere safe. This secret is like a password for your files. Be extra careful.");
    eprintln!("- Generated access token click Generate, copy somewhere safe.");
    eprintln!("Set the private/secret environment variable for you Linux terminal session:");
    eprintln!("    $ export DBX_OAUTH_TOKEN=xx.xxxxx ");
    eprintln!();
}
