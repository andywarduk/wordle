#![warn(missing_docs)]

//! Wordle helper

use dictionary::{Dictionary, LetterNext};

/// Arguments for the wordle helper
pub struct SolverArgs<'a> {
    /// Current board
    pub board: &'a str,
    /// Letters in the wrong place
    pub unplaced: &'a Option<String>,
    /// Letters not in the solution
    pub unused: &'a str,
    /// Dictionary to use
    pub dictionary: &'a Dictionary,
    /// Debug output
    pub debug: bool,
}

struct SolverRec<'a> {
    args: SolverArgs<'a>,
    board_elems: Vec<Option<u8>>,
    unused: [bool; 26],
    unplaced: Vec<u8>,
}

/// Find words in the provides dictionary using the provided letters
pub fn find_words(args: SolverArgs) -> Vec<String> {
    let mut result = Vec::new();

    // Dictionary entry element numbers for each letter
    let board_elems = args
        .board
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                Some(c as u8 - b'A')
            } else {
                None
            }
        })
        .collect::<Vec<Option<u8>>>();

    // Build used array
    let mut unused = [false; 26];

    for c in args.unused.chars() {
        unused[(c as u8 - b'A') as usize] = true;
    }

    // Vector of chosen letter elements
    let mut chosen = Vec::with_capacity(5);

    // Vector of unplaced letters
    let unplaced = if let Some(unplaced) = args.unplaced {
        unplaced.chars().map(|c| c as u8 - b'A').collect()
    } else {
        Vec::new()
    };

    // Start search recursion
    let rec = SolverRec {
        args,
        board_elems,
        unused,
        unplaced,
    };

    find_words_rec(&rec, 0, 0, &mut chosen, &mut result);

    result
}

fn find_words_rec(
    rec: &SolverRec,
    letter_elem: usize,
    dict_elem: usize,
    chosen: &mut Vec<u8>,
    result: &mut Vec<String>,
) {
    // Got a letter in this position?
    if let Some(letter) = rec.board_elems[letter_elem] {
        find_words_rec_letter(rec, letter_elem, dict_elem, chosen, letter, result);
    } else {
        for letter in 0u8..26u8 {
            if !rec.unused[letter as usize] {
                find_words_rec_letter(rec, letter_elem, dict_elem, chosen, letter, result);
            }
        }
    }
}

fn find_words_rec_letter(
    rec: &SolverRec,
    letter_elem: usize,
    dict_elem: usize,
    chosen: &mut Vec<u8>,
    letter: u8,
    result: &mut Vec<String>,
) {
    chosen.push(letter);

    // Walk the dictionary
    let dict_elem = rec
        .args
        .dictionary
        .lookup_elem_letter_num(dict_elem, letter);

    if rec.args.debug {
        debug_lookup(chosen, &dict_elem);
    }

    // Recurse to next letter
    match dict_elem {
        LetterNext::Next(e) | LetterNext::EndNext(e) => {
            find_words_rec(rec, letter_elem + 1, e as usize, chosen, result);
        }
        LetterNext::End => {
            if letter_elem == 4 {
                // Check we have all unplaced letters in the word
                let mut valid = true;

                for c in &rec.unplaced {
                    if !chosen.contains(c) {
                        valid = false;
                        break;
                    }
                }

                if valid {
                    // Add to results
                    result.push(chosen_string(chosen));
                }
            }
        }
        _ => (),
    }

    // SAFETY: length always decreasing and always removing the pushed entry above
    unsafe {
        chosen.set_len(chosen.len() - 1);
    }
}

#[inline]
fn chosen_string(chosen: &[u8]) -> String {
    chosen
        .iter()
        .map(|e| (*e + b'A') as char)
        .collect::<String>()
}

#[cold]
fn debug_lookup(chosen: &[u8], dict_elem: &LetterNext) {
    let string = chosen_string(chosen);
    let indent = string.len();

    println!("{:indent$}{} ({:?})", "", string, dict_elem);
}

#[cfg(test)]
mod tests {
    use dictionary::{Dictionary, LetterNext};

    use super::*;

    #[test]
    fn size_checks() {
        assert_eq!(8, std::mem::size_of::<LetterNext>());
    }
}
