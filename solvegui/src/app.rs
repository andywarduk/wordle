use dictionary::Dictionary;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, container, row, text, Column, Row};
use iced::Length::Fill;
use iced::{event, window, Color, Element, Event, Rectangle, Subscription, Task};
use once_cell::sync::Lazy;
use solveapp::SolveApp;

pub fn rungui(dictionary: Dictionary) -> iced::Result {
    iced::application("Wordle Solver", App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| App::new(dictionary))
}

const WORD_HEIGHT: u16 = 25;
const WORD_WIDTH: u16 = 90;

#[derive(Debug, Clone)]
enum Message {
    Quit,
    WindowResized,
    WordsBoundsFetched(Option<Rectangle>),
    LetterAdded(char),
    LetterRemoved,
    Toggle(usize, usize),
    ToggleCol(usize),
}

struct App {
    app: SolveApp,
    words_size: Option<Rectangle>,
}

impl App {
    fn new(dictionary: Dictionary) -> (Self, Task<Message>) {
        (
            Self {
                app: SolveApp::new(dictionary),
                words_size: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Quit => window::get_latest().and_then(window::close),
            Message::WindowResized => {
                container::visible_bounds(WORDS_CONTAINER.clone()).map(Message::WordsBoundsFetched)
            }
            Message::WordsBoundsFetched(rect) => {
                self.words_size = rect;
                Task::none()
            }
            Message::LetterAdded(c) => {
                if self.app.add(c) {
                    self.app.calculate()
                }
                Task::none()
            }
            Message::LetterRemoved => {
                if self.app.remove() {
                    self.app.calculate()
                }
                Task::none()
            }
            Message::Toggle(row, col) => {
                if self.app.toggle(row, col) {
                    self.app.calculate()
                }
                Task::none()
            }
            Message::ToggleCol(col) => {
                if self.app.toggle_col(col) {
                    self.app.calculate()
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            keyboard::on_key_press(|key, modifiers| {
                let mut res = None;

                if Self::no_modifiers(modifiers) {
                    match key.as_ref() {
                        Key::Named(Named::Escape) => res = Some(Message::Quit),
                        Key::Named(Named::Delete) | Key::Named(Named::Backspace) => {
                            res = Some(Message::LetterRemoved)
                        }
                        Key::Character(c) => {
                            if let Some(c) = c.chars().next() {
                                if c.is_ascii_uppercase() {
                                    res = Some(Message::LetterAdded(c));
                                } else if c.is_ascii_lowercase() {
                                    res = Some(Message::LetterAdded(c.to_ascii_uppercase()));
                                } else if ('1'..='9').contains(&c) {
                                    res = Some(Message::ToggleCol((c as u8 - b'1') as usize));
                                }
                            }
                        }
                        _ => (),
                    }
                }

                res
            }),
            event::listen_with(|event, _status, _window| match event {
                Event::Window(window::Event::Resized { .. }) => Some(Message::WindowResized),
                _ => None,
            }),
        ])
    }

    fn view(&self) -> Element<Message> {
        let board = self.app.board;

        // Draw the button grid
        let btn_grid: Element<Message> =
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
                            ..text::Style::default()
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
            .into();

        // Get word count
        let word_count = self.app.word_count();

        // Draw the words grid
        let words: Option<Element<Message>> = if let Some(words_size) = self.words_size {
            // How many columns?
            let cols_avail = (words_size.width / WORD_WIDTH as f32).floor() as usize;
            let rows_avail = (words_size.height / WORD_HEIGHT as f32).floor() as usize;

            if cols_avail > 0 && rows_avail > 0 && word_count > 0 {
                let draw_cols = (((word_count - 1) / rows_avail) + 1).min(cols_avail);

                let row = Row::with_children((0..draw_cols).map(|i| {
                    let start = i * rows_avail;

                    Column::with_children((start..word_count.min(start + rows_avail)).map(|j| {
                        text(self.app.get_word(j).unwrap())
                            .height(WORD_HEIGHT)
                            .width(WORD_WIDTH)
                            .into()
                    }))
                    .into()
                }));

                Some(row.into())
            } else {
                None
            }
        } else {
            None
        };

        // Create empty words grid if necessary
        let words = if let Some(words) = words {
            words
        } else {
            text("").into()
        };

        // Create word count text
        let words_txt: Element<Message> = if word_count > 0 {
            text!("Words found: {word_count}")
        } else {
            text("Type letters to fill the board\n\nBackspace to clear the last position\n\nToggle letters with the mouse or\npress 1-5 to toggle the column")
        }
        .into();

        // Draw the board container
        let board_box = container(Column::with_children([btn_grid, words_txt]).spacing(8))
            .height(Fill)
            .padding(10)
            .id(WORDS_CONTAINER.clone());

        // Draw the words container
        let words_box = container(words)
            .height(Fill)
            .width(Fill)
            .padding(10)
            .id(WORDS_CONTAINER.clone());

        // Create row with buttons grid and words
        let res: Element<Message> = row!(board_box, words_box).into();

        // to debug layout res.explain(Color::WHITE)
        res
    }

    fn no_modifiers(modifiers: Modifiers) -> bool {
        !modifiers.alt()
            && !modifiers.command()
            && !modifiers.control()
            && !modifiers.shift()
            && !modifiers.logo()
    }
}

static WORDS_CONTAINER: Lazy<container::Id> = Lazy::new(|| container::Id::new("words"));
