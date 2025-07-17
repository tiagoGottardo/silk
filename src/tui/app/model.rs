//! ## Model
//!
//! app model

use std::time::Duration;

use tuirealm::event::NoUserEvent;
use tuirealm::props::{Borders, Color};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, EventListenerCfg, Update};

use super::super::components::{Input, Menu};
use super::super::tui::{Id, MenuItem, Msg};

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
                        .constraints([Constraint::Length(3), Constraint::Min(1)])
                        .split(f.area());

                    self.app.view(&Id::Input, f, chunks[0]);
                    self.app.view(&Id::MainMenu, f, chunks[1]);
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
                Id::Input,
                Box::new(
                    Input::default()
                        .borders(Borders::default())
                        .foreground(Color::Green)
                        .label("Search on Youtube"),
                ),
                Vec::default(),
            )
            .is_ok()
        );

        assert!(
            app.mount(
                Id::MainMenu,
                Box::new(Menu::new(vec![
                    MenuItem::Search,
                    MenuItem::Feed,
                    MenuItem::Exit
                ])),
                Vec::default()
            )
            .is_ok()
        );

        assert!(app.active(&Id::MainMenu).is_ok());

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
                Msg::MenuSelected(item) => {
                    match item {
                        MenuItem::Exit => self.quit = true, // Terminate
                        MenuItem::Search => {
                            assert!(self.app.active(&Id::Input).is_ok());
                        }
                        _ => {}
                    }
                    None
                }
                Msg::Search(input) => {
                    self.quit = true;
                    None
                }
                Msg::Exit => {
                    assert!(self.app.active(&Id::MainMenu).is_ok());
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
