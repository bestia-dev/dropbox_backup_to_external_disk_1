//! dropbox_backup_to_external_disk.rs

// All work with input/output should be inside the bin part of the Rust project, and nothing in the lib part.
// Inside bin I should print on the screen and open or create Files. Then pass the Files to the lib part to operate on them.
// But to be interactive I cannot wait for a lib function to finish. The lib functions should be in another thread.
// Then send msg to the bin  main thread that print that to the screen.

use dropbox_backup_to_external_disk::*;
use std::env;

// define paths in bin, not in lib
static APP_CONFIG: AppConfig = AppConfig {
    path_list_base_local_path: "tmp/temp_data/list_base_local_path.csv",
    path_list_source_files: "tmp/temp_data/list_source_files.csv",
    path_list_destination_files: "tmp/temp_data/list_destination_files.csv",
    path_list_source_folders: "tmp/temp_data/list_source_folders.csv",
    path_list_destination_folders: "tmp/temp_data/list_destination_folders.csv",
    path_list_destination_readonly_files: "tmp/temp_data/list_destination_readonly_files.csv",
    path_list_for_download: "tmp/temp_data/list_for_download.csv",
    path_list_for_trash: "tmp/temp_data/list_for_trash.csv",
    path_list_for_correct_time: "tmp/temp_data/list_for_correct_time.csv",
    path_list_just_downloaded_or_moved: "tmp/temp_data/list_just_downloaded_or_moved.csv",
    path_list_for_trash_folders: "tmp/temp_data/list_for_trash_folders.csv",
    path_list_for_create_folders: "tmp/temp_data/list_for_create_folders.csv",
};

fn main() {
    pretty_env_logger::init();
    ctrlc::set_handler(move || {
        println!("terminated with ctrl+c. {}", *UNHIDE_CURSOR);
        std::process::exit(exitcode::OK);
    })
    .expect("Error setting Ctrl-C handler");

    //create the directory tmp/temp_data/
    std::fs::create_dir_all("tmp/temp_data").unwrap();

    let base_path = if std::path::Path::new(APP_CONFIG.path_list_base_local_path).exists() {
        std::fs::read_to_string(APP_CONFIG.path_list_base_local_path).unwrap()
    } else {
        String::new()
    };

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
                list_and_sync(path, &APP_CONFIG);
                ns_print_ms("list_and_sync", ns_started);
            }
            _ => println!("Unrecognized arguments. Try dropbox_backup_to_external_disk --help"),
        },
        Some("sync_only") => {
            let ns_started = ns_start("sync_only");
            print!("{}", *CLEAR_ALL);
            sync_only(&APP_CONFIG);
            ns_print_ms("sync_only", ns_started);
        }
        Some("remote_list") => {
            print!("{}", *CLEAR_ALL);
            println!("{}{}{}remote_list into {}{}", at_line(1), *CLEAR_LINE, *YELLOW, APP_CONFIG.path_list_source_files, *RESET,);
            let ns_started = ns_start("");
            test_connection();
            list_remote(&APP_CONFIG);
            ns_print_ms("remote_list", ns_started);
        }
        Some("local_list") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", *CLEAR_ALL);
                println!("{}{}{}local_list into {}{}", at_line(1), *CLEAR_LINE, *YELLOW, APP_CONFIG.path_list_destination_files, *RESET,);
                let ns_started = ns_start("");
                list_local(path, &APP_CONFIG);
                ns_print_ms("local_list", ns_started);
            }
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },
        Some("all_list") => match env::args().nth(2).as_deref() {
            Some(path) => {
                print!("{}", *CLEAR_ALL);
                println!("{}{}{}remote and local lists into tmp/temp_data{}", at_line(1), *CLEAR_LINE, *YELLOW, *RESET);
                let ns_started = ns_start("");
                test_connection();
                all_list_remote_and_local(path, &APP_CONFIG);
                ns_print_ms("all_list", ns_started);
            }
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },
        Some("read_only_toggle") => {
            let ns_started = ns_start("read_only_toggle");
            println!("{}read_only_toggle{}", *YELLOW, *RESET);
            // open file as read and write
            let mut file_destination_readonly_files = FileTxt::open_for_read_and_write(APP_CONFIG.path_list_destination_readonly_files).unwrap();
            read_only_toggle(&mut file_destination_readonly_files, &base_path);
            ns_print_ms("read_only_toggle", ns_started);
        }
        Some("compare_files") => {
            let ns_started = ns_start("compare lists");
            println!("{}compare remote and local files{}", *YELLOW, *RESET);
            compare_files(&APP_CONFIG);
            ns_print_ms("compare_files", ns_started);
        }
        Some("compare_folders") => {
            let ns_started = ns_start("compare_folders");
            println!("{}compare remote and local folders{}", *YELLOW, *RESET);
            let string_list_source_folder = std::fs::read_to_string(APP_CONFIG.path_list_source_folders).unwrap();
            let string_list_destination_folders = std::fs::read_to_string(APP_CONFIG.path_list_destination_folders).unwrap();
            let mut file_list_for_trash_folders = FileTxt::open_for_read_and_write(APP_CONFIG.path_list_for_trash_folders).unwrap();
            let mut file_list_for_create_folders = FileTxt::open_for_read_and_write(APP_CONFIG.path_list_for_create_folders).unwrap();
            compare_folders(
                &string_list_source_folder,
                &string_list_destination_folders,
                &mut file_list_for_trash_folders,
                &mut file_list_for_create_folders,
            );
            println!("Created files: list_for_trash_folders.csv and list_for_create_folders.csv");
            ns_print_ms("compare_folders", ns_started);
        }
        Some("create_folders") => {
            if base_path.is_empty() {
                println!("error: base_path is empty!");
            } else {
                let ns_started = ns_start(&format!("create_folders {}", APP_CONFIG.path_list_for_create_folders));
                let mut file_list_for_create_folders = FileTxt::open_for_read_and_write(APP_CONFIG.path_list_for_create_folders).unwrap();
                create_folders(&mut file_list_for_create_folders, &base_path);
                ns_print_ms("create_folders", ns_started);
            }
        }
        Some("trash_folders") => {
            if base_path.is_empty() {
                println!("error: base_path is empty!");
            } else {
                let ns_started = ns_start(&format!("trash_folders {}", APP_CONFIG.path_list_for_trash_folders));
                let mut file_list_for_trash_folders = FileTxt::open_for_read_and_write(APP_CONFIG.path_list_for_trash_folders).unwrap();
                trash_folders(&mut file_list_for_trash_folders, &base_path);
                ns_print_ms("trash_folders", ns_started);
            }
        }
        Some("move_or_rename_local_files") => {
            let ns_started = ns_start("move_or_rename_local_files");
            move_or_rename_local_files(&APP_CONFIG);
            ns_print_ms("move_or_rename_local_files", ns_started);
        }
        Some("trash_from_list") => {
            let ns_started = ns_start(&format!("trash from {}", APP_CONFIG.path_list_for_trash));
            trash_from_list(&APP_CONFIG);
            ns_print_ms("trash_from_list", ns_started);
        }
        Some("correct_time_from_list") => {
            let ns_started = ns_start(&format!("correct time of files from {}", APP_CONFIG.path_list_for_correct_time));
            correct_time_from_list(&APP_CONFIG);
            ns_print_ms("correct_time_from_list", ns_started);
        }
        Some("download_from_list") => {
            let ns_started = ns_start(&format!("download from {}", APP_CONFIG.path_list_for_download));
            download_from_list(&APP_CONFIG);
            ns_print_ms("download_from_list", ns_started);
        }
        Some("one_file_download") => match env::args().nth(2).as_deref() {
            Some(path) => download_one_file(path, &APP_CONFIG),
            _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
        },
        _ => println!("Unrecognized arguments. Try `dropbox_backup_to_external_disk --help`"),
    }
    // TODO: receive msg from other threads
}

/// sub-command for bash auto-completion of `cargo auto` using the crate `dev_bestia_cargo_completion`
/// `complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk`
/// `complete -p`  - shows all the completion commands
/// `complete -r xxx` - deletes a completion command
fn completion() {
    /// println one, more or all sub_commands
    fn completion_return_one_or_more_sub_commands(sub_commands: Vec<&str>, word_being_completed: &str) {
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

    if last_word.ends_with("dropbox_backup_to_external_disk") {
        let sub_commands = vec![
            "--help",
            "-h",
            "all_list",
            "compare_files",
            "compare_folders",
            "create_folders",
            "read_only_toggle",
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
            "trash_folders",
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
  Welcome to dropbox_backup_to_external_disk

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
{g} export DBX_OAUTH_TOKEN={rs}here paste the access token
  - test if the authentication works:
{g}dropbox_backup_to_external_disk test{rs}

  {y}Commands:{rs}
  Full list and sync - from dropbox to external disk
  This command has 2 phases. 
  1. First it lists all remote and local files. That can take a lot of time if you have lot of files.
  For faster work it uses concurrent threads. 
  If you interrupt the execution with ctrl+c in this phase, before the lists are completed, the lists are empty.
  You will need to rerun the command and wait for the lists to be fully completed.
  2. The second phase is the same as the command `sync_only`. 
  It can be interrupted with crl+c. The next `sync_only` will continue where it was interrupted.
{g}dropbox_backup_to_external_disk list_and_sync /mnt/d/DropBoxBackup1{rs}

  Sync only - one-way sync from dropbox to external disk
  It starts the sync only. Does NOT list again the remote and local files, the lists must already be completed 
  from the first command `list_and_sync`.
  It can be interrupted with crl+c. The next `sync_only` will continue where it was interrupted
{g}dropbox_backup_to_external_disk sync_only{rs}

  {y}Just for debugging purpose, you can run every step separately.{rs}
  Test connection and authorization:
{g}dropbox_backup_to_external_disk test{rs}
  List remote files from Dropbox to `{path_list_source_files}`:
{g}dropbox_backup_to_external_disk remote_list{rs}
  List local files to `{path_list_destination_files}`:
{g}dropbox_backup_to_external_disk local_list /mnt/d/DropBoxBackup1{rs}
  List all - both remote and local files to `temp_date/`:
{g}dropbox_backup_to_external_disk all_list /mnt/d/DropBoxBackup1{rs}  
  Read-only files toggle `{path_list_for_readonly}`:
{g}dropbox_backup_to_external_disk read_only_toggle  {rs}
  Compare file lists and generate `{path_list_for_download}`, `{path_list_for_trash}` and `{path_list_for_correct_time}`:
{g}dropbox_backup_to_external_disk compare_files{rs}
  Compare folders lists and generate `{path_list_for_trash_folders}`:
{g}dropbox_backup_to_external_disk compare_folders{rs}
  Create folders from `{path_list_for_create_folders}`:
{g}dropbox_backup_to_external_disk create_folders{rs}
  Move or rename local files if they are equal in trash_from_list and download_from_list:
{g}dropbox_backup_to_external_disk move_or_rename_local_files{rs}
  Move to trash from `{path_list_for_trash_folders}`:
{g}dropbox_backup_to_external_disk trash_folders{rs}
  Move to trash from `{path_list_for_trash}`:
{g}dropbox_backup_to_external_disk trash_from_list{rs}
  Correct time of files from `{path_list_for_correct_time}`:
{g}dropbox_backup_to_external_disk correct_time_from_list{rs}
  Download files from `{path_list_for_download}`:
{g}dropbox_backup_to_external_disk download_from_list{rs}
  One single file download:
{g}dropbox_backup_to_external_disk one_file_download <path>{rs}

  For bash auto-completion:
{g}alias dropbox_backup_to_external_disk=./dropbox_backup_to_external_disk{rs}
{g}complete -C "dropbox_backup_to_external_disk completion" dropbox_backup_to_external_disk{rs}

  Visit open-source repository: https://github.com/bestia-dev/dropbox_backup_to_external_disk
    "#,
        g = *GREEN,
        y = *YELLOW,
        rs = *RESET,
        path_list_source_files = APP_CONFIG.path_list_source_files,
        path_list_destination_files = APP_CONFIG.path_list_destination_files,
        path_list_for_download = APP_CONFIG.path_list_for_download,
        path_list_for_correct_time = APP_CONFIG.path_list_for_correct_time,
        path_list_for_trash = APP_CONFIG.path_list_for_trash,
        path_list_for_readonly = APP_CONFIG.path_list_destination_readonly_files,
        path_list_for_trash_folders = APP_CONFIG.path_list_destination_folders,
        path_list_for_create_folders = APP_CONFIG.path_list_for_create_folders,
        date = chrono::offset::Utc::now().format("%Y%m%dT%H%M%SZ"),
    );
}
