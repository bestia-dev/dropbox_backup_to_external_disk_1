//! terminal_ansi_mod.rs

// How to print on the same Line? Use the \r, but it does not work every time.
// https://www.lihaoyi.com/post/BuildyourownCommandLinewithANSIescapecodes.html
/*
            Move up/down is not working because some lines are too long and they change the cursor position
            It is better to use fixed row position numbers.
            Up: \x1b[{n}A moves cursor up by n
Down: \x1b[{n}B moves cursor down by n
Right: \x1b[{n}C moves cursor right by n
Left: \x1b[{n}D moves cursor left by n
Next Line: \x1b[{n}E moves cursor to beginning of line n lines down
Prev Line: \x1b[{n}F moves cursor to beginning of line n lines down
Set Column: \x1b[{n}G moves cursor to column n
Set Position: \x1b[{n};{m}H moves cursor to row n column m
Clear Screen: \x1b[{n}J clears the screen
n=0 clears from cursor until end of screen,
n=1 clears from cursor to beginning of screen
n=2 clears entire screen
Clear Line: \x1b[{n}K clears the current line
n=0 clears from cursor to end of line
n=1 clears from cursor to start of line
n=2 clears entire line

            */

pub fn ansi_clear_screen() {
    print!("\x1B[2J");
}

/// set row and clear line
pub fn ansi_set_row(row:u32)->String{
    format!("\x1b[{};{}H\x1b[2K", row, 1)
}