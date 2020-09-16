[comment]: # (lmake_md_to_doc_comments segment start A)

# dropbox_files

[comment]: # (lmake_cargo_toml_to_md start)

[comment]: # (lmake_cargo_toml_to_md end)

[comment]: # (lmake_lines_of_code start)

[comment]: # (lmake_lines_of_code end)

On my Dropbox "remote drive" I have more than 1 Terabyte of data in 200 000 files.  
I own now 4 notebooks and 2 android phones and 1 tablet and not a single one has an internal drive with more than 1 Terabyte. I use `Selective Sync` to sync only the bare minimum I temporarily need on the local device. But I want to have a backup of all of my data. I must have a backup. Better, I want to have 2 backups of all the data on external hard disks. So if Dropbox go bankrupt, I still have all my data.  
The original Dropbox Sync app works great for the internal HD, but is "not recommended" for external drives. I also need only one way sync: from remote to local. There exist apps for that:
- rclone
- dropbox_uploader

But I wanted to write something mine for fun, learning Rust and using my own apps.
I have a lot of files, so I wanted to list them first, then compare with the local files and finally download them. The delete part at the end is still todo.

## DropBox api2 - Stone sdk

Dropbox has made a `Stone` thing that contains all the API definition. From there is possible to generate code boilerplate for different languages for the api-client. 
For Rust there is this quasi official project:  
<https://crates.io/crates/dropbox-sdk>  

## Authorization OAuth2

Authorization on the internet is a mess. Dropbox uses OAuth2.
Every app must have its own `app key` and `app secret`. 
For commercial programs they probably embed them into the binary code somehow. But for OpenSource projects it is not possible to keep a secret. So the workaround is: every user must create a new `dropbox app`. It will be private for him only. Creating a new app is medium simple:
- login to dropbox.com
- App console <https://www.dropbox.com/developers/apps?_tk=pilot_lp&_ad=topbar4&_camp=myapps>
- Create app
- Scoped access
- Full dropbox
- choose a unique app name (it does not matter) ex. `dbx_files_20200916_181100`, write it somewhere safe
- Permission type, click Scoped App
- check files.content.read and files.metadata.read, Submit
- on the top return to Settings
- App secret click Show, copy App key and App secret somewhere safe. This secret is like a password.
- Generated access token click Generate, copy somewhere safe.

This is it. You have a `quasi-private` dropbox app only for you. 
This is medium complicated solution because of OpenSource of the project where secrets cannot be shared.
This app will stay forever in `development status` in dropbox, to be more private and secure.

## Try it

You should be logged in Linux with your account. So things you do are mostly not visible to others.
In Linux bash before starting this app write the token into this environment variable:
`$ export DBX_OAUTH_TOKEN=xx.xxxxx`
You can also skip this and give authorization with the interactive style.
You will need the `app key`, `app secret` and a browser.
Try the app like this:
`$ clear; cargo run --bin dropbox_files -- test`
or
`$ clear; cargo build`
`$ alias dropbox_files=target/debug/dropbox_files`
`$ dropbox_files test`
`$ unalias dropbox_files`

[comment]: # (lmake_md_to_doc_comments segment end A)

## Development

Repository:
<https://github.com/LucianoBestia/dropbox_files>  

## dropbox_files list_remote

List all the files from the remote Dropbox and saves to the file data/list_remote_files.csv.
Tab delimited with metadata: path (with name), datetime modified, size.
The path is not really case-sensitive. They try to make it case-preserve, but this apply only to the last part of the path. Before that it is random.
For big dropbox remotes it can take a while to complete.
