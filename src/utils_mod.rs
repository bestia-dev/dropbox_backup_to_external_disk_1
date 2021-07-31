//! utils_mod.rs
//! A module with often used functions. For every project I select only the functions I need for the project.

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use chrono::prelude::*;
use unwrap::unwrap;
use uncased::UncasedStr;

/// returns the now in nanoseconds
pub fn ns_start(text: &str) -> i64 {
    let now = Utc::now();
    if !text.is_empty() {
        println!(
            "{}: {}",
            Green.paint(&Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            Green.paint(text)
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
    let duration_ns = ns_elapsed(ns_start)/1_000_000;
    if !name.is_empty() {

        use num_format::{Locale, WriteFormatted};
        let mut string_duration_ns = String::new();
        unwrap!( string_duration_ns.write_formatted(&duration_ns, &Locale::en));
        
        println!("{:>15} {}: {}",Green.paint( string_duration_ns),Green.paint("ms"), Green.paint( name));
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
        unwrap!( string_duration_ns.write_formatted(&duration_ns, &Locale::en));
        
        println!("{:>15} {}: {}",Green.paint( string_duration_ns),Green.paint("ns"), Green.paint( name));
    }
    // return new now_ns
    Utc::now().timestamp_nanos()
}

/// ansi terminal - clears the line on the terminal from cursor to end of line
pub fn clear_line()->&'static str{
    "\x1b[0K"
}
/// ansi terminal - hide cursor
pub fn hide_cursor()->&'static str{
    "\x1b[?25l"
}
/// ansi terminal - unhide cursor
pub fn unhide_cursor()->&'static str{
    "\x1b[?25h"
}

// sort string lines case insensitive
pub fn sort_string_lines(output_string:&str)->String{
    let mut sorted_local: Vec<&str> = output_string.lines().collect();
    use rayon::prelude::*;
    sorted_local.par_sort_unstable_by(|a,b|{
        let aa: &UncasedStr = (*a).into();
        let bb: &UncasedStr = (*b).into();
        aa.cmp(bb)
    } );

    let joined = sorted_local.join("\n");  
    // return
    joined  
}