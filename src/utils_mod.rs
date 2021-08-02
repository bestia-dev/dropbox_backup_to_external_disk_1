//! utils_mod.rs
//! A module with often used functions. For every project I select only the functions I need for the project.

use std::io::Stdout;

#[allow(unused_imports)]
use chrono::prelude::*;
use lazy_static::lazy_static;
use termion::raw::RawTerminal;
use uncased::UncasedStr;
use unwrap::unwrap;

lazy_static! {
    pub static ref GREEN: String = termion::color::Fg(termion::color::Green).to_string();
    pub static ref YELLOW: String = termion::color::Fg(termion::color::Yellow).to_string();
    pub static ref RED: String = termion::color::Fg(termion::color::Red).to_string();
    pub static ref RESET: String = termion::color::Fg(termion::color::Reset).to_string();
    /// ansi terminal - clears the line on the terminal from cursor to end of line
    pub static ref CLEAR_LINE: String = termion::clear::CurrentLine.to_string();
    pub static ref CLEAR_ALL: String = termion::clear::All.to_string();
    //pub static ref HIDE_CURSOR: String = termion::cursor::Hide.to_string();
    pub static ref UNHIDE_CURSOR: String = termion::cursor::Show.to_string();
}

/// move cursor to line
pub fn at_line(y: u16) -> String {
    termion::cursor::Goto(1, y).to_string()
}

/// get cursor position from raw_mode, but return immediately to normal_mode
pub fn get_pos(
    hide_cursor_terminal: &mut termion::cursor::HideCursor<RawTerminal<Stdout>>,
) -> (u16, u16) {
    unwrap!(hide_cursor_terminal.activate_raw_mode());
    use termion::cursor::DetectCursorPos;
    // this can return error: Cursor position detection timed out.
    let (x, y) = unwrap!(hide_cursor_terminal.cursor_pos());
    unwrap!(hide_cursor_terminal.suspend_raw_mode());
    (x, y)
}

/// when changing cursor position it is good to hide the cursor
pub fn start_hide_cursor_terminal() -> termion::cursor::HideCursor<RawTerminal<Stdout>> {
    let hide_cursor = termion::cursor::HideCursor::from(
        termion::raw::IntoRawMode::into_raw_mode(std::io::stdout()).unwrap(),
    );
    unwrap!(hide_cursor.suspend_raw_mode());
    // return
    hide_cursor
}

/// returns the now in nanoseconds
pub fn ns_start(text: &str) -> i64 {
    let now = Utc::now();
    if !text.is_empty() {
        println!(
            "{}{}: {}{}",
            *GREEN,
            &Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            text,
            *RESET
        );
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

        println!(
            "{}{:>15} ms: {}{}",
            *GREEN, string_duration_ns, name, *RESET
        );
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

        println!(
            "{}{:>15} ns: {}{}",
            *GREEN, string_duration_ns, name, *RESET
        );
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
