use dictionary::Dictionary;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, container, row, text, Column, Lazy, Responsive, Row, Space};
use iced::window::icon::from_rgba;
use iced::window::{self, Settings as WinSettings};
use iced::{Color, Element, Length, Size, Subscription, Task};
use solveapp::{SolveApp, Words, BOARD_COLS, BOARD_ROWS};

/// Run the GUI solver
pub fn rungui(dictionary: Dictionary) -> iced::Result {
    // Build icon
    let icon = from_rgba(
        include_bytes!("../assets/wordle_logo_192x192.rgba").to_vec(),
        192,
        192,
    )
    .unwrap();

    // Work out min and initial dimensions
    let board_dim = |btn_count: usize| {
        ((BUTTON_DIM * btn_count as u16) + (BOARD_SPACING * (btn_count as u16 - 1)) + (PADDING * 2))
            as f32
    };

    let words_w = |word_count: u16| ((WORD_WIDTH * word_count) + (PADDING * 2)) as f32;

    let min_w = board_dim(BOARD_COLS);
    let min_h = board_dim(BOARD_ROWS);

    let w = min_w + words_w(4);
    let h = min_h * 1.5;

    // Run the app
    iced::application("Wordle Solver", App::update, App::view)
        .subscription(App::subscription)
        .window(WinSettings {
            icon: Some(icon),
            size: Size::new(w, h),
            min_size: Some(Size::new(min_w, min_h)),
            ..WinSettings::default()
        })
        .run_with(|| App::new(dictionary))
}

/// Dimension of board button
const BUTTON_DIM: u16 = 40;
/// Board button spacing
const BOARD_SPACING: u16 = 8;
/// Height of each word text element
const WORD_HEIGHT: u16 = 25;
/// Width of each word text element
const WORD_WIDTH: u16 = 90;
/// Element padding
const PADDING: u16 = 10;

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
        let words_txt: Element<Message> = match self.app.words().count() {
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
        let board_box = container(Column::with_children([
            btn_grid,
            Space::new(Length::Shrink, 16).into(),
            words_txt,
        ]))
        .height(Length::Fill)
        .padding(PADDING);

        // Draw the words container
        let words_box = container(words)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(PADDING);

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
        Lazy::new(self.app.board(), |board| {
            Column::with_children(board.iter().enumerate().map(|(rn, row)| {
                Row::with_children(row.iter().enumerate().map(|(cn, boardelem)| {
                    // Calculate enebled, character and colour from board element
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

                    // Create button with text
                    let mut button = button(text).width(BUTTON_DIM).height(BUTTON_DIM);

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
                .spacing(BOARD_SPACING)
                .into()
            }))
            .spacing(BOARD_SPACING)
        })
        .into()
    }

    // Draw the found words
    fn draw_words(&self) -> Element<Message> {
        // Create responsive container
        Responsive::new(|size| {
            // Dependency structure
            #[derive(Hash)]
            struct WordsDep<'a> {
                size: Size<usize>,
                words: &'a Words,
            }

            // How many rows and columns?
            let cols_avail = (size.width / WORD_WIDTH as f32).floor() as usize;
            let rows_avail = (size.height / WORD_HEIGHT as f32).floor() as usize;

            // Set dependency structure
            let dep = WordsDep {
                size: Size::new(cols_avail, rows_avail),
                words: self.app.words(),
            };

            // Create lazy content
            let content = Lazy::new(dep, |dep| {
                // Get size
                let size = dep.size;

                // Get words
                let words = dep.words;

                // Get word count
                let content: Option<Element<Message>> = match words.count() {
                    Some(word_count) if word_count > 0 => {
                        // Enough space to render some words?
                        if size.width > 0 && size.height > 0 {
                            // How many columns to draw?
                            let draw_cols = (((word_count - 1) / size.height) + 1).min(size.width);

                            // Create row layout containing columns
                            let row = Row::with_children((0..draw_cols).map(|i| {
                                // Calculate start word for this column
                                let start = i * size.height;

                                // Create the word column
                                Column::with_children(
                                    (start..word_count.min(start + size.height)).map(|j| {
                                        // Create text element with the found word
                                        text(self.app.get_word(j).unwrap())
                                            .height(WORD_HEIGHT)
                                            .width(WORD_WIDTH)
                                            .into()
                                    }),
                                )
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
                    None => Space::new(size.width as u16, size.height as u16).into(),
                }
            });

            content.into()
        })
        .into()
    }
}
