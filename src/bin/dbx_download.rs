//! dbx_download.rs

use dbx_download::*;

fn main() {
    env_logger::init();

    let bin_name = "dbx_download";

    match std::env::args().nth(1).as_deref() {
        None | Some("--help") | Some("-h") => print_help(&bin_name),
        Some("test") => test_connection(),
        Some("list_remote") => {
            let ns_started = ns_start("list_remote into data/list_remote_files.csv");
            list_remote();
            ns_print("list_remote", ns_started);
        }
        Some("list_local") => match std::env::args().nth(2).as_deref() {
            Some(path) => {
                let ns_started = ns_start("list_local into data/list_local_files.csv");
                list_local(path);
                ns_print("list_local", ns_started);
            }
            _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
        },
        Some("sort_lists") => {
            let ns_started = ns_start("sort lists");
            sort_lists();
            ns_print("sort_lists", ns_started);
        }
        Some("compare_sorted_lists") => {
            let ns_started = ns_start("compare sorted lists");
            compare_sorted_lists();
            ns_print("sort_lists", ns_started);
        }
        Some("download") => match std::env::args().nth(2).as_deref() {
            Some(path) => download(path),
            _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
        },
        Some("download_from_list") => {
            let ns_started = ns_start("download from data/list_for_download.csv");
            download_from_list();
            ns_print("download_from_list", ns_started);
        }

        _ => eprintln!("Unrecognized arguments. Try {} --help", &bin_name),
    }
}

fn print_help(bin_name: &str) {
    eprintln!("usage: $ {} <command> [options] [<args>]", bin_name);
    eprintln!("  ");

    eprintln!("View Help, usage:");
    eprintln!("  $ {} --help", bin_name);
    eprintln!("  $ {} -h", bin_name);
    eprintln!("  ");

    eprintln!("Test connection and authorization:");
    eprintln!("  $ {} test", bin_name);
    eprintln!("  ");

    eprintln!("List all files in your remote Dropbox to data/list_remote_files.csv:");
    eprintln!("  $ {} list_remote", bin_name);
    eprintln!("  ");

    eprintln!("List local files to data/list_local_files.csv:");
    eprintln!("  $ {} list_local /mnt/d/DropBoxBackup2", bin_name);
    eprintln!("  ");

    eprintln!("Sort lists lexical (case insensitive):");
    eprintln!("  $ {} sort_lists", bin_name);
    eprintln!("  ");

    eprintln!("Compare lists and create data/list_for_download.csv and data/list_for_delete.csv:");
    eprintln!("  $ {} compare_sorted_lists", bin_name);
    eprintln!("  ");

    eprintln!("Download a file:");
    eprintln!("  $ {} download <path>", bin_name);
    eprintln!();

    eprintln!("If a Dropbox OAuth token is given in the environment variable:");
    eprintln!("    $ export DBX_OAUTH_TOKEN=xx.xxxxx ");
    eprintln!("it will be used, otherwise you will be prompted for");
    eprintln!("authentication interactively.");
}
