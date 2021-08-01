//! utils_mod.rs
//! A module with often used functions. For every project I select only the functions I need for the project.

use std::io::Stdout;

#[allow(unused_imports)]
use chrono::prelude::*;
use lazy_static::lazy_static;
use termion::{input::MouseTerminal, raw::RawTerminal};
use uncased::UncasedStr;
use unwrap::unwrap;

lazy_static! {
    pub static ref GREEN: String = termion::color::Fg(termion::color::Green).to_string();
    pub static ref YELLOW: String = termion::color::Fg(termion::color::Yellow).to_string();
    pub static ref RESET: String = termion::color::Fg(termion::color::Reset).to_string();
    /// ansi terminal - clears the line on the terminal from cursor to end of line
    pub static ref CLEAR_LINE: String = "\x1b[0K".to_owned();
    pub static ref CLEAR_ALL: String = termion::clear::All.to_string();
    pub static ref HIDE_CURSOR: String = termion::cursor::Hide.to_string();
    pub static ref UNHIDE_CURSOR: String = termion::cursor::Show.to_string();
}
pub fn at_pos(x: u16, y: u16) -> String {
    termion::cursor::Goto(x, y).to_string()
}
pub fn at_line(y: u16) -> String {
    termion::cursor::Goto(1, y).to_string()
}

pub fn get_pos(mouse_terminal: &mut MouseTerminal<RawTerminal<Stdout>>) -> (u16, u16) {
    unwrap!(mouse_terminal.activate_raw_mode());
    use termion::cursor::DetectCursorPos;
    let (x, y) = unwrap!(mouse_terminal.cursor_pos());
    unwrap!(mouse_terminal.suspend_raw_mode());
    (x, y)
}

pub fn start_mouse_terminal() -> MouseTerminal<RawTerminal<Stdout>> {
    let mouse_terminal = termion::input::MouseTerminal::from(
        termion::raw::IntoRawMode::into_raw_mode(std::io::stdout()).unwrap(),
    );
    unwrap!(mouse_terminal.suspend_raw_mode());
    // return
    mouse_terminal
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
