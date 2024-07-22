#![warn(missing_docs)]

//! Wordle helper

use std::cmp;
use std::collections::HashMap;

use dictionary::{Dictionary, LetterNext, NEXT_NONE};

/// Number of columns on the board
pub const BOARD_COLS: usize = 5;

/// Number of rows on the board
pub const BOARD_ROWS: usize = 6;

/// Board element
#[derive(Clone, Copy, Debug, Hash)]
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
    contains: HashMap<u8, Contains>,
    unused: [bool; 26],
}

enum Contains {
    AtLeast(u8),
    Exactly(u8),
}

/// Find words in the provides dictionary using the provided letters
pub fn find_words(args: SolverArgs) -> Vec<LetterNext> {
    let mut result = Vec::new();

    // Correct letters
    let mut correct = [None; BOARD_COLS];

    // Incorrect letters
    let mut incorrect = [[false; 26]; BOARD_COLS];
    let mut contains = HashMap::new();

    // Unused letters
    let mut unused = [false; 26];

    // Lambda to add a letter to the row contains list
    let add_rowcontains = |rowcontains: &mut HashMap<u8, u8>, c| {
        rowcontains
            .entry(Dictionary::uchar_to_u8(c))
            .and_modify(|n| *n += 1)
            .or_insert(1);
    };

    // Iterate each row
    for row in args.board {
        let mut rowcontains = HashMap::new();

        // Iterate each letter in the row
        for (elem, col) in row.iter().enumerate() {
            match col {
                BoardElem::Gray(c) => unused[Dictionary::uchar_to_usize(*c)] = true,
                BoardElem::Yellow(c) => {
                    incorrect[elem][Dictionary::uchar_to_usize(*c)] = true;
                    add_rowcontains(&mut rowcontains, *c);
                }
                BoardElem::Green(c) => {
                    correct[elem] = Some(Dictionary::uchar_to_u8(*c));
                    add_rowcontains(&mut rowcontains, *c);
                }
                _ => (),
            }
        }

        // Build contains from rowcontains
        for (letter, count) in rowcontains.into_iter() {
            contains
                .entry(letter)
                .and_modify(|e| {
                    *e = match *e {
                        Contains::AtLeast(n) => Contains::AtLeast(cmp::max(n, count)),
                        Contains::Exactly(_) => panic!("Attempt to update Contains::Exactly"),
                    }
                })
                .or_insert(Contains::AtLeast(count));
        }
    }

    // Letter can be in contains and unused if guessed multiple times and the word contains fewer
    unused
        .iter_mut()
        .enumerate()
        .filter(|(_, unused)| **unused)
        .for_each(|(i, unused)| {
            if let Some(contains) = contains.get_mut(&(i as u8)) {
                // Set unused to false
                *unused = false;

                // Convert Contains AtLeast to Exactly
                *contains = match *contains {
                    Contains::AtLeast(n) => Contains::Exactly(n),
                    Contains::Exactly(_) => panic!("Already Contains::Exactly"),
                }
            }
        });

    // Start search recursion
    let rec = SolverRec {
        args,
        correct,
        incorrect,
        contains,
        unused,
    };

    find_words_rec(&rec, 0, 0, &mut result);

    result
}

fn find_words_rec(
    rec: &SolverRec,
    letter_elem: usize,
    dict_elem: usize,
    result: &mut Vec<LetterNext>,
) {
    // Got a letter in this position?
    if let Some(letter) = rec.correct[letter_elem] {
        find_words_rec_letter(rec, letter_elem, dict_elem, letter, result);
    } else {
        for letter in 0u8..26u8 {
            if !rec.unused[letter as usize] && !rec.incorrect[letter_elem][letter as usize] {
                find_words_rec_letter(rec, letter_elem, dict_elem, letter, result);
            }
        }
    }
}

fn find_words_rec_letter(
    rec: &SolverRec,
    letter_elem: usize,
    dict_elem: usize,
    letter: u8,
    result: &mut Vec<LetterNext>,
) {
    // Walk the dictionary
    let dict_elem = rec
        .args
        .dictionary
        .lookup_elem_letter_num(dict_elem, letter);

    if rec.args.debug {
        debug_lookup(rec.args.dictionary, dict_elem);
    }

    // Recurse to next letter
    if dict_elem != NEXT_NONE {
        if letter_elem == BOARD_COLS - 1 {
            // Check we have all unplaced letters in the word
            let mut valid = true;

            for (c, contains) in &rec.contains {
                let (count, exact) = match contains {
                    Contains::AtLeast(n) => (n, false),
                    Contains::Exactly(n) => (n, true),
                };

                if !rec
                    .args
                    .dictionary
                    .word_contains(dict_elem as usize, *c, *count, exact)
                {
                    valid = false;
                    break;
                }
            }

            if valid {
                // Add to results
                result.push(dict_elem);
            }
        } else {
            find_words_rec(rec, letter_elem + 1, dict_elem as usize, result);
        }
    }
}

#[cold]
fn debug_lookup(dictionary: &Dictionary, dict_elem: LetterNext) {
    let string = dictionary.get_word(dict_elem as usize);
    let indent = string.len();

    println!("{:indent$}{} ({:?})", "", string, dict_elem);
}
