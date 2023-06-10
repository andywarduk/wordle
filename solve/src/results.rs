use std::cmp::max;

use numformat::NumFormat;
#[cfg(any(unix, windows))]
use terminal_size::{terminal_size, Width};

pub fn print_results(mut words: Vec<String>) {
    // Sort words alphabetically
    words.sort();

    println!(
        "{} {} found",
        words.len().num_format(),
        if words.len() == 1 { "word" } else { "words" }
    );

    // Get terminal size
    let term_width = terminal_width();

    let cols = if term_width > 0 {
        max(1, term_width as usize / (5 + 2))
    } else {
        1
    };

    for line in words.chunks(cols) {
        println!("{}", line.join("  "))
    }
}

#[cfg(any(unix, windows))]
fn terminal_width() -> u16 {
    if let Some((Width(w), _)) = terminal_size() {
        w
    } else {
        0
    }
}

#[cfg(not(any(unix, windows)))]
fn terminal_width() -> u16 {
    0
}
