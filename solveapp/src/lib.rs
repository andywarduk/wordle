use dictionary::{Dictionary, LetterNext};
use solver::{find_words, SolverArgs};
pub use solver::{BoardElem, BOARD_COLS, BOARD_ROWS};

/// App holds the state of the application
pub struct SolveApp {
    /// Current board
    pub board: [[BoardElem; BOARD_COLS]; BOARD_ROWS],
    /// Current row
    row: usize,
    /// Current column
    col: usize,
    /// Dictionary
    dictionary: Dictionary,
    /// Words
    words: Option<Vec<LetterNext>>,
}

impl SolveApp {
    /// Creates the application
    pub fn new(dictionary: Dictionary) -> Self {
        Self {
            board: [[BoardElem::Empty; BOARD_COLS]; BOARD_ROWS],
            row: 0,
            col: 0,
            dictionary,
            words: None,
        }
    }

    /// Add a letter to the board
    pub fn add(&mut self, c: char) -> bool {
        // Any space left on the board?
        if self.row >= BOARD_ROWS {
            return false;
        }

        // Set board element to the letter
        // Search through board rows for matching letter in this column and copy if found
        self.board[self.row][self.col] = self
                    .board
                    .iter()
                    .find(|row| matches!(row[self.col], BoardElem::Green(oc) | BoardElem::Yellow(oc) if oc == c))
                    .map(|row| row[self.col])
                    .unwrap_or(BoardElem::Gray(c));

        // Move to the next board element
        self.col += 1;

        if self.col == BOARD_COLS {
            self.col = 0;
            self.row += 1;
        }

        true
    }

    /// Remove last letter from the board
    pub fn remove(&mut self) -> bool {
        // Any letters on this row?
        if self.col > 0 {
            // Yes - remove it
            self.col -= 1;
        } else if self.row > 0 {
            // No - move to last row
            self.row -= 1;
            self.col = BOARD_COLS - 1;
        } else {
            // No, and no previous row to move to
            return false;
        }

        // Set board element to empty
        self.board[self.row][self.col] = BoardElem::Empty;

        true
    }

    /// Toggle a column on the current row
    pub fn toggle_col(&mut self, colnum: usize) -> bool {
        let rownum = if colnum >= self.col {
            if self.row > 0 {
                Some(self.row - 1)
            } else {
                None
            }
        } else {
            Some(self.row)
        };

        if colnum < BOARD_COLS {
            if let Some(rownum) = rownum {
                self.toggle(rownum, colnum)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Toggle a board cell between Gray, Yellow and Green
    pub fn toggle(&mut self, rownum: usize, colnum: usize) -> bool {
        // Get the character we're toggling
        if let Some(c) = match self.board[rownum][colnum] {
            BoardElem::Gray(c) | BoardElem::Yellow(c) | BoardElem::Green(c) => Some(c),
            BoardElem::Empty => None,
        } {
            // Work out what to convert the board element to
            let new = match self.board[rownum][colnum] {
                BoardElem::Gray(c) => BoardElem::Yellow(c),
                BoardElem::Yellow(c) => {
                    if self
                        .board
                        .iter()
                        .any(|row| matches!(row[colnum], BoardElem::Green(_)))
                    {
                        BoardElem::Gray(c)
                    } else {
                        BoardElem::Green(c)
                    }
                }
                BoardElem::Green(c) => BoardElem::Gray(c),
                BoardElem::Empty => unreachable!(),
            };

            // Set new board element value on all rows where applicable
            for (rn, row) in self.board.iter_mut().enumerate() {
                match row[colnum] {
                    BoardElem::Gray(oc) | BoardElem::Yellow(oc) | BoardElem::Green(oc)
                        if oc == c =>
                    {
                        // If the letter appears elsewhere on the row, don't set automatically
                        if rn == rownum
                            || !row.iter().enumerate().any(|(cn, elem)| {
                                cn != colnum
                                    && matches!(*elem, BoardElem::Yellow(oc) | BoardElem::Green(oc) if oc == c)
                            })
                        {
                            row[colnum] = new;
                        }
                    }
                    _ => (),
                }
            }

            true
        } else {
            false
        }
    }

    /// Calculate valid words
    pub fn calculate(&mut self) {
        // Wait for at least one complete row
        if self.row > 0 {
            // Create solver arguments
            let args = SolverArgs {
                board: &self.board,
                dictionary: &self.dictionary,
                debug: false,
            };

            // Save the word list
            self.words = Some(find_words(args));
        } else {
            // Word list should be empty
            self.words = None;
        }
    }

    /// Get reference to the words list
    pub fn word_count(&self) -> usize {
        match &self.words {
            Some(words) => words.len(),
            _ => 0,
        }
    }

    /// Get word list word
    pub fn get_word(&self, elem: usize) -> Option<String> {
        if let Some(words) = &self.words {
            if elem < words.len() {
                Some(self.dictionary.get_word(words[elem] as usize))
            } else {
                None
            }
        } else {
            None
        }
    }
}
