#![warn(missing_docs)]

//! Countdown letters game solver

mod results;

use std::io;
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use dictionary::{Dictionary, WordSizeConstraint};
use numformat::NumFormat;
use solver::{find_words, SolverArgs};

use crate::results::print_results;

/// Countdown letters game solver
#[derive(Parser, Default)]
#[clap(author, version, about)]
struct Args {
    /// Current board
    #[clap(value_parser = validate_board)]
    board: String,

    /// Letters not in the solution
    #[clap(value_parser = validate_letters)]
    unused: String,

    /// Letters in the solution
    #[clap(value_parser = validate_letters)]
    unplaced: Option<String>,

    /// Word list file
    #[clap(
        short = 'd',
        long = "dictionary",
        default_value_t = default_dict().into(),
    )]
    dictionary_file: String,

    /// Verbose output
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,

    /// Debug output
    #[clap(long = "debug")]
    debug: bool,
}

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Check we have a dictionary
    if args.dictionary_file.is_empty() {
        eprintln!("No dictionary file given and none of the default dictionaries could be found.");
        eprintln!("Default dictionaries are:");

        for d in DICTS {
            eprintln!("  {d}");
        }

        std::process::exit(1);
    }

    // Load words
    let mut size = WordSizeConstraint::default();

    size.set_min(5);
    size.set_max(5);

    let dictionary = Dictionary::new_from_file(&args.dictionary_file, size, args.verbose)?;

    // Find words
    let start_time = Instant::now();

    let words = find_words(SolverArgs {
        board: &args.board,
        unplaced: &args.unplaced,
        unused: &args.unused,
        dictionary: &dictionary,
        debug: args.debug,
    });

    if args.verbose {
        println!(
            "Search took {} seconds",
            start_time.elapsed().as_secs_f64().num_format_sigdig(2)
        );
    }

    // Print results
    print_results(words);

    Ok(())
}

fn validate_board(s: &str) -> Result<String, String> {
    // Check minimum length
    if s.len() != 5 {
        Err("Board should contain 5 characters")?;
    }

    // Convert all letters to upper case
    let ustring = s
        .chars()
        .map(|c| c.to_ascii_uppercase())
        .collect::<String>();

    // Check we only have upper case ascii characters or '.'
    if !ustring.chars().all(|c| c.is_ascii_uppercase() || c == '.') {
        Err("Board letters must be A-Z or . only".to_string())?;
    }

    Ok(ustring)
}

fn validate_letters(s: &str) -> Result<String, String> {
    // Convert all letters to upper case
    let ustring = s
        .chars()
        .map(|c| c.to_ascii_uppercase())
        .collect::<String>();

    // Check we only have upper case ascii characters
    if !ustring.chars().all(|c| c.is_ascii_uppercase()) {
        Err("Letters must be A-Z only".to_string())?;
    }

    Ok(ustring)
}

const DICTS: [&str; 3] = [
    "words.txt",
    "words.txt.gz",
    "/etc/dictionaries-common/words",
];

fn default_dict() -> &'static str {
    DICTS
        .iter()
        .find(|d| dict_valid(d).is_some())
        .unwrap_or(&"")
}

fn dict_valid(dict: &str) -> Option<String> {
    if Path::new(dict).is_file() {
        Some(dict.into())
    } else {
        None
    }
}
