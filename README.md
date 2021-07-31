[comment]: # (lmake_md_to_doc_comments segment start A)

# dropbox_backup_to_external_disk

[comment]: # (lmake_cargo_toml_to_md start)

**one way sync from dropbox to an external disc**  
***[repo](https://github.com/lucianobestia/dropbox_backup_to_external_disk/); version: 0.1.288  date: 2021-07-31 authors: Luciano Bestia***  

[comment]: # (lmake_cargo_toml_to_md end)

[comment]: # (lmake_lines_of_code start)
[![Lines in Rust code](https://img.shields.io/badge/Lines_in_Rust-1009-green.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
[![Lines in Doc comments](https://img.shields.io/badge/Lines_in_Doc_comments-124-blue.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
[![Lines in Comments](https://img.shields.io/badge/Lines_in_comments-101-purple.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
[![Lines in examples](https://img.shields.io/badge/Lines_in_examples-0-yellow.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)
[![Lines in tests](https://img.shields.io/badge/Lines_in_tests-0-orange.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)

[comment]: # (lmake_lines_of_code end)

[![Licence](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/blob/master/LICENSE) [![Rust](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/workflows/RustAction/badge.svg)](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/)

On my Dropbox "remote drive" I have more than 1 Terabyte of data in 200 000 files.  
I own now 4 notebooks and 2 android phones and 1 tablet and not a single one has an internal drive with more than 1 Terabyte. I use dropbox `Selective Sync` to sync only the bare minimum I temporarily need on the local device. But I want to have a backup of all of my data. I must have a backup. Better, I want to have 2 backups of all the data on 2 external hard disks in different locations. So if Dropbox go bankrupt, I still have all my data.  
The original Dropbox Sync app works great for the internal HD, but is "not recommended" for external drives. I also need only one way sync: from remote to local. There exist apps for that:

- rclone
- dropbox_uploader

But I wanted to write something mine for fun, learning Rust and using my own apps.
I have a lot of files, so I wanted to list them first, then compare with the local files and finally download them. The trash part at the end will be "move to trash folder". So I can inspect what and how to remove it manually.  

## Try it

You should be logged in Linux terminal with your account. So things you do, are not visible to others.  
You will set some local environment variables that are private/secret to your linux Session.  
After you logout from you Linux session the local environment variables will be deleted.  
You have to be in the project folder where cargo.toml is.  
Build the CLI:  
`$ cargo make debug`  
Follow carefully the instructions.  
Before the first use, create your Dropbox app.  
Before every use generate your "short-lived access token" and in Linux bash write the "token" into the environment variable like this:  
`$ export DBX_OAUTH_TOKEN=here paste the token`  
Make a temporary alias for easy of use (it lasts only for this session lifetime) :  
`$ alias dropbox_backup_to_external_disk=target/debug/dropbox_backup_to_external_disk`  
Test the connection and permission:  
`$ dropbox_backup_to_external_disk test`  
  
Later, use `$ dropbox_backup_to_external_disk --help` to get all the instructions and commands.  

![screenshot_1](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/screenshot_1.png "screenshot_1") ![screenshot_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/screenshot_2.png "screenshot_2")  
![dropbox_1](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/dropbox_1.png "dropbox_1") ![dropbox_2](https://github.com/LucianoBestia/dropbox_backup_to_external_disk/raw/master/images/dropbox_2.png "dropbox_1" width=300)

## Warning

I don't know why, but WSL2 sometimes does not see all the folders of the external disk.  
Instead of 12.000 folders it sees only 28 ???  
Be careful !  
I then restart my Win10 and the problem magically disappears.

## Development

I use WSL2 on Win10 to develope and execute this CLI in Debian Linux.  
The external disk path from WSL2 looks like this: `/mnt/d/DropBoxBackup1`. CLI lists the local files metadata in `temp_data/list_local_files.csv`.  
List all the files metadata from the remote Dropbox to the file `temp_data/list_remote_files.csv`.
Tab delimited with metadata: path (with name), datetime modified, size.
The path is not really case-sensitive. They try to make it case-preserve, but this apply only to the last part of the path. Before that it is random.
For big dropbox remotes it can take a while to complete. After the first level folders are listed, I use 3 threads in a ThreadPool to get sub-folders recursively in parallel. It makes it much faster. Also the download of files is in parallel on multiple threads.  
The sorting of lists is also done in parallel with the crate Rayon.  
Once the lists are complete the CLI will compare them and create files:  
`list_for_correct_time.csv`  
`list_for_download.csv`  
`list_for_trash.csv`  
With this files the CLI will:  
`move_or_rename_local_files` using the content_hash to be sure they are equal  
`trash_from_list` will move the obsolete files into a trash folder  
`correct_time_from_list` sometimes it is needed  
`download_from_list` - this can take a lot of time and it can be stopped and restarted with the use of the `list_just_downloaded.csv`.  

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
Then every time before use we need an "access token" that is short-lived for security reasons.  

## rename or move

Often a file is renamed or moved to another folder. I can try to recognize if there is the same file in list_for_trash and list_for download, but I cannot use the file path or name. Instead the metadata size, date modified and content_hash must be the same.  

## REGEX adventure with non-breaking space and CRLF

We all know space. But there are other non-visible characters that are very similar and sometimes impossible to distinguish. Tab is one of them, but it is not so difficult to spot with a quick try.  
But nbsp non-breaking space, often used in HTML is a catastrophe. There is no way to tell it apart from the normal space. I used a regex to find a match with some spaces. It worked right for a years. Yesterday it didn't work. If I changed space to `\s` in the regex expression, it worked, but not with space. I tried everything and didn't find the cause. Finally I deleted and inserted the space. It works. But how? After a detailed analysis I discovered it was a non-breakable space. This is unicode 160 or \xa0, instead of normal space unicode 32 \x20. Now I will try to find them all and replace with normal space. What a crazy world.  
And another REGEX surprise. I try to have all text files delimited with the unix standard LF. But somehow the windows standard got mixed and I didn't recognize it. The regex for `end of line` $ didn't work for CRLF. When I changed it to LF, the good life is back and all works.

[comment]: # (lmake_md_to_doc_comments segment end A)
