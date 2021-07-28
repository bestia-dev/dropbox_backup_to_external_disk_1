//! utils_mod.rs

#[allow(unused_imports)]
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use chrono::prelude::*;
use unwrap::unwrap;
// use unwrap::unwrap;

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

pub fn ns_elapsed(ns_start: i64) -> i64 {
    let now_ns = Utc::now().timestamp_nanos();
    let duration_ns = now_ns - ns_start;
    // return
    duration_ns
}

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
