#![warn(missing_docs)]

//! Wordle helper

use dictionary::{Dictionary, LetterNext};

/// Number of columns on the board
pub const BOARD_COLS: usize = 5;

/// Number of rows on the board
pub const BOARD_ROWS: usize = 6;

/// Board element
#[derive(Clone, Copy, Debug)]
pub enum BoardElem {
    /// Empty board space
    Empty,
    /// Gray board space (letter not in solution)
    Gray(char),
    /// Yellow board space (letter in solution but in the wrong place)
    Yellow(char),
    /// Green board space (letter in solution and in the correct place)
    Green(char),
}

/// Arguments for the wordle helper
pub struct SolverArgs<'a> {
    /// Current board
    pub board: &'a [[BoardElem; BOARD_COLS]; BOARD_ROWS],
    /// Dictionary to use
    pub dictionary: &'a Dictionary,
    /// Debug output
    pub debug: bool,
}

struct SolverRec<'a> {
    args: SolverArgs<'a>,
    correct: [Option<u8>; BOARD_COLS],
    incorrect: [[bool; 26]; BOARD_COLS],
    contains: Vec<u8>,
    unused: [bool; 26],
}

/// Find words in the provides dictionary using the provided letters
pub fn find_words(args: SolverArgs) -> Vec<String> {
    let mut result = Vec::new();

    // Correct letters
    let mut correct = [None; BOARD_COLS];

    // Incorrect letters
    let mut incorrect = [[false; 26]; BOARD_COLS];
    let mut contains = Vec::new();

    // Unused letters
    let mut unused = [false; 26];

    for row in args.board {
        for (elem, col) in row.iter().enumerate() {
            match col {
                BoardElem::Gray(c) => unused[(*c as u8 - b'A') as usize] = true,
                BoardElem::Yellow(c) => {
                    incorrect[elem][(*c as u8 - b'A') as usize] = true;
                    contains.push(*c as u8 - b'A');
                }
                BoardElem::Green(c) => correct[elem] = Some(*c as u8 - b'A'),
                _ => (),
            }
        }
    }

    // Vector of chosen letter elements
    let mut chosen = Vec::with_capacity(BOARD_COLS);

    // Start search recursion
    let rec = SolverRec {
        args,
        correct,
        incorrect,
        contains,
        unused,
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
    if let Some(letter) = rec.correct[letter_elem] {
        find_words_rec_letter(rec, letter_elem, dict_elem, chosen, letter, result);
    } else {
        for letter in 0u8..26u8 {
            if !rec.unused[letter as usize] && !rec.incorrect[letter_elem][letter as usize] {
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
            if letter_elem == BOARD_COLS - 1 {
                // Check we have all unplaced letters in the word
                let mut valid = true;

                for c in &rec.contains {
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
