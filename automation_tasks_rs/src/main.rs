//! automation_tasks_rs with_lib

use cargo_auto_lib::*;
use unwrap::unwrap;

/// automation_tasks_rs with_lib
fn main() {
    exit_if_not_run_in_rust_project_root_directory();

    // get CLI arguments
    let mut args = std::env::args();
    // the zero argument is the name of the program
    let _arg_0 = args.next();
    match_arguments_and_call_tasks(args);
}

// region: match, help and completion. Take care to keep them in sync with the changes.

/// match arguments and call tasks functions
fn match_arguments_and_call_tasks(mut args: std::env::Args) {
    // the first argument is the user defined task: (no argument for help), build, release,...
    let arg_1 = args.next();
    match arg_1 {
        None => print_help(),
        Some(task) => {
            if &task == "completion" {
                completion();
            } else {
                println!("Running automation task: {}", &task);
                if &task == "build" || &task == "b" {
                    task_build();
                } else if &task == "release" || &task == "r" {
                    task_release();
                } else if &task == "docs" || &task == "doc" || &task == "d" {
                    task_docs();
                } else if &task == "github_new_release" {
                    task_github_new_release();
                } else {
                    println!("Task {} is unknown.", &task);
                    print_help();
                }
            }
        }
    }
}

/// write a comprehensible help for user defined tasks
fn print_help() {
    println!(
        r#"
User defined tasks in automation_tasks_rs:
cargo auto build - builds the crate in debug mode, fmt
cargo auto release - builds the crate in release mode, version from date, fmt, strip
cargo auto doc - builds the docs, copy to docs directory
cargo auto github_new_release - creates new release on github
  this task needs PAT (personal access token from github) in the env variable: `export GITHUB_TOKEN=paste_token_here`

Create alias for easy use when developing:
  $ alias {package_name}=target/debug/{package_name}
Create auto-completion:
  $ complete -C "{package_name} completion" {package_name}
  {package_name} --help - instructions especially for first use because of authentication
"#,
        package_name = package_name()
    );
}

/// sub-command for bash auto-completion of `cargo auto` using the crate `dev_bestia_cargo_completion`
fn completion() {
    let args: Vec<String> = std::env::args().collect();
    let word_being_completed = args[2].as_str();
    let last_word = args[3].as_str();

    if last_word == "cargo-auto" || last_word == "auto" {
        let sub_commands = vec!["build", "release", "doc", "github_new_release"];
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

// endregion: match, help and completion.

// region: tasks

/// example how to call a list of shell commands
fn task_build() {
    auto_version_from_date();
    #[rustfmt::skip]
    let shell_commands = [
        "cargo fmt", 
        "cargo build",
        &format!("target/debug/{package_name} --help" , package_name = package_name()),
        ];
    run_shell_commands(shell_commands.to_vec());
    println!(
        r#"
Create alias for easy use when developing:
  $  alias {package_name}=target/debug/{package_name}
Create auto-completion:
  $  complete -C "{package_name} completion" {package_name}

After `cargo auto build`, run the tests and the code. If ok, then `cargo auto release`
"#,
        package_name = package_name()
    );
}

/// example how to call one shell command and combine with rust code
fn task_release() {
    auto_version_from_date();
    auto_cargo_toml_to_md();
    auto_lines_of_code("");

    run_shell_command("cargo fmt");
    run_shell_command("cargo build --release");
    run_shell_command(&format!(
        "strip target/release/{package_name}",
        package_name = package_name()
    ));
    run_shell_command(&format!(
        "target/release/{package_name} --help",
        package_name = package_name()
    ));
    println!(
        r#"
Create alias for easy use when developing:
    $  alias {package_name}=target/release/{package_name}
Create auto-completion:
    $  complete -C "{package_name} completion" {package_name}

After `cargo auto release`, run the tests and the code. If ok, then `cargo auto doc`
"#,
        package_name = package_name()
    );
}

/// example how to call a list of shell commands and combine with rust code
fn task_docs() {
    auto_cargo_toml_to_md();
    auto_lines_of_code("");
    auto_md_to_doc_comments();
    #[rustfmt::skip]
    let shell_commands = [
        "cargo doc --no-deps --document-private-items --open",
        // copy target/doc into docs/ because it is github standard
        "rsync -a --info=progress2 --delete-after target/doc/ docs/",
        "echo Create simple index.html file in docs directory",
        &format!("echo \"<meta http-equiv=\\\"refresh\\\" content=\\\"0; url={}/index.html\\\" />\" > docs/index.html",package_name().replace("-","_")) ,
    ];
    run_shell_commands(shell_commands.to_vec());
    // message to help user with next move
    println!(
        r#"
After `cargo auto doc`, check `docs/index.html`. If ok, then `git commit -am"message"` and `git push`,
then manually create release on Github
"#
    );
}

/// create a new release on github with octocrab
/// the env variable GITHUB_TOKEN must be set `export GITHUB_TOKEN=paste_token_here`
fn task_github_new_release() {
    // async block with tokio and reqwest (octocrab choice)
    tokio::spawn(async move {
        use octocrab::Octocrab;

        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
        let octocrab = unwrap!(Octocrab::builder().personal_token(token).build());

        let body_md_text = &format!(
            r#"
From the [README.md]({package_repository}) instructions to install:
```bash
cd ~
mkdir {package_name}
cd {package_name}

curl -L https://github.com/{github_owner}/{package_name}/releases/latest/download/{package_name} --output {package_name}

chmod +x {package_name}
alias {package_name}=./{package_name}
complete -C "{package_name} completion" {package_name}
{package_name} --help
            ```            
            "#,
            package_repository = unwrap!(package_repository()),
            package_name = package_name(),
            github_owner = github_owner()
        );
        let _page = unwrap!(
            octocrab
                .repos(github_owner(), package_name())
                .releases()
                .create(&format!("v{}", package_version()))
                .target_commitish("main")
                .name(&format!("Version {}", package_version()))
                .body(body_md_text)
                .draft(false)
                .prerelease(false)
                // Send the request
                .send()
                .await
        );

        // upload asset
        let path_to_file = format!(
            "target/release/{package_name}",
            package_name = package_name()
        );
        let file = std::path::Path::new(&path_to_file);
        let file_name = file.file_name().unwrap().to_str().unwrap();
        let mut new_url = octocrab.base_url.clone();
        new_url.set_query(Some(format!("{}={}", "name", file_name).as_str()));

        let file_size = unwrap!(std::fs::metadata(file)).len();
        let file = unwrap!(tokio::fs::File::open(file).await);
        let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
        let body = reqwest::Body::wrap_stream(stream);

        let builder = octocrab
            .request_builder(new_url.as_str(), reqwest::Method::POST)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", file_size.to_string());

        let _response = unwrap!(builder.body(body).send().await);
    });
}

// endregion: tasks

// region: helper functions

/// check if run in rust project root directory error
/// there must be Cargo.toml and the directory automation_tasks_rs
/// exit with error message if not
fn exit_if_not_run_in_rust_project_root_directory() {
    if !(std::path::Path::new("automation_tasks_rs").exists()
        && std::path::Path::new("Cargo.toml").exists())
    {
        println!("Error: automation_tasks_rs must be called in the root directory of the rust project beside the Cargo.toml file and automation_tasks_rs directory.");
        // early exit
        std::process::exit(0);
    }
}

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

fn github_owner() -> String {
    match package_repository() {
        Some(repository) => {
            unwrap!(repository.trim_start_matches("https://").split("/").next()).to_string()
        }
        None => "".to_string(),
    }
}
// endregion: helper functions
