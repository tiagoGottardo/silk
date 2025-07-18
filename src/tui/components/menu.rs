use ratatui::style::Color;
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent};
use tuirealm::props::{Alignment, Borders, TableBuilder, TextModifiers, TextSpan};
use tuirealm::ratatui::layout::Rect;
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, State, StateValue,
};

use super::super::tui::Msg;

pub struct Menu {
    component: List,
    items: Vec<String>,
}

impl Menu {
    pub fn new(items: Vec<String>) -> Self {
        let mut table = TableBuilder::default();
        if let Some((last, head)) = items.split_last() {
            for item in head {
                table.add_col(TextSpan::from(item.clone())).add_row();
            }
            table.add_col(TextSpan::from(last.clone()));
        }

        Self {
            component: List::default()
                .foreground(Color::Reset)
                .background(Color::Reset)
                .highlighted_color(Color::Yellow)
                .highlighted_str(">> ")
                .modifiers(TextModifiers::BOLD)
                .scroll(true)
                .step(3)
                .borders(Borders::default())
                .title("Menu", Alignment::Center)
                .rewind(true)
                .rows(table.build()),
            items,
        }
    }
}

impl MockComponent for Menu {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        self.component.view(frame, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    fn state(&self) -> State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for Menu {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                self.perform(Cmd::Move(Direction::Down));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if let Some(item) = self.items.get(index) {
                        Some(Msg::MenuSelected(item.clone(), index))
                    } else {
                        Some(Msg::None)
                    }
                } else {
                    Some(Msg::None)
                }
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::AppClose),
            _ => Some(Msg::None),
        }
    }
}
