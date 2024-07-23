use std::io;

use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use dictionary::Dictionary;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::{Frame, Terminal};
use solveapp::{BoardElem, SolveApp, BOARD_COLS, BOARD_ROWS};

/// App holds the state of the application
pub struct App {
    /// Solve application
    app: SolveApp,
    /// Board rectange
    board_rect: Option<Rect>,
    /// Words rectange
    words_rect: Option<Rect>,
}

impl App {
    /// Board cell draw width
    const CELL_WIDTH: u16 = 5;
    /// Extra X dimension spacing
    const CELL_XSPACE: u16 = 2;
    /// Total width of a board cell
    const CELL_XTOTAL: u16 = Self::CELL_WIDTH + Self::CELL_XSPACE;

    /// Board cell draw height
    const CELL_HEIGHT: u16 = 3;
    /// Extra Y dimension spacing
    const CELL_YSPACE: u16 = 1;
    /// Total height of a board cell
    const CELL_YTOTAL: u16 = Self::CELL_HEIGHT + Self::CELL_YSPACE;

    /// Usage instructions
    const INSTRUCTIONS: &'static str = r#"
Wordle Solver
    
Fill the board on the left by pressing letter keys.

The colour of each letter can be toggled by clicking with the mouse or with the keys 1-5.

Press Escape to exit"#;

    /// Creates the application
    pub fn new(dictionary: Dictionary) -> Self {
        App {
            app: SolveApp::new(dictionary),
            board_rect: None,
            words_rect: None,
        }
    }

    /// Runs the application
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let mut render = true;
        let mut calculate = true;

        loop {
            // Need to recalculate?
            if calculate {
                self.app.calculate();

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
                        if self.app.add(c) {
                            calculate = true;
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() => {
                        // Lower case character
                        if self.app.add(c.to_ascii_uppercase()) {
                            calculate = true;
                        }
                    }
                    KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        // Number pressed
                        let col = (c as u8 - b'1') as usize;

                        if self.app.toggle_col(col) {
                            calculate = true;
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        // Backspace / delete pressed
                        if self.app.remove() {
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
                            if self.app.toggle(row, col) {
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
                            (BOARD_COLS as u16 * Self::CELL_XTOTAL) - Self::CELL_XSPACE + 2,
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

            if self.app.words().count().is_some() {
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
            }
        })?;

        Ok(())
    }

    /// Draws the board table
    fn board_table(&self, f: &mut Frame) {
        // Build board table contents
        let content = self.app.board().iter().enumerate().map(|(rn, row)| {
            // Build board table row
            Row::new(row.iter().map(|col| match col {
                BoardElem::Empty => Self::board_cell(' ', Color::DarkGray),
                BoardElem::Gray(c) => Self::board_cell(*c, Color::DarkGray),
                BoardElem::Yellow(c) => Self::board_cell(*c, Color::Yellow),
                BoardElem::Green(c) => Self::board_cell(*c, Color::Green),
            }))
            .height(Self::CELL_HEIGHT)
            .top_margin(if rn == 0 { 0 } else { 1 })
        });

        // Create the board table
        let table = Table::new(content, [Constraint::Length(Self::CELL_WIDTH); BOARD_COLS])
            .column_spacing(Self::CELL_XSPACE)
            .block(Block::default().borders(Borders::ALL).title("Board"));

        // Render the table
        f.render_widget(table, self.board_rect.unwrap());
    }

    /// Draws a single board cell
    fn board_cell<'b>(c: char, colour: Color) -> Cell<'b> {
        Cell::from(
            Text::from(format!("\n{}", c))
                .centered()
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(colour))
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
    fn words_table(&self, f: &mut Frame) {
        if let Some(rect) = self.words_rect {
            let words = self.app.words().count().unwrap();

            // Calculate the number of rows and columns
            let rows = rect.height as usize - 2;
            let cols = (rect.width as usize - 1) / (BOARD_COLS + 1);

            // Create spans
            let spans = (0..rows)
                .map(|row| {
                    Line::from(Span::styled(
                        (0..cols).fold(String::new(), |mut line, col| {
                            let elem = (col * rows) + row;

                            if elem < words {
                                if col > 0 {
                                    line.push(' ');
                                }
                                line.push_str(&self.app.get_word(elem).unwrap());
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
                    .title(format!("Words ({} found)", words)),
            );

            f.render_widget(para, rect);
        }
    }
}
