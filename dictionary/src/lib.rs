#![warn(missing_docs)]

//! Word list and loader functions

use std::fs::{read_link, symlink_metadata, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::time::Instant;

use flate2::bufread::GzDecoder;
use numformat::NumFormat;

/// Word end and next tree node indicators
#[derive(Copy, Clone, Debug)]
pub enum LetterNext {
    /// No word with this letter in this position
    None,
    /// Pointer to next tree node
    Next(u32),
    /// End of word indicator
    End,
    /// End of word and pointer to next tree node
    EndNext(u32),
}

/// Vector of next letters
type LetterVec = [LetterNext; 26];

/// Dictionary structure
pub struct Dictionary {
    words: usize,
    tree: Vec<LetterVec>,
}

impl Dictionary {
    /// Loads a dictionary from a file
    pub fn new_from_file(file: &str, size: WordSizeConstraint, verbose: bool) -> io::Result<Self> {
        let path_buf = PathBuf::from(file);

        if verbose {
            println!("Loading words from file {}", Self::file_spec(&path_buf)?);
        }

        // Create buf reader for the file
        Self::new_from_bufread(&mut BufReader::new(File::open(&path_buf)?), size, verbose)
    }

    /// Loads a dictionary from a string
    #[allow(dead_code)]
    pub fn new_from_string(
        string: &str,
        size: WordSizeConstraint,
        verbose: bool,
    ) -> io::Result<Self> {
        if verbose {
            println!("Loading words from string '{string}'");
        }

        Self::new_from_bufread(&mut BufReader::new(string.as_bytes()), size, verbose)
    }

    /// Loads a dictionary from a byte array
    #[allow(dead_code)]
    pub fn new_from_bytes(
        bytes: &[u8],
        size: WordSizeConstraint,
        verbose: bool,
    ) -> io::Result<Self> {
        if verbose {
            println!("Loading words from byte array (length {})", bytes.len());
        }

        Self::new_from_bufread(&mut BufReader::new(bytes), size, verbose)
    }

    /// Loads a dictionary from an entity implementing BufRead
    /// Handles gzip compressed buffers
    pub fn new_from_bufread(
        bufread: &mut dyn BufRead,
        size: WordSizeConstraint,
        verbose: bool,
    ) -> io::Result<Self> {
        // Get start time
        let start_time = Instant::now();

        // Fill the bufreader buffer
        let buf = bufread.fill_buf()?;

        // Check for gzip signature
        if buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b {
            // gzip compressed file
            if verbose {
                println!("Decompressing word list");
            }

            Self::new_from_bufread_internal(
                start_time,
                &mut BufReader::new(GzDecoder::new(bufread)),
                size,
                verbose,
            )
        } else {
            Self::new_from_bufread_internal(start_time, bufread, size, verbose)
        }
    }

    /// Loads a dictionary from an entity implementing BufRead
    fn new_from_bufread_internal(
        start_time: Instant,
        bufread: &mut dyn BufRead,
        size: WordSizeConstraint,
        verbose: bool,
    ) -> io::Result<Self> {
        let mut tree = Vec::new();

        let empty = [LetterNext::None; 26];

        let mut lines: usize = 0;
        let mut words: usize = 0;
        let mut too_short: usize = 0;
        let mut too_long: usize = 0;
        let mut wrong_case: usize = 0;

        tree.push(empty);

        // Iterate file lines
        for line in bufread.lines() {
            let line = line?;

            lines += 1;

            // Check length
            let length = line.len();

            if length > size.max {
                too_long += 1;
                continue;
            }

            if length < size.min {
                too_short += 1;
                continue;
            }

            // Make sure word consists of all lower case ascii characters
            if !Self::is_ascii_lower(&line) {
                wrong_case += 1;
                continue;
            }

            // Add this word to the tree
            words += 1;

            let mut cur_elem = 0;

            for (i, c) in line.chars().enumerate() {
                let letter = Self::lchar_to_elem(c);

                if i == length - 1 {
                    // Last character
                    tree[cur_elem][letter] = match tree[cur_elem][letter] {
                        LetterNext::None => LetterNext::End,
                        LetterNext::Next(n) => LetterNext::EndNext(n),
                        _ => panic!("Duplicate word {line}"),
                    }
                } else {
                    // Mid character
                    cur_elem = match tree[cur_elem][letter] {
                        LetterNext::None => {
                            tree.push(empty);
                            let e = tree.len() - 1;
                            tree[cur_elem][letter] = LetterNext::Next(e as u32);
                            e
                        }
                        LetterNext::End => {
                            tree.push(empty);
                            let e = tree.len() - 1;
                            tree[cur_elem][letter] = LetterNext::EndNext(e as u32);
                            e
                        }
                        LetterNext::Next(e) | LetterNext::EndNext(e) => e as usize,
                    };
                }
            }
        }

        let dictionary = Self { words, tree };

        if verbose {
            println!(
                "Dictionary read in {} seconds",
                start_time.elapsed().as_secs_f64().num_format_sigdig(2)
            );

            println!(
                "{} total words, ({} too short, {} too long, {} not all lower case)",
                lines.num_format(),
                too_short.num_format(),
                too_long.num_format(),
                wrong_case.num_format()
            );

            println!(
                "Dictionary words {}, tree nodes {} ({} bytes of {} allocated)",
                dictionary.word_count().num_format(),
                dictionary.tree_node_count().num_format(),
                dictionary.tree_mem_usage().num_format(),
                dictionary.tree_mem_alloc().num_format(),
            );
        }

        Ok(dictionary)
    }

    /// Returns the number of words stored in the dictionary
    pub fn word_count(&self) -> usize {
        self.words
    }

    /// Returns the size of the dictionary tree
    pub fn tree_node_count(&self) -> usize {
        self.tree.len()
    }

    /// Returns the used memory of the dictionary tree in bytes
    pub fn tree_mem_usage(&self) -> usize {
        self.tree_node_count() * std::mem::size_of::<LetterNext>()
    }

    /// Returns the allocated memory of the dictionary tree in bytes
    pub fn tree_mem_alloc(&self) -> usize {
        self.tree.capacity() * std::mem::size_of::<LetterNext>()
    }

    /// Looks up the letter number (0-25) in the dictionary tree node
    #[inline]
    pub fn lookup_elem_letter_num(&self, elem: usize, letter: u8) -> LetterNext {
        self.tree[elem][letter as usize]
    }

    #[inline]
    fn lchar_to_elem(c: char) -> usize {
        (c as u8 - b'a') as usize
    }

    #[inline]
    fn is_ascii_lower(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_lowercase())
    }

    fn file_spec(path: &PathBuf) -> io::Result<String> {
        let meta = symlink_metadata(path)?;

        if meta.is_symlink() {
            let target = read_link(path)?;

            Ok(format!(
                "{} -> {}",
                path.to_string_lossy(),
                Self::file_spec(&target)?
            ))
        } else {
            Ok(format!("{}", path.to_string_lossy()))
        }
    }
}

/// Word size constraints to use when loading a dictionary
pub struct WordSizeConstraint {
    min: usize,
    max: usize,
}

impl WordSizeConstraint {
    /// Sets the minimum length for a word
    pub fn set_min(&mut self, min: usize) {
        self.min = min;
    }

    /// Sets the maximum length for a word
    pub fn set_max(&mut self, max: usize) {
        self.max = max;
    }
}

impl Default for WordSizeConstraint {
    fn default() -> Self {
        Self {
            min: 0,
            max: usize::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    use super::*;

    fn gz_dict(string: &str) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(string.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn dict1() {
        // Create dictionary with one word in it "rust"
        let dictionary = Dictionary::new_from_string("rust", Default::default(), false).unwrap();

        test_dict1(dictionary)
    }

    #[test]
    fn dict1z() {
        // Create dictionary from compressed data with one word in it "rust"
        let dictionary =
            Dictionary::new_from_bytes(&gz_dict("rust"), Default::default(), false).unwrap();

        test_dict1(dictionary)
    }

    fn test_dict1(dictionary: Dictionary) {
        assert_eq!(dictionary.word_count(), 1);
        assert_eq!(dictionary.tree_node_count(), 4);
        assert_eq!(dictionary.tree_mem_usage(), 4 * 8);

        assert!(matches!(
            dictionary.lookup_elem_letter_num(0, b'R' - b'A'),
            LetterNext::Next(1)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(1, b'U' - b'A'),
            LetterNext::Next(2)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(2, b'S' - b'A'),
            LetterNext::Next(3)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(3, b'T' - b'A'),
            LetterNext::End
        ));
    }

    #[test]
    fn dict2() {
        // Create dictionary with two words, "rust" and "rusty"
        let dictionary =
            Dictionary::new_from_string("rust\nrusty", Default::default(), false).unwrap();

        test_dict2(dictionary);
    }

    #[test]
    fn dict2z() {
        // Create dictionary from compressed data with two words, "rust" and "rusty"
        let dictionary =
            Dictionary::new_from_bytes(&gz_dict("rust\nrusty"), Default::default(), false).unwrap();

        test_dict2(dictionary);
    }

    fn test_dict2(dictionary: Dictionary) {
        assert_eq!(dictionary.word_count(), 2);
        assert_eq!(dictionary.tree_node_count(), 5);
        assert_eq!(dictionary.tree_mem_usage(), 5 * 8);

        assert!(matches!(
            dictionary.lookup_elem_letter_num(0, b'R' - b'A'),
            LetterNext::Next(1)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(1, b'U' - b'A'),
            LetterNext::Next(2)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(2, b'S' - b'A'),
            LetterNext::Next(3)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(3, b'T' - b'A'),
            LetterNext::EndNext(4)
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(4, b'Y' - b'A'),
            LetterNext::End
        ));
    }
}
