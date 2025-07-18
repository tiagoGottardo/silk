//! ## Model
//!
//! app model

use std::time::Duration;

use tokio::sync::mpsc;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Borders, Color};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, EventListenerCfg, Update};

use crate::types::ContentItem;
use crate::youtube::search_content;

use super::super::components::{Input, Menu};
use super::super::tui::{Id, Msg};

pub struct Model<T>
where
    T: TerminalAdapter,
{
    pub app: Application<Id, Msg, NoUserEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<T>,
    pub search_result: Vec<ContentItem>,
    pub show_result: bool,
    pub tx: mpsc::Sender<Msg>,
}

impl Default for Model<CrosstermTerminalAdapter> {
    fn default() -> Self {
        let (tx, _rx) = mpsc::channel(1024);
        Self {
            app: Self::init_app(),
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("Cannot initialize terminal"),
            search_result: Vec::default(),
            show_result: false,
            tx,
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
                        .label("Search on Youtube")
                ),
                Vec::default()
            )
            .is_ok()
        );

        assert!(
            app.mount(
                Id::MainMenu,
                Box::new(Menu::new(vec![
                    "Search".to_string(),
                    "Feed".to_string(),
                    "Exit".to_string()
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
                Msg::MenuSelected(item, idx) => {
                    match self.show_result {
                        true => {
                            let mut content_item = self.search_result[idx].clone();

                            tokio::spawn(async move {
                                content_item.play().await;
                            });
                        }
                        false => match item.as_str() {
                            "Exit" => {
                                self.quit = true;
                            }
                            "Search" => {
                                assert!(self.app.active(&Id::Input).is_ok());
                            }
                            "Feed" => {}
                            _ => {}
                        },
                    }

                    None
                }
                Msg::Subscribe(_, idx) => {
                    let content_item = &mut self.search_result[idx];
                    let _ = content_item.subscribe();

                    None
                }
                Msg::Unsubscribe(_, idx) => {
                    let content_item = &mut self.search_result[idx];
                    let _ = content_item.unsubscribe();

                    None
                }
                Msg::Download(_, idx, veido_track) => {
                    let mut content_item = self.search_result[idx].clone();

                    tokio::spawn(async move {
                        content_item.download(veido_track).await;
                    });

                    None
                }
                Msg::Search(input) => {
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Ok(content) = search_content(&input).await {
                            tx.send(Msg::SearchResults(content)).await.ok();
                        }
                    });
                    None
                }
                Msg::SearchResults(content) => {
                    self.search_result = content.clone();

                    let menu_items = content
                        .iter()
                        .map(|content_item| match content_item {
                            ContentItem::Video(video) => video.title.clone(),
                            ContentItem::Channel(channel) => channel.username.clone(),
                            ContentItem::Playlist(playlist) => playlist.title.clone(),
                        })
                        .collect();

                    self.show_result = true;

                    assert!(
                        self.app
                            .remount(
                                Id::MainMenu,
                                Box::new(Menu::new(menu_items)),
                                Vec::default()
                            )
                            .is_ok()
                    );

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
