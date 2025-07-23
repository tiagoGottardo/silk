//! ## Input
// a tui-realm component to render an input

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
use unicode_segmentation::UnicodeSegmentation;

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

    fn get_formatted_input(&self) -> (String, u16) {
        let input = self.states.input.as_str();
        let cursor_col = self.get_cursor_column();
        (input.to_string(), cursor_col)
    }

    fn get_cursor_column(&self) -> u16 {
        let head = &self.states.input[..self.states.cursor_position];
        head.graphemes(true).count() as u16
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
            let mut style = Style::default().fg(foreground).bg(background);

            if focus {
                style = style.add_modifier(modifiers);
            }

            let (text_to_display, cursor_col) = self.get_formatted_input();
            frame.render_widget(
                Paragraph::new(text_to_display)
                    .block(get_block(borders, title, focus))
                    .style(style),
                area,
            );

            if focus {
                frame.set_cursor_position(Position::new(area.x + cursor_col + 1, area.y + 1));
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
                self.states.backspace();
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
pub struct OwnStates {
    input: String,
    cursor_position: usize,
}

impl OwnStates {
    fn append(&mut self, ch: char) {
        self.input.insert(self.cursor_position, ch);
        self.incr_cursor();
    }

    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.decr_cursor();
            self.input.remove(self.cursor_position);
        }
    }
    fn clear(&mut self) {
        self.input.clear();
        self.cursor_at_begin();
    }

    fn incr_cursor(&mut self) {
        if let Some((idx, _)) = self
            .input
            .grapheme_indices(true)
            .find(|(idx, _)| *idx > self.cursor_position)
        {
            self.cursor_position = idx;
        } else {
            self.cursor_at_end();
        }
    }

    fn decr_cursor(&mut self) {
        if let Some((idx, _)) = self
            .input
            .grapheme_indices(true)
            .take_while(|(idx, _)| *idx < self.cursor_position)
            .last()
        {
            self.cursor_position = idx;
        }
    }

    fn cursor_at_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    fn cursor_at_begin(&mut self) {
        self.cursor_position = 0;
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
