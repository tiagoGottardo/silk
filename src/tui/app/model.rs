//! ## Model
//!
//! app model

use std::time::{Duration};

use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{
    Application, EventListenerCfg, Update,
};

use super::super::components::Label;
use super::super::tui::{Id, Msg};

pub struct Model<T>
where
    T: TerminalAdapter,
{
    pub app: Application<Id, Msg, NoUserEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<T>,
}

impl Default for Model<CrosstermTerminalAdapter> {
    fn default() -> Self {
        Self {
            app: Self::init_app(),
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("Cannot initialize terminal"),
        }
    }
}

impl<T> Model<T>
where
    T: TerminalAdapter,
{
    pub fn view(&mut self) {
        assert!(
            self.terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(
                            [
                                Constraint::Length(1), // Label
                            ]
                            .as_ref(),
                        )
                        .split(f.area());
                    self.app.view(&Id::Label, f, chunks[0]);
                })
                .is_ok()
        );
    }

    fn init_app() -> Application<Id, Msg, NoUserEvent> {
        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(20), 3)
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );
        
        assert!(
            app.mount(
                Id::Label,
                Box::new(
                    Label::default()
                        .text("Press esc to exit")
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::LightYellow)
                        .modifiers(TextModifiers::BOLD),
                ),
                Vec::default(),
            )
            .is_ok()
        );

        assert!(app.active(&Id::Label).is_ok());

        app
    }
}

impl<T> Update<Msg> for Model<T>
where
    T: TerminalAdapter,
{
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            self.redraw = true;

            match msg {
                Msg::AppClose => {
                    self.quit = true; // Terminate
                    None
                }
                Msg::Clock => None,
                _ => None,
            }
        } else {
            None
        }
    }
}
