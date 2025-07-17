//! ## Input
//!
//! input component

use ratatui::layout::Position;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent};
use tuirealm::props::{Alignment, Borders, Color, Style, TextModifiers};
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Paragraph;
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
    StateValue,
};

use super::super::tui::Msg;
use super::get_block;

#[derive(Default)]
pub struct Input {
    props: Props,
    states: OwnStates,
}

impl Input {
    pub fn label<S>(mut self, label: S) -> Self
    where
        S: AsRef<str>,
    {
        self.attr(
            Attribute::Title,
            AttrValue::Title((label.as_ref().to_string(), Alignment::Left)),
        );
        self
    }

    pub fn alignment(mut self, a: Alignment) -> Self {
        self.attr(Attribute::TextAlign, AttrValue::Alignment(a));
        self
    }

    pub fn foreground(mut self, c: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(c));
        self
    }

    pub fn background(mut self, c: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(c));
        self
    }

    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }
}

impl MockComponent for Input {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            let foreground = self
                .props
                .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let background = self
                .props
                .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let modifiers = self
                .props
                .get_or(
                    Attribute::TextProps,
                    AttrValue::TextModifiers(TextModifiers::empty()),
                )
                .unwrap_text_modifiers();
            let title = self
                .props
                .get_or(
                    Attribute::Title,
                    AttrValue::Title((String::default(), Alignment::Center)),
                )
                .unwrap_title();
            let borders = self
                .props
                .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
                .unwrap_borders();
            let focus = self
                .props
                .get_or(Attribute::Focus, AttrValue::Flag(false))
                .unwrap_flag();

            let mut text_to_display = self.states.input.clone();
            if focus {
                text_to_display.push(' ');
            }

            frame.render_widget(
                Paragraph::new(text_to_display)
                    .style(
                        Style::default()
                            .fg(foreground)
                            .bg(background)
                            .add_modifier(modifiers),
                    )
                    .block(get_block(borders, title, focus)),
                area,
            );

            if focus {
                frame.set_cursor_position(Position::new(
                    area.x + self.states.cursor as u16 + 1u16,
                    area.y + 1,
                ));
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(StateValue::String(self.states.input.clone()))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Left) => {
                self.states.decr_cursor();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Right) => {
                self.states.incr_cursor();
                CmdResult::Changed(self.state())
            }
            Cmd::Delete => {
                self.states.del();
                CmdResult::Changed(self.state())
            }
            Cmd::Cancel => {
                self.states.clear();
                CmdResult::Custom("exit", self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            Cmd::Type(ch) => {
                self.states.append(ch);
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

#[derive(Default)]
struct OwnStates {
    input: String,
    cursor: usize,
}

impl OwnStates {
    fn append(&mut self, ch: char) {
        self.input.insert(self.cursor, ch);
        self.incr_cursor();
    }

    fn del(&mut self) {
        if self.cursor > 0 {
            self.decr_cursor();
            self.input.remove(self.cursor);
        }
    }

    fn clear(&mut self) {
        self.input.clear();
        self.cursor_at_begin();
    }

    fn incr_cursor(&mut self) {
        self.cursor = self.cursor.saturating_add(1);
        if self.cursor > self.input.len() {
            self.cursor_at_end();
        }
    }

    fn decr_cursor(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn cursor_at_begin(&mut self) {
        self.cursor = 0;
    }

    fn cursor_at_end(&mut self) {
        self.cursor = self.input.len();
    }
}

impl Component<Msg, NoUserEvent> for Input {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => Cmd::Move(Direction::Left),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => Cmd::Move(Direction::Right),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => Cmd::Delete,
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => Cmd::Type(ch),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => Cmd::Submit,
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Cmd::Cancel,
            _ => Cmd::None,
        };

        match self.perform(cmd) {
            CmdResult::Submit(State::One(StateValue::String(input))) => Some(Msg::Search(input)),
            CmdResult::Changed(_) => Some(Msg::None),
            CmdResult::Custom("exit", _) => Some(Msg::Exit),
            _ => None,
        }
    }
}
