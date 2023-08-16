// utils_mod.rs
//! A module with often used functions.

use std::io::Read;
use std::io::Stdout;

#[allow(unused_imports)]
use chrono::prelude::*;
use chrono::Duration;
use lazy_static::lazy_static;
use termion::raw::RawTerminal;
use uncased::UncasedStr;
use unwrap::unwrap;

lazy_static! {
    /// ansi color
    pub static ref GREEN: String = termion::color::Fg(termion::color::Green).to_string();
    /// ansi color
    pub static ref YELLOW: String = termion::color::Fg(termion::color::Yellow).to_string();
    /// ansi color
    pub static ref RED: String = termion::color::Fg(termion::color::Red).to_string();
    /// ansi reset color
    pub static ref RESET: String = termion::color::Fg(termion::color::Reset).to_string();
    /// ansi clear line
    pub static ref CLEAR_LINE: String = termion::clear::CurrentLine.to_string();
    /// ansi clear all
    pub static ref CLEAR_ALL: String = termion::clear::All.to_string();
    /// ansi unhide cursor
    pub static ref UNHIDE_CURSOR: String = termion::cursor::Show.to_string();
}

/// move cursor to line
pub fn at_line(y: u16) -> String {
    termion::cursor::Goto(1, y).to_string()
}

/// get cursor position from raw_mode, but return immediately to normal_mode
pub fn get_pos(hide_cursor_terminal: &mut termion::cursor::HideCursor<RawTerminal<Stdout>>) -> (u16, u16) {
    unwrap!(hide_cursor_terminal.activate_raw_mode());
    use termion::cursor::DetectCursorPos;
    // this can return error: Cursor position detection timed out.
    let (x, y) = unwrap!(hide_cursor_terminal.cursor_pos());
    unwrap!(hide_cursor_terminal.suspend_raw_mode());
    (x, y)
}

/// when changing cursor position it is good to hide the cursor
pub fn start_hide_cursor_terminal() -> termion::cursor::HideCursor<RawTerminal<Stdout>> {
    let hide_cursor = termion::cursor::HideCursor::from(termion::raw::IntoRawMode::into_raw_mode(std::io::stdout()).unwrap());
    unwrap!(hide_cursor.suspend_raw_mode());
    // return
    hide_cursor
}

/// returns the now in nanoseconds
pub fn ns_start(text: &str) -> i64 {
    let now = Utc::now();
    if !text.is_empty() {
        println!("{}{}: {}{}", *GREEN, &Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text, *RESET);
    }
    now.timestamp_nanos()
}

/// returns the elapsed nanoseconds
pub fn ns_elapsed(ns_start: i64) -> i64 {
    let now_ns = Utc::now().timestamp_nanos();
    let duration_ns = now_ns - ns_start;
    // return
    duration_ns
}

/// print elapsed time in milliseconds and returns the new now in nanoseconds
pub fn ns_print_ms(name: &str, ns_start: i64) -> i64 {
    // milliseconds
    let duration_ns = ns_elapsed(ns_start) / 1_000_000;
    if !name.is_empty() {
        use num_format::{Locale, WriteFormatted};
        let mut string_duration_ns = String::new();
        unwrap!(string_duration_ns.write_formatted(&duration_ns, &Locale::en));

        println!("{}{:>15} ms: {}{}", *GREEN, string_duration_ns, name, *RESET);
    }
    // return new now_ns
    Utc::now().timestamp_nanos()
}

/// print elapsed time in nanoseconds and returns the new now in nanoseconds
pub fn ns_print_ns(name: &str, ns_start: i64) -> i64 {
    // milliseconds
    let duration_ns = ns_elapsed(ns_start);
    if !name.is_empty() {
        use num_format::{Locale, WriteFormatted};
        let mut string_duration_ns = String::new();
        unwrap!(string_duration_ns.write_formatted(&duration_ns, &Locale::en));

        println!("{}{:>15} ns: {}{}", *GREEN, string_duration_ns, name, *RESET);
    }
    // return new now_ns
    Utc::now().timestamp_nanos()
}

/// sort string lines case insensitive
pub fn sort_string_lines(output_string: &str) -> String {
    let mut sorted_local: Vec<&str> = output_string.lines().collect();
    use rayon::prelude::*;
    sorted_local.par_sort_unstable_by(|a, b| {
        let aa: &UncasedStr = (*a).into();
        let bb: &UncasedStr = (*b).into();
        aa.cmp(bb)
    });

    let joined = sorted_local.join("\n");
    // return
    joined
}

/// shorten path for screen to avoid word-wrap
pub fn shorten_string(text: &str, x_max_char: u16) -> String {
    if text.chars().count() > x_max_char as usize {
        let x_half_in_char = (x_max_char / 2 - 2) as usize;
        let pos1_in_bytes = byte_pos_from_chars(text, x_half_in_char);
        let pos2_in_bytes = byte_pos_from_chars(text, text.chars().count() - x_half_in_char);
        return format!("{}...{}", &text[..pos1_in_bytes], &text[pos2_in_bytes..]);
    } else {
        return text.to_string();
    }
}

/// it is used for substring, because string slice are counted in bytes and not chars.
/// if we have multi-byte unicode characters we can get an error if the boundary is not on char boundary.
pub fn byte_pos_from_chars(text: &str, char_pos: usize) -> usize {
    text.char_indices().nth(char_pos).unwrap().0
}

use std::io::Write;
use std::thread;
use std::time;

use termion;
use termion::input::TermRead;

/// waits 5 seconds for the user to press any key then continues  
/// It is usable to make visible some data before going to the next step where the screen is cleaned.  
pub fn press_enter_to_continue_timeout_5_sec() {
    print!("press any key or wait 5 seconds to continue. 5..");
    let started = Utc::now();
    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut hide_cursor_terminal = crate::start_hide_cursor_terminal();
    unwrap!(hide_cursor_terminal.activate_raw_mode());

    // Use asynchronous stdin
    // The async_stdin opens a channel and then a thread with a loop to send keys to the receiver AsyncReader - async_stdin().
    // The thread stops when it tries to send a key, but the receiver does not exist any more: `send.send(i).is_err()`
    // Until there is no key in stdin it will not try to send and will not know that the receiver is dropped and the thread will live forever.
    // And that will create a panic on the next get_pos, that uses the same async_stdin. There cn be only one.
    let stdin = termion::async_stdin();
    let mut async_stdin_keys_receiver = stdin.keys();
    let mut count_seconds = 0;
    loop {
        // Read input (if any)
        let input = async_stdin_keys_receiver.next();

        // If any key was pressed
        if let Some(Ok(_key)) = input {
            break;
        }
        // if timeout 5 seconds passed
        let passed = Utc::now().signed_duration_since(started);
        if passed > Duration::seconds(1) && count_seconds < 1 {
            count_seconds += 1;
            print!("4..");
            hide_cursor_terminal.flush().unwrap();
            //raw_stdout.lock().flush().unwrap();
        } else if passed > Duration::seconds(2) && count_seconds < 2 {
            count_seconds += 1;
            print!("3..");
            hide_cursor_terminal.flush().unwrap();
            //raw_stdout.lock().flush().unwrap();
        } else if passed > Duration::seconds(3) && count_seconds < 3 {
            count_seconds += 1;
            print!("2..");
            hide_cursor_terminal.flush().unwrap();
            //raw_stdout.lock().flush().unwrap();
        } else if passed > Duration::seconds(4) && count_seconds < 4 {
            count_seconds += 1;
            print!("1..");
            hide_cursor_terminal.flush().unwrap();
            //raw_stdout.lock().flush().unwrap();
        } else if passed > Duration::seconds(5) {
            print!("0",);
            break;
        }
        // to avoid CPU overuse because of loop
        thread::sleep(time::Duration::from_millis(50));
    }
    // drop the AsyncReader (receiver), so the sender inside the thread will got an error on next send.
    // But sometimes there is no next send ! I need a way to write to stdin without the user and keyboard.
    // This ansi code on stdout "\x1B[6n" is:  Where is the cursor?
    // The reply goes to stdin.
    // This should end the loop and the thread waiting for stdin.
    drop(async_stdin_keys_receiver);
    print!("\x1B[6n");
    hide_cursor_terminal.flush().unwrap();
    // the thread will exit, but now the reply of our ansi code is written on the screen: ^[[48;25R
    // now I need to silently empty the stdin until R
    for x in std::io::stdin().keys() {
        if let Ok(y) = x {
            if let termion::event::Key::Char('R') = y {
                break;
            }
        }
    }

    unwrap!(hide_cursor_terminal.suspend_raw_mode());
    println!("");
}

/// object to work with text files
pub struct FileTxt {
    file_txt: std::fs::File,
}
impl FileTxt {
    pub fn from_file(file: std::fs::File) -> Self {
        FileTxt { file_txt: file }
    }
    /// if file not exist, it creates it
    pub fn open_for_read_and_write(path: &str) -> std::io::Result<Self> {
        let path = std::path::Path::new(path);
        if !path.exists() {
            std::fs::File::create(path).unwrap();
        }
        let file = std::fs::File::options().read(true).write(true).open(path)?;

        Ok(Self::from_file(file))
    }
    /// This method is similar to fs::read_to_string, but instead of a path it expects a File parameter
    /// So is possible to open a File in the bin part of the project and then pass it to the lib part of project.
    /// All input and output should be in the bin part of project and not in the lib.
    pub fn read_to_string(&mut self) -> std::io::Result<String> {
        let size = self.file_txt.metadata().map(|m| m.len()).unwrap_or(0);
        let mut string = String::with_capacity(size as usize);
        self.file_txt.read_to_string(&mut string)?;
        Ok(string)
    }

    pub fn write_str(&mut self, str: &str) -> std::io::Result<()> {
        self.file_txt.write_all(str.as_bytes())?;
        Ok(())
    }
}
