use std::error::Error;
use std::path::Path;

use app::rungui;
use clap::Parser;
use dictionary::Dictionary;

mod app;

/// Wordle solver
#[derive(Parser, Default)]
#[clap(author, version, about)]
struct Args {
    /// Word list file
    #[clap(
        short = 'd',
        long = "dictionary",
        default_value_t = default_dict().into(),
    )]
    dictionary_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
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
    let dictionary = Dictionary::new_from_file(&args.dictionary_file, false)?;

    // Run the gui
    rungui(dictionary)?;

    Ok(())
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
