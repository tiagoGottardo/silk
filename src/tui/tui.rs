//! ## Tui
use std::fmt;

use tuirealm::application::PollStrategy;
use tuirealm::{AttrValue, Attribute, Update};

use crate::tui::app;
use app::model::Model;

#[derive(Debug, PartialEq, Clone)]
pub enum MenuItem {
    Search,
    Feed,
    Exit,
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            MenuItem::Search => "Search",
            MenuItem::Feed => "Feed",
            MenuItem::Exit => "Exit",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    Clock,
    MenuSelected(MenuItem),
    MenuMoveUp,
    MenuMoveDown,
    AppExit,
    None,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Clock,
    MenuList,
    Label,
}

pub fn main() {
    let mut model = Model::default();

    let _ = model.terminal.enter_alternate_screen();
    let _ = model.terminal.enable_raw_mode();

    while !model.quit {
        // Tick
        match model.app.tick(PollStrategy::Once) {
            Err(err) => {
                assert!(
                    model
                        .app
                        .attr(
                            &Id::Label,
                            Attribute::Text,
                            AttrValue::String(format!("Application error: {err}")),
                        )
                        .is_ok()
                );
            }
            Ok(messages) if !messages.is_empty() => {
                model.redraw = true;
                for msg in messages {
                    let mut msg = Some(msg);
                    while msg.is_some() {
                        msg = model.update(msg);
                    }
                }
            }
            _ => {}
        }
        // Redraw
        if model.redraw {
            model.view();
            model.redraw = false;
        }
    }
    // Terminate terminal
    let _ = model.terminal.leave_alternate_screen();
    let _ = model.terminal.disable_raw_mode();
    let _ = model.terminal.clear_screen();
}
