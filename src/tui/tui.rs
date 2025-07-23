//! ## Tui

use crate::tui::app::model::Model;
use crate::types::ContentItem;
use tuirealm::application::PollStrategy;
use tuirealm::{AttrValue, Attribute, Update};

#[derive(PartialEq)]
pub enum Msg {
    AppClose,
    Exit,
    Clock,
    MenuSelected(String, usize),
    Subscribe(String, usize),
    Unsubscribe(String, usize),
    Download(String, usize, bool),
    Search(String),
    SearchResults(Vec<ContentItem>),
    None,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Clock,
    Menu,
    SearchMenu,
    Input,
    Label,
    SearchResults,
}

pub fn main() {
    let mut model = Model::default();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    model.tx = tx;

    let _ = model.terminal.enter_alternate_screen();
    let _ = model.terminal.enable_raw_mode();

    while !model.quit {
        // Tick
        let mut messages = vec![];
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }

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
            Ok(app_messages) => {
                messages.extend(app_messages);
            }
        }

        if !messages.is_empty() {
            model.redraw = true;
            for msg in messages {
                let mut msg = Some(msg);
                while msg.is_some() {
                    msg = model.update(msg);
                }
            }
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
