use std::io;

use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use dictionary::{Dictionary, LetterNext};
use solver::{find_words, BoardElem, SolverArgs, BOARD_COLS, BOARD_ROWS};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use tui::{Frame, Terminal};

/// App holds the state of the application
pub struct App {
    /// Current board
    board: [[BoardElem; BOARD_COLS]; BOARD_ROWS],
    /// Current row
    row: usize,
    /// Current column
    col: usize,
    /// Board rectange
    board_rect: Option<Rect>,
    /// Words rectange
    words_rect: Option<Rect>,
    /// Dictionary
    dictionary: Dictionary,
    /// Words
    words: Option<Vec<LetterNext>>,
}

impl App {
    /// Spacing between board table cells
    const CELL_SPACING: u16 = 1;

    /// Board cell draw width
    const CELL_WIDTH: u16 = 5;
    /// Extra X dimension spacing
    const CELL_XSPACE: u16 = 1;
    /// Total width of a board cell
    const CELL_XTOTAL: u16 = Self::CELL_WIDTH + Self::CELL_XSPACE + Self::CELL_SPACING;

    /// Board cell draw height
    const CELL_HEIGHT: u16 = 3;
    /// Extra Y dimension spacing
    const CELL_YSPACE: u16 = 0;
    /// Total height of a board cell
    const CELL_YTOTAL: u16 = Self::CELL_HEIGHT + Self::CELL_YSPACE + Self::CELL_SPACING;

    /// Usage instructions
    const INSTRUCTIONS: &str = r#"
Wordle Solver
    
Fill the board on the left by pressing letter keys.

The colour of each letter can be toggled by clicking with the mouse or with the keys 1-5.

Press Escape to exit"#;

    /// Creates the application
    pub fn new(dictionary: Dictionary) -> Self {
        App {
            board: [[BoardElem::Empty; BOARD_COLS]; BOARD_ROWS],
            row: 0,
            col: 0,
            board_rect: None,
            words_rect: None,
            dictionary,
            words: None,
        }
    }

    /// Runs the application
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let mut render = true;
        let mut calculate = true;

        loop {
            // Need to recalculate?
            if calculate {
                self.calculate();

                calculate = false;
                render = true;
            }

            // Need to render?
            if render {
                self.render(terminal)?;
                render = false;
            }

            // Get the next event
            let Ok(event) = event::read() else { continue };

            // Process the event
            match event {
                Event::Resize(..) => {
                    // Window is being resized
                    render = true;
                }
                Event::Key(event) => match event.code {
                    // Keyboard event
                    KeyCode::Esc => {
                        // Escape pressed
                        break Ok(());
                    }
                    KeyCode::Char(c) if c.is_ascii_uppercase() => {
                        // Upper case character
                        if self.add(c) {
                            calculate = true;
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() => {
                        // Lower case character
                        if self.add(c.to_ascii_uppercase()) {
                            calculate = true;
                        }
                    }
                    KeyCode::Char(c) if ('1'..='5').contains(&c) => {
                        // 1 to 5 pressed
                        let col = (c as u8 - b'1') as usize;

                        let row = if col >= self.col {
                            if self.row > 0 {
                                Some(self.row - 1)
                            } else {
                                None
                            }
                        } else {
                            Some(self.row)
                        };

                        if let Some(row) = row {
                            if self.toggle(row, col) {
                                calculate = true;
                            }
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        // Backspace / delete pressed
                        if self.remove() {
                            calculate = true;
                        }
                    }
                    _ => (),
                },
                Event::Mouse(event) => {
                    // Mouse event
                    if let MouseEventKind::Down(event::MouseButton::Left) = event.kind {
                        // Mouse left click - check for board hit
                        if let Some((row, col)) = self.board_hit(event.row, event.column) {
                            // Try and toggle the board element
                            if self.toggle(row, col) {
                                calculate = true;
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    /// Renders the next frame
    fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| {
            // Split the terminal in to two horizontal sections
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(
                            (BOARD_COLS as u16 * Self::CELL_XTOTAL)
                                - (Self::CELL_XSPACE + Self::CELL_SPACING)
                                + 2,
                        ),
                        Constraint::Min(BOARD_COLS as u16),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Save rectangles
            self.board_rect = Some(chunks[0]);
            self.words_rect = Some(chunks[1]);

            // Draw the board in the left hand section
            self.board_table(f);

            if self.words.is_some() {
                // Draw the word list in the right hand section
                self.words_table(f);
            } else {
                // Draw the instructions in the right hand section
                f.render_widget(
                    Paragraph::new(Text::styled(
                        Self::INSTRUCTIONS,
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("Instructions")),
                    self.words_rect.unwrap(),
                )
            };
        })?;

        Ok(())
    }

    /// Draws the board table
    fn board_table<B: Backend>(&self, f: &mut Frame<B>) {
        // Build board table contents
        let content = self
            .board
            .iter()
            .map(|row| {
                // Build board table row
                Row::new(
                    row.iter()
                        .map(|col| match col {
                            BoardElem::Empty => Self::board_cell(' ', Color::DarkGray),
                            BoardElem::Gray(c) => Self::board_cell(*c, Color::DarkGray),
                            BoardElem::Yellow(c) => Self::board_cell(*c, Color::Yellow),
                            BoardElem::Green(c) => Self::board_cell(*c, Color::Green),
                        })
                        .collect::<Vec<Cell>>(),
                )
                .height(4)
            })
            .collect::<Vec<Row>>();

        // Create the board table
        let table = Table::new(content)
            .widths(&[Constraint::Length(Self::CELL_WIDTH + Self::CELL_XSPACE); BOARD_COLS])
            .column_spacing(Self::CELL_SPACING)
            .block(Block::default().borders(Borders::ALL).title("Board"));

        // Render the table
        f.render_widget(table, self.board_rect.unwrap());
    }

    /// Draws a single board cell
    fn board_cell<'b>(c: char, colour: Color) -> Cell<'b> {
        Cell::from(Text::styled(
            format!("     \n  {}  \n     ", c),
            Style::default().bg(colour).add_modifier(Modifier::BOLD),
        ))
    }

    /// Tests if a board cell has been hit
    fn board_hit(&self, row: u16, col: u16) -> Option<(usize, usize)> {
        let mut result = None;

        // Make sure we have a Rect
        if let Some(board_rect) = self.board_rect {
            // Make sure the position is inside the rectangle
            if row > board_rect.top() && col > board_rect.left() {
                // Work out the hit element and offset within the element
                let col_elem = (col - (board_rect.left() + 1)) / Self::CELL_XTOTAL;
                let col_pos = (col - (board_rect.left() + 1)) % Self::CELL_XTOTAL;
                let row_elem = (row - (board_rect.top() + 1)) / Self::CELL_YTOTAL;
                let row_pos = (row - (board_rect.top() + 1)) % Self::CELL_YTOTAL;

                // Make sure the click is inside the drawn element
                if col_elem < BOARD_COLS as u16
                    && row_elem < BOARD_ROWS as u16
                    && col_pos < Self::CELL_WIDTH
                    && row_pos < Self::CELL_HEIGHT
                {
                    // Got a hit
                    result = Some((row_elem as usize, col_elem as usize))
                }
            }
        }

        result
    }

    /// Draw the words table
    fn words_table<B: Backend>(&self, f: &mut Frame<B>) {
        if let Some(rect) = self.words_rect {
            let words = &self.words.as_ref().unwrap();

            // Calculate the number of rows and columns
            let rows = rect.height as usize - 2;
            let cols = (rect.width as usize - 1) / (BOARD_COLS + 1);

            // Create spans
            let spans = (0..rows)
                .map(|row| {
                    Spans::from(Span::styled(
                        (0..cols).fold(String::new(), |mut line, col| {
                            let elem = (col * rows) + row;

                            if elem < words.len() {
                                if col > 0 {
                                    line.push(' ');
                                }
                                line.push_str(&self.dictionary.get_word(words[elem] as usize));
                            }

                            line
                        }),
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                })
                .collect::<Vec<_>>();

            // Create text content
            let content = Text::from(spans);

            let para = Paragraph::new(content).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Words ({} found)", words.len())),
            );

            f.render_widget(para, rect);
        }
    }

    /// Add a letter to the board
    fn add(&mut self, c: char) -> bool {
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
    fn remove(&mut self) -> bool {
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

    /// Toggle a board cell between Gray, Yellow and Green
    fn toggle(&mut self, row: usize, col: usize) -> bool {
        // Get the character we're toggling
        if let Some(c) = match self.board[row][col] {
            BoardElem::Gray(c) | BoardElem::Yellow(c) | BoardElem::Green(c) => Some(c),
            BoardElem::Empty => None,
        } {
            // Work out what to convert the board element to
            let new = match self.board[row][col] {
                BoardElem::Gray(c) => BoardElem::Yellow(c),
                BoardElem::Yellow(c) => {
                    if self
                        .board
                        .iter()
                        .any(|row| matches!(row[col], BoardElem::Green(_)))
                    {
                        BoardElem::Gray(c)
                    } else {
                        BoardElem::Green(c)
                    }
                }
                BoardElem::Green(c) => BoardElem::Gray(c),
                BoardElem::Empty => unreachable!(),
            };

            // Set new board element value
            for row in &mut self.board {
                match row[col] {
                    BoardElem::Gray(oc) | BoardElem::Yellow(oc) | BoardElem::Green(oc)
                        if oc == c =>
                    {
                        row[col] = new;
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
    fn calculate(&mut self) {
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
}
