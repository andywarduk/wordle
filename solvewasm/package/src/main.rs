use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

use clap::Parser;
use memmap2::Mmap;

mod multi;
use multi::multi_process;

mod single;
use single::single_process;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Start file
    #[arg()]
    start: PathBuf,

    /// Output directory
    #[arg()]
    outdir: PathBuf,

    /// Single page
    #[arg(short, long)]
    single: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line args
    let args = Cli::parse();

    // Set callback based on single or multiple assets
    let cb = if args.single { single_process } else { multi_process };

    // Create configuration
    let config = Config {
        outroot: args.outdir.clone(),
        callback: cb,
    };

    // Process the top level input file
    process_input_file(&config, args.start, 0)?;

    Ok(())
}

type ProcessCallback<R> =
    fn(config: &Config<R>, infile: PathBuf, mmap: &Mmap, depth: usize) -> Result<R, Box<dyn Error>>;

struct Config<R> {
    outroot: PathBuf,
    callback: ProcessCallback<R>,
}

fn process_input_file<R>(config: &Config<R>, infile: PathBuf, depth: usize) -> Result<R, Box<dyn Error>> {
    message(&format!("Processing file {}", infile.display()), depth);

    // Memory map the input file
    let mmap = {
        let file = File::open(&infile)?;

        unsafe { Mmap::map(&file)? }
    };

    // Call the callback to process the file content
    let result = (config.callback)(config, infile, &mmap, depth)?;

    Ok(result)
}

fn openout(outfile: &PathBuf) -> Result<File, Box<dyn Error>> {
    // Create the output directory if it doesn't exist
    if let Some(dir) = outfile.parent() {
        std::fs::create_dir_all(dir)?;
    };

    // Create the output file
    let f = File::create(outfile)?;

    Ok(f)
}

type ParseTextCallback<S> = fn(text: &str, state: &mut S) -> Result<(), Box<dyn Error>>;
type ParseLinkCallback<S> = fn(text: &str, &HashMap<String, String>, state: &mut S) -> Result<(), Box<dyn Error>>;

fn parse_file<S>(
    mmap: &Mmap,
    state: &mut S,
    textcb: ParseTextCallback<S>,
    linkcb: ParseLinkCallback<S>,
) -> Result<bool, Box<dyn Error>> {
    // Try and convert to UTF-8
    let ok = if let Ok(mapstr) = std::str::from_utf8(mmap) {
        // Converted OK - scan the content for $link() directives
        let mut pos = 0;

        loop {
            // Find the next $link() directive
            match &mapstr[pos..].find("$link(") {
                Some(idx) => {
                    // Isolate the text inside the directive
                    let idx = idx + pos;
                    let end = mapstr[idx..].find(')').unwrap() + idx;

                    // Call text callback with text before the directive
                    textcb(&mapstr[pos..idx], state)?;

                    // Extract the link and parameters
                    let linkspec = &mapstr[idx + 6..end];

                    // Split the link and parameters
                    let mut split = linkspec.split('|').collect::<Vec<_>>();

                    // Link is last
                    let link = split.pop().unwrap();

                    // Build a map of parameters
                    let parms = split
                        .iter()
                        .map(|parm| {
                            let mut split = parm.split('=');
                            (split.next().unwrap().to_string(), split.next().unwrap().to_string())
                        })
                        .collect::<HashMap<_, _>>();

                    // Call link callback
                    linkcb(link, &parms, state)?;

                    // Move past the directive
                    pos = end + 1;
                }
                None => {
                    // Call text callback with text after the last directive
                    textcb(&mapstr[pos..], state)?;

                    break;
                }
            }
        }

        true
    } else {
        // Failed to convert to UTF-8
        false
    };

    Ok(ok)
}

fn message(msg: &str, depth: usize) {
    let pad = String::from_utf8(vec![b' '; depth * 2]).unwrap();

    println!("{pad}{msg}");
}
