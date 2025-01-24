use std::{collections::HashMap, error::Error, fs::File, io::Write, os::unix::ffi::OsStrExt, path::PathBuf};

use memmap2::Mmap;

use crate::{Config, message, openout, parse_file, process_input_file};

/// State for processing to a multiple asset directory
struct MultiState<'a> {
    file: File,
    infile: &'a PathBuf,
    config: &'a Config<()>,
    depth: usize,
}

pub fn multi_process(config: &Config<()>, infile: PathBuf, mmap: &Mmap, depth: usize) -> Result<(), Box<dyn Error>> {
    // Build output file path
    let outfile = config.outroot.join(infile.file_name().unwrap());

    message(&format!("{} -> {}", infile.display(), outfile.display()), depth);

    // Create state for processing
    let mut state = MultiState {
        file: openout(&outfile)?,
        infile: &infile,
        config,
        depth,
    };

    // Try and parse the file
    if !parse_file(mmap, &mut state, multi_text, multi_link)? {
        // File is binary, just write it out
        message(&format!("{} is binary", infile.display()), depth);
        state.file.write_all(mmap)?;
    }

    Ok(())
}

fn multi_text(text: &str, state: &mut MultiState) -> Result<(), Box<dyn Error>> {
    // Write text to output file
    state.file.write_all(text.as_bytes())?;

    Ok(())
}

fn multi_link(link: &str, parms: &HashMap<String, String>, state: &mut MultiState) -> Result<(), Box<dyn Error>> {
    message(&format!("Processing link: {}", link), state.depth);

    // Build link file path
    let linkfile = state.infile.parent().unwrap().join(link);

    // Write URL prefix if provided
    if let Some(prefix) = parms.get("prefix") {
        state.file.write_all(prefix.as_bytes())?;
    }

    // Write link name
    state.file.write_all(linkfile.file_name().unwrap().as_bytes())?;

    // Process the linked file
    process_input_file(state.config, linkfile, state.depth + 1)?;

    Ok(())
}
