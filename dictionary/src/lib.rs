#![warn(missing_docs)]

//! Word list and loader functions

use std::fs::{read_link, symlink_metadata, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::PathBuf;

use flate2::bufread::GzDecoder;

/// Word next tree node
pub type LetterNext = u16;

/// No next letter
pub const NEXT_NONE: LetterNext = LetterNext::MAX;

/// Vector of next letters
struct LetterEnt {
    letter_vec: [LetterNext; 26],
    parent: LetterNext,
    letter: u8,
}

impl LetterEnt {
    fn new(letter: u8, parent: LetterNext) -> Self {
        Self {
            letter_vec: [NEXT_NONE; 26],
            letter,
            parent,
        }
    }
}

/// Dictionary structure
pub struct Dictionary {
    words: usize,
    tree: Vec<LetterEnt>,
}

impl Dictionary {
    /// Loads a dictionary from a file
    pub fn new_from_file(file: &str, verbose: bool) -> io::Result<Self> {
        let path_buf = PathBuf::from(file);

        if verbose {
            println!("Loading words from file {}", Self::file_spec(&path_buf)?);
        }

        // Create buf reader for the file
        Self::new_from_bufread(&mut BufReader::new(File::open(&path_buf)?), verbose)
    }

    /// Loads a dictionary from a string
    #[allow(dead_code)]
    pub fn new_from_string(string: &str, verbose: bool) -> io::Result<Self> {
        if verbose {
            println!("Loading words from string '{string}'");
        }

        Self::new_from_bufread(&mut BufReader::new(string.as_bytes()), verbose)
    }

    /// Loads a dictionary from a byte array
    #[allow(dead_code)]
    pub fn new_from_bytes(bytes: &[u8], verbose: bool) -> io::Result<Self> {
        if verbose {
            println!("Loading words from byte array (length {})", bytes.len());
        }

        Self::new_from_bufread(&mut BufReader::new(bytes), verbose)
    }

    /// Loads a dictionary from an entity implementing BufRead
    /// Handles gzip compressed buffers
    pub fn new_from_bufread(bufread: &mut dyn BufRead, verbose: bool) -> io::Result<Self> {
        // Fill the bufreader buffer
        let buf = bufread.fill_buf()?;

        // Check for gzip signature
        if buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b {
            // gzip compressed file
            if verbose {
                println!("Decompressing word list");
            }

            Self::new_from_bufread_internal(&mut BufReader::new(GzDecoder::new(bufread)), verbose)
        } else {
            Self::new_from_bufread_internal(bufread, verbose)
        }
    }

    /// Loads a dictionary from an entity implementing BufRead
    fn new_from_bufread_internal(bufread: &mut dyn BufRead, verbose: bool) -> io::Result<Self> {
        let mut tree = Vec::new();

        let mut lines: usize = 0;
        let mut words: usize = 0;
        let mut wrong_length: usize = 0;
        let mut wrong_case: usize = 0;

        tree.push(LetterEnt::new(0, NEXT_NONE));

        // Iterate file lines
        for line in bufread.lines() {
            let line = line?;

            lines += 1;

            // Check length
            let length = line.len();

            if length != 5 {
                wrong_length += 1;
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

            for c in line.chars() {
                let letter = Self::lchar_to_usize(c);

                cur_elem = match tree[cur_elem].letter_vec[letter] {
                    NEXT_NONE => {
                        tree.push(LetterEnt::new(letter as u8, cur_elem as LetterNext));
                        let e = tree.len() - 1;
                        tree[cur_elem].letter_vec[letter] = e as LetterNext;
                        e
                    }
                    e => e as usize,
                };
            }
        }

        let dictionary = Self { words, tree };

        if verbose {
            println!(
                "{} total words, ({} wrong length, {} not all lower case)",
                lines, wrong_length, wrong_case
            );

            println!(
                "Dictionary words {}, tree nodes {} ({} bytes of {} allocated)",
                dictionary.word_count(),
                dictionary.tree_node_count(),
                dictionary.tree_mem_usage(),
                dictionary.tree_mem_alloc(),
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
        self.tree_node_count() * std::mem::size_of::<LetterEnt>()
    }

    /// Returns the allocated memory of the dictionary tree in bytes
    pub fn tree_mem_alloc(&self) -> usize {
        self.tree.capacity() * std::mem::size_of::<LetterEnt>()
    }

    /// Looks up the letter number (0-25) in the dictionary tree node
    #[inline]
    pub fn lookup_elem_letter_num(&self, elem: usize, letter: u8) -> LetterNext {
        self.tree[elem].letter_vec[letter as usize]
    }

    /// Returns the word for a dictionary element
    #[inline]
    pub fn get_word(&self, elem: usize) -> String {
        let mut result = String::with_capacity(5);

        self.get_word_rec(elem, &mut result);

        result
    }

    #[inline]
    fn get_word_rec(&self, elem: usize, result: &mut String) {
        let next_elem = self.tree[elem].parent as usize;

        if next_elem != 0 {
            self.get_word_rec(next_elem, result);
        }

        result.push((self.tree[elem].letter + b'A') as char)
    }

    /// Tests if a word contains a given letter
    pub fn word_contains(&self, mut elem: usize, letter: u8) -> bool {
        let mut result: bool = false;

        while elem != 0 {
            if self.tree[elem].letter == letter {
                result = true;
                break;
            }

            elem = self.tree[elem].parent as usize;
        }

        result
    }

    /// Converts a lower case character to usize
    #[inline]
    pub fn lchar_to_usize(c: char) -> usize {
        (c as u8 - b'a') as usize
    }

    /// Converts an upper case character to usize
    #[inline]
    pub fn uchar_to_usize(c: char) -> usize {
        (c as u8 - b'A') as usize
    }

    /// Converts an upper case character to u8
    #[inline]
    pub fn uchar_to_u8(c: char) -> u8 {
        c as u8 - b'A'
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
        // Create dictionary with one word in it "rusty"
        let dictionary = Dictionary::new_from_string("rusty", false).unwrap();

        test_dict1(dictionary)
    }

    #[test]
    fn dict1z() {
        // Create dictionary from compressed data with one word in it "rusty"
        let dictionary = Dictionary::new_from_bytes(&gz_dict("rusty"), false).unwrap();

        test_dict1(dictionary)
    }

    fn test_dict1(dictionary: Dictionary) {
        assert_eq!(dictionary.word_count(), 1);
        assert_eq!(dictionary.tree_node_count(), 6);
        assert_eq!(dictionary.tree_mem_usage(), 6 * 56);

        assert!(matches!(
            dictionary.lookup_elem_letter_num(0, Dictionary::uchar_to_u8('R')),
            1
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(1, Dictionary::uchar_to_u8('U')),
            2
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(2, Dictionary::uchar_to_u8('S')),
            3
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(3, Dictionary::uchar_to_u8('T')),
            4
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(4, Dictionary::uchar_to_u8('Y')),
            5
        ));
    }

    #[test]
    fn dict2() {
        // Create dictionary with two words, "rusts" and "rusty"
        let dictionary = Dictionary::new_from_string("rusts\nrusty", false).unwrap();

        test_dict2(dictionary);
    }

    #[test]
    fn dict2z() {
        // Create dictionary from compressed data with two words, "rusts" and "rusty"
        let dictionary = Dictionary::new_from_bytes(&gz_dict("rusts\nrusty"), false).unwrap();

        test_dict2(dictionary);
    }

    fn test_dict2(dictionary: Dictionary) {
        assert_eq!(dictionary.word_count(), 2);
        assert_eq!(dictionary.tree_node_count(), 7);
        assert_eq!(dictionary.tree_mem_usage(), 7 * 56);

        assert!(matches!(
            dictionary.lookup_elem_letter_num(0, Dictionary::uchar_to_u8('R')),
            1
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(1, Dictionary::uchar_to_u8('U')),
            2
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(2, Dictionary::uchar_to_u8('S')),
            3
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(3, Dictionary::uchar_to_u8('T')),
            4
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(4, Dictionary::uchar_to_u8('Y')),
            6
        ));
        assert!(matches!(
            dictionary.lookup_elem_letter_num(4, Dictionary::uchar_to_u8('S')),
            5
        ));
    }
}
