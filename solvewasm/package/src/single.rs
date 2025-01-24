use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose};
use memmap2::Mmap;

use crate::{Config, message, openout, parse_file, process_input_file};

/// State for processing to a single file
struct SingleState<'a> {
    file: File,
    infile: &'a PathBuf,
    config: &'a Config<()>,
    depth: usize,
}

/// Top level file callback
pub fn single_process(config: &Config<()>, infile: PathBuf, mmap: &Mmap, depth: usize) -> Result<(), Box<dyn Error>> {
    // Build output file path
    let outfile = config.outroot.join(infile.file_name().unwrap());

    message(&format!("{} -> {}", infile.display(), outfile.display()), depth);

    // Create state for processing
    let mut state = SingleState {
        file: openout(&outfile)?,
        infile: &infile,
        config,
        depth,
    };

    // Try and parse the file
    if !parse_file(mmap, &mut state, single_text, single_link)? {
        // File is binary, just write it out
        message(&format!("{} is binary", infile.display()), depth);
        state.file.write_all(mmap)?;
    }

    Ok(())
}

fn single_text(text: &str, state: &mut SingleState) -> Result<(), Box<dyn Error>> {
    // Write text to output file
    state.file.write_all(text.as_bytes())?;

    Ok(())
}

fn single_link(link: &str, _parms: &HashMap<String, String>, state: &mut SingleState) -> Result<(), Box<dyn Error>> {
    message(&format!("  Processing link: {}", link), state.depth);

    // Build path to linked file
    let linkfile = state.infile.parent().unwrap().join(link);

    // Convert this file to base64 data URL
    let dataurl = convert_to_data_url(state.config, &linkfile, state.depth + 1)?;

    // Write data URL to output file
    state.file.write_all(dataurl.as_bytes())?;

    Ok(())
}

fn convert_to_data_url(config: &Config<()>, file: &Path, depth: usize) -> Result<String, Box<dyn Error>> {
    // Create configuration for data URL processing
    let data_url_config: Config<String> = Config {
        outroot: config.outroot.clone(),
        callback: convert_to_data_url_handler,
    };

    // Convert the link to a data URL
    let dataurl = process_input_file(&data_url_config, file.to_path_buf(), depth)?;

    Ok(dataurl)
}

/// State for recursive data URL processing
struct B64State<'a> {
    config: &'a Config<String>,
    infile: &'a PathBuf,
    content: String,
    depth: usize,
}

fn convert_to_data_url_handler(
    config: &Config<String>,
    infile: PathBuf,
    mmap: &Mmap,
    depth: usize,
) -> Result<String, Box<dyn Error>> {
    // Create state
    let mut state = B64State {
        config,
        infile: &infile,
        content: String::new(),
        depth,
    };

    // Try and parse the file
    let content = if parse_file(mmap, &mut state, convert_to_data_url_text, convert_to_data_url_link)? {
        // Ok - return the converted content
        state.content.as_bytes().to_vec()
    } else {
        // File is binary, just return the mmap
        mmap.to_vec()
    };

    // Work out the MIME type for the link
    let mime_type = match infile.extension() {
        Some(ext) => match ext.to_str() {
            Some("htm") => "text/html",
            Some("css") => "text/css",
            Some("js") => "text/javascript",
            Some("wasm") => "application/wasm",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            _ => "application/octet-stream",
        },
        None => "application/octet-stream",
    };

    // Build the data URL
    let dataurl = format!("data:{mime_type};base64,{}", general_purpose::STANDARD.encode(content));

    Ok(dataurl)
}

fn convert_to_data_url_text(text: &str, state: &mut B64State) -> Result<(), Box<dyn Error>> {
    // Append text to content
    state.content.push_str(text);

    Ok(())
}

fn convert_to_data_url_link(
    link: &str,
    _parms: &HashMap<String, String>,
    state: &mut B64State,
) -> Result<(), Box<dyn Error>> {
    message(&format!("  Processing link: {}", link), state.depth);

    // Build path to linked file
    let linkfile = state.infile.parent().unwrap().join(link);

    // Convert this file to base64 data URL
    let dataurl = process_input_file(state.config, linkfile.clone(), state.depth + 1)?;

    // Append data URL to content
    state.content.push_str(&dataurl);

    Ok(())
}
