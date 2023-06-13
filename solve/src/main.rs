use std::error::Error;
use std::io;
use std::path::Path;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode,
    enable_raw_mode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
};
use dictionary::Dictionary;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod app;

use app::App;

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

    /// Verbose output
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
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
    let dictionary = Dictionary::new_from_file(&args.dictionary_file, args.verbose)?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new(dictionary);
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

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
