[comment]: # (lmake_md_to_doc_comments segment start A)

# dropbox_backup_to_external_disk

[comment]: # (lmake_cargo_toml_to_md start)

[comment]: # (lmake_cargo_toml_to_md end)

[comment]: # (lmake_lines_of_code start)

[comment]: # (lmake_lines_of_code end)

On my Dropbox "remote drive" I have more than 1 Terabyte of data in 200 000 files.  
I own now 4 notebooks and 2 android phones and 1 tablet and not a single one has an internal drive with more than 1 Terabyte. I use dropbox `Selective Sync` to sync only the bare minimum I temporarily need on the local device. But I want to have a backup of all of my data. I must have a backup. Better, I want to have 2 backups of all the data on 2 external hard disks in different locations. So if Dropbox go bankrupt, I still have all my data.  
The original Dropbox Sync app works great for the internal HD, but is "not recommended" for external drives. I also need only one way sync: from remote to local. There exist apps for that:

- rclone
- dropbox_uploader

But I wanted to write something mine for fun, learning Rust and using my own apps.
I have a lot of files, so I wanted to list them first, then compare with the local files and finally download them. The trash part at the end will be "move to trash folder". So I can inspect what and how to remove it manually.  

## DropBox api2 - Stone sdk

Dropbox has made a `Stone` thing that contains all the API definition. From there is possible to generate code boilerplate for different languages for the api-client.  
For Rust there is this quasi official project:  
<https://crates.io/crates/dropbox-sdk>  

## Authorization OAuth2

Authorization on the internet is a mess. Dropbox api uses OAuth2.
Every app must be authorized on Dropbox and have its own `app key` and `app secret`.  
For commercial programs they probably embed them into the binary code somehow. But for OpenSource projects it is not possible to keep a secret. So the workaround is: every user must create a new `dropbox app` exclusive only to him. Creating a new app is simple. This app will stay forever in `development status` in dropbox, to be more private and secure. The  
`$ dropbox_backup_to_external_disk --help`  
has the detailed instructions.  
Then every time before use we need an "access token" that is temporary for security reasons.  

## Try it

You should be logged in Linux terminal with your account. So things you do are not visible to others.  
You will set some local environment variables that are private/secret to your linux Session.  
After you logout from you Linux session the local environment variables will be deleted.  
You have to be in the project folder where cargo.toml is.  
Build the CLI:  
`$ cargo make debug`  
Follow carefully the instructions.  
Before the first use create your Dropbox app.  
Before every use generate your "temporary access token" and in Linux bash write the "access token" into the environment variable like this:  
`$ export DBX_OAUTH_TOKEN=xx.xxxxx`  
Make an alias for easy of use:  
`$ alias dropbox_backup_to_external_disk=target/debug/dropbox_backup_to_external_disk`  
Test the connection and permission:  
`$ dropbox_backup_to_external_disk test`  
  
The list of commands is:  
One-way sync download (complete with all the steps):  
`$ dropbox_backup_to_external_disk one_way_sync /mnt/d/DropBoxBackup2`  
  
For debugging purpose, you can run every step separately.  
Test connection and authorization:  
`$ dropbox_backup_to_external_disk test`  
  
List all files in your remote Dropbox to temp_data/list_remote_files.csv:  
`$ dropbox_backup_to_external_disk list_remote`  
  
List local files to temp_data/list_local_files.csv:  
`$ dropbox_backup_to_external_disk list_local /mnt/d/DropBoxBackup2`  
  
Compare lists and create temp_data/list_for_download.csv, temp_data/  list_for_trash.csv and temp_data/list_for_correct_time.csv:  
`$ dropbox_backup_to_external_disk compare_sorted_lists`  
  
Correct time of files from temp_data/list_for_correct_time.csv:  
`$ dropbox_backup_to_external_disk correct_time_from_list`  

Move to trash folder from temp_data/list_for_trash.csv:  
`$ dropbox_backup_to_external_disk trash_from_list`  

Download files from temp_data/list_for_download.csv:  
`$ dropbox_backup_to_external_disk download_from_list`  

Download one single file:  
`$ dropbox_backup_to_external_disk download <path>`  

[comment]: # (lmake_md_to_doc_comments segment end A)

## list_remote

List all the files from the remote Dropbox and saves to the file `temp_data/list_remote_files.csv`.
Tab delimited with metadata: path (with name), datetime modified, size.
The path is not really case-sensitive. They try to make it case-preserve, but this apply only to the last part of the path. Before that it is random.
For big dropbox remotes it can take a while to complete.
