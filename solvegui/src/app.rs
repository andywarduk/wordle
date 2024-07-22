use dictionary::Dictionary;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, container, row, text, Column, Responsive, Row, Space};
use iced::window::icon::from_rgba;
use iced::window::{self, Settings as WinSettings};
use iced::{Color, Element, Length, Subscription, Task};
use solveapp::SolveApp;

/// Run the GUI solver
pub fn rungui(dictionary: Dictionary) -> iced::Result {
    // Build icon
    let icon = from_rgba(
        include_bytes!("../assets/wordle_logo_192x192.rgba").to_vec(),
        192,
        192,
    )
    .unwrap();

    // Run the app
    iced::application("Wordle Solver", App::update, App::view)
        .subscription(App::subscription)
        .window(WinSettings {
            icon: Some(icon),
            ..WinSettings::default()
        })
        .run_with(|| App::new(dictionary))
}

/// Height of each word text element
const WORD_HEIGHT: u16 = 25;
/// Width of each word text element
const WORD_WIDTH: u16 = 90;

#[derive(Debug, Clone)]
enum Message {
    Quit,
    LetterAdded(char),
    LetterRemoved,
    Toggle(usize, usize),
    ToggleCol(usize),
}

struct App {
    app: SolveApp,
}

impl App {
    /// Create new GUI app
    fn new(dictionary: Dictionary) -> (Self, Task<Message>) {
        (
            Self {
                app: SolveApp::new(dictionary),
            },
            Task::none(),
        )
    }

    /// Update the state given a message
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Quit => window::get_latest().and_then(window::close),
            Message::LetterAdded(c) => {
                // Add letter to the board
                if self.app.add(c) {
                    self.app.calculate()
                }
                Task::none()
            }
            Message::LetterRemoved => {
                // Remove last letter from the board
                if self.app.remove() {
                    self.app.calculate()
                }
                Task::none()
            }
            Message::Toggle(row, col) => {
                // Toggle a letter at position
                if self.app.toggle(row, col) {
                    self.app.calculate()
                }
                Task::none()
            }
            Message::ToggleCol(col) => {
                // Toggle last letter in the column
                if self.app.toggle_col(col) {
                    self.app.calculate()
                }
                Task::none()
            }
        }
    }

    // Add subscriptions
    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to keyboard events
        keyboard::on_key_press(|key, modifiers| {
            let mut res = None;

            // Check no modifiers
            if Self::no_modifiers(modifiers) {
                match key.as_ref() {
                    Key::Named(Named::Escape) => res = Some(Message::Quit),
                    Key::Named(Named::Delete) | Key::Named(Named::Backspace) => {
                        // Delete / backspace
                        res = Some(Message::LetterRemoved)
                    }
                    Key::Character(c) => {
                        if let Some(c) = c.chars().next() {
                            if c.is_ascii_uppercase() {
                                // Upper case ascii character (A-Z)
                                res = Some(Message::LetterAdded(c));
                            } else if c.is_ascii_lowercase() {
                                // Lower case ascii character (a-z)
                                res = Some(Message::LetterAdded(c.to_ascii_uppercase()));
                            } else if ('1'..='9').contains(&c) {
                                // Number
                                res = Some(Message::ToggleCol((c as u8 - b'1') as usize));
                            }
                        }
                    }
                    _ => (),
                }
            }

            res
        })
    }

    // Create view from state
    fn view(&self) -> Element<Message> {
        // Draw the button grid
        let btn_grid = self.draw_board();

        // Draw the words grid
        let words = self.draw_words();

        // Create word count text
        let words_txt: Element<Message> = match self.app.word_count() {
            Some(word_count) => text!("Words found: {word_count}"),
            None => text(
                "\
                Type letters to fill the board\n\n\
                Backspace to clear the last position\n\n\
                Toggle letters with the mouse or\npress 1-5 to toggle the column\
                ",
            ),
        }
        .into();

        // Draw the board container
        let board_box = container(Column::with_children([btn_grid, words_txt]).spacing(8))
            .height(Length::Fill)
            .padding(10);

        // Draw the words container
        let words_box = container(words)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(10);

        // Create row with buttons grid and words
        let res: Element<Message> = row!(board_box, words_box).into();

        // to debug layout res.explain(Color::WHITE)
        res
    }

    // Return true if no key modifiers present
    fn no_modifiers(modifiers: Modifiers) -> bool {
        !modifiers.alt()
            && !modifiers.command()
            && !modifiers.control()
            && !modifiers.shift()
            && !modifiers.logo()
    }

    // Draw the wordle board
    fn draw_board(&self) -> Element<Message> {
        let board = self.app.board;

        Column::with_children(board.iter().enumerate().map(|(rn, row)| {
            Row::with_children(row.iter().enumerate().map(|(cn, boardelem)| {
                let (enabled, button_char, colour) = match boardelem {
                    solveapp::BoardElem::Empty => (false, ' ', None),
                    solveapp::BoardElem::Gray(c) => {
                        (true, *c, Some(Color::from_rgb(0.3, 0.3, 0.3)))
                    }
                    solveapp::BoardElem::Yellow(c) => {
                        (true, *c, Some(Color::from_rgb(0.8, 0.8, 0.0)))
                    }
                    solveapp::BoardElem::Green(c) => {
                        (true, *c, Some(Color::from_rgb(0.0, 0.8, 0.0)))
                    }
                };

                // Create button text (white)
                let text = text(button_char.to_string())
                    .center()
                    .size(20)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
                        // ..text::Style::default()
                    });

                // Create button
                let mut button = button(text).width(40).height(40);

                // Add click event to toggle
                if enabled {
                    button = button.on_press_with(move || Message::Toggle(rn, cn));
                }

                // Set button colour
                if let Some(colour) = colour {
                    button = button.style(move |_theme, _status| {
                        button::Style::default().with_background(colour)
                    });
                }

                button.into()
            }))
            .spacing(8)
            .into()
        }))
        .spacing(8)
        .into()
    }

    // Draw the found words
    fn draw_words(&self) -> Element<Message> {
        Responsive::new(|words_size| {
            // Get word count
            let content = match self.app.word_count() {
                Some(word_count) if word_count > 0 => {
                    // How many rows and columns?
                    let cols_avail = (words_size.width / WORD_WIDTH as f32).floor() as usize;
                    let rows_avail = (words_size.height / WORD_HEIGHT as f32).floor() as usize;

                    // Enough space to render some words?
                    if cols_avail > 0 && rows_avail > 0 {
                        // How many columns to draw?
                        let draw_cols = (((word_count - 1) / rows_avail) + 1).min(cols_avail);

                        // Create row layout containing columns
                        let row = Row::with_children((0..draw_cols).map(|i| {
                            // Calculate start word for this column
                            let start = i * rows_avail;

                            // Create the word column
                            Column::with_children((start..word_count.min(start + rows_avail)).map(
                                |j| {
                                    // Create text element with the found word
                                    text(self.app.get_word(j).unwrap())
                                        .height(WORD_HEIGHT)
                                        .width(WORD_WIDTH)
                                        .into()
                                },
                            ))
                            .into()
                        }));

                        Some(row.into())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // Draw space element if no words found
            match content {
                Some(elem) => elem,
                None => Space::new(words_size.width, words_size.height).into(),
            }
        })
        .into()
    }
}